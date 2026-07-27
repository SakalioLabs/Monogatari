import type { SceneRoleplayDefinition } from './sceneRoleplay'
import {
  clearScopedBrowserDrafts,
  loadScopedBrowserDrafts,
  saveScopedBrowserDrafts,
} from './browserProjectDrafts'

const BROWSER_ROLEPLAY_DRAFT_KEY = 'monogatari:scene-roleplay-authoring-catalog:v1'

export function loadBrowserRoleplayDrafts(): SceneRoleplayDefinition[] | null {
  return loadScopedBrowserDrafts(BROWSER_ROLEPLAY_DRAFT_KEY, isSceneRoleplayDefinition)
}

export function saveBrowserRoleplayDrafts(definitions: SceneRoleplayDefinition[]): void {
  saveScopedBrowserDrafts(BROWSER_ROLEPLAY_DRAFT_KEY, definitions)
}

export function clearBrowserRoleplayDrafts(): void {
  clearScopedBrowserDrafts(BROWSER_ROLEPLAY_DRAFT_KEY)
}

export const sceneRoleplayDraftStorageKey = BROWSER_ROLEPLAY_DRAFT_KEY

function isSceneRoleplayDefinition(value: unknown): value is SceneRoleplayDefinition {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return false
  const definition = value as Record<string, unknown>
  return definition.schema === 'monogatari-scene-roleplay/v1'
    && typeof definition.id === 'string'
    && typeof definition.title === 'string'
    && Array.isArray(definition.nodes)
}
