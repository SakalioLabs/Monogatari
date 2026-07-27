import { beforeEach, describe, expect, it, vi } from 'vitest'

const { loadStoryCharactersMock } = vi.hoisted(() => ({
  loadStoryCharactersMock: vi.fn(),
}))

vi.mock('../storyContent', () => ({
  loadStoryCharacters: loadStoryCharactersMock,
}))

vi.mock('../tauri', () => ({
  hasTauriRuntime: () => false,
  invokeCommand: vi.fn(),
}))

import {
  deleteKnowledgeEntryDefinition,
  loadKnowledgeAuthoringCatalog,
  saveKnowledgeEntryDefinition,
  type KnowledgeEntryDefinition,
} from '../knowledgeContent'
import {
  activateBrowserProjectScope,
  saveScopedBrowserDrafts,
} from '../browserProjectDrafts'

const browserDraftKey = 'monogatari:knowledge-authoring-catalog:v1'

function entry(overrides: Partial<KnowledgeEntryDefinition> = {}): KnowledgeEntryDefinition {
  return {
    id: 'agent_context',
    title: 'Agent Context',
    category: 'world_lore',
    content: 'Stable project context.',
    tags: ['agent'],
    importance: 0.8,
    metadata: {},
    related_entries: [],
    ...overrides,
  }
}

function seedBrowserDrafts(entries: KnowledgeEntryDefinition[]): void {
  activateBrowserProjectScope(projectManifest)
  saveScopedBrowserDrafts(browserDraftKey, entries)
}

const projectManifest = {
  schema: 'monogatari-web-project-assets/v1',
  character_files: [],
  scene_files: [],
  dialogue_files: [],
  roleplay_files: [],
  campaign_files: [],
  ending_files: [],
  knowledge_files: [],
  event_catalogs: [],
}

describe('browser knowledge content transport', () => {
  beforeEach(() => {
    window.localStorage.clear()
    vi.stubGlobal('fetch', vi.fn(async () => new Response(
      JSON.stringify(projectManifest),
      { status: 200 },
    )))
    loadStoryCharactersMock.mockReset()
    loadStoryCharactersMock.mockResolvedValue([])
  })

  it('persists normalized entries behind optimistic fingerprints', async () => {
    seedBrowserDrafts([])
    const empty = await loadKnowledgeAuthoringCatalog()
    const saved = await saveKnowledgeEntryDefinition(entry({
      title: ' Agent Context ',
      category: ' WORLD_LORE ',
      tags: ['agent', 'agent', ' delivery '],
    }), null, empty.catalog_fingerprint)

    expect(saved.browser_draft).toBe(true)
    expect(saved.entries).toEqual([entry({ tags: ['agent', 'delivery'] })])
    expect(JSON.parse(window.localStorage.getItem(browserDraftKey) || 'null').entries)
      .toEqual(saved.entries)
    await expect(saveKnowledgeEntryDefinition(
      entry({ id: 'stale_entry' }),
      null,
      empty.catalog_fingerprint,
    )).rejects.toThrow('Knowledge catalog changed')
  })

  it('protects references from other browser knowledge entries', async () => {
    seedBrowserDrafts([
      entry(),
      entry({ id: 'agent_route', title: 'Agent Route', related_entries: ['agent_context'] }),
    ])
    const current = await loadKnowledgeAuthoringCatalog()

    await expect(deleteKnowledgeEntryDefinition('agent_context', current.catalog_fingerprint))
      .rejects.toThrow('knowledge:agent_route')
    expect(JSON.parse(window.localStorage.getItem(browserDraftKey) || '{}').entries).toHaveLength(2)
  })

  it('protects references from browser-authored character drafts', async () => {
    seedBrowserDrafts([entry()])
    loadStoryCharactersMock.mockResolvedValue([{
      id: 'agent_guard',
      name: 'Agent Guard',
      knowledge_refs: ['agent_context'],
    }])
    const current = await loadKnowledgeAuthoringCatalog()

    await expect(deleteKnowledgeEntryDefinition('agent_context', current.catalog_fingerprint))
      .rejects.toThrow('character:agent_guard')
    expect(loadStoryCharactersMock).toHaveBeenCalledOnce()
    expect(JSON.parse(window.localStorage.getItem(browserDraftKey) || '{}').entries).toEqual([entry()])
  })
})
