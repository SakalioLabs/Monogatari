<template>
  <div class="game-container">
    <div class="scene-backdrop" :style="sceneBackdropStyle">
      <CharacterModelView
        v-if="activeSceneModel3dPath"
        class="scene-model-backdrop"
        :model-path="activeSceneModel3dPath"
        presentation="scene"
        @load-error="markRendererAssetFailed"
      />
      <div class="scene-horizon"></div>
    </div>

    <header class="game-topbar">
      <button class="control-btn icon-control" :title="t('game.home', 'Home')" :aria-label="t('game.home', 'Home')" @click="$router.push('/')"><House :size="16" /></button>
      <div class="scene-meta">
        <span class="eyebrow">{{ t('game.story-mode', 'Playtest') }}</span>
        <strong>{{ activeRoleplaySnapshot?.definition.title || dialogueState?.speaker || currentCharacter?.name || activeScene?.name || t('game.demo-scene', 'Demo Scene') }}</strong>
      </div>
      <div class="top-actions">
        <button class="control-btn library-trigger" :title="t('game.story-library', 'Story library')" :aria-label="t('game.story-library', 'Story library')" @click="openStoryLibrary"><LibraryBig :size="16" /></button>
        <button
          v-if="!activeRoleplaySnapshot && currentCharacter && dialogueState?.is_active"
          class="control-btn icon-control npc-trigger"
          data-testid="npc-trigger"
          :title="t('npc.open', 'Talk to {name}', { name: currentCharacter.name })"
          :aria-label="t('npc.open', 'Talk to {name}', { name: currentCharacter.name })"
          @click="openNpcConversation"
        ><MessageCircleMore :size="16" /></button>
        <button v-if="desktopRuntime" class="control-btn" @click="saveGame"><Save :size="15" />{{ t('game.save', 'Save') }}</button>
        <button v-if="desktopRuntime" class="control-btn" @click="openLoadDialog"><FolderOpen :size="15" />{{ t('game.load', 'Load') }}</button>
        <button class="control-btn" @click="$router.push('/backlog')"><History :size="15" />{{ t('nav.backlog', 'Backlog') }}</button>
        <button class="control-btn" @click="toggleSettings"><SlidersHorizontal :size="15" />{{ t('game.tune', 'Tune') }}</button>
        <span v-if="desktopRuntime && dialogueState?.is_active" class="auto-save-badge" :title="t('game.auto-save-active', 'Auto-save active')"><Cloud :size="15" /></span>
      </div>
    </header>

    <main class="stage" :class="{ 'roleplay-active': activeRoleplaySnapshot }">
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
          <img :src="currentSpritePath" :alt="currentCharacter?.name || t('game.character-sprite', 'Character sprite')" />
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

      <section class="dialogue-area" :class="{ 'roleplay-dialogue-area': activeRoleplaySnapshot }">
        <SceneRoleplayPanel
          v-if="activeRoleplaySnapshot"
          :snapshot="activeRoleplaySnapshot"
          :desktop-runtime="desktopRuntime"
          :characters="characters"
          :endings="storyEndings"
          :locale="locale"
          :scene-name="activeScene?.name || null"
          @update="updateActiveRoleplay"
          @node-change="syncRoleplayNode"
          @emotion="applyNpcEmotion"
          @ending="completeRoleplayEnding"
          @restart="restartActiveRoleplay"
        />
        <div v-else-if="dialogueState?.is_active" class="dialogue-box">
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
          <h1>{{ t('game.monogatari-runtime', 'Monogatari Runtime') }}</h1>
          <p>{{ activeScene ? activeScene.background_path || t('game.active-scene-ready', 'Active scene is ready.') : t('game.runtime-desc', 'AI-ready visual novel playback with dialogue state, Live2D staging, and saves.') }}</p>
          <button class="btn btn-primary btn-lg" :disabled="isLoading" @click="openStoryLibrary">
            <span v-if="isLoading" class="loading-spinner"></span>
            <LibraryBig v-else :size="16" />
            <span>{{ isLoading ? t('game.loading', 'Loading') : t('game.choose-story', 'Choose story') }}</span>
          </button>
        </div>
      </section>
    </main>

    <Transition name="fade">
      <div v-if="toastMessage" class="toast" @click="toastMessage = null">{{ toastMessage }}</div>
    </Transition>

    <Transition name="fade">
      <div v-if="showStoryLibrary" class="modal-overlay" @click.self="showStoryLibrary = false">
        <div class="modal story-library-modal">
          <div class="modal-head">
            <div>
              <span class="eyebrow">{{ t('game.story-library', 'Story library') }}</span>
              <strong>{{ t('game.gated-unlocks', '{unlocked} / {total} gated unlocks', { unlocked: storyAccess.unlocked_gated_content_count, total: storyAccess.gated_content_count }) }}</strong>
            </div>
            <button class="close-btn" :title="t('common.close', 'Close')" :aria-label="t('common.close', 'Close')" @click="showStoryLibrary = false"><X :size="16" /></button>
          </div>
          <div class="library-tabs" role="tablist" :aria-label="t('game.story-content-type', 'Story content type')">
            <button :class="{ active: libraryTab === 'roleplays' }" @click="libraryTab = 'roleplays'">{{ t('game.roleplays', 'Roleplays') }}</button>
            <button :class="{ active: libraryTab === 'scenes' }" @click="libraryTab = 'scenes'">{{ t('game.scenes', 'Scenes') }}</button>
            <button :class="{ active: libraryTab === 'dialogues' }" @click="libraryTab = 'dialogues'">{{ t('game.dialogues', 'Dialogues') }}</button>
            <button :class="{ active: libraryTab === 'endings' }" @click="libraryTab = 'endings'">{{ t('game.endings', 'Endings') }}</button>
          </div>
          <div class="story-content-list">
            <button
              v-for="roleplay in libraryTab === 'roleplays' ? storyRoleplays : []"
              :key="roleplay.id"
              class="story-content-row"
              :class="{ active: activeRoleplaySnapshot?.definition.id === roleplay.id }"
              :disabled="isLoading"
              @click="startSceneRoleplay(roleplay)"
            >
              <span class="content-mark" aria-hidden="true"><MessageCircleMore :size="15" /></span>
              <span class="content-copy">
                <strong>{{ roleplay.title }}</strong>
                <small>{{ t('game.node-count', '{count} nodes', { count: roleplay.nodes.length }) }}</small>
              </span>
              <span class="content-status">{{ t('game.play', 'Play') }}</span>
            </button>
            <button
              v-for="scene in libraryTab === 'scenes' ? storyScenes : []"
              :key="scene.id"
              class="story-content-row"
              :class="{ locked: !scene.access.unlocked, active: activeScene?.id === scene.id }"
              :disabled="!scene.access.unlocked || isLoading"
              @click="enterScene(scene)"
            >
              <span class="content-mark">{{ scene.access.unlocked ? 'SC' : 'LK' }}</span>
              <span class="content-copy">
                <strong>{{ scene.name }}</strong>
                <small>{{ scene.id }}<template v-if="scene.access.gated"> · {{ unlockHint(scene.access) }}</template></small>
              </span>
              <span class="content-status">{{ activeScene?.id === scene.id ? t('game.active', 'Active') : scene.access.unlocked ? t('game.enter', 'Enter') : t('game.locked', 'Locked') }}</span>
            </button>
            <button
              v-for="dialogue in libraryTab === 'dialogues' ? storyDialogues : []"
              :key="dialogue.id"
              class="story-content-row"
              :class="{ locked: !dialogue.access.unlocked }"
              :disabled="!dialogue.access.unlocked || isLoading"
              @click="startStoryDialogue(dialogue)"
            >
              <span class="content-mark">{{ dialogue.access.unlocked ? 'DL' : 'LK' }}</span>
              <span class="content-copy">
                <strong>{{ dialogue.title }}</strong>
                <small>{{ t('game.node-count', '{count} nodes', { count: dialogue.node_count }) }}<template v-if="dialogue.access.gated"> · {{ unlockHint(dialogue.access) }}</template></small>
              </span>
              <span class="content-status">{{ dialogue.access.unlocked ? t('game.play', 'Play') : t('game.locked', 'Locked') }}</span>
            </button>
            <button
              v-for="ending in libraryTab === 'endings' ? storyEndings : []"
              :key="ending.id"
              class="story-content-row"
              :class="{ locked: !ending.access.unlocked }"
              :disabled="!ending.access.unlocked || isLoading"
              @click="startEnding(ending)"
            >
              <span class="content-mark">{{ ending.access.unlocked ? 'EN' : 'LK' }}</span>
              <span class="content-copy">
                <strong>{{ ending.title }}</strong>
                <small>{{ ending.description }}<template v-if="ending.access.gated"> · {{ unlockHint(ending.access) }}</template></small>
              </span>
              <span class="content-status">{{ ending.access.unlocked ? t('game.view', 'View') : t('game.locked', 'Locked') }}</span>
            </button>
            <p v-if="activeLibraryItems === 0" class="no-saves">{{ t('game.empty-library', 'No {type} are available in this project.', { type: libraryTypeLabel(libraryTab) }) }}</p>
          </div>
        </div>
      </div>
    </Transition>

    <Transition name="fade">
      <div v-if="errorMessage" class="error-toast" @click="errorMessage = null">{{ errorMessage }}</div>
    </Transition>

    <Transition name="fade">
      <div v-if="showLoadDialog" class="modal-overlay" @click.self="showLoadDialog = false">
        <div class="modal">
          <div class="modal-head">
            <span class="eyebrow">{{ t('game.saves', 'Saves') }}</span>
            <button class="close-btn" :title="t('common.close', 'Close')" :aria-label="t('common.close', 'Close')" @click="showLoadDialog = false"><X :size="16" /></button>
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
          <div class="pause-title">{{ t('game.paused', 'Paused') }}</div>
          <div class="pause-actions">
            <button class="pause-btn primary" @click="showPause = false"><Play :size="15" />{{ t('game.resume', 'Resume') }}</button>
            <button v-if="desktopRuntime" class="pause-btn" @click="saveGame(); showPause = false"><Save :size="15" />{{ t('game.save', 'Save') }}</button>
            <button v-if="desktopRuntime" class="pause-btn" @click="openLoadDialog(); showPause = false"><FolderOpen :size="15" />{{ t('game.load', 'Load') }}</button>
            <button class="pause-btn" @click="$router.push('/backlog')"><History :size="15" />{{ t('nav.backlog', 'Backlog') }}</button>
            <button class="pause-btn" @click="showSettings = true; showPause = false"><SlidersHorizontal :size="15" />{{ t('game.settings', 'Settings') }}</button>
            <button class="pause-btn secondary" @click="$router.push('/title')"><PanelsTopLeft :size="15" />{{ t('game.title-screen', 'Title screen') }}</button>
          </div>
        </div>
      </div>
    </Transition>

    <NpcConversationPanel
      :open="showNpcConversation"
      :character="currentCharacter"
      :desktop-runtime="desktopRuntime"
      :locale="locale"
      @close="showNpcConversation = false"
      @emotion="applyNpcEmotion"
      @story-progress="loadStoryLibrary"
    />

    <Transition name="slide">
      <aside v-if="showSettings" class="settings-panel">
        <div class="settings-header">
          <div>
            <span class="eyebrow">{{ t('game.playback', 'Playback') }}</span>
            <h3>{{ t('game.settings', 'Settings') }}</h3>
          </div>
          <button class="close-btn" :title="t('common.close', 'Close')" :aria-label="t('common.close', 'Close')" @click="showSettings = false"><X :size="16" /></button>
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
import { useRoute } from 'vue-router'
import {
  Cloud,
  FolderOpen,
  History,
  House,
  LibraryBig,
  MessageCircleMore,
  PanelsTopLeft,
  Play,
  Save,
  SlidersHorizontal,
  X,
} from '@lucide/vue'
import Live2DCanvas from '../components/Live2DCanvas.vue'
import CharacterModelView from '../components/CharacterModelView.vue'
import NpcConversationPanel from '../components/NpcConversationPanel.vue'
import SceneRoleplayPanel from '../components/SceneRoleplayPanel.vue'
import { hasTauriRuntime, invokeCommand } from '../lib/tauri'
import { useI18n } from '../lib/i18n'
import { resolveAssetUrl } from '../lib/assets'
import { selectCharacterRendererAsset } from '../lib/rendererAssets'
import { reloadStoryEventCatalog } from '../lib/storyEvents'
import { loadStoryContentAccess, type StoryContentAccessEntry, type StoryContentAccessSnapshot } from '../lib/storyAccess'
import { createStoryTextPlaybackController } from '../lib/storyTextPlayback'
import {
  advanceBrowserDialogue,
  applyBrowserRelationshipChanges,
  selectBrowserDialogueChoice,
  startBrowserDialogue,
  type BrowserDialogueTransition,
  type BrowserDialogueRuntime,
  type DialogueState,
} from '../lib/storyPlaytest'
import {
  loadStoryDialogues,
  loadStoryEndings,
  loadStoryCharacters,
  loadStoryScenes,
  type StoryDialogueInfo,
  type StoryEndingInfo,
  type StoryCharacterInfo,
  type StorySceneInfo,
} from '../lib/storyContent'
import {
  loadSceneRoleplays,
  startBrowserSceneRoleplay,
  type SceneRoleplayDefinition,
  type SceneRoleplayNode,
  type SceneRoleplaySnapshot,
} from '../lib/sceneRoleplay'

const { locale, t } = useI18n()
const route = useRoute()
const desktopRuntime = hasTauriRuntime()

type CharacterInfo = StoryCharacterInfo & {
  live2d_model_path: string | null
  model_3d_path: string | null
  portrait_path: string | null
  sprite_path: string | null
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
  model_3d_path?: string | null
  bgm_path: string | null
  weather: string | null
  time_of_day: string | null
  tags: string[]
  source: string
  background_exists: boolean
  absolute_background_path: string | null
  model_3d_exists?: boolean
  absolute_model_3d_path?: string | null
}

interface StoryEndingLaunch {
  ending: StoryEndingInfo
  scene: SceneInfo
  dialogue: DialogueState
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
const showStoryLibrary = ref(false)
const showSettings = ref(false)
const showPause = ref(false)
const showNpcConversation = ref(false)
const saves = ref<SaveInfo[]>([])
const errorMessage = ref<string | null>(null)
const toastMessage = ref<string | null>(null)
const isLoading = ref(false)
const failedRendererAssets = ref<Record<string, true>>({})
const storyScenes = ref<StorySceneInfo[]>([])
const storyDialogues = ref<StoryDialogueInfo[]>([])
const storyEndings = ref<StoryEndingInfo[]>([])
const storyRoleplays = ref<SceneRoleplayDefinition[]>([])
const activeRoleplaySnapshot = ref<SceneRoleplaySnapshot | null>(null)
const libraryTab = ref<'roleplays' | 'scenes' | 'dialogues' | 'endings'>('roleplays')
const storyAccess = ref<StoryContentAccessSnapshot>({
  schema: 'monogatari-story-content-access/v1',
  catalog_fingerprint: '',
  progress_fingerprint: '',
  gated_content_count: 0,
  unlocked_gated_content_count: 0,
  locked_content_count: 0,
  entries: [],
})
const webActiveDialogue = ref<StoryDialogueInfo | null>(null)
const webDialogueRuntime = ref<BrowserDialogueRuntime | null>(null)
const webDialogueFlags = ref<Record<string, boolean>>({})

const settings = ref({
  textSpeed: 30,
  autoPlay: false,
  autoPlaySpeed: 3000,
  bgmVolume: 80,
  sfxVolume: 80,
  voiceVolume: 80,
})

const textPlayback = createStoryTextPlaybackController({
  scheduler: {
    setInterval: (callback, delay) => window.setInterval(callback, delay),
    clearInterval: (timerId) => window.clearInterval(timerId),
    setTimeout: (callback, delay) => window.setTimeout(callback, delay),
    clearTimeout: (timerId) => window.clearTimeout(timerId),
  },
  readTextIntervalMs: () => settings.value.textSpeed,
  readAutoAdvanceDelayMs: () => settings.value.autoPlaySpeed,
  shouldAutoAdvance: () => settings.value.autoPlay
    && !activeRoleplaySnapshot.value
    && !showNpcConversation.value
    && dialogueState.value?.choices.length === 0,
  onTextChange: (text) => { displayedText.value = text },
  onTypingChange: (typing) => { isTyping.value = typing },
  onAutoAdvance: () => { void advanceDialogue() },
})

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
      background: 'hsl(210 10% 16%)',
    }
  }

  const seed = Array.from(activeScene.value.id).reduce((sum, char) => sum + char.charCodeAt(0), 0)
  const hueA = (seed * 17) % 360
  return {
    background: `hsl(${hueA} 8% 16%)`,
  }
})

const activeSceneModel3dPath = computed(() => {
  const path = activeScene.value?.absolute_model_3d_path || activeScene.value?.model_3d_path
  if (!path) return null
  const resolved = resolveAssetUrl(path)
  if (!resolved || failedRendererAssets.value[path] || failedRendererAssets.value[resolved]) return null
  return resolved
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
const activeLibraryItems = computed(() => {
  if (libraryTab.value === 'roleplays') return storyRoleplays.value.length
  if (libraryTab.value === 'scenes') return storyScenes.value.length
  return libraryTab.value === 'dialogues' ? storyDialogues.value.length : storyEndings.value.length
})

function formatTime(timestamp: string): string {
  try {
    return new Date(timestamp).toLocaleString(locale.value)
  } catch {
    return timestamp
  }
}

function markRendererAssetFailed(payload: { path: string | null; message: string }) {
  const path = payload.path?.trim()
  if (!path) return
  failedRendererAssets.value = { ...failedRendererAssets.value, [path]: true }
  errorMessage.value = t('game.renderer-fallback', '{message}; using the next available renderer asset.', { message: payload.message })
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
    characters.value = (await loadStoryCharacters()).map(character => ({
      ...character,
      live2d_model_path: character.live2d_model_path ?? null,
      model_3d_path: character.model_3d_path ?? null,
      portrait_path: character.portrait_path ?? null,
      sprite_path: character.sprite_path ?? null,
    }))
    if (!currentCharacter.value && characters.value.length > 0) {
      currentCharacter.value = characters.value.find(character => (
        character.live2d_model_path || character.model_3d_path || character.sprite_path
      )) || characters.value[0]
      currentExpression.value = currentCharacter.value.emotion || 'neutral'
    }
  } catch (e) {
    console.error(e)
  }
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
      scene_id: null,
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
    await syncDialogueScene(dialogueState.value)
    syncCurrentCharacter()
    if (dialogueState.value?.text) {
      typewriterEffect(dialogueState.value.text)
    }
  } catch (e) {
    console.error('Failed to get dialogue state:', e)
  }
}

function typewriterEffect(text: string) {
  textPlayback.start(text)
}

function unlockHint(access: StoryContentAccessEntry): string {
  return access.unlocked
    ? t('game.unlocked', 'Unlocked')
    : t('game.requires-events', 'Requires {events}', { events: access.unlock_event_ids.join(', ') })
}

function libraryTypeLabel(type: 'roleplays' | 'scenes' | 'dialogues' | 'endings'): string {
  if (type === 'roleplays') return t('game.roleplays', 'roleplays')
  if (type === 'scenes') return t('game.scenes', 'scenes')
  if (type === 'dialogues') return t('game.dialogues', 'dialogues')
  return t('game.endings', 'endings')
}

async function loadStoryLibrary() {
  try {
    await reloadStoryEventCatalog()
    const [roleplays, scenes, dialogues, endings, access] = await Promise.all([
      loadSceneRoleplays(),
      loadStoryScenes(),
      loadStoryDialogues(),
      loadStoryEndings(),
      loadStoryContentAccess(),
    ])
    storyRoleplays.value = roleplays
    storyScenes.value = scenes
    storyDialogues.value = dialogues
    storyEndings.value = endings
    storyAccess.value = access
  } catch (error) {
    errorMessage.value = t('game.unable-load-library', 'Unable to load story library: {error}', { error: String(error) })
  }
}

async function openStoryLibrary() {
  isLoading.value = true
  await loadStoryLibrary()
  isLoading.value = false
  showStoryLibrary.value = true
}

async function enterScene(scene: StorySceneInfo) {
  isLoading.value = true
  errorMessage.value = null
  try {
    activeRoleplaySnapshot.value = null
    activeScene.value = await invokeCommand<SceneInfo>('enter_story_scene', { sceneId: scene.id }, scene)
    localStorage.setItem(activeSceneStorageKey, JSON.stringify(activeScene.value))
    showStoryLibrary.value = false
    toastMessage.value = t('game.entered-scene', 'Entered {scene}', { scene: scene.name })
  } catch (error) {
    errorMessage.value = String(error)
  } finally {
    isLoading.value = false
  }
}

async function startSceneRoleplay(definition: SceneRoleplayDefinition) {
  isLoading.value = true
  errorMessage.value = null
  try {
    const snapshot = desktopRuntime
      ? await invokeCommand<SceneRoleplaySnapshot>('start_scene_roleplay', { roleplayId: definition.id })
      : startBrowserSceneRoleplay(definition)
    activeRoleplaySnapshot.value = snapshot
    dialogueState.value = null
    webActiveDialogue.value = null
    webDialogueRuntime.value = null
    displayedText.value = ''
    textPlayback.cancel()
    await syncRoleplayNode(snapshot.current_node)
    showStoryLibrary.value = false
  } catch (error) {
    errorMessage.value = t('game.unable-start-dialogue', 'Unable to start dialogue: {error}', { error: String(error) })
  } finally {
    isLoading.value = false
  }
}

function updateActiveRoleplay(snapshot: SceneRoleplaySnapshot) {
  activeRoleplaySnapshot.value = snapshot
}

async function syncRoleplayNode(node: SceneRoleplayNode) {
  const scene = storyScenes.value.find(candidate => candidate.id === node.scene_id)
  if (!scene) {
    errorMessage.value = `Scene roleplay references unavailable scene "${node.scene_id}".`
    return
  }
  if (activeScene.value?.id !== scene.id) {
    activeScene.value = await invokeCommand<SceneInfo>('enter_story_scene', { sceneId: scene.id }, scene)
    localStorage.setItem(activeSceneStorageKey, JSON.stringify(activeScene.value))
  }
  currentCharacter.value = characters.value.find(character => character.id === node.character_id) || null
  currentExpression.value = currentCharacter.value?.emotion || 'neutral'
}

function completeRoleplayEnding(endingId: string) {
  const ending = storyEndings.value.find(candidate => candidate.id === endingId)
  toastMessage.value = ending
    ? t('game.ending-started', 'Ending: {title}', { title: ending.title })
    : t('game.ending-started', 'Ending: {title}', { title: endingId })
}

async function restartActiveRoleplay() {
  const definition = activeRoleplaySnapshot.value?.definition
  if (definition) await startSceneRoleplay(definition)
}

async function startStoryDialogue(dialogue: StoryDialogueInfo, previewNodeId?: string | null) {
  isLoading.value = true
  errorMessage.value = null
  try {
    activeRoleplaySnapshot.value = null
    if (hasTauriRuntime()) {
      dialogueState.value = await invokeCommand<DialogueState>('start_dialogue', { dialogueId: dialogue.id })
    } else {
      webActiveDialogue.value = dialogue
      await applyBrowserDialogueTransition(startBrowserDialogue(dialogue, webDialogueFlags.value, previewNodeId))
    }
    const state = dialogueState.value
    if (!state) throw new Error('Dialogue runtime did not return a state.')
    syncCurrentCharacter()
    currentExpression.value = state.emotion || state.live2d_expression || currentExpression.value
    typewriterEffect(state.text)
    showStoryLibrary.value = false
  } catch (e) {
    errorMessage.value = t('game.unable-start-dialogue', 'Unable to start dialogue: {error}', { error: String(e) })
  } finally {
    isLoading.value = false
  }
}

async function startEnding(ending: StoryEndingInfo) {
  isLoading.value = true
  errorMessage.value = null
  try {
    activeRoleplaySnapshot.value = null
    const fallbackScene = storyScenes.value.find((scene) => scene.id === ending.scene_id) || activeScene.value
    if (!fallbackScene) throw new Error(t('game.ending-scene-unavailable', 'Ending scene {id} is unavailable', { id: ending.scene_id }))
    const fallbackDialogue = storyDialogues.value.find((dialogue) => dialogue.id === ending.dialogue_id)
    if (!fallbackDialogue) throw new Error(t('game.ending-dialogue-unavailable', 'Ending dialogue {id} is unavailable', { id: ending.dialogue_id }))
    const launch = await invokeCommand<StoryEndingLaunch>('start_story_ending', { endingId: ending.id }, () => ({
      ending,
      scene: fallbackScene,
      dialogue: startBrowserDialogue(fallbackDialogue, webDialogueFlags.value).state,
    }))
    activeScene.value = launch.scene
    dialogueState.value = launch.dialogue
    if (!hasTauriRuntime()) {
      webActiveDialogue.value = fallbackDialogue
      await applyBrowserDialogueTransition(startBrowserDialogue(fallbackDialogue, webDialogueFlags.value))
    }
    const state = dialogueState.value
    if (!state) throw new Error('Ending runtime did not return a dialogue state.')
    syncCurrentCharacter()
    typewriterEffect(state.text)
    showStoryLibrary.value = false
    toastMessage.value = t('game.ending-started', 'Ending: {title}', { title: ending.title })
  } catch (error) {
    errorMessage.value = String(error)
  } finally {
    isLoading.value = false
  }
}

async function selectChoice(index: number) {
  try {
    if (!hasTauriRuntime() && webActiveDialogue.value) {
      if (!webDialogueRuntime.value) throw new Error('Browser dialogue runtime is unavailable.')
      const transition = selectBrowserDialogueChoice(webActiveDialogue.value, webDialogueRuntime.value, index)
      applyBrowserChoiceRelationships(transition.relationship_changes)
      await applyBrowserDialogueTransition(transition)
      syncCurrentCharacter()
      typewriterEffect(transition.state.text)
    } else {
      await invokeCommand<void>('select_choice', { choiceIndex: index })
      await updateDialogueState()
    }
  } catch (e) {
    errorMessage.value = String(e)
  }
}

async function advanceDialogue() {
  if (textPlayback.complete()) return
  try {
    if (!hasTauriRuntime() && webActiveDialogue.value) {
      if (!webDialogueRuntime.value) throw new Error('Browser dialogue runtime is unavailable.')
      const transition = advanceBrowserDialogue(webActiveDialogue.value, webDialogueRuntime.value)
      if (transition.blocked_reason === 'choice_required') return
      await applyBrowserDialogueTransition(transition)
      if (transition.state.is_active) {
        syncCurrentCharacter()
        typewriterEffect(transition.state.text)
      } else {
        displayedText.value = ''
      }
    } else {
      await invokeCommand<void>('advance_dialogue')
      await updateDialogueState()
    }
  } catch (e) {
    errorMessage.value = String(e)
  }
}

async function applyBrowserDialogueTransition(transition: BrowserDialogueTransition) {
  webDialogueRuntime.value = transition.runtime
  webDialogueFlags.value = transition.runtime.flags
  dialogueState.value = transition.state
  await syncDialogueScene(transition.state)
  if (transition.completed) webActiveDialogue.value = null
}

async function syncDialogueScene(state: DialogueState | null) {
  const sceneId = state?.scene_id?.trim()
  if (!sceneId || activeScene.value?.id === sceneId) return
  const scene = storyScenes.value.find((item) => item.id === sceneId)
  if (!scene) throw new Error(`Dialogue references unavailable scene "${sceneId}".`)
  activeScene.value = await invokeCommand<SceneInfo>('enter_story_scene', { sceneId }, scene)
  localStorage.setItem(activeSceneStorageKey, JSON.stringify(activeScene.value))
}

function applyBrowserChoiceRelationships(changes: Record<string, number>) {
  if (Object.keys(changes).length === 0) return
  const activeCharacterId = currentCharacter.value?.id
  characters.value = applyBrowserRelationshipChanges(characters.value, changes)
  currentCharacter.value = characters.value.find((character) => character.id === activeCharacterId) || currentCharacter.value
}

const QUICK_SAVE_ID = 'quick_save_0'
const AUTO_SAVE_ID = 'auto_save_0'

async function saveQuick() {
  try {
    await invokeCommand<string>('save_game', { saveName: t('game.quick-save-name', 'Quick Save'), saveId: QUICK_SAVE_ID })
    toastMessage.value = t('game.quick-saved', 'Quick saved')
  } catch (e) { errorMessage.value = String(e) }
}

async function quickLoad() {
  try {
    await loadGame(QUICK_SAVE_ID)
    toastMessage.value = t('game.quick-loaded', 'Quick save loaded')
  } catch (e) {
    errorMessage.value = t('game.no-quick-save', 'No quick save found')
  }
}

async function saveGame() {
  try {
    const name = t('game.save-name', 'Save {date}', { date: new Date().toLocaleString(locale.value) })
    const saveId = await invokeCommand<string>('save_game', { saveName: name })
    toastMessage.value = t('game.saved-id', 'Saved {id}', { id: saveId })
    await loadSaves()
  } catch (e) {
    errorMessage.value = String(e)
  }
}

async function loadGame(saveId: string) {
  try {
    await invokeCommand<void>('load_game', { saveId })
    showLoadDialog.value = false
    toastMessage.value = t('game.save-loaded', 'Save loaded')
    await loadActiveScene()
    await updateDialogueState()
    await loadStoryLibrary()
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

function openNpcConversation() {
  if (!currentCharacter.value || !dialogueState.value?.is_active) return
  showStoryLibrary.value = false
  showLoadDialog.value = false
  showSettings.value = false
  showPause.value = false
  showNpcConversation.value = true
}

function applyNpcEmotion(emotion: string) {
  currentExpression.value = emotion.trim() || currentExpression.value
}

function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false
  return target instanceof HTMLInputElement
    || target instanceof HTMLTextAreaElement
    || target.isContentEditable
}

function handleKeydown(e: KeyboardEvent) {
  if (showNpcConversation.value) {
    if (e.key === 'Escape') {
      e.preventDefault()
      showNpcConversation.value = false
    }
    return
  }
  if (isEditableTarget(e.target)) return
  if (e.key === 'Escape') {
    e.preventDefault()
    showPause.value = !showPause.value
    return
  }
  if (activeRoleplaySnapshot.value) return
  // Quick save/load
  if (desktopRuntime && e.key === 's' && !e.ctrlKey && !e.metaKey) {
    e.preventDefault()
    if (dialogueState.value?.is_active) {
      saveQuick()
    }
    return
  }
  if (desktopRuntime && e.key === 'l' && !e.ctrlKey && !e.metaKey) {
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
  await loadStoryLibrary()
  await syncDialogueScene(dialogueState.value)
  const previewSceneId = route.query.previewScene
  const previewRoleplayId = route.query.previewRoleplay
  const previewDialogueId = route.query.previewDialogue
  const previewNodeId = route.query.previewNode
  const previewEndingId = route.query.previewEnding
  if (!desktopRuntime && route.query.authoring === '1') {
    const roleplay = typeof previewRoleplayId === 'string'
      ? storyRoleplays.value.find(item => item.id === previewRoleplayId)
      : typeof previewDialogueId === 'string' && typeof previewNodeId !== 'string'
        ? storyRoleplays.value.find(item => roleplayStem(item.id) === roleplayStem(previewDialogueId))
        : undefined
    if (roleplay) {
      await startSceneRoleplay(roleplay)
    } else if (typeof previewSceneId === 'string') {
      const scene = storyScenes.value.find((item) => item.id === previewSceneId)
      if (scene) await enterScene(scene)
    } else if (typeof previewDialogueId === 'string') {
      const dialogue = storyDialogues.value.find((item) => item.id === previewDialogueId)
      if (dialogue) await startStoryDialogue(
        dialogue,
        typeof previewNodeId === 'string' ? previewNodeId : undefined,
      )
    } else if (typeof previewEndingId === 'string') {
      const ending = storyEndings.value.find((item) => item.id === previewEndingId)
      if (ending) await startEnding(ending)
    }
  }
  window.addEventListener('keydown', handleKeydown)
  
  if (desktopRuntime) {
    autoSaveTimer = window.setInterval(async () => {
      if (dialogueState.value?.is_active) {
        try {
          const name = t('game.auto-save-name', 'Auto-save {time}', {
            time: new Date().toLocaleTimeString(locale.value, { hour: '2-digit', minute: '2-digit' }),
          })
          await invokeCommand<string>('save_game', { saveName: name, saveId: AUTO_SAVE_ID })
        } catch (e) { console.error('Auto-save failed:', e) }
      }
    }, 120000)
  }
})

function roleplayStem(id: string): string {
  return id.replace(/_(?:roleplay|dialogue)$/, '')
}

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
  textPlayback.dispose()
  if (autoSaveTimer) clearInterval(autoSaveTimer)
})
</script>

<style scoped>
.game-container {
  --surface-0: #101314;
  --surface-1: #171b1c;
  --surface-2: #222829;
  --surface-3: #303839;
  --surface-4: #414b4c;
  --text-primary: #f4f3ed;
  --text-secondary: #c9ccc7;
  --text-tertiary: #9da39f;
  --border: rgba(235, 231, 214, 0.2);
  --brand: #d8b969;
  --brand-light: #f0d78e;
  --brand-strong: #ffe6a3;
  --success: #80b49a;
  --shadow-lg: 0 18px 48px rgba(0, 0, 0, 0.38);
  position: relative;
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  height: 100vh;
  height: 100svh;
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
  overflow: hidden;
}

.scene-model-backdrop {
  position: absolute;
  inset: 0;
  min-height: 0;
  border-radius: 0;
}

.scene-horizon { z-index: 1; pointer-events: none; }

.scene-horizon {
  background: rgba(15,17,21,0.46);
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
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 7px;
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

.library-trigger {
  width: 34px;
  padding-inline: 0;
}

.icon-control,
.close-btn {
  width: 34px;
  padding-inline: 0;
}

.auto-save-badge {
  display: inline-grid;
  place-items: center;
  width: 30px;
  height: 30px;
  color: var(--success);
}

.stage {
  position: relative;
  z-index: 1;
  display: grid;
  grid-template-rows: minmax(0, 1fr) auto;
  min-height: 0;
}

.stage.roleplay-active {
  grid-template-columns: minmax(280px, 0.82fr) minmax(480px, 1.18fr);
  grid-template-rows: minmax(0, 1fr);
}

.roleplay-active .model-area {
  grid-column: 1;
  grid-row: 1;
}

.roleplay-active .roleplay-dialogue-area {
  grid-column: 2;
  grid-row: 1;
  min-height: 0;
  padding: 16px;
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
  min-height: 0;
  display: grid;
  place-items: end center;
  overflow: hidden;
}

.sprite-stage img {
  display: block;
  width: min(460px, 76vw);
  height: min(680px, 62vh);
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

.story-library-modal {
  width: min(720px, calc(100vw - 32px));
  max-height: min(680px, calc(100vh - 48px));
}

.story-library-modal .modal-head strong {
  display: block;
  margin-top: 3px;
  font-size: 14px;
}

.library-tabs {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  margin-bottom: 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  overflow: hidden;
}

.library-tabs button {
  min-height: 36px;
  border: 0;
  border-right: 1px solid var(--border);
  background: var(--surface-2);
  color: var(--text-secondary);
  cursor: pointer;
  font: inherit;
  font-size: 12px;
  font-weight: 700;
}

.library-tabs button:last-child { border-right: 0; }
.library-tabs button.active { background: var(--surface-3); color: var(--brand-light); }

.story-content-list {
  display: grid;
  gap: 7px;
}

.story-content-row {
  width: 100%;
  min-height: 64px;
  display: grid;
  grid-template-columns: 36px minmax(0, 1fr) auto;
  gap: 11px;
  align-items: center;
  padding: 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  color: var(--text-primary);
  cursor: pointer;
  text-align: left;
}

.story-content-row:hover:not(:disabled),
.story-content-row.active { border-color: var(--brand); background: var(--surface-3); }
.story-content-row.locked { opacity: .68; cursor: not-allowed; }
.content-mark { display: grid; place-items: center; width: 34px; height: 34px; border-radius: var(--radius-sm); background: var(--surface-3); color: var(--brand-light); font-family: var(--font-mono); font-size: 10px; font-weight: 900; }
.locked .content-mark { color: var(--text-tertiary); }
.content-copy { min-width: 0; display: grid; gap: 3px; }
.content-copy strong, .content-copy small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.content-copy small { color: var(--text-tertiary); font-size: 10px; }
.content-status { color: var(--text-tertiary); font-size: 10px; font-weight: 800; text-transform: uppercase; }

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

.pause-overlay {
  position: fixed;
  inset: 0;
  z-index: 70;
  display: grid;
  place-items: center;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(8px);
}

.pause-panel {
  width: min(340px, calc(100vw - 32px));
  padding: 20px;
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  background: var(--surface-1);
  box-shadow: var(--shadow-lg);
}

.pause-title {
  margin-bottom: 14px;
  color: var(--text-primary);
  font-size: 18px;
  font-weight: 850;
}

.pause-actions {
  display: grid;
  gap: 7px;
}

.pause-btn {
  display: flex;
  align-items: center;
  gap: 9px;
  min-height: 40px;
  padding: 8px 11px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  color: var(--text-secondary);
  cursor: pointer;
  font: inherit;
  font-size: 12px;
  font-weight: 750;
  text-align: left;
}

.pause-btn:hover { border-color: var(--brand); color: var(--text-primary); }
.pause-btn.primary { border-color: var(--brand); background: var(--brand); color: var(--surface-0); }
.pause-btn.secondary { margin-top: 4px; background: transparent; }

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
  border: 1px solid rgba(255,255,255,0.18);
  background: rgba(32,33,36,0.96);
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
    grid-template-columns: auto minmax(0, 1fr);
    gap: 8px;
    padding: 9px 10px;
  }

  .top-actions {
    grid-column: 1 / -1;
    flex-wrap: nowrap;
    overflow-x: auto;
    padding-bottom: 2px;
  }

  .top-actions .control-btn { flex: 0 0 auto; }
  .stage { min-height: 0; }
  .stage.roleplay-active {
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: minmax(150px, 32vh) minmax(0, 1fr);
  }
  .roleplay-active .model-area { grid-column: 1; grid-row: 1; padding: 8px; }
  .roleplay-active .roleplay-dialogue-area { grid-column: 1; grid-row: 2; padding: 8px; }

  .dialogue-area {
    padding: 12px;
  }

  .dialogue-text {
    font-size: 16px;
  }

  .story-content-row { grid-template-columns: 34px minmax(0, 1fr); }
  .content-status { display: none; }
}
</style>
