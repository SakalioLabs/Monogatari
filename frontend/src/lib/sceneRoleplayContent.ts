import type { SceneRoleplayDefinition } from './sceneRoleplay'

const BROWSER_ROLEPLAY_DRAFT_KEY = 'monogatari:scene-roleplay-authoring-catalog:v1'

export function loadBrowserRoleplayDrafts(): SceneRoleplayDefinition[] | null {
  if (typeof localStorage === 'undefined') return null
  const stored = localStorage.getItem(BROWSER_ROLEPLAY_DRAFT_KEY)
  if (!stored) return null
  try {
    const parsed = JSON.parse(stored)
    return Array.isArray(parsed) ? parsed as SceneRoleplayDefinition[] : null
  } catch {
    return null
  }
}

export function saveBrowserRoleplayDrafts(definitions: SceneRoleplayDefinition[]): void {
  if (typeof localStorage === 'undefined') {
    throw new Error('Browser roleplay drafts require local storage.')
  }
  localStorage.setItem(BROWSER_ROLEPLAY_DRAFT_KEY, JSON.stringify(definitions))
}

export function clearBrowserRoleplayDrafts(): void {
  if (typeof localStorage !== 'undefined') localStorage.removeItem(BROWSER_ROLEPLAY_DRAFT_KEY)
}

export const sceneRoleplayDraftStorageKey = BROWSER_ROLEPLAY_DRAFT_KEY
