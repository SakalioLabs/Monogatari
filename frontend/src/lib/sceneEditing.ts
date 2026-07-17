import {
  hasSceneIdCollision,
  type SceneAssetIssue,
  type SceneAuthoringEntry,
} from './sceneAuthoring'
import type { SceneDefinition } from './storyContent'

export type SceneSourceFilter = 'all' | 'authored' | 'inferred'

export type SceneDraftWarningCode =
  | 'no_visual'
  | 'unresolved_background'
  | 'unresolved_model_3d'

export interface SceneDraftWarning {
  code: SceneDraftWarningCode
}

export function cloneSceneDefinition(scene: SceneDefinition): SceneDefinition {
  return {
    id: scene.id,
    name: scene.name,
    background_path: scene.background_path,
    model_3d_path: scene.model_3d_path || null,
    bgm_path: scene.bgm_path,
    weather: scene.weather,
    time_of_day: scene.time_of_day,
    tags: [...scene.tags],
  }
}

export function sceneDefinitionFromEntry(
  entry: SceneAuthoringEntry,
): SceneDefinition {
  return cloneSceneDefinition(entry)
}

export function sceneDraftSnapshot(
  scene: SceneDefinition | null | undefined,
): string {
  return scene ? JSON.stringify(scene) : ''
}

export function filterSceneAuthoringEntries(
  scenes: readonly SceneAuthoringEntry[],
  query: string,
  sourceFilter: SceneSourceFilter,
): SceneAuthoringEntry[] {
  const needle = query.trim().toLowerCase()
  return scenes.filter((scene) => {
    const sourceMatches = sourceFilter === 'all'
      || (sourceFilter === 'authored' && scene.metadata_authored)
      || (sourceFilter === 'inferred' && !scene.metadata_authored)
    return sourceMatches && (!needle
      || scene.id.toLowerCase().includes(needle)
      || scene.name.toLowerCase().includes(needle)
      || scene.tags.some((tag) => tag.toLowerCase().includes(needle)))
  })
}

export function nextSceneId(
  existingIds: readonly string[],
  base = 'new_scene',
): string {
  const normalizedBase = base.trim() || 'new_scene'
  if (!hasSceneIdCollision(existingIds, normalizedBase)) return normalizedBase
  let index = 2
  while (hasSceneIdCollision(existingIds, `${normalizedBase}_${index}`)) index += 1
  return `${normalizedBase}_${index}`
}

export function createSceneDraft(
  existingIds: readonly string[],
  name: string,
): SceneDefinition {
  return {
    id: nextSceneId(existingIds),
    name,
    background_path: null,
    model_3d_path: null,
    bgm_path: null,
    weather: null,
    time_of_day: null,
    tags: [],
  }
}

export function duplicateSceneDraft(
  source: SceneDefinition,
  existingIds: readonly string[],
  name: string,
): SceneDefinition {
  return {
    ...cloneSceneDefinition(source),
    id: nextSceneId(existingIds, `${source.id}_copy`),
    name,
  }
}

export function parseSceneTags(value: string): string[] {
  return [...new Set(value.split(',').map((tag) => tag.trim()).filter(Boolean))]
}

export function sceneDraftWarnings(
  draft: SceneDefinition | null | undefined,
  selectedEntry: SceneAuthoringEntry | null | undefined,
): SceneDraftWarning[] {
  if (!draft) return []
  const warnings: SceneDraftWarning[] = []
  if (!draft.background_path?.trim() && !draft.model_3d_path?.trim()) warnings.push({ code: 'no_visual' })
  if (selectedEntry?.background_path === draft.background_path && !selectedEntry.background_exists) {
    warnings.push({ code: 'unresolved_background' })
  }
  if (draft.model_3d_path?.trim()
    && selectedEntry?.model_3d_path === draft.model_3d_path
    && !selectedEntry?.model_3d_exists) {
    warnings.push({ code: 'unresolved_model_3d' })
  }
  return warnings
}

export function relevantSceneAssetIssues(
  issues: readonly SceneAssetIssue[],
  selectedSceneId: string | null,
): SceneAssetIssue[] {
  return issues.filter((issue) => !issue.scene_id || issue.scene_id === selectedSceneId)
}
