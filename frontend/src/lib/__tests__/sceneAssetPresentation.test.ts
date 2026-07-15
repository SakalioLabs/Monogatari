import { describe, expect, it } from 'vitest'

import {
  activateSceneAssetState,
  addFailedScenePreviewUrl,
  filterSceneAssets,
  formatSceneAssetBytes,
  normalizeActiveSceneAssetState,
  sceneAssetIssueTarget,
  sceneAssetMetrics,
  sceneAssetName,
} from '../sceneAssetPresentation'
import type { SceneAssetCatalog, SceneAssetInfo } from '../sceneAssets'

function scene(overrides: Partial<SceneAssetInfo> = {}): SceneAssetInfo {
  return {
    id: 'studio_night',
    name: 'Studio Night',
    background_path: 'assets/backgrounds/studio_night.svg',
    bgm_path: null,
    weather: null,
    time_of_day: 'night',
    tags: ['studio'],
    source: 'metadata',
    background_exists: true,
    absolute_background_path: null,
    ...overrides,
  }
}

function catalog(): SceneAssetCatalog {
  return {
    project_path: null,
    valid: false,
    error_count: 1,
    warning_count: 2,
    scenes: [
      scene(),
      scene({ id: 'agent_route', name: 'Agent Route', tags: ['delivery'] }),
      scene({ id: 'missing', name: 'Missing', background_exists: false }),
    ],
    backgrounds: [],
    issues: [],
  }
}

describe('scene asset presentation domain', () => {
  it('filters active, missing, and query views independently', () => {
    const scenes = catalog().scenes
    expect(filterSceneAssets(scenes, '', 'active', 'agent_route')).toEqual([scenes[1]])
    expect(filterSceneAssets(scenes, '', 'missing', null)).toEqual([scenes[2]])
    expect(filterSceneAssets(scenes, ' DELIVERY ', 'all', null)).toEqual([scenes[1]])
  })

  it('derives issue and broken-scene metrics', () => {
    expect(sceneAssetMetrics(catalog())).toEqual({ issue_count: 3, broken_scene_count: 1 })
    expect(sceneAssetMetrics(null)).toEqual({ issue_count: 0, broken_scene_count: 0 })
  })

  it('normalizes active state and bounds immutable history', () => {
    const sourceCatalog = catalog()
    const staleScene = scene({ id: 'agent_route', name: 'Stale Name' })
    const normalized = normalizeActiveSceneAssetState({
      scene: staleScene,
      scene_history: ['studio_night'],
    }, sourceCatalog)
    expect(normalized.scene).toBe(sourceCatalog.scenes[1])

    const activated = activateSceneAssetState(normalized, sourceCatalog.scenes[2], 2)
    expect(activated.scene).toBe(sourceCatalog.scenes[2])
    expect(activated.scene_history).toEqual(['studio_night', 'missing'])
    expect(normalized.scene_history).toEqual(['studio_night'])
  })

  it('tracks failed preview URLs without duplication', () => {
    expect(addFailedScenePreviewUrl(['/a.svg'], '/b.svg')).toEqual(['/a.svg', '/b.svg'])
    expect(addFailedScenePreviewUrl(['/a.svg'], '/a.svg')).toEqual(['/a.svg'])
    expect(addFailedScenePreviewUrl(['/a.svg'], null)).toEqual(['/a.svg'])
  })

  it('shapes names, issue targets, and byte labels', () => {
    const scenes = catalog().scenes
    expect(sceneAssetName(scenes, 'agent_route')).toBe('Agent Route')
    expect(sceneAssetName(scenes, 'unknown')).toBe('unknown')
    expect(sceneAssetIssueTarget({ severity: 'error', code: 'x', scene_id: null, path: 'a.svg', message: 'x' }))
      .toBe('a.svg')
    expect(formatSceneAssetBytes(512)).toBe('512 B')
    expect(formatSceneAssetBytes(2048)).toBe('2 KB')
    expect(formatSceneAssetBytes(1.5 * 1024 * 1024)).toBe('1.5 MB')
  })
})
