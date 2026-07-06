import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invokeCommand } from '../lib/tauri'

interface Character {
  id: string
  name: string
  description: string
  emotion: string
  live2d_model_path: string | null
}

interface DialogueState {
  is_active: boolean
  speaker: string | null
  text: string
  emotion: string | null
  choices: { index: number; text: string }[]
  live2d_expression: string | null
}

const emptyDialogueState: DialogueState = {
  is_active: false,
  speaker: null,
  text: '',
  emotion: null,
  choices: [],
  live2d_expression: null,
}

export const useGameStore = defineStore('game', () => {
  const characters = ref<Character[]>([])
  const currentCharacter = ref<Character | null>(null)
  const dialogueState = ref<DialogueState | null>(null)
  const isLoading = ref(false)
  const activeSceneId = ref<string | null>(null)
  const saves = ref<{ id: string; timestamp: string }[]>([])

  async function loadCharacters() {
    try {
      characters.value = await invokeCommand<Character[]>('get_characters', undefined, [])
    } catch (e) {
      console.error('Failed to load characters:', e)
    }
  }

  async function setCurrentCharacter(id: string) {
    try {
      currentCharacter.value = await invokeCommand<Character>('get_character', { characterId: id })
    } catch (e) {
      console.error('Failed to get character:', e)
    }
  }

  async function refreshDialogueState() {
    try {
      dialogueState.value = await invokeCommand<DialogueState>('get_dialogue_state', undefined, emptyDialogueState)
    } catch (e) {
      console.error('Failed to get dialogue state:', e)
    }
  }

  async function startDialogue(dialogueId: string) {
    isLoading.value = true
    try {
      dialogueState.value = await invokeCommand<DialogueState>('start_dialogue', { dialogueId })
    } catch (e) {
      console.error('Failed to start dialogue:', e)
    } finally {
      isLoading.value = false
    }
  }

  async function advanceDialogue() {
    try {
      dialogueState.value = await invokeCommand<DialogueState>('advance_dialogue')
    } catch (e) {
      console.error('Failed to advance dialogue:', e)
    }
  }

  async function selectChoice(index: number) {
    try {
      dialogueState.value = await invokeCommand<DialogueState>('select_choice', { choiceIndex: index })
    } catch (e) {
      console.error('Failed to select choice:', e)
    }
  }

  async function saveGame(slotId: string) {
    isLoading.value = true
    try {
      return await invokeCommand('save_game', { slotId })
    } catch (e) {
      console.error('Save failed:', e)
      return null
    } finally {
      isLoading.value = false
    }
  }

  async function loadGame(slotId: string) {
    isLoading.value = true
    try {
      return await invokeCommand('load_game', { slotId })
    } catch (e) {
      console.error('Load failed:', e)
      return null
    } finally {
      isLoading.value = false
    }
  }

  async function listSaves() {
    try {
      saves.value = await invokeCommand<{ id: string; timestamp: string }[]>('list_saves', undefined, [])
    } catch {
      saves.value = []
    }
    return saves.value
  }

  async function deleteSave(slotId: string) {
    try {
      await invokeCommand('delete_save', { slotId })
      saves.value = saves.value.filter(s => s.id !== slotId)
    } catch (e) {
      console.error('Delete save failed:', e)
    }
  }

  async function setActiveScene(sceneId: string) {
    try {
      await invokeCommand('set_scene', { sceneId })
      activeSceneId.value = sceneId
    } catch (e) {
      console.error('Scene change failed:', e)
    }
  }

  async function getRelationshipScore(characterId: string): Promise<number> {
    try {
      return await invokeCommand<number>('get_relationship_score', { characterId }, 0)
    } catch {
      return 0
    }
  }

  return {
    characters,
    currentCharacter,
    dialogueState,
    isLoading,
    activeSceneId,
    saves,
    loadCharacters,
    setCurrentCharacter,
    refreshDialogueState,
    startDialogue,
    advanceDialogue,
    selectChoice,
    saveGame,
    loadGame,
    listSaves,
    deleteSave,
    setActiveScene,
    getRelationshipScore,
  }
})
