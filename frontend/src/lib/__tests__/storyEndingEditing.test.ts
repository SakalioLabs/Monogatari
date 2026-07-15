import { describe, expect, it } from 'vitest'

import type { StoryEndingDefinition } from '../storyContent'
import type { StoryEndingAuthoringEntry } from '../storyEndings'
import {
  cloneStoryEndingDefinition,
  createStoryEndingDraft,
  duplicateStoryEndingDraft,
  filterStoryEndingEntries,
  nextStoryEndingId,
  storyEndingCoverageWarnings,
  storyEndingDefinitionFromEntry,
  storyEndingDraftSnapshot,
  validateStoryEndingReferences,
} from '../storyEndingEditing'
import {
  hasStoryEndingIdCollision,
  normalizeStoryEndingDefinition,
  validateStoryEndingDefinition,
} from '../storyEndings'

function ending(overrides: Partial<StoryEndingDefinition> = {}): StoryEndingDefinition {
  return {
    schema: 'monogatari-story-ending/v1',
    id: 'truth_route',
    title: 'Truth Route',
    description: 'The complete route.',
    scene_id: 'station',
    dialogue_id: 'truth_dialogue',
    ...overrides,
  }
}

function entry(overrides: Partial<StoryEndingAuthoringEntry> = {}): StoryEndingAuthoringEntry {
  return {
    ...ending(),
    source_path: 'endings/truth_route.json',
    content_fingerprint: 'sha256',
    access: {
      content_type: 'ending',
      content_id: 'truth_route',
      gated: true,
      unlocked: false,
      unlock_event_ids: ['truth_unlocked'],
    },
    ...overrides,
  }
}

describe('Story Ending draft editing', () => {
  it('clones authoring entries without retaining metadata or mutable identity', () => {
    const source = entry()
    const definition = storyEndingDefinitionFromEntry(source)
    const cloned = cloneStoryEndingDefinition(definition)
    cloned.title = 'Changed'

    expect(source.title).toBe('Truth Route')
    expect(definition).not.toHaveProperty('source_path')
    expect(definition).not.toHaveProperty('access')
    expect(storyEndingDraftSnapshot(definition)).toContain('truth_route')
  })

  it('filters trimmed text and allocates IDs with portable case-folded collisions', () => {
    const endings = [entry(), entry({ id: 'quiet_route', title: 'Quiet Route', description: 'A silent ending.' })]
    expect(filterStoryEndingEntries(endings, '  SILENT ')).toEqual([endings[1]])
    expect(hasStoryEndingIdCollision(['Finale'], 'finale')).toBe(true)
    expect(nextStoryEndingId(['New_Ending', 'new_ending_2'])).toBe('new_ending_3')
  })

  it('creates and duplicates isolated drafts with real reference defaults', () => {
    const created = createStoryEndingDraft(
      ['new_ending'],
      'New Ending',
      'A conclusion.',
      'station',
      'intro',
    )
    expect(created).toMatchObject({ id: 'new_ending_2', scene_id: 'station', dialogue_id: 'intro' })

    const source = ending()
    const copy = duplicateStoryEndingDraft(source, ['truth_route_copy'], 'Truth Route Copy')
    copy.description = 'Changed'
    expect(copy.id).toBe('truth_route_copy_2')
    expect(source.description).toBe('The complete route.')
  })

  it('normalizes transport payloads and validates bounded definitions', () => {
    expect(normalizeStoryEndingDefinition(ending({
      id: ' truth_route ',
      title: ' Truth Route ',
      description: ' Complete. ',
      scene_id: ' station ',
      dialogue_id: ' intro ',
    }))).toEqual(ending({ dialogue_id: 'intro', description: 'Complete.' }))
    expect(validateStoryEndingDefinition(ending())).toEqual([])
    expect(validateStoryEndingDefinition(ending({ id: '../bad', title: ' ' }))).toEqual(expect.arrayContaining([
      'Ending ID must be a portable 1-128 character id.',
      'Title must contain 1-256 characters.',
    ]))
  })
})

describe('Story Ending project references', () => {
  it('returns stable missing-reference and case-folded collision evidence', () => {
    expect(validateStoryEndingReferences(ending({
      id: 'TRUTH_ROUTE',
      scene_id: 'missing_scene',
      dialogue_id: 'missing_dialogue',
    }), {
      original_ending_id: null,
      existing_ending_ids: ['truth_route'],
      scene_ids: ['station'],
      dialogue_ids: ['truth_dialogue'],
    })).toEqual([
      { code: 'scene_missing', target_id: 'missing_scene' },
      { code: 'dialogue_missing', target_id: 'missing_dialogue' },
      { code: 'id_collision', target_id: 'TRUTH_ROUTE' },
    ])
  })

  it('reports unlock coverage gaps for persisted routes only', () => {
    const access = (ids: string[]) => ({
      content_type: 'scene' as const,
      content_id: 'content',
      gated: ids.length > 0,
      unlocked: false,
      unlock_event_ids: ids,
    })
    expect(storyEndingCoverageWarnings({
      original_ending_id: 'truth_route',
      ending_unlock_event_ids: ['truth_unlocked'],
      scene_access: access([]),
      dialogue_access: { ...access(['truth_unlocked']), content_type: 'dialogue' },
    })).toEqual([{ code: 'scene_not_unlocked', event_id: 'truth_unlocked' }])
    expect(storyEndingCoverageWarnings({
      original_ending_id: 'open_route',
      ending_unlock_event_ids: [],
      scene_access: null,
      dialogue_access: null,
    })).toEqual([{ code: 'no_unlock' }])
    expect(storyEndingCoverageWarnings({
      original_ending_id: null,
      ending_unlock_event_ids: [],
      scene_access: null,
      dialogue_access: null,
    })).toEqual([])
  })
})
