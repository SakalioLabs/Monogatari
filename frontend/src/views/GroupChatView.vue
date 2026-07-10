<template>
  <div class="group-chat">
    <header class="page-header">
      <div>
        <span class="eyebrow">{{ t('group.eyebrow', 'Live test') }}</span>
        <h1>{{ t('group.title', 'Group Chat') }}</h1>
        <p>{{ t('group.subtitle', 'Test how multiple characters respond, relate, and stay in voice together.') }}</p>
      </div>
      <div class="session-controls">
        <button v-if="!session" class="btn btn-primary" :disabled="selectedIds.length < 2" @click="startSession">
          <Play :size="15" />
          {{ t('group.start', 'Start Group Chat') }}
        </button>
        <button v-else class="btn btn-danger" @click="endSession">
          <LogOut :size="15" />
          {{ t('group.end', 'End session') }}
        </button>
      </div>
    </header>
    <div v-if="errorMessage" class="group-error" @click="errorMessage = null">{{ errorMessage }}</div>

    <section v-if="!session" class="character-select">
      <div class="selection-head">
        <div>
          <span class="eyebrow">{{ t('group.setup', 'Session setup') }}</span>
          <strong>{{ t('group.select-chars', 'Select characters') }}</strong>
        </div>
        <span class="selection-count">{{ t('group.selected-count', '{count} selected', { count: selectedIds.length }) }}</span>
      </div>
      <p class="select-hint">{{ t('group.select-hint', 'Select at least two characters to start a group conversation.') }}</p>
      <div class="character-grid">
        <button
          v-for="character in available"
          :key="character[0]"
          class="char-card"
          :class="{ selected: selectedIds.includes(character[0]) }"
          :aria-pressed="selectedIds.includes(character[0])"
          @click="toggle(character[0])"
        >
          <span class="char-avatar">{{ character[1].charAt(0) }}</span>
          <span class="char-copy">
            <strong>{{ character[1] }}</strong>
            <small>{{ character[0] }}</small>
          </span>
          <Check v-if="selectedIds.includes(character[0])" class="selected-mark" :size="15" />
        </button>
        <div v-if="available.length === 0" class="empty-state">
          <UsersRound :size="28" />
          <span>{{ t('group.empty', 'No characters are available for group chat.') }}</span>
        </div>
      </div>
    </section>

    <section v-else class="chat-area">
      <div class="participants-bar">
        <span class="participants-label">{{ t('group.characters', 'Participants') }}</span>
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
          <div class="msg-content"><LoaderCircle class="spinner" :size="15" />{{ t('group.thinking', 'Characters are thinking...') }}</div>
        </div>
      </div>
      <div class="input-area">
        <input v-model="input" class="input" :placeholder="t('group.placeholder', 'Message the group...')" @keyup.enter="send" :disabled="loading" />
        <button class="btn btn-primary" @click="send" :disabled="!input.trim() || loading">
          <Send :size="15" />
          {{ t('group.send', 'Send') }}
        </button>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, onMounted, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { Check, LoaderCircle, LogOut, Play, Send, UsersRound } from '@lucide/vue'
import { hasTauriRuntime, invokeCommand } from '../lib/tauri'
import { useI18n } from '../lib/i18n'
import { loadStoryCharacters } from '../lib/storyContent'

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
  mind_contract_applied?: boolean
  knowledge_context_pinned?: boolean
  pinned_knowledge_ref_count?: number
  pinned_knowledge_ref_ids?: string[]
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
const errorMessage = ref<string | null>(null)
const messagesEl = ref<HTMLElement>()
const currentEmotion = ref('neutral')
const relationshipScores = ref<Record<string, number>>({})
let streamUnlisteners: UnlistenFn[] = []

async function loadCharacters() {
  try {
    errorMessage.value = null
    const characters = await loadStoryCharacters()
    available.value = characters.map(character => [character.id, character.name])
  } catch (e) {
    available.value = []
    errorMessage.value = String(e)
  }
}
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
    errorMessage.value = null
    session.value = await invokeCommand<GroupSession>('start_group_chat', { characterIds: selectedIds.value }, {
      session_id: 'browser-preview',
      character_ids: [...selectedIds.value],
      messages: [],
      active: true,
    })
  } catch (e) {
    errorMessage.value = String(e)
  }
}

function endSession() { session.value = null }

function groupSafetyFlags(trace: ChatSafetyTrace) {
  return [
    { key: 'mind', label: t('group.safety.mind', 'Mind'), active: !!trace.mind_contract_applied },
    { key: 'knowledge', label: t('group.safety.knowledge', 'Knowledge'), active: !!trace.knowledge_context_pinned },
    { key: 'input', label: t('group.safety.input', 'Input'), active: trace.input_prompt_injection_detected || trace.input_private_reasoning_request_detected },
    { key: 'response', label: t('group.safety.response', 'Response'), active: trace.response_guard_applied },
    { key: 'memory', label: t('group.safety.memory', 'Memory'), active: trace.memory_guard_applied },
    { key: 'relation', label: t('group.safety.relation', 'Relation'), active: trace.relationship_delta_blocked },
  ]
}

function groupSafetySummary(trace: ChatSafetyTrace) {
  const notes = trace.guard_notes || []
  const refSummary = trace.pinned_knowledge_ref_ids?.length
    ? t('group.safety.refs', 'Refs {refs}', { refs: trace.pinned_knowledge_ref_ids.join(', ') })
    : ''
  if (!notes.length || notes.includes('no_runtime_safety_interventions')) {
    const clear = t('group.safety.clear', 'No interventions')
    return refSummary ? `${clear} / ${refSummary}` : clear
  }
  return [...notes.map(formatSafetyNote), refSummary].filter(Boolean).join(' / ')
}

function formatSafetyNote(note: string) {
  return note
    .replace(/_/g, ' ')
    .replace(/\b\w/g, (ch) => ch.toUpperCase())
}

async function send() {
  if (!input.value.trim() || !session.value || loading.value) return
  loading.value = true
  errorMessage.value = null
  try {
    const message = input.value.trim()
    session.value = await invokeCommand<GroupSession>(
      'send_group_message',
      { session: session.value, message },
      () => browserGroupReply(session.value as GroupSession, message),
    )
    input.value = ''
    await nextTick()
    if (messagesEl.value) messagesEl.value.scrollTop = messagesEl.value.scrollHeight
  } catch (e) {
    errorMessage.value = String(e)
  } finally {
    loading.value = false
  }
}

function browserGroupReply(currentSession: GroupSession, message: string): GroupSession {
  const timestamp = new Date().toISOString()
  return {
    ...currentSession,
    messages: [
      ...currentSession.messages,
      {
        role: 'player',
        character_id: null,
        character_name: t('backlog.player', 'Player'),
        content: message,
        emotion: null,
        timestamp,
      },
      {
        role: 'system',
        character_id: null,
        character_name: t('backlog.system', 'System'),
        content: t('group.preview-response', 'Browser preview received the message. Use the desktop runtime for model responses.'),
        emotion: null,
        timestamp,
      },
    ],
  }
}

onMounted(async () => {
  await loadCharacters()
  if (hasTauriRuntime()) await attachGroupStreamListeners()
})
</script>

<style scoped>
.group-chat {
  display: flex;
  flex-direction: column;
  gap: 18px;
  width: min(1180px, 100%);
  min-height: calc(100dvh - 54px);
  margin: 0 auto;
  padding: 32px 36px 48px;
}
.page-header { display: flex; align-items: flex-start; justify-content: space-between; gap: 18px; }
.page-header h1 { margin: 3px 0 0; color: var(--text-primary); font-size: 28px; line-height: 1.15; }
.page-header p { max-width: 640px; margin: 7px 0 0; color: var(--text-secondary); font-size: 13px; line-height: 1.55; }
.eyebrow { display: block; color: var(--text-tertiary); font-size: 11px; font-weight: 800; text-transform: uppercase; }
.session-controls { flex-shrink: 0; }
.btn { display: inline-flex; align-items: center; justify-content: center; gap: 7px; }
.group-error { padding: 10px 12px; border: 1px solid rgba(239,68,68,0.28); background: rgba(239,68,68,0.08); color: var(--danger); border-radius: var(--radius); font-size: 12px; cursor: pointer; overflow-wrap: anywhere; }
.character-select { display: grid; gap: 14px; }
.selection-head { display: flex; align-items: flex-end; justify-content: space-between; gap: 14px; padding-bottom: 12px; border-bottom: 1px solid var(--border); }
.selection-head strong { display: block; margin-top: 3px; color: var(--text-primary); font-size: 16px; }
.selection-count { color: var(--text-tertiary); font-size: 11px; font-weight: 700; }
.select-hint { margin: 0; color: var(--text-secondary); font-size: 12px; }
.character-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 10px; }
.char-card { position: relative; display: grid; grid-template-columns: 42px minmax(0, 1fr) 18px; align-items: center; gap: 11px; min-height: 72px; padding: 13px; border-radius: var(--radius); border: 1px solid var(--border); background: var(--surface-1); color: var(--text-primary); cursor: pointer; font: inherit; text-align: left; transition: all var(--transition-fast); }
.char-card:hover { border-color: var(--border-light); background: var(--surface-2); }
.char-card.selected { border-color: var(--brand); background: rgba(45,212,191,0.08); }
.char-avatar { width: 42px; height: 42px; border-radius: var(--radius); background: var(--surface-3); display: flex; align-items: center; justify-content: center; font-weight: 800; font-size: 17px; color: var(--brand-light); }
.char-copy { display: grid; gap: 3px; min-width: 0; }
.char-copy strong, .char-copy small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.char-copy strong { font-size: 13px; }
.char-copy small { color: var(--text-tertiary); font-family: var(--font-mono); font-size: 10px; }
.selected-mark { color: var(--brand-light); }
.empty-state { grid-column: 1 / -1; display: grid; place-items: center; gap: 8px; min-height: 220px; color: var(--text-tertiary); font-size: 12px; text-align: center; }
.chat-area { flex: 1; display: flex; flex-direction: column; gap: 12px; min-height: 520px; }
.participants-bar { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; padding-bottom: 10px; border-bottom: 1px solid var(--border); }
.participants-label { margin-right: 3px; color: var(--text-tertiary); font-size: 10px; font-weight: 800; text-transform: uppercase; }
.participant-tag { padding: 3px 10px; border-radius: 100px; font-size: 11px; font-weight: 600; background: var(--surface-3); color: var(--text-secondary); text-transform: uppercase; }
.messages { flex: 1; min-height: 300px; overflow-y: auto; display: flex; flex-direction: column; gap: 10px; padding: 8px 2px; }
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
.input-area { display: flex; gap: 8px; padding-top: 12px; border-top: 1px solid var(--border); }
.input-area .input { flex: 1; min-width: 0; }
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
  display: inline-block;
  margin-right: 7px;
  vertical-align: -3px;
  animation: spin 0.8s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }

@media (max-width: 720px) {
  .group-chat { min-height: calc(100dvh - 54px); padding: 22px 16px 96px; }
  .page-header p { display: none; }
  .session-controls .btn { min-height: 34px; }
  .character-grid { grid-template-columns: 1fr; }
  .chat-area { min-height: 620px; }
  .message { max-width: 92%; }
}

@media (max-width: 440px) {
  .page-header { flex-direction: column; }
  .session-controls, .session-controls .btn { width: 100%; }
  .input-area .btn { width: 42px; padding-inline: 0; font-size: 0; }
}
</style>
