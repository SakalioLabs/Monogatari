import {
  loadSceneAuthoringCatalog,
  type SceneAuthoringCatalogSnapshot,
} from './sceneAuthoring'
import { hasTauriRuntime, invokeCommand } from './tauri'

export interface SceneAssetInfo {
  id: string
  name: string
  background_path: string | null
  bgm_path: string | null
  weather: string | null
  time_of_day: string | null
  tags: string[]
  source: string
  background_exists: boolean
  absolute_background_path: string | null
}

export interface BackgroundAssetInfo {
  id: string
  file_name: string
  relative_path: string
  absolute_path: string
  extension: string
  file_size: number
  linked_scene_id: string | null
}

export interface SceneAssetCatalogIssue {
  severity: string
  code: string
  scene_id: string | null
  path: string | null
  message: string
}

export interface SceneAssetCatalog {
  project_path: string | null
  valid: boolean
  error_count: number
  warning_count: number
  scenes: SceneAssetInfo[]
  backgrounds: BackgroundAssetInfo[]
  issues: SceneAssetCatalogIssue[]
}

export interface ActiveSceneAssetState {
  scene: SceneAssetInfo | null
  scene_history: string[]
}

const activeSceneStorageKey = 'monogatari.activeScene'

export async function loadSceneAssetCatalog(): Promise<SceneAssetCatalog> {
  if (hasTauriRuntime()) return invokeCommand<SceneAssetCatalog>('list_scene_assets')
  return buildBrowserSceneAssetCatalog(await loadSceneAuthoringCatalog())
}

export async function loadActiveSceneAssetState(
  catalog: SceneAssetCatalog,
): Promise<ActiveSceneAssetState> {
  if (hasTauriRuntime()) return invokeCommand<ActiveSceneAssetState>('get_current_scene')

  const storedValue = window.localStorage.getItem(activeSceneStorageKey)
  const storedId = browserActiveSceneId(storedValue)
  const scene = catalog.scenes.find((item) => item.id === storedId) || catalog.scenes[0] || null
  if ((storedValue && !storedId) || (storedId && scene?.id !== storedId)) {
    window.localStorage.removeItem(activeSceneStorageKey)
  }
  return { scene, scene_history: scene ? [scene.id] : [] }
}

export async function setActiveSceneAsset(scene: SceneAssetInfo): Promise<SceneAssetInfo> {
  if (hasTauriRuntime()) {
    return invokeCommand<SceneAssetInfo>('set_scene', { sceneId: scene.id })
  }
  window.localStorage.setItem(activeSceneStorageKey, JSON.stringify({ id: scene.id }))
  return scene
}

export function buildBrowserSceneAssetCatalog(
  snapshot: SceneAuthoringCatalogSnapshot,
): SceneAssetCatalog {
  const scenes = snapshot.scenes.map((scene): SceneAssetInfo => ({
    id: scene.id,
    name: scene.name,
    background_path: scene.background_path,
    bgm_path: scene.bgm_path,
    weather: scene.weather,
    time_of_day: scene.time_of_day,
    tags: [...scene.tags],
    source: scene.metadata_authored ? 'metadata' : 'background',
    background_exists: scene.background_exists,
    absolute_background_path: scene.absolute_background_path,
  }))
  const backgroundByPath = new Map<string, BackgroundAssetInfo>()
  for (const scene of scenes) {
    const relativePath = scene.background_path
    if (!relativePath || !scene.background_exists || backgroundByPath.has(relativePath)) continue
    const fileName = relativePath.split('/').at(-1) || relativePath
    const extension = fileName.includes('.') ? fileName.split('.').at(-1)?.toLowerCase() || '' : ''
    backgroundByPath.set(relativePath, {
      id: scene.id,
      file_name: fileName,
      relative_path: relativePath,
      absolute_path: scene.absolute_background_path || '',
      extension,
      file_size: 0,
      linked_scene_id: scene.id,
    })
  }
  const issues = snapshot.issues.map((issue) => ({ ...issue }))
  const errorCount = issues.filter((issue) => issue.severity.toLowerCase() === 'error').length
  const warningCount = issues.filter((issue) => issue.severity.toLowerCase() === 'warning').length
  return {
    project_path: null,
    valid: errorCount === 0,
    error_count: errorCount,
    warning_count: warningCount,
    scenes,
    backgrounds: [...backgroundByPath.values()].sort((left, right) => (
      left.relative_path.localeCompare(right.relative_path)
    )),
    issues,
  }
}

function browserActiveSceneId(raw: string | null): string | null {
  if (!raw) return null
  try {
    const value = JSON.parse(raw) as unknown
    if (!value || typeof value !== 'object' || Array.isArray(value)) return null
    const id = (value as Record<string, unknown>).id
    return typeof id === 'string' && id ? id : null
  } catch {
    return null
  }
}
