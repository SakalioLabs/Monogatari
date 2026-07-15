import { describe, expect, it } from 'vitest'
import { reactive } from 'vue'

import type { StoryEventDocument, StoryEventDraft } from '../storyEvents'
import {
  appendStoryEvent,
  appendStoryEventAction,
  applyStoryEventMetadata,
  cloneStoryEventDocument,
  createStoryEventAction,
  deleteStoryEvent,
  duplicateStoryEvent,
  filterStoryEvents,
  nextStoryEventId,
  parseStoryEventMetadata,
  removeStoryEventAction,
  replaceStoryEventAction,
  setStoryEventGate,
  storyEventActionCount,
  storyEventDocumentSnapshot,
  storyEventDocumentWarnings,
  storyEventGateEnabled,
  storyEventLockedTargetCount,
  storyEventMetadataChanged,
  storyEventMetadataText,
  storyEventTypes,
  toggleStoryEventCharacter,
  validateStoryEventDocument,
} from '../storyEventEditing'

function event(overrides: Partial<StoryEventDraft> = {}): StoryEventDraft {
  return {
    event_id: 'first_event',
    event_type: 'special_dialogue',
    description: 'A tested story milestone.',
    data: { nested: { ready: true } },
    actions: [{ type: 'unlock_scene', scene_id: 'garden' }],
    character_ids: ['aoi'],
    repeatable: false,
    rule: { min_relationship: 0.5 },
    ...overrides,
  }
}

function document(events: StoryEventDraft[] = [event()]): StoryEventDocument {
  return { schema: 'monogatari-story-event-catalog/v1', events }
}

const references = {
  character_ids: ['aoi', 'emi'],
  scene_ids: ['garden', 'station'],
  dialogue_ids: ['intro'],
  ending_ids: ['truth'],
}

describe('Story Event document editing', () => {
  it('clones reactive event documents without retaining nested references', () => {
    const source = reactive(document())
    const cloned = cloneStoryEventDocument(source)

    cloned.events[0].actions![0] = { type: 'unlock_scene', scene_id: 'station' }
    cloned.events[0].character_ids!.push('emi')
    ;(cloned.events[0].data!.nested as { ready: boolean }).ready = false
    expect(source.events[0].actions![0]).toEqual({ type: 'unlock_scene', scene_id: 'garden' })
    expect(source.events[0].character_ids).toEqual(['aoi'])
    expect(source.events[0].data!.nested).toEqual({ ready: true })
  })

  it('filters events, merges event types, and derives catalog metrics', () => {
    const source = document([
      event(),
      event({
        event_id: 'second_event',
        event_type: 'custom_gate',
        description: 'Quiet station route.',
        actions: [
          { type: 'unlock_scene', scene_id: 'garden' },
          { type: 'unlock_scene', scene_id: 'garden' },
          { type: 'unlock_dialogue', dialogue_id: 'intro' },
          { type: 'set_flag', flag: 'route.ready', value: true },
        ],
      }),
    ])

    expect(filterStoryEvents(source, '  STATION ', 'custom_gate').map(({ index }) => index)).toEqual([1])
    expect(storyEventTypes(source)).toContain('custom_gate')
    expect(storyEventActionCount(source)).toBe(5)
    expect(storyEventLockedTargetCount(source)).toBe(2)
  })

  it('appends, duplicates, and deletes isolated documents with stable selection', () => {
    const source = document([event({ event_id: 'new_event' })])
    const appended = appendStoryEvent(source, 'New milestone.')
    expect(appended).toMatchObject({ selected_index: 1, changed: true })
    expect(appended.document.events[1]).toMatchObject({
      event_id: 'new_event_2',
      event_type: 'special_dialogue',
      rule: { min_relationship: 0.5 },
    })

    const duplicated = duplicateStoryEvent(appended.document, 1)
    expect(duplicated.document.events[2].event_id).toBe('new_event_2_copy')
    duplicated.document.events[2].description = 'Changed copy.'
    expect(appended.document.events[1].description).toBe('New milestone.')

    const deleted = deleteStoryEvent(duplicated.document, 2)
    expect(deleted).toMatchObject({ selected_index: 1, changed: true })
    expect(deleted.document.events).toHaveLength(2)
    expect(source.events).toHaveLength(1)
    expect(nextStoryEventId(deleted.document.events, 'new_event')).toBe('new_event_3')
  })
})

describe('Story Event trigger and action editing', () => {
  it('toggles every trigger gate without mutating the source event', () => {
    const source = event({ rule: {} })
    const relationship = setStoryEventGate(source, 'relationship', true)
    const score = setStoryEventGate(relationship, 'score', true)
    const evaluation = setStoryEventGate(score, 'evaluation', true)

    expect(storyEventGateEnabled(evaluation, 'relationship')).toBe(true)
    expect(storyEventGateEnabled(evaluation, 'score')).toBe(true)
    expect(storyEventGateEnabled(evaluation, 'evaluation')).toBe(true)
    expect(evaluation.rule).toEqual({
      min_relationship: 0.5,
      score_metric: 'overall',
      min_score: 0.7,
      min_evaluation_count: 1,
    })
    expect(setStoryEventGate(evaluation, 'score', false).rule).not.toHaveProperty('min_score')
    expect(source.rule).toEqual({})
  })

  it('toggles sorted character scopes immutably', () => {
    const source = event({ character_ids: ['emi'] })
    const added = toggleStoryEventCharacter(source, 'aoi')
    const removed = toggleStoryEventCharacter(added, 'emi')
    expect(added.character_ids).toEqual(['aoi', 'emi'])
    expect(removed.character_ids).toEqual(['aoi'])
    expect(source.character_ids).toEqual(['emi'])
  })

  it('creates, appends, replaces, and removes typed actions immutably', () => {
    const source = event({ actions: [] })
    const scene = createStoryEventAction('unlock_scene', { scene_id: 'garden' })
    const dialogue = createStoryEventAction('unlock_dialogue', { dialogue_id: 'intro' })
    const ending = createStoryEventAction('unlock_ending', { ending_id: 'truth' })
    const flag = createStoryEventAction('set_flag', {})
    expect([scene, dialogue, ending, flag]).toEqual([
      { type: 'unlock_scene', scene_id: 'garden' },
      { type: 'unlock_dialogue', dialogue_id: 'intro' },
      { type: 'unlock_ending', ending_id: 'truth' },
      { type: 'set_flag', flag: 'story.event_complete', value: true },
    ])

    const appended = appendStoryEventAction(source, scene)
    const replaced = replaceStoryEventAction(appended, 0, dialogue)
    const removed = removeStoryEventAction(replaced, 0)
    expect(appended.actions).toEqual([scene])
    expect(replaced.actions).toEqual([dialogue])
    expect(removed.actions).toEqual([])
    expect(source.actions).toEqual([])
  })
})

describe('Story Event metadata and validation', () => {
  it('tracks semantic metadata edits and applies them without structuredClone', () => {
    const source = reactive(event({ data: { route: 'open' } }))
    expect(storyEventMetadataText(source)).toBe('{\n  "route": "open"\n}')
    expect(storyEventMetadataChanged(source, '{ "route": "open" }')).toBe(false)
    expect(storyEventMetadataChanged(source, '{"route":"closed"}')).toBe(true)
    expect(storyEventMetadataChanged(source, '{')).toBe(true)
    expect(parseStoryEventMetadata('[]')).toEqual({ value: null, error: 'invalid_object' })

    const applied = applyStoryEventMetadata(source, '{"route":"closed"}')
    expect(applied).toMatchObject({ changed: true, error: null })
    expect(applied.event.data).toEqual({ route: 'closed' })
    expect(source.data).toEqual({ route: 'open' })
  })

  it('accepts a complete valid document and returns stable warning evidence', () => {
    const source = document([
      event(),
      event({ event_id: 'warning_event', actions: [], rule: {} }),
    ])
    expect(validateStoryEventDocument(source, references)).toEqual([])
    expect(storyEventDocumentWarnings(source)).toEqual([
      { code: 'no_effects', event_id: 'warning_event' },
      { code: 'no_trigger', event_id: 'warning_event' },
    ])
    expect(storyEventDocumentSnapshot(source)).toContain('first_event')
  })

  it('reports identity, scope, threshold, action, and reference failures by code', () => {
    const broken = document([
      event({
        event_id: 'bad/id',
        event_type: 'bad type',
        description: '',
        character_ids: ['missing', 'missing'],
        rule: { min_relationship: 2, score_metric: 'overall', min_score: 2, min_evaluation_count: -1 },
        actions: [
          { type: 'unlock_scene', scene_id: 'missing' },
          { type: 'unlock_scene', scene_id: 'missing' },
          { type: 'unlock_dialogue', dialogue_id: 'missing' },
          { type: 'unlock_ending', ending_id: 'missing' },
          { type: 'set_flag', flag: 'bad/flag', value: true },
        ],
      }),
      event({ event_id: 'bad/id', rule: { score_metric: 'overall' } }),
    ])
    const codes = validateStoryEventDocument(broken, references).map(({ code }) => code)
    expect(codes).toEqual(expect.arrayContaining([
      'invalid_id',
      'duplicate_id',
      'invalid_type',
      'description',
      'duplicate_scope',
      'unknown_character',
      'relationship_threshold',
      'score_pair',
      'score_threshold',
      'evaluation_count',
      'duplicate_action',
      'unknown_scene',
      'unknown_dialogue',
      'unknown_ending',
      'invalid_flag',
    ]))
  })

  it('enforces the frontend catalog count boundary', () => {
    const events = Array.from({ length: 513 }, (_, index) => event({ event_id: `event_${index}` }))
    expect(validateStoryEventDocument(document(events), references)[0]).toEqual({ code: 'catalog_limit' })
  })
})
