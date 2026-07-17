import type { StoryContentAccessEntry } from './storyAccess'
import {
  loadBrowserSceneDrafts,
  loadStoryDialogues,
  loadStoryEndings,
  loadStoryScenes,
  saveBrowserSceneDrafts,
  type SceneDefinition,
  type StorySceneInfo,
} from './storyContent'
import { hasTauriRuntime, invokeCommand } from './tauri'

export const SCENE_AUTHORING_CATALOG_SCHEMA = 'monogatari-scene-authoring-catalog/v1'

export interface SceneAssetIssue {
  severity: string
  code: string
  scene_id: string | null
  path: string | null
  message: string
}

export interface SceneAuthoringEntry extends SceneDefinition {
  source_path: string | null
  content_fingerprint: string
  metadata_authored: boolean
  background_exists: boolean
  absolute_background_path: string | null
  model_3d_exists?: boolean
  absolute_model_3d_path?: string | null
  access: StoryContentAccessEntry
}

export interface SceneAuthoringCatalogSnapshot {
  schema: string
  catalog_fingerprint: string
  scene_count: number
  metadata_scene_count: number
  inferred_scene_count: number
  scenes: SceneAuthoringEntry[]
  issues: SceneAssetIssue[]
}

const PORTABLE_ID = /^[A-Za-z0-9_.-]{1,128}$/
const BACKGROUND_EXTENSION = /\.(png|jpe?g|webp|bmp|gif|svg)$/i
const MODEL_3D_EXTENSION = /\.(glb|gltf)$/i
const AUDIO_EXTENSION = /\.(mp3|ogg|wav|m4a|aac|flac)$/i

export async function loadSceneAuthoringCatalog(): Promise<SceneAuthoringCatalogSnapshot> {
  if (hasTauriRuntime()) {
    return invokeCommand<SceneAuthoringCatalogSnapshot>('get_scene_authoring_catalog')
  }
  return browserCatalogSnapshot()
}

export async function saveSceneDefinition(
  scene: SceneDefinition,
  originalSceneId: string | null,
  expectedCatalogFingerprint: string,
): Promise<SceneAuthoringCatalogSnapshot> {
  const normalized = normalizeSceneDefinition(scene)
  const issues = validateSceneDefinition(normalized)
  if (issues.length > 0) throw new Error(issues[0])
  if (hasTauriRuntime()) {
    return invokeCommand<SceneAuthoringCatalogSnapshot>('save_scene_definition', {
      scene: normalized,
      originalSceneId,
      expectedCatalogFingerprint,
    })
  }

  const current = await browserCatalogSnapshot()
  ensureExpectedFingerprint(current, expectedCatalogFingerprint)
  const definitions = current.scenes.map(sceneDefinition)
  const existingIndex = definitions.findIndex((item) => item.id === normalized.id)
  if (originalSceneId) {
    if (originalSceneId !== normalized.id) {
      throw new Error('Scene ids are immutable after creation. Duplicate the scene to use a new id.')
    }
    if (existingIndex < 0) throw new Error(`Scene "${originalSceneId}" no longer exists. Reload first.`)
    definitions.splice(existingIndex, 1, normalized)
  } else {
    if (hasSceneIdCollision(definitions.map((item) => item.id), normalized.id)) {
      throw new Error(`Scene "${normalized.id}" already exists.`)
    }
    definitions.push(normalized)
  }
  definitions.sort((left, right) => left.id.localeCompare(right.id))
  saveBrowserSceneDrafts(definitions)
  return browserCatalogSnapshot()
}

export async function deleteSceneDefinition(
  sceneId: string,
  expectedCatalogFingerprint: string,
): Promise<SceneAuthoringCatalogSnapshot> {
  if (hasTauriRuntime()) {
    return invokeCommand<SceneAuthoringCatalogSnapshot>('delete_scene_definition', {
      sceneId,
      expectedCatalogFingerprint,
    })
  }

  const current = await browserCatalogSnapshot()
  ensureExpectedFingerprint(current, expectedCatalogFingerprint)
  const target = current.scenes.find((scene) => scene.id === sceneId)
  if (!target) throw new Error(`Scene "${sceneId}" does not exist.`)
  const references = target.access.unlock_event_ids.map((eventId) => `event:${eventId}`)
  const [dialogues, endings] = await Promise.all([loadStoryDialogues(), loadStoryEndings()])
  references.push(...sceneDialogueReferences(dialogues, sceneId))
  references.push(...endings.filter((ending) => ending.scene_id === sceneId).map((ending) => `ending:${ending.id}`))
  if (references.length > 0) {
    throw new Error(`Scene "${sceneId}" is still referenced by: ${references.join(', ')}.`)
  }
  saveBrowserSceneDrafts(current.scenes.filter((scene) => scene.id !== sceneId).map(sceneDefinition))
  return browserCatalogSnapshot()
}

export function sceneDialogueReferences(
  dialogues: readonly { id: string; nodes?: Record<string, { scene_id?: string | null }> }[],
  sceneId: string,
): string[] {
  const references = dialogues.flatMap((dialogue) => Object.entries(dialogue.nodes || {})
    .filter(([, node]) => node.scene_id === sceneId)
    .map(([nodeId]) => `dialogue:${dialogue.id}/${nodeId}`))
  return [...new Set(references)].sort()
}

export function normalizeSceneDefinition(scene: SceneDefinition): SceneDefinition {
  return {
    id: scene.id.trim(),
    name: scene.name.trim(),
    background_path: optionalText(scene.background_path),
    model_3d_path: optionalText(scene.model_3d_path),
    bgm_path: optionalText(scene.bgm_path),
    weather: optionalText(scene.weather),
    time_of_day: optionalText(scene.time_of_day),
    tags: [...new Set(scene.tags.map((tag) => tag.trim()).filter(Boolean))].sort(),
  }
}

export function validateSceneDefinition(scene: SceneDefinition): string[] {
  const issues: string[] = []
  if (!PORTABLE_ID.test(scene.id) || scene.id.trim() !== scene.id) {
    issues.push('Scene ID must be a portable 1-128 character id.')
  }
  validateText(issues, 'Name', scene.name, 1, 256)
  if (scene.weather !== null) validateText(issues, 'Weather', scene.weather, 1, 64)
  if (scene.time_of_day !== null) validateText(issues, 'Time of day', scene.time_of_day, 1, 64)
  if (scene.tags.length > 64) issues.push('A scene can contain at most 64 tags.')
  scene.tags.forEach((tag) => validateText(issues, 'Tag', tag, 1, 64))
  if (scene.background_path !== null) {
    if (!isPortableProjectPath(scene.background_path)) {
      issues.push('Background path must use portable project-relative segments.')
    } else if (!BACKGROUND_EXTENSION.test(scene.background_path)) {
      issues.push('Background path must use a supported image extension.')
    }
  }
  if (scene.model_3d_path !== null && scene.model_3d_path !== undefined) {
    if (!isPortableProjectPath(scene.model_3d_path)) {
      issues.push('3D model path must use portable project-relative segments.')
    } else if (!MODEL_3D_EXTENSION.test(scene.model_3d_path)) {
      issues.push('3D model path must use .glb or .gltf.')
    }
  }
  if (scene.bgm_path !== null) {
    if (!isPortableProjectPath(scene.bgm_path)) {
      issues.push('BGM path must use portable project-relative segments.')
    } else if (!AUDIO_EXTENSION.test(scene.bgm_path)) {
      issues.push('BGM path must use a supported audio extension.')
    }
  }
  return [...new Set(issues)]
}

export function hasSceneIdCollision(
  existingIds: readonly string[],
  candidateId: string,
): boolean {
  const portableKey = candidateId.trim().toLowerCase()
  return Boolean(portableKey)
    && existingIds.some((id) => id.trim().toLowerCase() === portableKey)
}

async function browserCatalogSnapshot(): Promise<SceneAuthoringCatalogSnapshot> {
  const draftActive = loadBrowserSceneDrafts() !== null
  const scenes = await loadStoryScenes()
  const entries = scenes.map((scene) => authoringEntry(scene, draftActive))
    .sort((left, right) => left.id.localeCompare(right.id))
  return {
    schema: SCENE_AUTHORING_CATALOG_SCHEMA,
    catalog_fingerprint: browserFingerprint(entries.map((scene) => ({
      source_path: scene.source_path,
      metadata_authored: scene.metadata_authored,
      scene: sceneDefinition(scene),
    }))),
    scene_count: entries.length,
    metadata_scene_count: entries.length,
    inferred_scene_count: 0,
    scenes: entries,
    issues: [],
  }
}

function authoringEntry(scene: StorySceneInfo, draftActive: boolean): SceneAuthoringEntry {
  const definition = sceneDefinition(scene)
  return {
    ...definition,
    source_path: `${draftActive ? 'browser-draft/' : ''}scenes/${scene.id}.json`,
    content_fingerprint: browserFingerprint(definition),
    metadata_authored: true,
    background_exists: scene.background_exists,
    absolute_background_path: scene.absolute_background_path,
    model_3d_exists: Boolean(scene.model_3d_exists),
    absolute_model_3d_path: scene.absolute_model_3d_path || null,
    access: scene.access,
  }
}

function sceneDefinition(scene: SceneDefinition): SceneDefinition {
  return normalizeSceneDefinition(scene)
}

function optionalText(value: string | null | undefined): string | null {
  const normalized = value?.trim() || ''
  return normalized || null
}

function validateText(issues: string[], label: string, value: string, min: number, max: number): void {
  const length = [...value].length
  if (length < min || length > max || /[\u0000-\u001f\u007f]/.test(value)) {
    issues.push(`${label} must contain ${min}-${max} non-control characters.`)
  }
}

function isPortableProjectPath(value: string): boolean {
  return value.trim() === value
    && value.length > 0
    && !value.includes('\\')
    && !value.includes(':')
    && !/[\u0000-\u001f\u007f]/.test(value)
    && value.split('/').every((segment) => segment.length > 0
      && segment !== '.'
      && segment !== '..'
      && /^[A-Za-z0-9._-]+$/.test(segment))
}

function ensureExpectedFingerprint(current: SceneAuthoringCatalogSnapshot, expected: string): void {
  if (current.catalog_fingerprint !== expected) {
    throw new Error('Scene catalog changed since it was opened. Reload before saving.')
  }
}

function browserFingerprint(value: unknown): string {
  const text = JSON.stringify(value)
  let left = 0x811c9dc5
  let right = 0x9e3779b9
  for (let index = 0; index < text.length; index += 1) {
    const code = text.charCodeAt(index)
    left = Math.imul(left ^ code, 0x01000193) >>> 0
    right = Math.imul(right ^ (code + index), 0x85ebca6b) >>> 0
  }
  return `browser-${left.toString(16).padStart(8, '0')}${right.toString(16).padStart(8, '0')}`
}
