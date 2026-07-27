export const BROWSER_PROJECT_DRAFT_SCHEMA = 'monogatari-browser-project-drafts/v1'

export interface BrowserProjectManifestIdentity {
  project_scope?: string
  character_files?: string[]
  scene_files?: string[]
  dialogue_files?: string[]
  roleplay_files?: string[]
  campaign_files?: string[]
  ending_files?: string[]
  knowledge_files?: string[]
  event_catalogs?: string[]
}

let activeProjectScope: string | null = null

export function activateBrowserProjectScope(
  manifest: BrowserProjectManifestIdentity,
): string {
  activeProjectScope = browserProjectScope(manifest)
  return activeProjectScope
}

export function loadScopedBrowserDrafts<T>(
  storageKey: string,
  isEntry: (value: unknown) => value is T,
): T[] | null {
  if (typeof window === 'undefined' || !activeProjectScope) return null
  const raw = window.localStorage.getItem(storageKey)
  if (raw === null) return null
  try {
    const value = JSON.parse(raw) as Record<string, unknown>
    if (value.schema !== BROWSER_PROJECT_DRAFT_SCHEMA
      || value.project_scope !== activeProjectScope
      || !Array.isArray(value.entries)) {
      return null
    }
    const entries = value.entries.filter(isEntry)
    return entries.length === value.entries.length ? entries : null
  } catch {
    return null
  }
}

export function saveScopedBrowserDrafts<T>(storageKey: string, entries: T[]): void {
  if (typeof window === 'undefined') return
  if (!activeProjectScope) {
    throw new Error('Browser project content must be loaded before saving authoring drafts.')
  }
  window.localStorage.setItem(storageKey, JSON.stringify({
    schema: BROWSER_PROJECT_DRAFT_SCHEMA,
    project_scope: activeProjectScope,
    entries,
  }))
}

export function clearScopedBrowserDrafts(storageKey: string): void {
  if (typeof window !== 'undefined') window.localStorage.removeItem(storageKey)
}

function browserProjectScope(manifest: BrowserProjectManifestIdentity): string {
  if (/^project-[a-f0-9]{16,64}$/.test(manifest.project_scope || '')) {
    return manifest.project_scope as string
  }
  const fields = [
    manifest.character_files,
    manifest.scene_files,
    manifest.dialogue_files,
    manifest.roleplay_files,
    manifest.campaign_files,
    manifest.ending_files,
    manifest.knowledge_files,
    manifest.event_catalogs,
  ].map(paths => [...(paths || [])].sort())
  const source = JSON.stringify(fields)
  let hash = 0x811c9dc5
  for (let index = 0; index < source.length; index += 1) {
    hash = Math.imul(hash ^ source.charCodeAt(index), 0x01000193)
  }
  return `catalog-${(hash >>> 0).toString(16).padStart(8, '0')}`
}
