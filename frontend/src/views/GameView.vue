<template>
  <div class="game-container">
    <!-- Background Layer -->
    <div class="background-layer">
      <div class="bg-gradient"></div>
    </div>

    <!-- Live2D Model Area -->
    <div class="model-area">
      <Live2DCanvas
        v-if="currentCharacter"
        :model-path="currentCharacter.live2d_model_path"
        :expression="currentExpression"
        :motion="currentMotion"
      />
      <div v-else class="placeholder">
        <div class="placeholder-icon">🎭</div>
        <p>等待加载角色...</p>
      </div>
    </div>

    <!-- Dialogue Area -->
    <div class="dialogue-area">
      <div v-if="dialogueState?.is_active" class="dialogue-box">
        <div class="speaker-name" v-if="dialogueState.speaker">
          <span class="speaker-icon">💬</span>
          {{ dialogueState.speaker }}
        </div>
        <div class="dialogue-text">
          {{ displayedText }}
          <span v-if="isTyping" class="cursor">▎</span>
        </div>

        <!-- Choices -->
        <div v-if="dialogueState.choices.length > 0" class="choices">
          <button
            v-for="(choice, idx) in dialogueState.choices"
            :key="choice.index"
            class="choice-btn"
            :style="{ animationDelay: `${idx * 0.1}s` }"
            @click="selectChoice(choice.index)"
          >
            <span class="choice-number">{{ idx + 1 }}</span>
            {{ choice.text }}
          </button>
        </div>

        <!-- Advance hint -->
        <div v-else class="advance-hint" @click="advanceDialogue">
          <span class="hint-icon">▼</span>
          点击或按空格继续
        </div>
      </div>

      <!-- No dialogue state -->
      <div v-else class="no-dialogue">
        <div class="welcome-card">
          <h2>🎮 LLM Galgame Engine</h2>
          <p>AI驱动的视觉小说引擎</p>
          <button class="btn btn-primary btn-lg" @click="startDemoDialogue" :disabled="isLoading">
            <span v-if="isLoading" class="loading-spinner"></span>
            <span v-else>▶</span>
            {{ isLoading ? '加载中...' : '开始演示' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Error Message -->
    <Transition name="fade">
      <div v-if="errorMessage" class="error-toast" @click="errorMessage = null">
        <span class="error-icon">⚠️</span>
        {{ errorMessage }}
        <span class="error-close">✕</span>
      </div>
    </Transition>

    <!-- Controls -->
    <div class="game-controls">
      <button class="control-btn" @click="$router.push('/')" title="返回主页">
        🏠
      </button>
      <button class="control-btn" @click="saveGame" title="保存游戏">
        💾
      </button>
      <button class="control-btn" @click="showLoadDialog = true" title="加载游戏">
        📂
      </button>
      <button class="control-btn" @click="toggleSettings" title="设置">
        ⚙️
      </button>
    </div>

    <!-- Load Dialog -->
    <Transition name="fade">
      <div v-if="showLoadDialog" class="modal-overlay" @click.self="showLoadDialog = false">
        <div class="card modal">
          <h3>📂 加载游戏</h3>
          <div class="save-list">
            <div
              v-for="save in saves"
              :key="save.save_id"
              class="save-item"
              @click="loadGame(save.save_id)"
            >
              <div class="save-icon">💾</div>
              <div class="save-info">
                <span class="save-name">{{ save.save_name }}</span>
                <span class="save-time">{{ formatTime(save.timestamp) }}</span>
              </div>
            </div>
            <p v-if="saves.length === 0" class="no-saves">
              <span>📭</span> 暂无存档
            </p>
          </div>
          <button class="btn btn-secondary" @click="showLoadDialog = false">关闭</button>
        </div>
      </div>
    </Transition>

    <!-- Settings Panel -->
    <Transition name="slide">
      <div v-if="showSettings" class="settings-panel">
        <div class="settings-header">
          <h3>⚙️ 设置</h3>
          <button class="close-btn" @click="showSettings = false">✕</button>
        </div>
        <div class="settings-content">
          <div class="setting-group">
            <label>文字速度</label>
            <input type="range" v-model="settings.textSpeed" min="10" max="100" />
            <span>{{ settings.textSpeed }}ms</span>
          </div>
          <div class="setting-group">
            <label>自动播放</label>
            <label class="switch">
              <input type="checkbox" v-model="settings.autoPlay" />
              <span class="slider"></span>
            </label>
          </div>
          <div class="setting-group">
            <label>BGM音量</label>
            <input type="range" v-model="settings.bgmVolume" min="0" max="100" />
            <span>{{ settings.bgmVolume }}%</span>
          </div>
          <div class="setting-group">
            <label>音效音量</label>
            <input type="range" v-model="settings.sfxVolume" min="0" max="100" />
            <span>{{ settings.sfxVolume }}%</span>
          </div>
          <div class="setting-group">
            <label>语音音量</label>
            <input type="range" v-model="settings.voiceVolume" min="0" max="100" />
            <span>{{ settings.voiceVolume }}%</span>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import Live2DCanvas from '../components/Live2DCanvas.vue'

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
}

interface SaveInfo {
  save_id: string
  save_name: string
  timestamp: string
  current_scene: string | null
}

const dialogueState = ref<DialogueState | null>(null)
const currentCharacter = ref<CharacterInfo | null>(null)
const currentExpression = ref<string>('neutral')
const currentMotion = ref<string>('idle')
const displayedText = ref('')
const isTyping = ref(false)
const showLoadDialog = ref(false)
const showSettings = ref(false)
const saves = ref<SaveInfo[]>([])
const errorMessage = ref<string | null>(null)
const isLoading = ref(false)

// Settings
const settings = ref({
  textSpeed: 30,
  autoPlay: false,
  autoPlaySpeed: 3000,
  bgmVolume: 80,
  sfxVolume: 80,
  voiceVolume: 80,
})

let typingTimer: number | null = null

function formatTime(timestamp: string): string {
  try {
    const date = new Date(timestamp)
    return date.toLocaleString('zh-CN')
  } catch {
    return timestamp
  }
}

async function updateDialogueState() {
  try {
    dialogueState.value = await invoke('get_dialogue_state')
    if (dialogueState.value?.live2d_expression) {
      currentExpression.value = dialogueState.value.live2d_expression
    }
    if (dialogueState.value?.text) {
      typewriterEffect(dialogueState.value.text)
    }
  } catch (e) {
    console.error('Failed to get dialogue state:', e)
  }
}

function typewriterEffect(text: string) {
  if (typingTimer) clearInterval(typingTimer)
  displayedText.value = ''
  isTyping.value = true
  let i = 0
  typingTimer = window.setInterval(() => {
    if (i < text.length) {
      displayedText.value += text[i]
      i++
    } else {
      if (typingTimer) clearInterval(typingTimer)
      isTyping.value = false
    }
  }, 30)
}

async function startDemoDialogue() {
  isLoading.value = true
  errorMessage.value = null
  try {
    await invoke('start_dialogue', { dialogueId: 'meeting_sakura' })
    await updateDialogueState()
  } catch (e) {
    console.error('Failed to start dialogue:', e)
    errorMessage.value = `无法开始对话: ${e}`
  } finally {
    isLoading.value = false
  }
}

async function selectChoice(index: number) {
  try {
    await invoke('select_choice', { choiceIndex: index })
    await updateDialogueState()
  } catch (e) {
    console.error('Failed to select choice:', e)
  }
}

async function advanceDialogue() {
  if (isTyping.value) {
    // Skip typing animation
    if (typingTimer) clearInterval(typingTimer)
    isTyping.value = false
    if (dialogueState.value?.text) {
      displayedText.value = dialogueState.value.text
    }
    return
  }
  try {
    await invoke('advance_dialogue')
    await updateDialogueState()
  } catch (e) {
    console.error('Failed to advance dialogue:', e)
  }
}

async function saveGame() {
  try {
    const name = `存档 ${new Date().toLocaleString('zh-CN')}`
    const saveId = await invoke<string>('save_game', { saveName: name })
    alert(`游戏已保存: ${saveId}`)
  } catch (e) {
    console.error('Failed to save:', e)
  }
}

async function loadGame(saveId: string) {
  try {
    await invoke('load_game', { saveId })
    showLoadDialog.value = false
    await updateDialogueState()
  } catch (e) {
    console.error('Failed to load:', e)
  }
}

async function loadSaves() {
  try {
    saves.value = await invoke('list_saves')
  } catch (e) {
    console.error('Failed to list saves:', e)
  }
}

function toggleSettings() {
  showSettings.value = !showSettings.value
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === ' ' || e.key === 'Enter') {
    e.preventDefault()
    advanceDialogue()
  }
}

onMounted(() => {
  updateDialogueState()
  loadSaves()
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
  if (typingTimer) clearInterval(typingTimer)
})
</script>

<style scoped>
.game-container {
  height: 100vh;
  display: flex;
  flex-direction: column;
  position: relative;
  overflow: hidden;
}

.background-layer {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 0;
}

.bg-gradient {
  width: 100%;
  height: 100%;
  background: linear-gradient(180deg, #0a0a1a 0%, #1a1a3e 50%, #0a0a1a 100%);
}

.model-area {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  z-index: 1;
}

.placeholder {
  text-align: center;
  color: var(--text-muted);
}

.placeholder-icon {
  font-size: 64px;
  margin-bottom: 16px;
  opacity: 0.5;
}

.dialogue-area {
  padding: 20px;
  position: relative;
  z-index: 2;
}

.dialogue-box {
  background: rgba(22, 33, 62, 0.95);
  border: 1px solid rgba(108, 92, 231, 0.3);
  border-radius: 16px;
  padding: 24px;
  max-width: 800px;
  margin: 0 auto;
  backdrop-filter: blur(10px);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
}

.speaker-name {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
  color: var(--primary);
  margin-bottom: 12px;
  padding: 4px 12px;
  background: rgba(108, 92, 231, 0.1);
  border-radius: 8px;
}

.speaker-icon {
  font-size: 14px;
}

.dialogue-text {
  font-size: 18px;
  line-height: 1.8;
  min-height: 60px;
  color: #e0e0e0;
}

.cursor {
  color: var(--primary);
  animation: blink 0.8s infinite;
  font-weight: bold;
}

@keyframes blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0; }
}

.choices {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-top: 20px;
}

.choice-btn {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 20px;
  background: rgba(108, 92, 231, 0.1);
  border: 1px solid rgba(108, 92, 231, 0.3);
  border-radius: 12px;
  color: var(--text);
  cursor: pointer;
  text-align: left;
  transition: all 0.3s;
  animation: slideIn 0.3s ease forwards;
  opacity: 0;
  transform: translateX(-20px);
}

@keyframes slideIn {
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

.choice-btn:hover {
  background: rgba(108, 92, 231, 0.3);
  border-color: var(--primary);
  transform: translateX(4px);
}

.choice-number {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: var(--primary);
  border-radius: 50%;
  font-size: 14px;
  font-weight: bold;
  flex-shrink: 0;
}

.advance-hint {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  margin-top: 16px;
  color: var(--text-muted);
  font-size: 14px;
  cursor: pointer;
  transition: color 0.2s;
}

.advance-hint:hover {
  color: var(--primary);
}

.hint-icon {
  animation: bounce 1.5s infinite;
}

@keyframes bounce {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(4px); }
}

.no-dialogue {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px;
}

.welcome-card {
  text-align: center;
  background: rgba(22, 33, 62, 0.9);
  border: 1px solid rgba(108, 92, 231, 0.3);
  border-radius: 20px;
  padding: 40px 60px;
  backdrop-filter: blur(10px);
}

.welcome-card h2 {
  font-size: 28px;
  margin-bottom: 12px;
  background: linear-gradient(135deg, var(--primary), var(--secondary));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.welcome-card p {
  color: var(--text-muted);
  margin-bottom: 24px;
}

.btn-lg {
  padding: 14px 32px;
  font-size: 18px;
}

.game-controls {
  display: flex;
  gap: 8px;
  padding: 12px 20px;
  background: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(10px);
  position: relative;
  z-index: 2;
}

.control-btn {
  padding: 8px 16px;
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  color: var(--text);
  cursor: pointer;
  font-size: 18px;
  transition: all 0.2s;
}

.control-btn:hover {
  background: rgba(108, 92, 231, 0.3);
  border-color: var(--primary);
}

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  backdrop-filter: blur(4px);
}

.modal {
  width: 420px;
  max-height: 500px;
  overflow-y: auto;
}

.modal h3 {
  margin-bottom: 20px;
  color: var(--primary);
  font-size: 20px;
}

.save-list {
  margin-bottom: 20px;
}

.save-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px;
  border-bottom: 1px solid var(--border);
  cursor: pointer;
  transition: background 0.2s;
  border-radius: 8px;
}

.save-item:hover {
  background: rgba(108, 92, 231, 0.1);
}

.save-icon {
  font-size: 24px;
}

.save-info {
  display: flex;
  flex-direction: column;
}

.save-name {
  font-weight: 500;
}

.save-time {
  font-size: 12px;
  color: var(--text-muted);
}

.no-saves {
  text-align: center;
  color: var(--text-muted);
  padding: 30px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.no-saves span {
  font-size: 32px;
  opacity: 0.5;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

/* Settings Panel */
.settings-panel {
  position: fixed;
  top: 0;
  right: 0;
  width: 320px;
  height: 100vh;
  background: rgba(22, 33, 62, 0.98);
  border-left: 1px solid var(--border);
  z-index: 50;
  display: flex;
  flex-direction: column;
  backdrop-filter: blur(10px);
}

.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border-bottom: 1px solid var(--border);
}

.settings-header h3 {
  color: var(--primary);
  font-size: 18px;
}

.close-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 20px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  transition: all 0.2s;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text);
}

.settings-content {
  padding: 20px;
  overflow-y: auto;
  flex: 1;
}

.setting-group {
  margin-bottom: 20px;
}

.setting-group label {
  display: block;
  font-size: 14px;
  color: var(--text-muted);
  margin-bottom: 8px;
}

.setting-group input[type="range"] {
  width: 100%;
  height: 6px;
  -webkit-appearance: none;
  background: var(--bg-input);
  border-radius: 3px;
  outline: none;
}

.setting-group input[type="range"]::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 18px;
  height: 18px;
  background: var(--primary);
  border-radius: 50%;
  cursor: pointer;
}

.setting-group span {
  display: inline-block;
  margin-top: 4px;
  font-size: 12px;
  color: var(--text-muted);
}

/* Toggle Switch */
.switch {
  position: relative;
  display: inline-block;
  width: 48px;
  height: 24px;
}

.switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--bg-input);
  transition: 0.3s;
  border-radius: 24px;
}

.slider:before {
  position: absolute;
  content: "";
  height: 18px;
  width: 18px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  transition: 0.3s;
  border-radius: 50%;
}

input:checked + .slider {
  background-color: var(--primary);
}

input:checked + .slider:before {
  transform: translateX(24px);
}

.slide-enter-active,
.slide-leave-active {
  transition: transform 0.3s ease;
}

.slide-enter-from,
.slide-leave-to {
  transform: translateX(100%);
}

/* Error Toast */
.error-toast {
  position: fixed;
  top: 20px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(225, 112, 85, 0.95);
  color: white;
  padding: 12px 20px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  gap: 10px;
  z-index: 200;
  cursor: pointer;
  backdrop-filter: blur(10px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  max-width: 500px;
}

.error-icon {
  font-size: 18px;
}

.error-close {
  margin-left: 10px;
  opacity: 0.7;
}

.error-close:hover {
  opacity: 1;
}

/* Loading Spinner */
.loading-spinner {
  display: inline-block;
  width: 16px;
  height: 16px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Disabled button */
.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
