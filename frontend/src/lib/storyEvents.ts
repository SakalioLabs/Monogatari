import builtInCatalog from '../../../data/events/story_events.json'
import { hasTauriRuntime, invokeCommand } from './tauri'

export const STORY_EVENT_CATALOG_SCHEMA_V1 = 'monogatari-story-event-catalog/v1'

export type StoryEventAction =
  | { type: 'unlock_scene'; scene_id: string }
  | { type: 'unlock_dialogue'; dialogue_id: string }
  | { type: 'unlock_ending'; ending_id: string }
  | { type: 'set_flag'; flag: string; value: boolean }

export interface EventTriggerRule {
  event_id: string
  event_type: string
  rule_fingerprint?: string | null
  min_relationship?: number | null
  score_metric?: string | null
  min_score?: number | null
  min_evaluation_count?: number | null
  character_ids?: string[]
  repeatable?: boolean
}

export interface StoryEventDefinition {
  event_id: string
  event_type: string
  description: string
  data: Record<string, unknown>
  actions: StoryEventAction[]
  rule: EventTriggerRule
  source_path: string
}

export interface StoryEventCatalogSnapshot {
  schema: string
  source: string
  event_count: number
  catalog_fingerprint: string
  events: StoryEventDefinition[]
}

export interface StoryEventRuleDraft {
  min_relationship?: number
  score_metric?: 'friendliness' | 'engagement' | 'creativity' | 'overall'
  min_score?: number
  min_evaluation_count?: number
}

export interface StoryEventDraft {
  event_id: string
  event_type: string
  description: string
  data?: Record<string, unknown>
  actions?: StoryEventAction[]
  character_ids?: string[]
  repeatable?: boolean
  rule?: StoryEventRuleDraft
}

export interface StoryEventDocument {
  schema: string
  events: StoryEventDraft[]
}

const portableIdPattern = /^[A-Za-z0-9_.-]+$/

function portableId(value: unknown, label: string): string {
  const normalized = String(value ?? '')
  if (!normalized || normalized.trim() !== normalized || normalized.length > 128 || !portableIdPattern.test(normalized)) {
    throw new Error(`Invalid story event ${label}: ${normalized || '<empty>'}`)
  }
  return normalized
}

function normalizeActions(event: StoryEventDocument['events'][number], data: Record<string, unknown>): StoryEventAction[] {
  const actions: StoryEventAction[] = []
  const seen = new Set<string>()
  const append = (action: StoryEventAction) => {
    const key = JSON.stringify(action)
    if (seen.has(key)) throw new Error(`Duplicate story event action: ${key}`)
    seen.add(key)
    actions.push(action)
  }

  for (const raw of event.actions || []) {
    if (!raw || typeof raw !== 'object' || Array.isArray(raw)) throw new Error('Invalid story event action')
    const action = raw as Record<string, unknown>
    const allowedFields = action.type === 'unlock_scene'
      ? ['type', 'scene_id']
      : action.type === 'unlock_dialogue'
        ? ['type', 'dialogue_id']
        : action.type === 'unlock_ending'
          ? ['type', 'ending_id']
          : action.type === 'set_flag'
            ? ['type', 'flag', 'value']
            : []
    if (Object.keys(action).some((field) => !allowedFields.includes(field))) {
      throw new Error(`Unknown field in story event action: ${String(action.type)}`)
    }
    if (action.type === 'unlock_scene') {
      append({ type: 'unlock_scene', scene_id: portableId(action.scene_id, 'action scene_id') })
    } else if (action.type === 'unlock_dialogue') {
      append({ type: 'unlock_dialogue', dialogue_id: portableId(action.dialogue_id, 'action dialogue_id') })
    } else if (action.type === 'unlock_ending') {
      append({ type: 'unlock_ending', ending_id: portableId(action.ending_id, 'action ending_id') })
    } else if (action.type === 'set_flag' && typeof action.value === 'boolean') {
      append({ type: 'set_flag', flag: portableId(action.flag, 'action flag'), value: action.value })
    } else {
      throw new Error(`Unsupported story event action: ${String(action.type)}`)
    }
  }

  for (const [field, type] of [
    ['unlock_scene', 'unlock_scene'],
    ['dialogue_id', 'unlock_dialogue'],
    ['unlock_ending', 'unlock_ending'],
  ] as const) {
    if (!(field in data)) continue
    if (typeof data[field] !== 'string') throw new Error(`Legacy story event data.${field} must be a string`)
    const value = portableId(data[field], `legacy data ${field}`)
    const action: StoryEventAction = type === 'unlock_scene'
      ? { type, scene_id: value }
      : type === 'unlock_dialogue'
        ? { type, dialogue_id: value }
        : { type, ending_id: value }
    const key = JSON.stringify(action)
    if (!seen.has(key)) {
      seen.add(key)
      actions.push(action)
    }
  }
  if (actions.length > 64) throw new Error('Story event has more than 64 actions')
  return actions
}

function webCatalogUrl(): string {
  const base = import.meta.env.BASE_URL || '/'
  const baseUrl = base === './'
    ? new URL('./', window.location.href)
    : new URL(base, window.location.origin)
  return new URL('events/story_events.json', baseUrl).toString()
}

function documentSnapshot(document: StoryEventDocument, source: string): StoryEventCatalogSnapshot {
  if (document.schema !== STORY_EVENT_CATALOG_SCHEMA_V1 || !Array.isArray(document.events)) {
    throw new Error(`Unsupported story event catalog schema: ${String(document.schema)}`)
  }
  if (document.events.length > 512) throw new Error('Story event catalog has more than 512 events')

  const seen = new Set<string>()
  const events = document.events.map((event): StoryEventDefinition => {
    const eventId = portableId(event.event_id, 'event_id')
    const eventType = portableId(event.event_type, 'event_type')
    const description = String(event.description || '')
    if (!eventId || !eventType || !description.trim() || description.length > 2048 || seen.has(eventId)) {
      throw new Error(`Invalid or duplicate story event: ${eventId || '<empty>'}`)
    }
    seen.add(eventId)
    const characterIds = [...(event.character_ids || [])].map((id) => portableId(id, 'character_id')).sort()
    if (characterIds.length > 128 || new Set(characterIds).size !== characterIds.length) {
      throw new Error(`Invalid or duplicate character scope in story event: ${eventId}`)
    }
    validateRule(eventId, event.rule || {})
    return {
      event_id: eventId,
      event_type: eventType,
      description,
      data: event.data && typeof event.data === 'object' && !Array.isArray(event.data) ? event.data : {},
      actions: normalizeActions(
        event,
        event.data && typeof event.data === 'object' && !Array.isArray(event.data) ? event.data : {},
      ),
      source_path: source,
      rule: {
        event_id: eventId,
        event_type: eventType,
        ...(event.rule || {}),
        character_ids: characterIds,
        repeatable: Boolean(event.repeatable),
      },
    }
  }).sort((left, right) => left.event_id.localeCompare(right.event_id))

  return {
    schema: STORY_EVENT_CATALOG_SCHEMA_V1,
    source,
    event_count: events.length,
    catalog_fingerprint: '',
    events,
  }
}

function validateRule(eventId: string, rule: StoryEventRuleDraft): void {
  if (rule.min_relationship !== undefined
    && (!Number.isFinite(rule.min_relationship) || rule.min_relationship < -1 || rule.min_relationship > 1)) {
    throw new Error(`Story event ${eventId} relationship threshold must be between -1 and 1`)
  }
  const hasMetric = rule.score_metric !== undefined
  const hasScore = rule.min_score !== undefined
  if (hasMetric !== hasScore) throw new Error(`Story event ${eventId} score metric and threshold must be set together`)
  if (hasMetric && !['friendliness', 'engagement', 'creativity', 'overall'].includes(String(rule.score_metric))) {
    throw new Error(`Story event ${eventId} has an unsupported score metric`)
  }
  if (hasScore && (!Number.isFinite(rule.min_score) || Number(rule.min_score) < 0 || Number(rule.min_score) > 1)) {
    throw new Error(`Story event ${eventId} score threshold must be between 0 and 1`)
  }
  if (rule.min_evaluation_count !== undefined
    && (!Number.isInteger(rule.min_evaluation_count) || rule.min_evaluation_count < 0 || rule.min_evaluation_count > 1_000_000)) {
    throw new Error(`Story event ${eventId} evaluation count must be an integer between 0 and 1000000`)
  }
}

export function storyEventCatalogDocument(snapshot: StoryEventCatalogSnapshot): StoryEventDocument {
  return {
    schema: STORY_EVENT_CATALOG_SCHEMA_V1,
    events: snapshot.events.map((event) => ({
      event_id: event.event_id,
      event_type: event.event_type,
      description: event.description,
      ...(Object.keys(event.data || {}).length > 0 ? { data: structuredClone(event.data) } : {}),
      ...(event.actions.length > 0 ? { actions: structuredClone(event.actions) } : {}),
      ...((event.rule.character_ids?.length || 0) > 0 ? { character_ids: [...(event.rule.character_ids || [])] } : {}),
      ...(event.rule.repeatable ? { repeatable: true } : {}),
      rule: {
        ...(event.rule.min_relationship !== undefined && event.rule.min_relationship !== null
          ? { min_relationship: event.rule.min_relationship } : {}),
        ...(event.rule.score_metric ? { score_metric: event.rule.score_metric as StoryEventRuleDraft['score_metric'] } : {}),
        ...(event.rule.min_score !== undefined && event.rule.min_score !== null
          ? { min_score: event.rule.min_score } : {}),
        ...(event.rule.min_evaluation_count !== undefined && event.rule.min_evaluation_count !== null
          ? { min_evaluation_count: event.rule.min_evaluation_count } : {}),
      },
    })),
  }
}

const browserDraftKey = 'monogatari.storyEventCatalogDraft.v1'

export async function loadStoryEventCatalog(): Promise<StoryEventCatalogSnapshot> {
  if (hasTauriRuntime()) {
    return invokeCommand<StoryEventCatalogSnapshot>('get_story_event_catalog')
  }

  try {
    const localDraft = localStorage.getItem(browserDraftKey)
    if (localDraft) return documentSnapshot(JSON.parse(localDraft) as StoryEventDocument, 'web_local_draft')
    const response = await fetch(webCatalogUrl(), { cache: 'no-cache' })
    if (!response.ok) throw new Error(`HTTP ${response.status}`)
    return documentSnapshot(await response.json() as StoryEventDocument, 'web_project')
  } catch {
    return documentSnapshot(builtInCatalog as StoryEventDocument, 'web_bundled_fallback')
  }
}

export async function reloadStoryEventCatalog(): Promise<StoryEventCatalogSnapshot> {
  if (hasTauriRuntime()) {
    return invokeCommand<StoryEventCatalogSnapshot>('reload_story_event_catalog')
  }
  return loadStoryEventCatalog()
}

export async function saveStoryEventCatalog(
  document: StoryEventDocument,
  expectedCatalogFingerprint: string,
): Promise<StoryEventCatalogSnapshot> {
  if (hasTauriRuntime()) {
    return invokeCommand<StoryEventCatalogSnapshot>('save_story_event_catalog', {
      document,
      expectedCatalogFingerprint,
    })
  }
  const snapshot = documentSnapshot(document, 'web_local_draft')
  localStorage.setItem(browserDraftKey, JSON.stringify(document))
  return snapshot
}
