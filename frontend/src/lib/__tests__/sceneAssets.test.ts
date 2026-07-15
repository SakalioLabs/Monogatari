import { beforeEach, describe, expect, it } from 'vitest'

import {
  buildBrowserSceneAssetCatalog,
  loadActiveSceneAssetState,
  setActiveSceneAsset,
} from '../sceneAssets'
import type { SceneAuthoringCatalogSnapshot, SceneAuthoringEntry } from '../sceneAuthoring'

function scene(overrides: Partial<SceneAuthoringEntry> = {}): SceneAuthoringEntry {
  return {
    id: 'studio_night',
    name: 'Studio Night',
    background_path: 'assets/backgrounds/studio_night.svg',
    bgm_path: null,
    weather: null,
    time_of_day: 'night',
    tags: ['studio'],
    source_path: 'scenes/studio_night.json',
    content_fingerprint: 'scene-sha',
    metadata_authored: true,
    background_exists: true,
    absolute_background_path: null,
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

function snapshot(): SceneAuthoringCatalogSnapshot {
  return {
    schema: 'monogatari-scene-authoring-catalog/v1',
    catalog_fingerprint: 'catalog-sha',
    scene_count: 3,
    metadata_scene_count: 2,
    inferred_scene_count: 1,
    scenes: [
      scene(),
      scene({ id: 'shared_stage', name: 'Shared Stage', source_path: null, metadata_authored: false }),
      scene({
        id: 'missing_stage',
        name: 'Missing Stage',
        background_path: 'assets/backgrounds/missing.svg',
        background_exists: false,
      }),
    ],
    issues: [
      { severity: 'error', code: 'missing', scene_id: 'missing_stage', path: 'assets/backgrounds/missing.svg', message: 'Missing' },
      { severity: 'warning', code: 'unused', scene_id: null, path: null, message: 'Unused' },
    ],
  }
}

describe('scene asset catalog transport', () => {
  beforeEach(() => window.localStorage.clear())

  it('builds the browser catalog from real authoring entries without sample duplication', () => {
    const catalog = buildBrowserSceneAssetCatalog(snapshot())

    expect(catalog.scenes).toHaveLength(3)
    expect(catalog.scenes.map((item) => item.source)).toEqual(['metadata', 'background', 'metadata'])
    expect(catalog.backgrounds).toEqual([{
      id: 'studio_night',
      file_name: 'studio_night.svg',
      relative_path: 'assets/backgrounds/studio_night.svg',
      absolute_path: '',
      extension: 'svg',
      file_size: 0,
      linked_scene_id: 'studio_night',
    }])
    expect(catalog).toMatchObject({ valid: false, error_count: 1, warning_count: 1 })
  })

  it('loads and persists browser active Scene identity through the shared catalog', async () => {
    const catalog = buildBrowserSceneAssetCatalog(snapshot())
    expect((await loadActiveSceneAssetState(catalog)).scene?.id).toBe('studio_night')

    await setActiveSceneAsset(catalog.scenes[1])
    expect(JSON.parse(window.localStorage.getItem('monogatari.activeScene') || 'null'))
      .toEqual({ id: 'shared_stage' })
    expect((await loadActiveSceneAssetState(catalog)).scene?.id).toBe('shared_stage')
  })

  it('clears stale browser active Scene identities', async () => {
    const catalog = buildBrowserSceneAssetCatalog(snapshot())
    window.localStorage.setItem('monogatari.activeScene', JSON.stringify({ id: 'removed_scene' }))

    expect((await loadActiveSceneAssetState(catalog)).scene?.id).toBe('studio_night')
    expect(window.localStorage.getItem('monogatari.activeScene')).toBeNull()

    window.localStorage.setItem('monogatari.activeScene', '{broken')
    expect((await loadActiveSceneAssetState(catalog)).scene?.id).toBe('studio_night')
    expect(window.localStorage.getItem('monogatari.activeScene')).toBeNull()
  })
})
