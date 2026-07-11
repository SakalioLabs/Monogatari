import { describe, expect, it } from 'vitest'

import {
  contentAccess,
  deriveStoryContentAccess,
  type StoryContentAccessSnapshot,
} from '../storyAccess'
import type { StoryProgressSnapshot } from '../storyProgress'

function progress(overrides: Partial<StoryProgressSnapshot> = {}): StoryProgressSnapshot {
  return {
    schema: 'monogatari-story-progress/v1',
    catalog_fingerprint: 'catalog',
    progress_fingerprint: 'progress',
    applied_event_count: 0,
    total_application_count: 0,
    applied_events: [],
    unlocked_scene_ids: [],
    unlocked_dialogue_ids: [],
    unlocked_ending_ids: [],
    ...overrides,
  }
}

describe('story access derivation', () => {
  it('deduplicates unlock sources and reports stable lock counts', () => {
    const snapshot = deriveStoryContentAccess('catalog', [
      { event_id: 'event_b', actions: [{ type: 'unlock_scene', scene_id: 'studio' }] },
      { event_id: 'event_a', actions: [
        { type: 'unlock_scene', scene_id: 'studio' },
        { type: 'unlock_ending', ending_id: 'finale' },
      ] },
    ], progress({
      unlocked_scene_ids: ['studio'],
      unlocked_dialogue_ids: ['always_open'],
    }))

    expect(snapshot).toMatchObject({
      gated_content_count: 2,
      unlocked_gated_content_count: 1,
      locked_content_count: 1,
    })
    expect(contentAccess(snapshot, 'scene', 'studio')).toEqual({
      content_type: 'scene',
      content_id: 'studio',
      gated: true,
      unlocked: true,
      unlock_event_ids: ['event_a', 'event_b'],
    })
    expect(contentAccess(snapshot, 'dialogue', 'always_open').gated).toBe(false)
  })

  it('treats content absent from the gate catalog as backward-compatible open content', () => {
    const snapshot: StoryContentAccessSnapshot = {
      schema: 'monogatari-story-content-access/v1',
      catalog_fingerprint: '',
      progress_fingerprint: '',
      gated_content_count: 0,
      unlocked_gated_content_count: 0,
      locked_content_count: 0,
      entries: [],
    }
    expect(contentAccess(snapshot, 'dialogue', 'legacy')).toMatchObject({
      gated: false,
      unlocked: true,
    })
  })
})
