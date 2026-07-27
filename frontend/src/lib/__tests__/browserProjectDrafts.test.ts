import { beforeEach, describe, expect, it } from 'vitest'

import { activateBrowserProjectScope } from '../browserProjectDrafts'
import { createSceneRoleplayDraft } from '../sceneRoleplayAuthoring'
import {
  loadBrowserRoleplayDrafts,
  saveBrowserRoleplayDrafts,
  sceneRoleplayDraftStorageKey,
} from '../sceneRoleplayContent'

describe('browser project draft isolation', () => {
  beforeEach(() => {
    window.localStorage.clear()
  })

  it('ignores legacy and foreign-project live roleplay drafts', () => {
    const definition = createSceneRoleplayDraft()
    window.localStorage.setItem(sceneRoleplayDraftStorageKey, JSON.stringify([definition]))
    activateBrowserProjectScope(manifest('/roleplays/project_a.json'))
    expect(loadBrowserRoleplayDrafts()).toBeNull()

    saveBrowserRoleplayDrafts([definition])
    expect(loadBrowserRoleplayDrafts()?.map(roleplay => roleplay.id)).toEqual([definition.id])

    activateBrowserProjectScope(manifest('/roleplays/project_b.json'))
    expect(loadBrowserRoleplayDrafts()).toBeNull()
  })
})

function manifest(roleplayPath: string) {
  return {
    character_files: [],
    scene_files: [],
    dialogue_files: [],
    roleplay_files: [roleplayPath],
    campaign_files: [],
    ending_files: [],
    knowledge_files: [],
    event_catalogs: [],
  }
}
