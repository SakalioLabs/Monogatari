import { loadStoryCharacters } from './storyContent'
import { hasTauriRuntime, invokeCommand } from './tauri'

export const KNOWLEDGE_AUTHORING_SCHEMA_V1 = 'monogatari-knowledge-authoring/v1'

export interface KnowledgeEntryDefinition {
  id: string
  category: string
  title: string
  content: string
  tags: string[]
  importance: number
  metadata: Record<string, unknown>
  related_entries: string[]
}

export interface KnowledgeCatalogSnapshot {
  schema: string
  catalog_fingerprint: string
  entries: KnowledgeEntryDefinition[]
  browser_draft?: boolean
}

interface WebProjectManifest {
  schema: string
  knowledge_files?: string[]
}

const browserDraftKey = 'monogatari:knowledge-authoring-catalog:v1'

export async function loadKnowledgeAuthoringCatalog(): Promise<KnowledgeCatalogSnapshot> {
  if (hasTauriRuntime()) {
    return invokeCommand<KnowledgeCatalogSnapshot>('get_knowledge_authoring_catalog')
  }

  const browserDrafts = loadBrowserDrafts()
  if (browserDrafts) return snapshot(browserDrafts, true)

  const manifest = await fetchJson<WebProjectManifest>(projectUrl('project-assets.json'))
  if (manifest.schema !== 'monogatari-web-project-assets/v1') {
    throw new Error(`Unsupported project content manifest: ${String(manifest.schema)}`)
  }
  const documents = await Promise.all((manifest.knowledge_files || []).map(file => (
    fetchJson<unknown>(projectUrl(file))
  )))
  const entries = documents.flatMap(document => normalizeKnowledgeDocument(document))
  return snapshot(uniqueEntries(entries))
}

export async function saveKnowledgeEntryDefinition(
  entry: KnowledgeEntryDefinition,
  originalEntryId: string | null,
  expectedCatalogFingerprint: string,
): Promise<KnowledgeCatalogSnapshot> {
  const normalized = normalizeKnowledgeEntry(entry)
  validateKnowledgeEntry(normalized)
  if (hasTauriRuntime()) {
    return invokeCommand<KnowledgeCatalogSnapshot>('save_knowledge_entry_definition', {
      entry: normalized,
      originalEntryId,
      expectedCatalogFingerprint,
    })
  }

  const current = await loadKnowledgeAuthoringCatalog()
  requireFingerprint(current, expectedCatalogFingerprint)
  if (originalEntryId && originalEntryId !== entry.id) {
    throw new Error('Knowledge entry ids cannot change after creation.')
  }
  if (!originalEntryId && current.entries.some(item => item.id === normalized.id)) {
    throw new Error(`Knowledge entry ${normalized.id} already exists.`)
  }
  const knownIds = new Set(current.entries.map(item => item.id))
  knownIds.add(normalized.id)
  validateKnowledgeRelations(normalized, knownIds)
  const entries = current.entries.filter(item => item.id !== (originalEntryId || normalized.id))
  entries.push(normalized)
  saveBrowserDrafts(entries)
  return snapshot(entries, true)
}

export async function deleteKnowledgeEntryDefinition(
  entryId: string,
  expectedCatalogFingerprint: string,
): Promise<KnowledgeCatalogSnapshot> {
  if (hasTauriRuntime()) {
    return invokeCommand<KnowledgeCatalogSnapshot>('delete_knowledge_entry_definition', {
      entryId,
      expectedCatalogFingerprint,
    })
  }

  const current = await loadKnowledgeAuthoringCatalog()
  requireFingerprint(current, expectedCatalogFingerprint)
  const references = current.entries
    .filter(entry => entry.id !== entryId && entry.related_entries.includes(entryId))
    .map(entry => `knowledge:${entry.id}`)
  references.push(...await loadBrowserCharacterKnowledgeReferences(entryId))
  if (references.length > 0) {
    throw new Error(`Knowledge entry ${entryId} is still referenced by ${[...new Set(references)].sort().join(', ')}.`)
  }
  const entries = current.entries.filter(entry => entry.id !== entryId)
  if (entries.length === current.entries.length) throw new Error(`Knowledge entry ${entryId} does not exist.`)
  saveBrowserDrafts(entries)
  return snapshot(entries, true)
}

export async function resetBrowserKnowledgeDrafts(): Promise<KnowledgeCatalogSnapshot> {
  if (hasTauriRuntime()) return loadKnowledgeAuthoringCatalog()
  window.localStorage.removeItem(browserDraftKey)
  return loadKnowledgeAuthoringCatalog()
}

function baseUrl(): URL {
  const base = import.meta.env.BASE_URL || '/'
  return base === './' ? new URL('./', window.location.href) : new URL(base, window.location.origin)
}

function projectUrl(relativePath: string): string {
  return new URL(relativePath.replace(/^\/+/, ''), baseUrl()).toString()
}

async function fetchJson<T>(url: string): Promise<T> {
  const response = await fetch(url, { cache: 'no-cache' })
  if (!response.ok) throw new Error(`${url} returned HTTP ${response.status}`)
  return response.json() as Promise<T>
}

function normalizeKnowledgeDocument(value: unknown): KnowledgeEntryDefinition[] {
  if (Array.isArray(value)) return value.map(normalizeKnowledgeEntry)
  if (value && typeof value === 'object') return [normalizeKnowledgeEntry(value)]
  return []
}

function normalizeKnowledgeEntry(value: unknown): KnowledgeEntryDefinition {
  const input = value && typeof value === 'object' ? value as Record<string, unknown> : {}
  return {
    id: String(input.id || '').trim(),
    category: String(input.category || 'other').trim().toLowerCase(),
    title: String(input.title || '').trim(),
    content: String(input.content || '').trim(),
    tags: stringList(input.tags),
    importance: clampImportance(input.importance),
    metadata: plainObject(input.metadata),
    related_entries: stringList(input.related_entries ?? input.relatedEntries),
  }
}

function validateKnowledgeEntry(entry: KnowledgeEntryDefinition) {
  if (!/^[a-z0-9_-]{1,128}$/.test(entry.id)) {
    throw new Error('Knowledge entry ids must use lowercase letters, numbers, underscores, or hyphens.')
  }
  validateText(entry.title, 'title', 1, 256, false)
  validateText(entry.content, 'content', 1, 16_384, true)
  if (!/^[a-z0-9_-]{1,64}$/.test(entry.category)) {
    throw new Error('Knowledge categories must use lowercase letters, numbers, underscores, or hyphens.')
  }
  if (!Number.isFinite(entry.importance) || entry.importance < 0 || entry.importance > 1) {
    throw new Error('Knowledge importance must be between 0 and 1.')
  }
  if (entry.tags.length > 64) throw new Error('Knowledge entries can contain at most 64 tags.')
  entry.tags.forEach(tag => validateText(tag, 'tag', 1, 64, false))
  entry.related_entries.forEach(relatedId => {
    if (!/^[a-z0-9_-]{1,128}$/.test(relatedId)) {
      throw new Error(`Related knowledge entry id ${relatedId} is invalid.`)
    }
  })
}

function validateKnowledgeRelations(entry: KnowledgeEntryDefinition, knownIds: Set<string>) {
  for (const relatedId of entry.related_entries) {
    if (relatedId === entry.id) throw new Error('Knowledge entries cannot reference themselves.')
    if (!knownIds.has(relatedId)) throw new Error(`Related knowledge entry ${relatedId} does not exist.`)
  }
}

function validateText(value: string, field: string, min: number, max: number, allowMultiline: boolean) {
  const length = [...value].length
  const disallowedControl = allowMultiline
    ? /[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f]/.test(value)
    : /[\u0000-\u001f\u007f]/.test(value)
  if (length < min || length > max || disallowedControl) {
    throw new Error(`Knowledge ${field} must contain ${min} to ${max} characters without control characters.`)
  }
}

function stringList(value: unknown): string[] {
  if (!Array.isArray(value)) return []
  return [...new Set(value.map(item => String(item).trim()).filter(Boolean))]
}

function plainObject(value: unknown): Record<string, unknown> {
  return value && typeof value === 'object' && !Array.isArray(value)
    ? structuredClone(value as Record<string, unknown>)
    : {}
}

async function loadBrowserCharacterKnowledgeReferences(entryId: string): Promise<string[]> {
  return (await loadStoryCharacters())
    .filter((character) => (character.knowledge_refs || character.knowledge || []).includes(entryId))
    .map((character) => `character:${character.id}`)
}

function clampImportance(value: unknown): number {
  const numeric = Number(value)
  return Number.isFinite(numeric) ? Math.max(0, Math.min(1, numeric)) : 0.5
}

function uniqueEntries(entries: KnowledgeEntryDefinition[]): KnowledgeEntryDefinition[] {
  const byId = new Map<string, KnowledgeEntryDefinition>()
  for (const entry of entries) {
    if (entry.id && entry.title && entry.content) byId.set(entry.id, entry)
  }
  return [...byId.values()].sort((left, right) => left.id.localeCompare(right.id))
}

function snapshot(entries: KnowledgeEntryDefinition[], browserDraft = false): KnowledgeCatalogSnapshot {
  const normalized = uniqueEntries(entries.map(normalizeKnowledgeEntry))
  return {
    schema: KNOWLEDGE_AUTHORING_SCHEMA_V1,
    catalog_fingerprint: fingerprint(normalized),
    entries: normalized,
    browser_draft: browserDraft,
  }
}

function fingerprint(entries: KnowledgeEntryDefinition[]): string {
  const value = JSON.stringify(entries)
  let hash = 2166136261
  for (let index = 0; index < value.length; index += 1) {
    hash ^= value.charCodeAt(index)
    hash = Math.imul(hash, 16777619)
  }
  return `browser-${(hash >>> 0).toString(16).padStart(8, '0')}`
}

function loadBrowserDrafts(): KnowledgeEntryDefinition[] | null {
  const raw = window.localStorage.getItem(browserDraftKey)
  if (raw === null) return null
  try {
    const value = JSON.parse(raw) as unknown
    return Array.isArray(value) ? value.map(normalizeKnowledgeEntry) : null
  } catch {
    return null
  }
}

function saveBrowserDrafts(entries: KnowledgeEntryDefinition[]) {
  window.localStorage.setItem(browserDraftKey, JSON.stringify(uniqueEntries(entries)))
}

function requireFingerprint(current: KnowledgeCatalogSnapshot, expected: string) {
  if (current.catalog_fingerprint !== expected) {
    throw new Error('Knowledge catalog changed; reload before saving.')
  }
}
