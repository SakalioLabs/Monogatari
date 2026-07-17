import { reactive } from 'vue'
import { describe, expect, it } from 'vitest'

import {
  cloneSceneDefinition,
  createSceneDraft,
  duplicateSceneDraft,
  filterSceneAuthoringEntries,
  nextSceneId,
  parseSceneTags,
  relevantSceneAssetIssues,
  sceneDefinitionFromEntry,
  sceneDraftSnapshot,
  sceneDraftWarnings,
} from '../sceneEditing'
import type { SceneAssetIssue, SceneAuthoringEntry } from '../sceneAuthoring'
import type { SceneDefinition } from '../storyContent'

function definition(overrides: Partial<SceneDefinition> = {}): SceneDefinition {
  return {
    id: 'studio_night',
    name: 'Studio Night',
    background_path: 'assets/backgrounds/studio_night.svg',
    model_3d_path: null,
    bgm_path: 'assets/audio/studio.ogg',
    weather: null,
    time_of_day: 'night',
    tags: ['studio'],
    ...overrides,
  }
}

function entry(overrides: Partial<SceneAuthoringEntry> = {}): SceneAuthoringEntry {
  return {
    ...definition(),
    source_path: 'scenes/studio_night.json',
    content_fingerprint: 'scene-sha',
    metadata_authored: true,
    background_exists: true,
    absolute_background_path: null,
    model_3d_exists: false,
    absolute_model_3d_path: null,
    access: {
      content_type: 'scene',
      content_id: 'studio_night',
      gated: false,
      unlocked: true,
      unlock_event_ids: [],
    },
    ...overrides,
  }
}

describe('scene editing domain', () => {
  it('clones reactive definitions without retaining tag references', () => {
    const source = reactive(definition())
    const draft = cloneSceneDefinition(source)
    draft.tags.push('changed')
    expect(source.tags).toEqual(['studio'])

    const fromEntry = sceneDefinitionFromEntry(reactive(entry()))
    expect(fromEntry).toEqual(definition())
    expect(fromEntry).not.toHaveProperty('source_path')
  })

  it('tracks canonical draft snapshots', () => {
    const draft = definition()
    const baseline = sceneDraftSnapshot(draft)
    draft.name = 'Changed'
    expect(sceneDraftSnapshot(draft)).not.toBe(baseline)
    expect(sceneDraftSnapshot(null)).toBe('')
  })

  it('filters source and trimmed query evidence', () => {
    const scenes = [
      entry(),
      entry({ id: 'agent_route', name: 'Agent Route', tags: ['delivery'], metadata_authored: false }),
    ]
    expect(filterSceneAuthoringEntries(scenes, ' DELIVERY ', 'inferred')).toEqual([scenes[1]])
    expect(filterSceneAuthoringEntries(scenes, 'studio', 'authored')).toEqual([scenes[0]])
    expect(filterSceneAuthoringEntries(scenes, '', 'all')).toEqual(scenes)
  })

  it('allocates IDs with portable case-folded collisions', () => {
    expect(nextSceneId(['new_scene', 'NEW_SCENE_2'])).toBe('new_scene_3')
    expect(nextSceneId(['agent_route'], 'AGENT_ROUTE')).toBe('AGENT_ROUTE_2')
    expect(nextSceneId([], 'agent_route')).toBe('agent_route')
  })

  it('creates and duplicates isolated drafts', () => {
    const created = createSceneDraft(['NEW_SCENE'], 'New Scene')
    expect(created).toEqual({
      id: 'new_scene_2',
      name: 'New Scene',
      background_path: null,
      model_3d_path: null,
      bgm_path: null,
      weather: null,
      time_of_day: null,
      tags: [],
    })

    const source = definition()
    const duplicate = duplicateSceneDraft(source, ['STUDIO_NIGHT_COPY'], 'Studio Night Copy')
    duplicate.tags.push('copy')
    expect(duplicate.id).toBe('studio_night_copy_2')
    expect(source.tags).toEqual(['studio'])
  })

  it('parses stable tag lists', () => {
    expect(parseSceneTags(' outdoor, calm, outdoor, , route-a '))
      .toEqual(['outdoor', 'calm', 'route-a'])
  })

  it('returns stable background warnings and relevant diagnostics', () => {
    expect(sceneDraftWarnings(definition({ background_path: null }), null))
      .toEqual([{ code: 'no_visual' }])
    expect(sceneDraftWarnings(definition(), entry({ background_exists: false })))
      .toEqual([{ code: 'unresolved_background' }])

    const issues: SceneAssetIssue[] = [
      { severity: 'error', code: 'global', scene_id: null, path: null, message: 'Global' },
      { severity: 'warning', code: 'selected', scene_id: 'studio_night', path: null, message: 'Selected' },
      { severity: 'warning', code: 'other', scene_id: 'other', path: null, message: 'Other' },
    ]
    expect(relevantSceneAssetIssues(issues, 'studio_night')).toEqual(issues.slice(0, 2))
  })
})
