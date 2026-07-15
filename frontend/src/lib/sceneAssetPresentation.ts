import type {
  ActiveSceneAssetState,
  SceneAssetCatalog,
  SceneAssetCatalogIssue,
  SceneAssetInfo,
} from './sceneAssets'

export type SceneAssetFilter = 'all' | 'active' | 'missing'

export interface SceneAssetMetrics {
  issue_count: number
  broken_scene_count: number
}

export function sceneAssetMetrics(catalog: SceneAssetCatalog | null | undefined): SceneAssetMetrics {
  const scenes = catalog?.scenes || []
  return {
    issue_count: (catalog?.error_count || 0) + (catalog?.warning_count || 0),
    broken_scene_count: scenes.filter((scene) => Boolean(
      scene.background_path && !scene.background_exists,
    )).length,
  }
}

export function filterSceneAssets(
  scenes: readonly SceneAssetInfo[],
  query: string,
  filter: SceneAssetFilter,
  activeSceneId: string | null,
): SceneAssetInfo[] {
  const needle = query.trim().toLowerCase()
  return scenes.filter((scene) => {
    if (filter === 'active' && activeSceneId !== scene.id) return false
    if (filter === 'missing' && (!scene.background_path || scene.background_exists)) return false
    return !needle
      || scene.id.toLowerCase().includes(needle)
      || scene.name.toLowerCase().includes(needle)
      || (scene.background_path || '').toLowerCase().includes(needle)
      || scene.tags.some((tag) => tag.toLowerCase().includes(needle))
  })
}

export function normalizeActiveSceneAssetState(
  state: ActiveSceneAssetState,
  catalog: SceneAssetCatalog,
): ActiveSceneAssetState {
  const scene = state.scene
    ? catalog.scenes.find((item) => item.id === state.scene?.id) || state.scene
    : null
  return { scene, scene_history: [...(state.scene_history || [])] }
}

export function activateSceneAssetState(
  state: ActiveSceneAssetState | null | undefined,
  scene: SceneAssetInfo,
  historyLimit = 24,
): ActiveSceneAssetState {
  return {
    scene,
    scene_history: [...(state?.scene_history || []), scene.id].slice(-historyLimit),
  }
}

export function addFailedScenePreviewUrl(
  failedUrls: readonly string[],
  url: string | null | undefined,
): string[] {
  return url && !failedUrls.includes(url) ? [...failedUrls, url] : [...failedUrls]
}

export function sceneAssetName(
  scenes: readonly SceneAssetInfo[],
  id: string,
): string {
  return scenes.find((scene) => scene.id === id)?.name || id
}

export function sceneAssetIssueTarget(issue: SceneAssetCatalogIssue): string | null {
  return issue.scene_id || issue.path || null
}

export function formatSceneAssetBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}
