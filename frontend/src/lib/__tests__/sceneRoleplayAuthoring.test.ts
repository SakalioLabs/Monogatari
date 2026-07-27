import { beforeEach, describe, expect, it, vi } from 'vitest'

import {
  cloneSceneRoleplayDefinition,
  createRoleplayCondition,
  createSceneRoleplayDraft,
  deleteSceneRoleplayDefinition,
  loadSceneRoleplayAuthoringCatalog,
  saveSceneRoleplayDefinition,
  sceneRoleplayDraftSnapshot,
  validateSceneRoleplayDefinition,
} from '../sceneRoleplayAuthoring'
import {
  saveBrowserRoleplayDrafts,
  sceneRoleplayDraftStorageKey,
} from '../sceneRoleplayContent'

describe('scene roleplay authoring domain', () => {
  beforeEach(() => {
    localStorage.removeItem(sceneRoleplayDraftStorageKey)
    vi.unstubAllGlobals()
  })

  it('creates an editable live NPC graph with independent inference budgets', () => {
    const draft = createSceneRoleplayDraft(['new_roleplay'])

    expect(draft.id).toBe('new_roleplay_2')
    expect(draft.nodes).toHaveLength(1)
    expect(draft.nodes[0]).toMatchObject({
      player_goal: expect.any(String),
      character_goal: expect.any(String),
      timeout_target: { kind: 'ending' },
    })
    expect(draft.inference.npc_max_tokens).toBeLessThan(draft.inference.max_context_characters)
    expect(validateSceneRoleplayDefinition(draft)).toEqual([])
  })

  it('deep-clones definitions and creates typed route conditions', () => {
    const source = createSceneRoleplayDraft()
    const cloned = cloneSceneRoleplayDefinition(source)
    cloned.nodes[0].situation = 'Changed'

    expect(source.nodes[0].situation).not.toBe('Changed')
    expect(createRoleplayCondition('score_at_least', source, source.nodes[0])).toEqual({
      kind: 'score_at_least',
      dimension_id: 'story_progress',
      value: 1,
    })
    expect(createRoleplayCondition('evidence_observed', source, source.nodes[0])).toEqual({
      kind: 'evidence_observed',
      evidence_id: '',
    })
  })

  it('persists browser drafts with optimistic catalog fingerprints', async () => {
    const definition = createSceneRoleplayDraft()
    stubProjectCatalog([])
    await loadSceneRoleplayAuthoringCatalog()
    saveBrowserRoleplayDrafts([definition])
    const before = await loadSceneRoleplayAuthoringCatalog()
    const edited = cloneSceneRoleplayDefinition(definition)
    edited.title = 'Edited live roleplay'

    const saved = await saveSceneRoleplayDefinition(
      edited,
      definition.id,
      before.catalog_fingerprint,
    )
    expect(saved.roleplays[0].definition.title).toBe('Edited live roleplay')
    expect(saved.catalog_fingerprint).not.toBe(before.catalog_fingerprint)
    await expect(saveSceneRoleplayDefinition(
      edited,
      definition.id,
      before.catalog_fingerprint,
    )).rejects.toThrow(/changed since it was opened/)

    const removed = await deleteSceneRoleplayDefinition(
      definition.id,
      saved.catalog_fingerprint,
    )
    expect(removed.roleplays).toEqual([])
  })

  it('rejects browser deletion while a Campaign references the roleplay', async () => {
    const definition = createSceneRoleplayDraft()
    stubProjectCatalog([{
      schema: 'monogatari-roleplay-campaign/v1',
      id: 'chapter_campaign',
      title: 'Chapter campaign',
      start_entry_id: 'chapter',
      entries: [{
        id: 'chapter',
        roleplay_id: definition.id,
        routes: [{
          ending_id: definition.exhaustion_ending_id,
          target: { kind: 'complete' },
        }],
      }],
    }])
    await loadSceneRoleplayAuthoringCatalog()
    saveBrowserRoleplayDrafts([definition])
    const catalog = await loadSceneRoleplayAuthoringCatalog()

    await expect(deleteSceneRoleplayDefinition(
      definition.id,
      catalog.catalog_fingerprint,
    )).rejects.toThrow(/used by Campaign: chapter_campaign/)
    expect((await loadSceneRoleplayAuthoringCatalog()).roleplays).toHaveLength(1)
  })

  it('produces stable snapshots and actionable graph validation', () => {
    const definition = createSceneRoleplayDraft()
    expect(sceneRoleplayDraftSnapshot(definition)).toBe(sceneRoleplayDraftSnapshot(
      cloneSceneRoleplayDefinition(definition),
    ))

    definition.nodes[0].score_rules[0].dimension_id = 'missing'
    expect(validateSceneRoleplayDefinition(definition).join(' ')).toContain('Unknown score dimension')
  })
})

function stubProjectCatalog(campaigns: unknown[]): void {
  vi.stubGlobal('fetch', vi.fn(async (input: RequestInfo | URL) => {
    const url = String(input)
    if (url.endsWith('/project-assets.json')) {
      return new Response(JSON.stringify({
        schema: 'monogatari-web-project-assets/v1',
        character_files: [],
        scene_files: [],
        dialogue_files: [],
        roleplay_files: [],
        campaign_files: campaigns.map((_, index) => `/campaigns/${index}.json`),
        ending_files: [],
        knowledge_files: [],
        event_catalogs: [],
      }), { status: 200 })
    }
    const campaignIndex = campaigns.findIndex((_, index) => url.endsWith(`/campaigns/${index}.json`))
    if (campaignIndex >= 0) {
      return new Response(JSON.stringify(campaigns[campaignIndex]), { status: 200 })
    }
    return new Response('', { status: 404 })
  }))
}
