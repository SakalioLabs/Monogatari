import builtInCatalog from '../../../data/events/story_events.json'
import { hasTauriRuntime, invokeCommand } from './tauri'

export const STORY_EVENT_CATALOG_SCHEMA_V1 = 'monogatari-story-event-catalog/v1'

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

interface StoryEventDocument {
  schema: string
  events: Array<{
    event_id: string
    event_type: string
    description: string
    data?: Record<string, unknown>
    character_ids?: string[]
    repeatable?: boolean
    rule?: Omit<EventTriggerRule, 'event_id' | 'event_type' | 'rule_fingerprint' | 'character_ids' | 'repeatable'>
  }>
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

  const seen = new Set<string>()
  const events = document.events.map((event): StoryEventDefinition => {
    const eventId = String(event.event_id || '').trim()
    const eventType = String(event.event_type || '').trim()
    const description = String(event.description || '').trim()
    if (!eventId || !eventType || !description || seen.has(eventId)) {
      throw new Error(`Invalid or duplicate story event: ${eventId || '<empty>'}`)
    }
    seen.add(eventId)
    return {
      event_id: eventId,
      event_type: eventType,
      description,
      data: event.data && typeof event.data === 'object' ? event.data : {},
      source_path: source,
      rule: {
        event_id: eventId,
        event_type: eventType,
        ...(event.rule || {}),
        character_ids: [...(event.character_ids || [])].sort(),
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

export async function loadStoryEventCatalog(): Promise<StoryEventCatalogSnapshot> {
  if (hasTauriRuntime()) {
    return invokeCommand<StoryEventCatalogSnapshot>('get_story_event_catalog')
  }

  try {
    const response = await fetch(webCatalogUrl(), { cache: 'no-cache' })
    if (!response.ok) throw new Error(`HTTP ${response.status}`)
    return documentSnapshot(await response.json() as StoryEventDocument, 'web_project')
  } catch {
    return documentSnapshot(builtInCatalog as StoryEventDocument, 'web_bundled_fallback')
  }
}
