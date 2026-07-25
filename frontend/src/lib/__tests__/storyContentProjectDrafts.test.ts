import { beforeEach, describe, expect, it, vi } from 'vitest'

import {
  loadStoryCharacters,
  saveBrowserCharacterDrafts,
} from '../storyContent'

const draftKey = 'monogatari:character-authoring-catalog:v1'

describe('browser project authoring drafts', () => {
  beforeEach(() => {
    window.localStorage.clear()
    vi.restoreAllMocks()
  })

  it('ignores legacy and foreign-project drafts while retaining current-project drafts', async () => {
    let characterPath = '/characters/aqua.json'
    vi.stubGlobal('fetch', vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input)
      if (url.endsWith('/project-assets.json')) {
        return jsonResponse({
          schema: 'monogatari-web-project-assets/v1',
          character_files: [characterPath],
          scene_files: [],
          dialogue_files: [],
          roleplay_files: [],
          campaign_files: [],
          ending_files: [],
          knowledge_files: [],
        })
      }
      if (url.endsWith(characterPath)) {
        const id = characterPath.includes('aqua') ? 'aqua' : 'sakura'
        return jsonResponse({ id, name: id === 'aqua' ? 'Aqua' : 'Sakura' })
      }
      return new Response('', { status: 404 })
    }))

    window.localStorage.setItem(draftKey, JSON.stringify([{ id: 'echo_nine', name: 'Echo Nine' }]))
    expect((await loadStoryCharacters()).map(character => character.id)).toEqual(['aqua'])

    saveBrowserCharacterDrafts([character('eris', 'Eris')])
    expect((await loadStoryCharacters()).map(character => character.id)).toEqual(['eris'])
    expect(JSON.parse(window.localStorage.getItem(draftKey) || '{}')).toMatchObject({
      schema: 'monogatari-browser-project-drafts/v1',
      entries: [{ id: 'eris', name: 'Eris' }],
    })

    characterPath = '/characters/sakura.json'
    expect((await loadStoryCharacters()).map(character => character.id)).toEqual(['sakura'])
  })
})

function jsonResponse(value: unknown): Response {
  return new Response(JSON.stringify(value), {
    status: 200,
    headers: { 'Content-Type': 'application/json' },
  })
}

function character(id: string, name: string) {
  return {
    id,
    name,
    description: '',
    emotion: 'neutral',
    portrait_path: null,
    sprite_path: null,
  }
}
