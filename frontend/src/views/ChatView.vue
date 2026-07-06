<template>
  <div class="chat-container">
    <div class="chat-header">
      <button class="back-btn" @click="$router.push('/')">Back</button>
      <div class="character-info" v-if="selectedCharacter">
        <span class="char-name">{{ selectedCharacter.name }}</span>
        <span class="char-emotion">{{ currentEmotion }}</span>
        <span class="relationship" :class="relationshipClass">Relation: {{ relationshipScore.toFixed(1) }}</span>
      </div>
      <div class="header-actions">
        <button class="btn btn-secondary btn-sm" @click="showEvaluation = !showEvaluation">Score</button>
        <button class="btn btn-secondary btn-sm" @click="clearChat">Clear</button>
      </div>
    </div>

    <div class="chat-body">
      <!-- Character selector -->
      <div v-if="!selectedCharacter" class="character-select">
        <h2>Choose a character to chat with</h2>
        <div class="char-grid">
          <div
            v-for="char in characters"
            :key="char.id"
            class="char-card"
            @click="selectCharacter(char)"
          >
            <div class="char-avatar">{{ char.name[0] }}</div>
            <div class="char-name">{{ char.name }}</div>
            <div class="char-desc">{{ char.description }}</div>
          </div>
        </div>
      </div>

      <!-- Chat messages -->
      <div v-else class="messages-area" ref="messagesRef">
        <div
          v-for="(msg, i) in messages"
          :key="i"
          class="message"
          :class="msg.role === 'player' ? 'msg-player' : 'msg-character'"
        >
          <div class="msg-avatar">{{ msg.role === 'player' ? 'P' : selectedCharacter.name[0] }}</div>
          <div class="msg-bubble">
            <div class="msg-content">{{ msg.content }}</div>
            <div class="msg-meta">
              <span v-if="msg.emotion" class="msg-emotion">{{ msg.emotion }}</span>
              <span class="msg-time">{{ formatTime(msg.timestamp) }}</span>
            </div>
          </div>
        </div>
        <div v-if="isLoading" class="message msg-character">
          <div class="msg-avatar">{{ selectedCharacter.name[0] }}</div>
          <div class="msg-bubble typing">
            <span class="dot"></span><span class="dot"></span><span class="dot"></span>
          </div>
        </div>
      </div>

      <!-- Input area -->
      <div v-if="selectedCharacter" class="input-area">
        <textarea
          ref="inputRef"
          v-model="inputText"
          @keydown.enter.exact.prevent="sendMessage"
          placeholder="Type your message..."
          rows="1"
        ></textarea>
        <button class="send-btn" @click="sendMessage" :disabled="!inputText.trim() || isLoading">
          Send
        </button>
      </div>
    </div>

    <!-- Event notification -->
    <Transition name="fade">
      <div v-if="activeEvent" class="event-toast" @click="activeEvent = null">
        <div class="event-icon">&#9733;</div>
        <div class="event-text">
          <strong>{{ activeEvent.description }}</strong>
          <p v-if="activeEvent.data.unlock_scene">New scene unlocked!</p>
          <p v-if="activeEvent.data.dialogue_id">Special dialogue available!</p>
        </div>
      </div>
    </Transition>

    <!-- Evaluation panel -->
    <Transition name="slide">
      <div v-if="showEvaluation && evaluation" class="eval-panel">
        <h3>Conversation Score</h3>
        <div class="eval-bar">
          <label>Friendliness</label>
          <div class="bar-track"><div class="bar-fill" :style="{ width: evaluation.friendliness * 100 + '%' }"></div></div>
          <span>{{ (evaluation.friendliness * 100).toFixed(0) }}%</span>
        </div>
        <div class="eval-bar">
          <label>Engagement</label>
          <div class="bar-track"><div class="bar-fill engagement" :style="{ width: evaluation.engagement * 100 + '%' }"></div></div>
          <span>{{ (evaluation.engagement * 100).toFixed(0) }}%</span>
        </div>
        <div class="eval-bar">
          <label>Creativity</label>
          <div class="bar-track"><div class="bar-fill creativity" :style="{ width: evaluation.creativity * 100 + '%' }"></div></div>
          <span>{{ (evaluation.creativity * 100).toFixed(0) }}%</span>
        </div>
        <p class="eval-summary">{{ evaluation.summary }}</p>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface ChatMessage {
  role: string
  content: string
  emotion: string | null
  timestamp: string
}
interface ChatResponse {
  character_response: string
  emotion: string
  relationship_delta: number
  evaluation: ConversationEvaluation | null
  triggered_events: TriggeredEvent[]
}
interface ConversationEvaluation {
  friendliness: number
  engagement: number
  creativity: number
  overall_score: number
  summary: string
}
interface TriggeredEvent {
  event_id: string
  event_type: string
  description: string
  data: Record<string, any>
}
interface CharacterInfo {
  id: string
  name: string
  description: string
  emotion: string
  live2d_model_path: string | null
}

const characters = ref<CharacterInfo[]>([])
const selectedCharacter = ref<CharacterInfo | null>(null)
const messages = ref<ChatMessage[]>([])
const inputText = ref('')
const isLoading = ref(false)
const currentEmotion = ref('neutral')
const relationshipScore = ref(0)
const evaluation = ref<ConversationEvaluation | null>(null)
const showEvaluation = ref(false)
const activeEvent = ref<TriggeredEvent | null>(null)
const messagesRef = ref<HTMLDivElement>()
const inputRef = ref<HTMLTextAreaElement>()

const relationshipClass = ref('')

function formatTime(ts: string): string {
  try { return new Date(ts).toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' }) }
  catch { return '' }
}

function scrollToBottom() {
  nextTick(() => {
    if (messagesRef.value) messagesRef.value.scrollTop = messagesRef.value.scrollHeight
  })
}

async function selectCharacter(char: CharacterInfo) {
  selectedCharacter.value = char
  currentEmotion.value = char.emotion || 'neutral'
  try {
    messages.value = await invoke('get_chat_history', { characterId: char.id })
    relationshipScore.value = await invoke('get_relationship_score', { characterId: char.id })
    updateRelationshipClass()
  } catch (e) { console.error(e) }
  scrollToBottom()
  nextTick(() => inputRef.value?.focus())
}

async function sendMessage() {
  if (!inputText.value.trim() || !selectedCharacter.value || isLoading.value) return
  const text = inputText.value.trim()
  inputText.value = ''
  messages.value.push({ role: 'player', content: text, emotion: null, timestamp: new Date().toISOString() })
  scrollToBottom()
  isLoading.value = true
  try {
    const resp = await invoke<ChatResponse>('send_chat_message', {
      characterId: selectedCharacter.value.id,
      message: text,
    })
    messages.value.push({
      role: 'character',
      content: resp.character_response,
      emotion: resp.emotion,
      timestamp: new Date().toISOString(),
    })
    currentEmotion.value = resp.emotion
    relationshipScore.value += resp.relationship_delta
    updateRelationshipClass()
    if (resp.evaluation) evaluation.value = resp.evaluation
    if (resp.triggered_events.length > 0) activeEvent.value = resp.triggered_events[0]
  } catch (e: any) {
    messages.value.push({ role: 'character', content: `Error: ${e}`, emotion: null, timestamp: new Date().toISOString() })
  } finally {
    isLoading.value = false
    scrollToBottom()
  }
}

function updateRelationshipClass() {
  if (relationshipScore.value >= 0.6) relationshipClass.value = 'rel-high'
  else if (relationshipScore.value >= 0.3) relationshipClass.value = 'rel-mid'
  else relationshipClass.value = 'rel-low'
}

async function clearChat() {
  if (!selectedCharacter.value) return
  await invoke('clear_chat_history', { characterId: selectedCharacter.value.id })
  messages.value = []
  evaluation.value = null
}

onMounted(async () => {
  try { characters.value = await invoke('get_characters') } catch (e) { console.error(e) }
})
</script>

<style scoped>
.chat-container { height: 100vh; display: flex; flex-direction: column; background: var(--bg-dark, #0a0a1a); }
.chat-header { display: flex; align-items: center; justify-content: space-between; padding: 10px 20px; background: var(--bg-card, #16213e); border-bottom: 1px solid var(--border, #2a2a4a); }
.back-btn { background: none; border: 1px solid var(--border, #2a2a4a); color: var(--text-muted, #888); padding: 6px 12px; border-radius: 6px; cursor: pointer; }
.back-btn:hover { border-color: var(--primary, #6c5ce7); color: var(--primary, #6c5ce7); }
.character-info { display: flex; align-items: center; gap: 12px; }
.char-name { font-weight: 600; color: var(--primary, #6c5ce7); font-size: 16px; }
.char-emotion { font-size: 13px; color: var(--text-muted, #888); padding: 2px 8px; background: rgba(108,92,231,0.1); border-radius: 10px; }
.relationship { font-size: 13px; padding: 2px 8px; border-radius: 10px; }
.rel-high { background: rgba(0,184,148,0.15); color: #00b894; }
.rel-mid { background: rgba(253,203,110,0.15); color: #fdcb6e; }
.rel-low { background: rgba(225,112,85,0.15); color: #e17055; }
.header-actions { display: flex; gap: 8px; }
.btn-sm { padding: 5px 10px; font-size: 12px; }
.chat-body { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
.character-select { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 40px; }
.character-select h2 { color: var(--primary, #6c5ce7); margin-bottom: 24px; }
.char-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 16px; max-width: 800px; }
.char-card { background: var(--bg-card, #16213e); border: 1px solid var(--border, #2a2a4a); border-radius: 12px; padding: 20px; text-align: center; cursor: pointer; transition: all 0.2s; }
.char-card:hover { border-color: var(--primary, #6c5ce7); transform: translateY(-2px); box-shadow: 0 4px 16px rgba(108,92,231,0.2); }
.char-avatar { width: 56px; height: 56px; border-radius: 50%; background: linear-gradient(135deg, var(--primary, #6c5ce7), var(--secondary, #00cec9)); display: flex; align-items: center; justify-content: center; margin: 0 auto 12px; font-size: 24px; color: white; font-weight: bold; }
.char-card .char-name { font-weight: 600; color: var(--text, #e0e0e0); margin-bottom: 4px; }
.char-desc { font-size: 12px; color: var(--text-muted, #888); }
.messages-area { flex: 1; overflow-y: auto; padding: 20px; display: flex; flex-direction: column; gap: 12px; }
.message { display: flex; gap: 10px; max-width: 70%; animation: fadeIn 0.3s ease; }
.msg-player { align-self: flex-end; flex-direction: row-reverse; }
.msg-character { align-self: flex-start; }
.msg-avatar { width: 36px; height: 36px; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-weight: bold; font-size: 14px; flex-shrink: 0; }
.msg-player .msg-avatar { background: var(--primary, #6c5ce7); color: white; }
.msg-character .msg-avatar { background: var(--secondary, #00cec9); color: white; }
.msg-bubble { padding: 12px 16px; border-radius: 16px; }
.msg-player .msg-bubble { background: rgba(108,92,231,0.2); border: 1px solid rgba(108,92,231,0.3); border-bottom-right-radius: 4px; }
.msg-character .msg-bubble { background: rgba(22,33,62,0.95); border: 1px solid var(--border, #2a2a4a); border-bottom-left-radius: 4px; }
.msg-content { font-size: 15px; line-height: 1.6; color: var(--text, #e0e0e0); }
.msg-meta { display: flex; gap: 8px; margin-top: 6px; font-size: 11px; color: var(--text-muted, #666); }
.msg-emotion { padding: 1px 6px; background: rgba(0,206,201,0.15); border-radius: 8px; }
.typing { display: flex; gap: 4px; align-items: center; padding: 16px 20px; }
.dot { width: 8px; height: 8px; border-radius: 50%; background: var(--text-muted, #888); animation: bounce 1.4s infinite; }
.dot:nth-child(2) { animation-delay: 0.2s; }
.dot:nth-child(3) { animation-delay: 0.4s; }
.input-area { display: flex; gap: 10px; padding: 16px 20px; background: var(--bg-card, #16213e); border-top: 1px solid var(--border, #2a2a4a); }
.input-area textarea { flex: 1; background: var(--bg-input, #1a1a3e); border: 1px solid var(--border, #2a2a4a); border-radius: 12px; padding: 10px 14px; color: var(--text, #e0e0e0); font-size: 15px; resize: none; outline: none; font-family: inherit; }
.input-area textarea:focus { border-color: var(--primary, #6c5ce7); }
.send-btn { padding: 10px 24px; background: var(--primary, #6c5ce7); color: white; border: none; border-radius: 12px; cursor: pointer; font-size: 15px; font-weight: 600; transition: all 0.2s; }
.send-btn:hover:not(:disabled) { background: #5a4bd1; }
.send-btn:disabled { opacity: 0.5; cursor: not-allowed; }
.event-toast { position: fixed; top: 20px; left: 50%; transform: translateX(-50%); background: linear-gradient(135deg, rgba(108,92,231,0.95), rgba(0,206,201,0.95)); color: white; padding: 16px 24px; border-radius: 16px; display: flex; align-items: center; gap: 12px; z-index: 200; cursor: pointer; box-shadow: 0 8px 32px rgba(0,0,0,0.4); max-width: 500px; animation: slideDown 0.5s ease; }
.event-icon { font-size: 28px; }
.event-text strong { display: block; margin-bottom: 4px; }
.event-text p { font-size: 13px; opacity: 0.9; margin: 0; }
.eval-panel { position: fixed; right: 0; top: 60px; width: 300px; background: var(--bg-card, #16213e); border-left: 1px solid var(--border, #2a2a4a); padding: 20px; z-index: 50; height: calc(100vh - 60px); overflow-y: auto; }
.eval-panel h3 { color: var(--primary, #6c5ce7); margin-bottom: 16px; }
.eval-bar { margin-bottom: 14px; }
.eval-bar label { display: block; font-size: 12px; color: var(--text-muted, #888); margin-bottom: 4px; }
.bar-track { height: 8px; background: var(--bg-input, #1a1a3e); border-radius: 4px; overflow: hidden; }
.bar-fill { height: 100%; background: var(--primary, #6c5ce7); border-radius: 4px; transition: width 0.5s ease; }
.bar-fill.engagement { background: var(--secondary, #00cec9); }
.bar-fill.creativity { background: #fdcb6e; }
.eval-bar span { font-size: 12px; color: var(--text-muted, #888); }
.eval-summary { font-size: 13px; color: var(--text-muted, #888); margin-top: 16px; padding-top: 12px; border-top: 1px solid var(--border, #2a2a4a); }
@keyframes fadeIn { from { opacity: 0; transform: translateY(8px); } to { opacity: 1; transform: translateY(0); } }
@keyframes bounce { 0%, 80%, 100% { transform: translateY(0); } 40% { transform: translateY(-6px); } }
@keyframes slideDown { from { opacity: 0; transform: translate(-50%, -20px); } to { opacity: 1; transform: translate(-50%, 0); } }
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
.slide-enter-active, .slide-leave-active { transition: transform 0.3s ease; }
.slide-enter-from, .slide-leave-to { transform: translateX(100%); }
</style>
