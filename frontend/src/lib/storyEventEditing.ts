import { cloneJsonRecord, isJsonRecord, parseJsonRecord } from './jsonValue'
import type {
  StoryEventAction,
  StoryEventDocument,
  StoryEventDraft,
  StoryEventRuleDraft,
} from './storyEvents'

export type StoryEventGate = 'relationship' | 'score' | 'evaluation'

export interface StoryEventReferenceIds {
  character_ids: readonly string[]
  scene_ids: readonly string[]
  dialogue_ids: readonly string[]
  ending_ids: readonly string[]
}

export interface StoryEventActionDefaults {
  scene_id?: string
  dialogue_id?: string
  ending_id?: string
}

export interface StoryEventListItem {
  event: StoryEventDraft
  index: number
}

export interface StoryEventDocumentEditResult {
  document: StoryEventDocument
  selected_index: number
  changed: boolean
}

export type StoryEventEditingIssueCode =
  | 'catalog_limit'
  | 'invalid_id'
  | 'duplicate_id'
  | 'invalid_type'
  | 'description'
  | 'duplicate_scope'
  | 'unknown_character'
  | 'relationship_threshold'
  | 'score_pair'
  | 'score_threshold'
  | 'evaluation_count'
  | 'duplicate_action'
  | 'unknown_scene'
  | 'unknown_dialogue'
  | 'unknown_ending'
  | 'invalid_flag'

export interface StoryEventEditingIssue {
  code: StoryEventEditingIssueCode
  event_id?: string
  target_id?: string
  action_type?: StoryEventAction['type']
}

export type StoryEventEditingWarningCode = 'no_effects' | 'no_trigger'

export interface StoryEventEditingWarning {
  code: StoryEventEditingWarningCode
  event_id: string
}

export interface StoryEventMetadataResult {
  value: Record<string, unknown> | null
  error: 'invalid_object' | null
}

export interface StoryEventMetadataApplyResult extends StoryEventMetadataResult {
  event: StoryEventDraft
  changed: boolean
}

const PORTABLE_ID = /^[A-Za-z0-9_.-]+$/
const DEFAULT_EVENT_TYPES = [
  'relationship_milestone',
  'special_dialogue',
  'cumulative_achievement',
]
const STORY_EVENT_ACTION_TYPES: readonly string[] = [
  'unlock_scene',
  'unlock_dialogue',
  'unlock_ending',
  'set_flag',
]

export function cloneStoryEventDocument(document: StoryEventDocument): StoryEventDocument {
  return {
    schema: document.schema,
    events: document.events.map(cloneStoryEventDraft),
  }
}

export function cloneStoryEventDraft(event: StoryEventDraft): StoryEventDraft {
  return {
    event_id: event.event_id,
    event_type: event.event_type,
    description: event.description,
    data: cloneJsonRecord(isJsonRecord(event.data) ? event.data : {}),
    actions: (event.actions || []).map(cloneStoryEventAction),
    character_ids: [...(event.character_ids || [])],
    repeatable: Boolean(event.repeatable),
    rule: { ...(event.rule || {}) },
  }
}

export function storyEventDocumentSnapshot(document: StoryEventDocument): string {
  return JSON.stringify(document)
}

export function storyEventTypes(document: StoryEventDocument): string[] {
  return [...new Set([
    ...DEFAULT_EVENT_TYPES,
    ...document.events.map((event) => event.event_type).filter(Boolean),
  ])].sort()
}

export function filterStoryEvents(
  document: StoryEventDocument,
  search: string,
  typeFilter: string,
): StoryEventListItem[] {
  const needle = search.trim().toLocaleLowerCase()
  return document.events
    .map((event, index) => ({ event, index }))
    .filter(({ event }) => !typeFilter || event.event_type === typeFilter)
    .filter(({ event }) => !needle
      || `${event.event_id} ${event.event_type} ${event.description}`.toLocaleLowerCase().includes(needle))
}

export function storyEventActionCount(document: StoryEventDocument): number {
  return document.events.reduce((sum, event) => sum + (event.actions?.length || 0), 0)
}

export function storyEventLockedTargetCount(document: StoryEventDocument): number {
  return new Set(document.events.flatMap((event) => (event.actions || [])
    .flatMap((action) => actionTargetKey(action)))).size
}

export function nextStoryEventId(
  events: readonly StoryEventDraft[],
  base = 'new_event',
): string {
  const existing = new Set(events.map((event) => event.event_id))
  if (!existing.has(base)) return base
  let index = 2
  while (existing.has(`${base}_${index}`)) index += 1
  return `${base}_${index}`
}

export function appendStoryEvent(
  source: StoryEventDocument,
  description: string,
): StoryEventDocumentEditResult {
  const document = cloneStoryEventDocument(source)
  document.events.push(cloneStoryEventDraft({
    event_id: nextStoryEventId(document.events),
    event_type: 'special_dialogue',
    description,
    actions: [],
    rule: { min_relationship: 0.5 },
  }))
  return { document, selected_index: document.events.length - 1, changed: true }
}

export function duplicateStoryEvent(
  source: StoryEventDocument,
  selectedIndex: number,
): StoryEventDocumentEditResult {
  if (!source.events[selectedIndex]) {
    return { document: source, selected_index: selectedIndex, changed: false }
  }
  const document = cloneStoryEventDocument(source)
  const copy = cloneStoryEventDraft(document.events[selectedIndex])
  copy.event_id = nextStoryEventId(document.events, `${copy.event_id}_copy`)
  document.events.splice(selectedIndex + 1, 0, copy)
  return { document, selected_index: selectedIndex + 1, changed: true }
}

export function deleteStoryEvent(
  source: StoryEventDocument,
  selectedIndex: number,
): StoryEventDocumentEditResult {
  if (!source.events[selectedIndex]) {
    return { document: source, selected_index: selectedIndex, changed: false }
  }
  const document = cloneStoryEventDocument(source)
  document.events.splice(selectedIndex, 1)
  return {
    document,
    selected_index: Math.min(selectedIndex, document.events.length - 1),
    changed: true,
  }
}

export function storyEventGateEnabled(
  event: StoryEventDraft | null | undefined,
  gate: StoryEventGate,
): boolean {
  if (gate === 'relationship') return event?.rule?.min_relationship !== undefined
  if (gate === 'score') return event?.rule?.score_metric !== undefined || event?.rule?.min_score !== undefined
  return event?.rule?.min_evaluation_count !== undefined
}

export function setStoryEventGate(
  source: StoryEventDraft,
  gate: StoryEventGate,
  enabled: boolean,
): StoryEventDraft {
  const event = cloneStoryEventDraft(source)
  const rule = event.rule || (event.rule = {})
  if (gate === 'relationship') {
    if (enabled) rule.min_relationship = 0.5
    else delete rule.min_relationship
  } else if (gate === 'score') {
    if (enabled) {
      rule.score_metric = 'overall'
      rule.min_score = 0.7
    } else {
      delete rule.score_metric
      delete rule.min_score
    }
  } else if (enabled) {
    rule.min_evaluation_count = 1
  } else {
    delete rule.min_evaluation_count
  }
  return event
}

export function toggleStoryEventCharacter(
  source: StoryEventDraft,
  characterId: string,
): StoryEventDraft {
  const event = cloneStoryEventDraft(source)
  const scopes = event.character_ids || (event.character_ids = [])
  const index = scopes.indexOf(characterId)
  if (index >= 0) scopes.splice(index, 1)
  else scopes.push(characterId)
  scopes.sort()
  return event
}

export function createStoryEventAction(
  type: StoryEventAction['type'],
  defaults: StoryEventActionDefaults,
): StoryEventAction {
  if (type === 'unlock_dialogue') return { type, dialogue_id: defaults.dialogue_id || '' }
  if (type === 'unlock_ending') return { type, ending_id: defaults.ending_id || 'new_ending' }
  if (type === 'set_flag') return { type, flag: 'story.event_complete', value: true }
  return { type: 'unlock_scene', scene_id: defaults.scene_id || '' }
}

export function isStoryEventActionType(value: string): value is StoryEventAction['type'] {
  return STORY_EVENT_ACTION_TYPES.includes(value)
}

export function appendStoryEventAction(
  source: StoryEventDraft,
  action: StoryEventAction,
): StoryEventDraft {
  const event = cloneStoryEventDraft(source)
  event.actions?.push(cloneStoryEventAction(action))
  return event
}

export function replaceStoryEventAction(
  source: StoryEventDraft,
  index: number,
  action: StoryEventAction,
): StoryEventDraft {
  const event = cloneStoryEventDraft(source)
  if (event.actions?.[index]) event.actions[index] = cloneStoryEventAction(action)
  return event
}

export function removeStoryEventAction(
  source: StoryEventDraft,
  index: number,
): StoryEventDraft {
  const event = cloneStoryEventDraft(source)
  if (Number.isInteger(index) && index >= 0) event.actions?.splice(index, 1)
  return event
}

export function parseStoryEventMetadata(value: string): StoryEventMetadataResult {
  const parsed = parseJsonRecord(value)
  return parsed
    ? { value: parsed, error: null }
    : { value: null, error: 'invalid_object' }
}

export function storyEventMetadataText(event: StoryEventDraft | null | undefined): string {
  return JSON.stringify(isJsonRecord(event?.data) ? event.data : {}, null, 2)
}

export function storyEventMetadataChanged(
  event: StoryEventDraft | null | undefined,
  value: string,
): boolean {
  if (!event) return false
  const parsed = parseStoryEventMetadata(value)
  if (!parsed.value) return true
  return JSON.stringify(parsed.value) !== JSON.stringify(isJsonRecord(event.data) ? event.data : {})
}

export function applyStoryEventMetadata(
  source: StoryEventDraft,
  value: string,
): StoryEventMetadataApplyResult {
  const parsed = parseStoryEventMetadata(value)
  if (!parsed.value) return { ...parsed, event: source, changed: false }
  const changed = JSON.stringify(parsed.value) !== JSON.stringify(isJsonRecord(source.data) ? source.data : {})
  if (!changed) return { ...parsed, event: source, changed: false }
  const event = cloneStoryEventDraft(source)
  event.data = parsed.value
  return { ...parsed, event, changed: true }
}

export function validateStoryEventDocument(
  document: StoryEventDocument,
  references: StoryEventReferenceIds,
): StoryEventEditingIssue[] {
  const issues: StoryEventEditingIssue[] = []
  const ids = new Set<string>()
  const sceneIds = new Set(references.scene_ids)
  const dialogueIds = new Set(references.dialogue_ids)
  const endingIds = new Set(references.ending_ids)
  const characterIds = new Set(references.character_ids)
  if (document.events.length > 512) issues.push({ code: 'catalog_limit' })

  for (const event of document.events) {
    const eventId = event.event_id
    if (!PORTABLE_ID.test(eventId) || eventId.length > 128) issues.push({ code: 'invalid_id', event_id: eventId })
    if (ids.has(eventId)) issues.push({ code: 'duplicate_id', event_id: eventId })
    ids.add(eventId)
    if (!PORTABLE_ID.test(event.event_type) || event.event_type.length > 128) {
      issues.push({ code: 'invalid_type', event_id: eventId })
    }
    if (!event.description?.trim() || event.description.length > 2048) {
      issues.push({ code: 'description', event_id: eventId })
    }

    const scopes = event.character_ids || []
    if (new Set(scopes).size !== scopes.length) issues.push({ code: 'duplicate_scope', event_id: eventId })
    for (const characterId of scopes) {
      if (!characterIds.has(characterId)) {
        issues.push({ code: 'unknown_character', event_id: eventId, target_id: characterId })
      }
    }

    validateStoryEventRule(eventId, event.rule || {}, issues)
    const actionKeys = new Set<string>()
    for (const action of event.actions || []) {
      const key = JSON.stringify(action)
      if (actionKeys.has(key)) {
        issues.push({ code: 'duplicate_action', event_id: eventId, action_type: action.type })
      }
      actionKeys.add(key)
      if (action.type === 'unlock_scene' && !sceneIds.has(action.scene_id)) {
        issues.push({ code: 'unknown_scene', event_id: eventId, target_id: action.scene_id })
      } else if (action.type === 'unlock_dialogue' && !dialogueIds.has(action.dialogue_id)) {
        issues.push({ code: 'unknown_dialogue', event_id: eventId, target_id: action.dialogue_id })
      } else if (action.type === 'unlock_ending' && !endingIds.has(action.ending_id)) {
        issues.push({ code: 'unknown_ending', event_id: eventId, target_id: action.ending_id })
      } else if (action.type === 'set_flag' && !PORTABLE_ID.test(action.flag)) {
        issues.push({ code: 'invalid_flag', event_id: eventId })
      }
    }
  }
  return issues
}

export function storyEventDocumentWarnings(
  document: StoryEventDocument,
): StoryEventEditingWarning[] {
  const warnings: StoryEventEditingWarning[] = []
  for (const event of document.events) {
    const rule = event.rule || {}
    if (!event.actions?.length) warnings.push({ code: 'no_effects', event_id: event.event_id })
    if (rule.min_relationship === undefined && rule.min_score === undefined && rule.min_evaluation_count === undefined) {
      warnings.push({ code: 'no_trigger', event_id: event.event_id })
    }
  }
  return warnings
}

function cloneStoryEventAction(action: StoryEventAction): StoryEventAction {
  if (action.type === 'unlock_scene') return { type: action.type, scene_id: action.scene_id }
  if (action.type === 'unlock_dialogue') return { type: action.type, dialogue_id: action.dialogue_id }
  if (action.type === 'unlock_ending') return { type: action.type, ending_id: action.ending_id }
  return { type: action.type, flag: action.flag, value: action.value }
}

function actionTargetKey(action: StoryEventAction): string[] {
  if (action.type === 'unlock_scene') return [`scene:${action.scene_id}`]
  if (action.type === 'unlock_dialogue') return [`dialogue:${action.dialogue_id}`]
  if (action.type === 'unlock_ending') return [`ending:${action.ending_id}`]
  return []
}

function validateStoryEventRule(
  eventId: string,
  rule: StoryEventRuleDraft,
  issues: StoryEventEditingIssue[],
): void {
  if (rule.min_relationship !== undefined
    && (!Number.isFinite(rule.min_relationship) || rule.min_relationship < -1 || rule.min_relationship > 1)) {
    issues.push({ code: 'relationship_threshold', event_id: eventId })
  }
  if ((rule.score_metric === undefined) !== (rule.min_score === undefined)) {
    issues.push({ code: 'score_pair', event_id: eventId })
  }
  if (rule.min_score !== undefined
    && (!Number.isFinite(rule.min_score) || rule.min_score < 0 || rule.min_score > 1)) {
    issues.push({ code: 'score_threshold', event_id: eventId })
  }
  if (rule.min_evaluation_count !== undefined
    && (!Number.isInteger(rule.min_evaluation_count)
      || rule.min_evaluation_count < 0
      || rule.min_evaluation_count > 1_000_000)) {
    issues.push({ code: 'evaluation_count', event_id: eventId })
  }
}
