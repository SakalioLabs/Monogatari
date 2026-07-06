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
        <span v-for="id in session.character_ids" :key="id" class="participant-tag">{{ id }}</span>
      </div>
      <div class="messages" ref="messagesEl">
        <div v-for="(m, i) in session.messages" :key="i" class="message" :class="m.role">
          <div class="msg-sender">{{ m.character_name }}</div>
          <div class="msg-content">{{ m.content }}</div>
          <div v-if="m.emotion" class="msg-emotion">{{ m.emotion }}</div>
        </div>
        <div v-if="loading" class="message system">
          <div class="msg-content"><span class="spinner"></span> Characters are thinking...</div>
        </div>
      </div>
      <div class="input-area">
        <input v-model="input" class="input" placeholder="Type a message to the group..." @keyup.enter="send" :disabled="loading" />
        <button class="btn btn-primary" @click="send" :disabled="!input.trim() || loading">Send</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from 'vue'
import { invokeCommand } from '../lib/tauri'

interface GroupMessage {
  role: string
  character_id: string | null
  character_name: string
  content: string
  emotion: string | null
  timestamp: string
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

async function loadCharacters() {
  try {
    available.value = await invokeCommand<[string, string][]>('get_group_chat_characters', {}, [])
  } catch { available.value = [] }
}
loadCharacters()

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
.input-area { display: flex; gap: 8px; }
.input-area .input { flex: 1; }
</style>
