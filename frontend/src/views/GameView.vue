<template>
  <div class="game-container">
    <div class="scene-backdrop" :style="sceneBackdropStyle">
      <div class="scene-horizon"></div>
    </div>

    <header class="game-topbar">
      <button class="control-btn" title="Dashboard" @click="$router.push('/')">{{ t('game.home', 'Home') }}</button>
      <div class="scene-meta">
        <span class="eyebrow">{{ t('game.story-mode', 'Story Mode') }}</span>
        <strong>{{ dialogueState?.speaker || currentCharacter?.name || activeScene?.name || 'Demo Scene' }}</strong>
      </div>
      <div class="top-actions">
        <button class="control-btn" title="Save" @click="saveGame">{{ t('game.save', 'Save') }}</button>
        <button class="control-btn" title="Load" @click="openLoadDialog">{{ t('game.load', 'Load') }}</button>
        <button class="control-btn" title="Backlog" @click="$router.push('/backlog')">{{ t('nav.backlog', 'Backlog') }}</button>
        <button class="control-btn" title="Settings" @click="toggleSettings">{{ t('game.tune', 'Tune') }}</button>
        <span v-if="dialogueState?.is_active" class="auto-save-badge" title="Auto-save active">&#128190;</span>
      </div>
    </header>

    <main class="stage">
      <section class="model-area">
        <Live2DCanvas
          v-if="currentLive2dPath"
          :model-path="currentLive2dPath"
          :expression="currentExpression"
          :motion="currentMotion"
          @load-error="markRendererAssetFailed"
        />
        <CharacterModelView
          v-else-if="currentModel3dPath"
          :model-path="currentModel3dPath"
          :expression="currentExpression"
          :motion="currentMotion"
          @load-error="markRendererAssetFailed"
        />
        <div v-else-if="currentSpritePath" class="sprite-stage">
          <img :src="currentSpritePath" :alt="currentCharacter?.name || 'Character sprite'" />
        </div>
        <CharacterModelView
          v-else-if="currentCharacter"
          :model-path="null"
          :expression="currentExpression"
          :motion="currentMotion"
        />
        <div v-else class="model-placeholder">
          <span class="empty-mark">VN</span>
          <strong>{{ t('game.no-character', 'No character on stage') }}</strong>
          <span>{{ t('game.waiting', 'Scene runtime is waiting.') }}</span>
        </div>
      </section>

      <section class="dialogue-area">
        <div v-if="dialogueState?.is_active" class="dialogue-box">
          <div class="speaker-name" v-if="dialogueState.speaker">
            <span>{{ dialogueState.speaker }}</span>
            <small>{{ dialogueState.emotion || currentExpression }}</small>
          </div>

          <p class="dialogue-text">
            {{ displayedText }}
            <span v-if="isTyping" class="cursor"></span>
          </p>

          <div v-if="dialogueState.choices.length > 0" class="choices">
            <button
              v-for="(choice, idx) in dialogueState.choices"
              :key="choice.index"
              class="choice-btn"
              :style="{ animationDelay: `${idx * 0.06}s` }"
              @click="selectChoice(choice.index)"
            >
              <span class="choice-number">{{ idx + 1 }}</span>
              <span>{{ choice.text }}</span>
            </button>
          </div>

          <button v-else class="advance-hint" @click="advanceDialogue">
            {{ isTyping ? t('game.complete-line', 'Complete line') : t('game.continue-text', 'Continue') }}
          </button>
        </div>

        <div v-else class="scene-empty">
          <span class="empty-mark">M</span>
          <h1>Monogatari Runtime</h1>
          <p>{{ activeScene ? activeScene.background_path || 'Active scene is ready.' : t('game.runtime-desc', 'AI-ready visual novel playback with dialogue state, Live2D staging, and saves.') }}</p>
          <button class="btn btn-primary btn-lg" :disabled="isLoading" @click="startDemoDialogue">
            <span v-if="isLoading" class="loading-spinner"></span>
            <span>{{ isLoading ? t('game.loading', 'Loading') : t('game.start-demo', 'Start Demo') }}</span>
          </button>
        </div>
      </section>
    </main>

    <Transition name="fade">
      <div v-if="toastMessage" class="toast" @click="toastMessage = null">{{ toastMessage }}</div>
    </Transition>

    <Transition name="fade">
      <div v-if="errorMessage" class="error-toast" @click="errorMessage = null">{{ errorMessage }}</div>
    </Transition>

    <Transition name="fade">
      <div v-if="showLoadDialog" class="modal-overlay" @click.self="showLoadDialog = false">
        <div class="modal">
          <div class="modal-head">
            <span class="eyebrow">{{ t('game.saves', 'Saves') }}</span>
            <button class="close-btn" @click="showLoadDialog = false">{{ t('common.close', 'Close') }}</button>
          </div>
          <div class="save-list">
            <button v-for="save in saves" :key="save.save_id" class="save-item" @click="loadGame(save.save_id)">
              <span class="save-name">{{ save.save_name }}</span>
              <span class="save-time">{{ formatTime(save.timestamp) }}</span>
            </button>
            <p v-if="saves.length === 0" class="no-saves">{{ t('game.no-saves', 'No saves yet.') }}</p>
          </div>
        </div>
      </div>
    </Transition>

    <Transition name="fade">
      <div v-if="showPause" class="pause-overlay" @click.self="showPause = false">
        <div class="pause-panel">
          <div class="pause-title">Paused</div>
          <div class="pause-actions">
            <button class="pause-btn primary" @click="showPause = false">Resume</button>
            <button class="pause-btn" @click="saveGame(); showPause = false">Save</button>
            <button class="pause-btn" @click="openLoadDialog(); showPause = false">Load</button>
            <button class="pause-btn" @click="$router.push('/backlog')">Backlog</button>
            <button class="pause-btn" @click="showSettings = true; showPause = false">Settings</button>
            <button class="pause-btn secondary" @click="$router.push('/title')">Title Screen</button>
          </div>
        </div>
      </div>
    </Transition>

    <Transition name="slide">
      <aside v-if="showSettings" class="settings-panel">
        <div class="settings-header">
          <div>
            <span class="eyebrow">{{ t('game.playback', 'Playback') }}</span>
            <h3>{{ t('game.settings', 'Settings') }}</h3>
          </div>
          <button class="close-btn" @click="showSettings = false">{{ t('common.close', 'Close') }}</button>
        </div>
        <div class="settings-content">
          <label class="setting-group">
            <span>{{ t('game.text-speed', 'Text speed') }}</span>
            <input type="range" v-model="settings.textSpeed" min="10" max="100" />
            <b>{{ settings.textSpeed }}ms</b>
          </label>
          <label class="setting-line">
            <span>{{ t('game.auto-play', 'Auto play') }}</span>
            <input type="checkbox" v-model="settings.autoPlay" />
          </label>
          <label class="setting-group">
            <span>{{ t('game.bgm-volume', 'BGM volume') }}</span>
            <input type="range" v-model="settings.bgmVolume" min="0" max="100" />
            <b>{{ settings.bgmVolume }}%</b>
          </label>
          <label class="setting-group">
            <span>{{ t('game.sfx-volume', 'SFX volume') }}</span>
            <input type="range" v-model="settings.sfxVolume" min="0" max="100" />
            <b>{{ settings.sfxVolume }}%</b>
          </label>
          <label class="setting-group">
            <span>{{ t('game.voice-volume', 'Voice volume') }}</span>
            <input type="range" v-model="settings.voiceVolume" min="0" max="100" />
            <b>{{ settings.voiceVolume }}%</b>
          </label>
        </div>
      </aside>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import Live2DCanvas from '../components/Live2DCanvas.vue'
import CharacterModelView from '../components/CharacterModelView.vue'
import { invokeCommand } from '../lib/tauri'
import { useI18n } from '../lib/i18n'
import { resolveAssetUrl } from '../lib/assets'
import { selectCharacterRendererAsset } from '../lib/rendererAssets'

const { t } = useI18n()

interface DialogueState {
  is_active: boolean
  speaker: string | null
  text: string
  emotion: string | null
  choices: { index: number; text: string }[]
  live2d_expression: string | null
}

interface CharacterInfo {
  id: string
  name: string
  description: string
  emotion: string
  live2d_model_path: string | null
  model_3d_path: string | null
  portrait_path: string | null
  sprite_path: string | null
  sprite_paths?: Record<string, string>
}

interface SaveInfo {
  save_id: string
  save_name: string
  timestamp: string
  current_scene: string | null
}

interface SceneInfo {
  id: string
  name: string
  background_path: string | null
  bgm_path: string | null
  weather: string | null
  time_of_day: string | null
  tags: string[]
  source: string
  background_exists: boolean
  absolute_background_path: string | null
}

interface ActiveScene {
  scene: SceneInfo | null
  scene_history: string[]
}

const dialogueState = ref<DialogueState | null>(null)
const characters = ref<CharacterInfo[]>([])
const currentCharacter = ref<CharacterInfo | null>(null)
const activeScene = ref<SceneInfo | null>(null)
const currentExpression = ref('neutral')
const currentMotion = ref('idle')
const displayedText = ref('')
const isTyping = ref(false)
const showLoadDialog = ref(false)
const showSettings = ref(false)
const showPause = ref(false)
const saves = ref<SaveInfo[]>([])
const errorMessage = ref<string | null>(null)
const toastMessage = ref<string | null>(null)
const isLoading = ref(false)
const failedRendererAssets = ref<Record<string, true>>({})

const settings = ref({
  textSpeed: 30,
  autoPlay: false,
  autoPlaySpeed: 3000,
  bgmVolume: 80,
  sfxVolume: 80,
  voiceVolume: 80,
})

let typingTimer: number | null = null
let autoPlayTimer: number | null = null
let autoSaveTimer: number | null = null
const activeSceneStorageKey = 'monogatari.activeScene'

const sceneBackdropStyle = computed(() => {
  if (!activeScene.value) return {}

  const bgPath = activeScene.value.absolute_background_path || activeScene.value.background_path
  const bgUrl = resolveAssetUrl(bgPath)
  if (bgUrl) {
    return {
      background: `url("${bgUrl}") center / cover no-repeat`,
    }
  }

  if (bgPath) {
    return {
      background: 'linear-gradient(180deg, hsl(210 28% 18%), hsl(225 32% 10%))',
    }
  }

  const seed = Array.from(activeScene.value.id).reduce((sum, char) => sum + char.charCodeAt(0), 0)
  const hueA = (seed * 17) % 360
  const hueB = (hueA + 44) % 360
  const hueC = (hueA + 172) % 360
  return {
    background:
      `linear-gradient(180deg, hsl(${hueA} 44% 18%), hsl(${hueB} 42% 10%)), ` +
      `radial-gradient(circle at 50% 72%, hsl(${hueC} 62% 36% / 0.36), transparent 38%)`,
  }
})

const currentRendererAsset = computed(() =>
  selectCharacterRendererAsset(currentCharacter.value, {
    expression: currentExpression.value,
    validatePaths: true,
    blockedPaths: Object.keys(failedRendererAssets.value),
  })
)
const currentLive2dPath = computed(() =>
  currentRendererAsset.value.mode === 'live2d' ? currentRendererAsset.value.resolvedUrl : null
)
const currentModel3dPath = computed(() =>
  currentRendererAsset.value.mode === 'model3d' ? currentRendererAsset.value.resolvedUrl : null
)
const currentSpritePath = computed(() =>
  currentRendererAsset.value.mode === 'sprite' ? currentRendererAsset.value.resolvedUrl : null
)

function formatTime(timestamp: string): string {
  try {
    return new Date(timestamp).toLocaleString('zh-CN')
  } catch {
    return timestamp
  }
}

function markRendererAssetFailed(payload: { path: string | null; message: string }) {
  const path = payload.path?.trim()
  if (!path) return
  failedRendererAssets.value = { ...failedRendererAssets.value, [path]: true }
  errorMessage.value = `${payload.message}; falling back to the next renderer asset.`
}

watch(() => currentCharacter.value?.id, () => {
  failedRendererAssets.value = {}
})

async function loadActiveScene() {
  try {
    const active = await invokeCommand<ActiveScene>('get_current_scene', undefined, previewActiveScene)
    activeScene.value = active.scene
  } catch (e) {
    console.error('Failed to load active scene:', e)
  }
}

function previewActiveScene(): ActiveScene {
  const stored = localStorage.getItem(activeSceneStorageKey)
  if (!stored) return { scene: null, scene_history: [] }
  try {
    const scene = JSON.parse(stored) as SceneInfo
    return { scene, scene_history: [scene.id] }
  } catch {
    localStorage.removeItem(activeSceneStorageKey)
    return { scene: null, scene_history: [] }
  }
}

async function loadCharacters() {
  try {
    characters.value = await invokeCommand<CharacterInfo[]>('get_characters', undefined, previewCharacters)
    if (!currentCharacter.value && characters.value.length > 0) {
      currentCharacter.value = characters.value[0]
      currentExpression.value = currentCharacter.value.emotion || 'neutral'
    }
  } catch (e) {
    console.error(e)
  }
}

function previewCharacters(): CharacterInfo[] {
  return [
    {
      id: 'sakura',
      name: 'Sakura',
      description: 'Browser preview character for renderer fallback checks.',
      emotion: 'happy',
      live2d_model_path: null,
      model_3d_path: null,
      portrait_path: null,
      sprite_path: null,
      sprite_paths: {},
    },
  ]
}

function syncCurrentCharacter() {
  const speaker = dialogueState.value?.speaker
  if (!speaker) return
  currentCharacter.value =
    characters.value.find((char) => char.name === speaker || char.id === speaker) || currentCharacter.value
}

async function updateDialogueState() {
  try {
    dialogueState.value = await invokeCommand<DialogueState>('get_dialogue_state', undefined, {
      is_active: false,
      speaker: null,
      text: '',
      emotion: null,
      choices: [],
      live2d_expression: null,
    })
    if (dialogueState.value?.live2d_expression) {
      currentExpression.value = dialogueState.value.live2d_expression
    }
    if (dialogueState.value?.emotion) {
      currentExpression.value = dialogueState.value.emotion
    }
    syncCurrentCharacter()
    if (dialogueState.value?.text) {
      typewriterEffect(dialogueState.value.text)
    }
  } catch (e) {
    console.error('Failed to get dialogue state:', e)
  }
}

function typewriterEffect(text: string) {
  if (typingTimer) clearInterval(typingTimer)
  if (autoPlayTimer) clearTimeout(autoPlayTimer)
  displayedText.value = ''
  isTyping.value = true
  let i = 0
  typingTimer = window.setInterval(() => {
    if (i < text.length) {
      displayedText.value += text[i]
      i += 1
    } else {
      if (typingTimer) clearInterval(typingTimer)
      isTyping.value = false
      if (settings.value.autoPlay && dialogueState.value?.choices.length === 0) {
        autoPlayTimer = window.setTimeout(advanceDialogue, settings.value.autoPlaySpeed)
      }
    }
  }, settings.value.textSpeed)
}

async function startDemoDialogue() {
  isLoading.value = true
  errorMessage.value = null
  try {
    await invokeCommand<void>('start_dialogue', { dialogueId: 'meeting_sakura' })
    await updateDialogueState()
  } catch (e) {
    errorMessage.value = `Unable to start dialogue: ${e}`
  } finally {
    isLoading.value = false
  }
}

async function selectChoice(index: number) {
  try {
    await invokeCommand<void>('select_choice', { choiceIndex: index })
    await updateDialogueState()
  } catch (e) {
    errorMessage.value = String(e)
  }
}

async function advanceDialogue() {
  if (isTyping.value) {
    if (typingTimer) clearInterval(typingTimer)
    isTyping.value = false
    displayedText.value = dialogueState.value?.text || displayedText.value
    return
  }
  try {
    await invokeCommand<void>('advance_dialogue')
    await updateDialogueState()
  } catch (e) {
    errorMessage.value = String(e)
  }
}

const QUICK_SAVE_ID = 'quick_save_0'

async function saveQuick() {
  try {
    await invokeCommand<string>('save_game', { saveName: 'Quick Save' })
    toastMessage.value = 'Quick saved'
  } catch (e) { errorMessage.value = String(e) }
}

async function quickLoad() {
  try {
    await loadGame(QUICK_SAVE_ID)
    toastMessage.value = 'Quick loaded'
  } catch (e) {
    errorMessage.value = 'No quick save found'
  }
}

async function saveGame() {
  try {
    const name = `Save ${new Date().toLocaleString('zh-CN')}`
    const saveId = await invokeCommand<string>('save_game', { saveName: name })
    toastMessage.value = `Saved ${saveId}`
    await loadSaves()
  } catch (e) {
    errorMessage.value = String(e)
  }
}

async function loadGame(saveId: string) {
  try {
    await invokeCommand<void>('load_game', { saveId })
    showLoadDialog.value = false
    toastMessage.value = 'Save loaded'
    await loadActiveScene()
    await updateDialogueState()
  } catch (e) {
    errorMessage.value = String(e)
  }
}

async function loadSaves() {
  try {
    saves.value = await invokeCommand<SaveInfo[]>('list_saves', undefined, [])
  } catch (e) {
    console.error('Failed to list saves:', e)
  }
}

async function openLoadDialog() {
  await loadSaves()
  showLoadDialog.value = true
}

function toggleSettings() {
  showSettings.value = !showSettings.value
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    e.preventDefault()
    showPause.value = !showPause.value
    return
  }
  // Quick save/load
  if (e.key === 's' && !e.ctrlKey && !e.metaKey) {
    e.preventDefault()
    if (dialogueState.value?.is_active) {
      saveQuick()
    }
    return
  }
  if (e.key === 'l' && !e.ctrlKey && !e.metaKey) {
    e.preventDefault()
    quickLoad()
    return
  }
  if (e.key === ' ' || e.key === 'Enter') {
    e.preventDefault()
    advanceDialogue()
  }
}

onMounted(async () => {
  await loadActiveScene()
  await loadCharacters()
  await updateDialogueState()
  await loadSaves()
  window.addEventListener('keydown', handleKeydown)
  
  // Auto-save every 2 minutes during active dialogue
  autoSaveTimer = window.setInterval(async () => {
    if (dialogueState.value?.is_active) {
      try {
        const name = 'Auto-save ' + new Date().toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
        await invokeCommand<string>('save_game', { saveName: name })
      } catch (e) { console.error('Auto-save failed:', e) }
    }
  }, 120000)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
  if (typingTimer) clearInterval(typingTimer)
  if (autoPlayTimer) clearTimeout(autoPlayTimer)
  if (autoSaveTimer) clearInterval(autoSaveTimer)
})
</script>

<style scoped>
.game-container {
  position: relative;
  height: 100vh;
  overflow: hidden;
  color: var(--text-primary);
  background: var(--surface-0);
}

.scene-backdrop,
.scene-horizon {
  position: absolute;
  inset: 0;
}

.scene-backdrop {
  background: var(--surface-0);
}

.scene-horizon {
  background:
    linear-gradient(180deg, rgba(96,165,250,0.16), transparent 42%),
    radial-gradient(circle at 50% 78%, rgba(45,212,191,0.16), transparent 34%),
    linear-gradient(180deg, rgba(21,25,34,0.28), rgba(15,17,21,0.82));
}

.game-topbar {
  position: relative;
  z-index: 4;
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  gap: 14px;
  align-items: center;
  padding: 14px 18px;
  border-bottom: 1px solid rgba(255,255,255,0.08);
  background: rgba(15,17,21,0.72);
  backdrop-filter: blur(16px);
}

.scene-meta {
  min-width: 0;
}

.scene-meta strong {
  display: block;
  overflow: hidden;
  color: var(--text-primary);
  font-size: 16px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.eyebrow {
  display: block;
  color: var(--text-tertiary);
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0;
  text-transform: uppercase;
}

.top-actions {
  display: flex;
  gap: 8px;
}

.control-btn,
.close-btn {
  min-height: 34px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: rgba(21,25,34,0.86);
  color: var(--text-secondary);
  cursor: pointer;
  font: inherit;
  font-weight: 700;
  padding: 6px 10px;
}

.control-btn:hover,
.close-btn:hover {
  border-color: var(--brand);
  color: var(--brand-light);
}

.stage {
  position: relative;
  z-index: 1;
  display: grid;
  grid-template-rows: minmax(0, 1fr) auto;
  height: calc(100vh - 63px);
}

.model-area {
  min-height: 0;
  display: grid;
  place-items: center;
  padding: 24px;
}

.model-placeholder {
  display: grid;
  gap: 8px;
  place-items: center;
  color: var(--text-tertiary);
}

.model-placeholder strong {
  color: var(--text-primary);
}

.sprite-stage {
  width: min(460px, 76vw);
  height: min(680px, 62vh);
  display: grid;
  place-items: end center;
}

.sprite-stage img {
  display: block;
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
  filter: drop-shadow(0 28px 36px rgba(0,0,0,0.46));
}

.dialogue-area {
  padding: 20px;
}

.dialogue-box,
.scene-empty {
  max-width: 920px;
  margin: 0 auto;
  border: 1px solid rgba(255,255,255,0.12);
  border-radius: var(--radius-lg);
  background: rgba(21,25,34,0.88);
  backdrop-filter: blur(18px);
  box-shadow: var(--shadow-lg);
}

.dialogue-box {
  padding: 20px;
}

.speaker-name {
  display: inline-flex;
  gap: 10px;
  align-items: baseline;
  margin-bottom: 12px;
  color: var(--brand-light);
  font-weight: 800;
}

.speaker-name small {
  color: var(--text-tertiary);
  font-size: 12px;
  font-weight: 700;
}

.dialogue-text {
  min-height: 76px;
  color: var(--text-primary);
  font-size: 18px;
  line-height: 1.75;
  white-space: pre-wrap;
}

.cursor {
  display: inline-block;
  width: 8px;
  height: 1.1em;
  margin-left: 4px;
  vertical-align: -0.15em;
  background: var(--brand);
  animation: blink 0.8s infinite;
}

.choices {
  display: grid;
  gap: 10px;
  margin-top: 18px;
}

.choice-btn {
  display: grid;
  grid-template-columns: 30px minmax(0, 1fr);
  gap: 12px;
  align-items: center;
  width: 100%;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-2);
  color: var(--text-primary);
  cursor: pointer;
  padding: 12px;
  text-align: left;
  opacity: 0;
  transform: translateY(8px);
  animation: optionIn 0.22s ease forwards;
}

.choice-btn:hover {
  border-color: var(--brand);
  background: var(--surface-3);
}

.choice-number {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: var(--radius-sm);
  background: var(--brand);
  color: var(--surface-0);
  font-weight: 900;
}

.advance-hint {
  width: 100%;
  margin-top: 16px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-2);
  color: var(--text-secondary);
  cursor: pointer;
  font-weight: 800;
  padding: 11px;
}

.advance-hint:hover {
  border-color: var(--brand);
  color: var(--brand-light);
}

.scene-empty {
  display: grid;
  place-items: center;
  gap: 12px;
  padding: 34px;
  text-align: center;
}

.scene-empty h1 {
  color: var(--text-primary);
  font-size: 28px;
}

.scene-empty p {
  max-width: 520px;
  color: var(--text-secondary);
}

.empty-mark {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 44px;
  height: 44px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-2);
  color: var(--brand-light);
  font-family: var(--font-mono);
  font-weight: 900;
}

.modal-overlay {
  position: fixed;
  inset: 0;
  z-index: 40;
  display: grid;
  place-items: center;
  background: rgba(0,0,0,0.66);
  backdrop-filter: blur(5px);
}

.modal {
  width: min(460px, calc(100vw - 32px));
  max-height: min(560px, calc(100vh - 80px));
  overflow: auto;
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  background: var(--surface-1);
  box-shadow: var(--shadow-lg);
  padding: 18px;
}

.modal-head,
.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  margin-bottom: 16px;
}

.save-list {
  display: grid;
  gap: 8px;
}

.save-item {
  display: grid;
  gap: 3px;
  width: 100%;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-2);
  color: var(--text-primary);
  cursor: pointer;
  padding: 12px;
  text-align: left;
}

.save-item:hover {
  border-color: var(--brand);
}

.save-name {
  font-weight: 800;
}

.save-time,
.no-saves {
  color: var(--text-tertiary);
  font-size: 12px;
}

.settings-panel {
  position: fixed;
  z-index: 35;
  top: 0;
  right: 0;
  width: min(340px, 100vw);
  height: 100vh;
  border-left: 1px solid var(--border);
  background: rgba(21,25,34,0.98);
  box-shadow: var(--shadow-lg);
  padding: 20px;
}

.settings-header h3 {
  color: var(--text-primary);
}

.settings-content {
  display: grid;
  gap: 18px;
}

.setting-group,
.setting-line {
  display: grid;
  gap: 8px;
  color: var(--text-secondary);
  font-weight: 700;
}

.setting-line {
  grid-template-columns: 1fr auto;
  align-items: center;
}

.setting-group input[type='range'] {
  width: 100%;
  accent-color: var(--brand);
}

.setting-group b {
  color: var(--text-tertiary);
  font-size: 12px;
}

.toast,
.error-toast {
  position: fixed;
  z-index: 80;
  left: 50%;
  transform: translateX(-50%);
  min-width: min(420px, calc(100vw - 32px));
  border-radius: var(--radius);
  box-shadow: var(--shadow-lg);
  padding: 12px 14px;
  color: white;
  text-align: center;
}

.toast {
  top: 82px;
  border: 1px solid rgba(45,212,191,0.36);
  background: rgba(15,118,110,0.96);
}

.error-toast {
  bottom: 18px;
  border: 1px solid rgba(239,68,68,0.42);
  background: rgba(127,29,29,0.96);
}

.loading-spinner {
  display: inline-block;
  width: 16px;
  height: 16px;
  border: 2px solid rgba(255,255,255,0.35);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

.btn:disabled,
.control-btn:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.22s;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.slide-enter-active,
.slide-leave-active {
  transition: transform 0.24s ease;
}

.slide-enter-from,
.slide-leave-to {
  transform: translateX(100%);
}

@keyframes blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0; }
}

@keyframes optionIn {
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

@media (max-width: 720px) {
  .game-topbar {
    grid-template-columns: 1fr;
  }

  .top-actions {
    flex-wrap: wrap;
  }

  .dialogue-area {
    padding: 12px;
  }

  .dialogue-text {
    font-size: 16px;
  }
}
</style>
