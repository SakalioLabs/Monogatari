<template>
  <div class="group-chat">
    <header class="group-chat-header">
      <h2>Group Chat</h2>
      <div class="session-controls">
        <button v-if="!session" class="btn btn-primary" :disabled="selectedIds.length < 2" @click="startSession">Start Group Chat</button>
        <button v-else class="btn btn-danger" @click="endSession">End Session</button>
      </div>
    </header>

    <div v-if="!session" class="character-select">
      <p class="select-hint">Select 2 or more characters to start a group conversation.</p>
      <div class="character-grid">
        <div v-for="c in available" :key="c[0]" class="char-card" :class="{ selected: selectedIds.includes(c[0]) }" @click="toggle(c[0])">
          <div class="char-avatar">{{ c[1].charAt(0) }}</div>
          <span class="char-name">{{ c[1] }}</span>
        </div>
      </div>
    </div>

    <div v-else class="chat-area">
      <div class="participants-bar">
        <span v-for="id in session.character_ids" :key="id" class="participant-tag">
          <span class="participant-dot"></span>{{ id }}
          <span v-if="relationshipScores[id]" class="rel-score">{{ relationshipScores[id]?.toFixed(1) }}</span>
        </span>
      </div>
      <div class="messages" ref="messagesEl">
        <div v-for="(m, i) in session.messages" :key="i" class="message" :class="m.role">
          <div class="msg-sender">{{ m.character_name }}</div>
          <div class="msg-content">{{ m.content }}</div>
          <div v-if="m.emotion" class="msg-emotion">
            <span class="emotion-dot"></span>{{ m.emotion }}
          </div>
          <div v-if="m.safety_trace" class="group-safety-trace">
            <span
              v-for="flag in groupSafetyFlags(m.safety_trace)"
              :key="flag.key"
              class="group-safety-chip"
              :class="{ active: flag.active }"
            >{{ flag.label }}</span>
            <small>{{ groupSafetySummary(m.safety_trace) }}</small>
          </div>
        </div>
        <div v-if="loading" class="message system">
          <div class="msg-content"><span class="spinner"></span> Characters are thinking...</div>
        </div>
      </div>
      <div class="input-area">
        <input v-model="input" class="input" placeholder="Type a message to the group..." @keyup.enter="send" :disabled="loading" />
        <button class="btn btn-primary" @click="send" :disabled="!input.trim() || loading">{{ t("group.send", "Send") }}</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, onMounted, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { invokeCommand } from '../lib/tauri'
import { useI18n } from '../lib/i18n'

const { t } = useI18n()

interface GroupMessage {
  role: string
  character_id: string | null
  character_name: string
  content: string
  emotion: string | null
  timestamp: string
  safety_trace?: ChatSafetyTrace | null
}

interface ChatSafetyTrace {
  input_wrapped_as_untrusted: boolean
  input_prompt_injection_detected: boolean
  input_private_reasoning_request_detected: boolean
  response_guard_applied: boolean
  private_reasoning_blocked: boolean
  identity_drift_blocked: boolean
  style_drift_blocked: boolean
  memory_guard_applied: boolean
  relationship_delta_blocked: boolean
  stream_guard_applied: boolean
  guard_notes: string[]
}

interface GroupSession {
  session_id: string
  character_ids: string[]
  messages: GroupMessage[]
  active: boolean
}

const available = ref<[string, string][]>([])
const selectedIds = ref<string[]>([])
const session = ref<GroupSession | null>(null)
const input = ref('')
const loading = ref(false)
const messagesEl = ref<HTMLElement>()
const currentEmotion = ref('neutral')
const relationshipScores = ref<Record<string, number>>({})
let streamUnlisteners: UnlistenFn[] = []

async function loadCharacters() {
  try {
    available.value = await invokeCommand<[string, string][]>('get_group_chat_characters', {}, [])
  } catch { available.value = [] }
}
loadCharacters()

function cleanupStreamListeners() {
  for (const u of streamUnlisteners) u()
  streamUnlisteners = []
}

async function attachGroupStreamListeners() {
  cleanupStreamListeners()
  streamUnlisteners = await Promise.all([
    listen<string>('chat-chunk', (event) => {
      if (session.value && session.value.messages.length > 0) {
        const lastMsg = session.value.messages[session.value.messages.length - 1]
        if (lastMsg.role === 'character') {
          lastMsg.content += event.payload
        }
      }
    }),
    listen<string>('chat-emotion', (event) => {
      currentEmotion.value = event.payload || currentEmotion.value
    }),
  ])
}

onUnmounted(cleanupStreamListeners)

function toggle(id: string) {
  const idx = selectedIds.value.indexOf(id)
  if (idx >= 0) selectedIds.value.splice(idx, 1)
  else selectedIds.value.push(id)
}

async function startSession() {
  if (selectedIds.value.length < 2) return
  try {
    session.value = await invokeCommand<GroupSession>('start_group_chat', { characterIds: selectedIds.value })
  } catch (e) { console.error(e) }
}

function endSession() { session.value = null }

function groupSafetyFlags(trace: ChatSafetyTrace) {
  return [
    { key: 'input', label: 'Input', active: trace.input_prompt_injection_detected || trace.input_private_reasoning_request_detected },
    { key: 'response', label: 'Response', active: trace.response_guard_applied },
    { key: 'memory', label: 'Memory', active: trace.memory_guard_applied },
    { key: 'relation', label: 'Relation', active: trace.relationship_delta_blocked },
  ]
}

function groupSafetySummary(trace: ChatSafetyTrace) {
  const notes = trace.guard_notes || []
  if (!notes.length || notes.includes('no_runtime_safety_interventions')) return 'No interventions'
  return notes.map(formatSafetyNote).join(' / ')
}

function formatSafetyNote(note: string) {
  return note
    .replace(/_/g, ' ')
    .replace(/\b\w/g, (ch) => ch.toUpperCase())
}

async function send() {
  if (!input.value.trim() || !session.value || loading.value) return
  loading.value = true
  try {
    session.value = await invokeCommand<GroupSession>('send_group_message', { session: session.value, message: input.value })
    input.value = ''
    await nextTick()
    if (messagesEl.value) messagesEl.value.scrollTop = messagesEl.value.scrollHeight
  } catch (e) { console.error(e) }
  loading.value = false
}
</script>

<style scoped>
.group-chat { height: 100%; display: flex; flex-direction: column; padding: 24px; gap: 16px; }
.group-chat-header { display: flex; align-items: center; justify-content: space-between; }
.group-chat-header h2 { font-size: 20px; font-weight: 700; }
.select-hint { color: var(--text-secondary); margin-bottom: 16px; font-size: 13px; }
.character-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(140px, 1fr)); gap: 12px; }
.char-card { display: flex; flex-direction: column; align-items: center; gap: 8px; padding: 16px; border-radius: var(--radius); border: 1px solid var(--border); background: var(--surface-1); cursor: pointer; transition: all var(--transition-fast); }
.char-card:hover { border-color: var(--border-light); background: var(--surface-2); }
.char-card.selected { border-color: var(--brand); background: rgba(45,212,191,0.08); }
.char-avatar { width: 44px; height: 44px; border-radius: 50%; background: var(--surface-3); display: flex; align-items: center; justify-content: center; font-weight: 700; font-size: 18px; color: var(--brand-light); }
.char-name { font-size: 13px; font-weight: 600; }
.chat-area { flex: 1; display: flex; flex-direction: column; gap: 12px; min-height: 0; }
.participants-bar { display: flex; gap: 6px; flex-wrap: wrap; }
.participant-tag { padding: 3px 10px; border-radius: 100px; font-size: 11px; font-weight: 600; background: var(--surface-3); color: var(--text-secondary); text-transform: uppercase; }
.messages { flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 10px; padding: 8px 0; }
.message { max-width: 80%; padding: 10px 14px; border-radius: var(--radius); }
.message.player { align-self: flex-end; background: rgba(45,212,191,0.12); border: 1px solid rgba(45,212,191,0.2); }
.message.character { align-self: flex-start; background: var(--surface-2); border: 1px solid var(--border); }
.message.system { align-self: center; background: var(--surface-3); font-style: italic; }
.msg-sender { font-size: 11px; font-weight: 700; color: var(--brand-light); margin-bottom: 4px; text-transform: uppercase; }
.msg-content { font-size: 13px; line-height: 1.6; }
.msg-emotion { font-size: 10px; color: var(--text-tertiary); margin-top: 4px; text-transform: uppercase; }
.group-safety-trace {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 5px;
  margin-top: 7px;
}
.group-safety-chip {
  padding: 2px 6px;
  border-radius: 999px;
  background: var(--surface-3);
  color: var(--text-tertiary);
  font-size: 9px;
  font-weight: 800;
  text-transform: uppercase;
}
.group-safety-chip.active {
  background: rgba(245,158,11,0.14);
  color: var(--warning);
}
.group-safety-trace small {
  flex-basis: 100%;
  color: var(--text-tertiary);
  font-size: 10px;
  line-height: 1.45;
  overflow-wrap: anywhere;
}
.input-area { display: flex; gap: 8px; }
.input-area .input { flex: 1; }
.participant-dot {
  display: inline-block; width: 6px; height: 6px; border-radius: 50%;
  background: var(--success); margin-right: 4px;
}
.rel-score {
  margin-left: 6px; padding: 1px 6px; border-radius: 100px;
  font-size: 9px; font-weight: 800; background: rgba(45,212,191,0.12);
  color: var(--brand-light);
}
.emotion-dot {
  display: inline-block; width: 5px; height: 5px; border-radius: 50%;
  background: var(--warning); margin-right: 4px;
}
.spinner {
  display: inline-block; width: 14px; height: 14px;
  border: 2px solid rgba(255,255,255,0.3); border-top-color: white;
  border-radius: 50%; animation: spin 0.8s linear infinite;
  margin-right: 8px; vertical-align: middle;
}
@keyframes spin { to { transform: rotate(360deg); } }
</style>
