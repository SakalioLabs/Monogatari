import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

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

export const useGameStore = defineStore('game', () => {
  const characters = ref<Character[]>([])
  const currentCharacter = ref<Character | null>(null)
  const dialogueState = ref<DialogueState | null>(null)
  const isLoading = ref(false)

  async function loadCharacters() {
    try {
      characters.value = await invoke('get_characters')
    } catch (e) {
      console.error('Failed to load characters:', e)
    }
  }

  async function setCurrentCharacter(id: string) {
    try {
      currentCharacter.value = await invoke('get_character', { characterId: id })
    } catch (e) {
      console.error('Failed to get character:', e)
    }
  }

  async function refreshDialogueState() {
    try {
      dialogueState.value = await invoke('get_dialogue_state')
    } catch (e) {
      console.error('Failed to get dialogue state:', e)
    }
  }

  async function startDialogue(dialogueId: string) {
    isLoading.value = true
    try {
      dialogueState.value = await invoke('start_dialogue', { dialogueId })
    } catch (e) {
      console.error('Failed to start dialogue:', e)
    } finally {
      isLoading.value = false
    }
  }

  async function advanceDialogue() {
    try {
      dialogueState.value = await invoke('advance_dialogue')
    } catch (e) {
      console.error('Failed to advance dialogue:', e)
    }
  }

  async function selectChoice(index: number) {
    try {
      dialogueState.value = await invoke('select_choice', { choiceIndex: index })
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
