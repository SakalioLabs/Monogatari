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

  return {
    characters,
    currentCharacter,
    dialogueState,
    isLoading,
    loadCharacters,
    setCurrentCharacter,
    refreshDialogueState,
    startDialogue,
    advanceDialogue,
    selectChoice,
  }
})
