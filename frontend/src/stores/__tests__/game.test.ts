import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'

vi.mock('../../lib/tauri', () => ({
  invokeCommand: vi.fn(),
}))

import { invokeCommand } from '../../lib/tauri'
import { useGameStore } from '../game'

const invokeMock = vi.mocked(invokeCommand)

describe('game store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invokeMock.mockReset()
    vi.spyOn(console, 'error').mockImplementation(() => undefined)
  })

  it('loads characters and forwards command fallbacks', async () => {
    invokeMock.mockResolvedValueOnce([{
      id: 'aoi',
      name: 'Aoi',
      description: 'Writer',
      emotion: 'neutral',
      live2d_model_path: null,
    }])
    const store = useGameStore()

    await store.loadCharacters()

    expect(store.characters.map((character) => character.id)).toEqual(['aoi'])
    expect(invokeMock).toHaveBeenCalledWith('get_characters', undefined, [])
  })

  it('keeps loading active until dialogue startup settles', async () => {
    let resolve!: (value: unknown) => void
    invokeMock.mockImplementationOnce(() => new Promise((done) => { resolve = done }))
    const store = useGameStore()

    const pending = store.startDialogue('intro')
    expect(store.isLoading).toBe(true)
    resolve({ is_active: true, speaker: 'Aoi', text: 'Hello', emotion: null, choices: [], live2d_expression: null })
    await pending

    expect(store.isLoading).toBe(false)
    expect(store.dialogueState?.text).toBe('Hello')
    expect(invokeMock).toHaveBeenCalledWith('start_dialogue', { dialogueId: 'intro' })
  })

  it('returns null and resets loading after save or load failures', async () => {
    invokeMock.mockRejectedValue(new Error('offline'))
    const store = useGameStore()

    await expect(store.saveGame('Slot')).resolves.toBeNull()
    expect(store.isLoading).toBe(false)
    await expect(store.loadGame('slot-1')).resolves.toBeNull()
    expect(store.isLoading).toBe(false)
  })

  it('updates save and scene state only after successful commands', async () => {
    const store = useGameStore()
    store.saves = [
      { save_id: 'keep', timestamp: '1' },
      { save_id: 'delete', timestamp: '2' },
    ]
    invokeMock.mockResolvedValueOnce(undefined)
    await store.deleteSave('delete')
    expect(store.saves.map((save) => save.save_id)).toEqual(['keep'])

    invokeMock.mockResolvedValueOnce(undefined)
    await store.setActiveScene('studio')
    expect(store.activeSceneId).toBe('studio')

    invokeMock.mockRejectedValueOnce(new Error('scene rejected'))
    await store.setActiveScene('private')
    expect(store.activeSceneId).toBe('studio')
  })

  it('uses stable browser-safe fallbacks for save lists and relationship scores', async () => {
    const store = useGameStore()
    invokeMock.mockRejectedValue(new Error('unavailable'))

    await expect(store.listSaves()).resolves.toEqual([])
    await expect(store.getRelationshipScore('aoi')).resolves.toBe(0)
  })
})
