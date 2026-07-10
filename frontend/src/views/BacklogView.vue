<template>
  <div class="backlog-view">
    <header class="backlog-header">
      <div>
        <span class="eyebrow">{{ t('backlog.eyebrow', 'Playback history') }}</span>
        <h1>{{ t('backlog.heading', 'Conversation History') }}</h1>
      </div>
      <div class="header-actions">
        <div class="filter-group" role="group" :aria-label="t('backlog.filter-label', 'Message filter')">
          <button class="filter-btn" :class="{ active: filter === 'all' }" @click="filter = 'all'">{{ t('backlog.filter-all', 'All') }}</button>
          <button class="filter-btn" :class="{ active: filter === 'player' }" @click="filter = 'player'">{{ t('backlog.filter-player', 'Player Only') }}</button>
          <button class="filter-btn" :class="{ active: filter === 'character' }" @click="filter = 'character'">{{ t('backlog.filter-character', 'Character Only') }}</button>
        </div>
        <button class="btn btn-secondary btn-sm" @click="$router.push('/game')">
          <ArrowLeft :size="14" />
          {{ t('common.back', 'Back') }}
        </button>
      </div>
    </header>

    <div class="character-select" v-if="characters.length > 0">
      <button
        v-for="char in characters"
        :key="char.id"
        class="char-chip"
        :class="{ active: selectedCharId === char.id }"
        @click="selectCharacter(char.id)"
      >
        <span class="chip-avatar" :style="{ background: colorFor(char.id) }">{{ initials(char.name) }}</span>
        <span>{{ char.name }}</span>
      </button>
    </div>

    <main class="backlog-body" ref="bodyRef">
      <div v-if="filteredMessages.length === 0" class="empty-backlog">
        <BookOpen :size="32" />
        <p>{{ t('backlog.empty', 'No conversation history yet.') }}</p>
        <button class="btn btn-primary btn-sm" @click="$router.push('/chat')">
          <MessageCircle :size="14" />
          {{ t('backlog.start-chat', 'Start chatting') }}
        </button>
      </div>

      <article
        v-for="(msg, i) in filteredMessages"
        :key="i"
        class="backlog-entry"
        :class="'entry-' + msg.role"
      >
        <div class="entry-avatar" :style="{ background: avatarColor(msg) }">
          {{ avatarLabel(msg) }}
        </div>
        <div class="entry-content">
          <div class="entry-head">
            <strong class="entry-name">{{ roleName(msg) }}</strong>
            <span class="entry-emotion" v-if="msg.emotion">{{ msg.emotion }}</span>
            <time class="entry-time">{{ formatTime(msg.timestamp) }}</time>
          </div>
          <p class="entry-text">{{ msg.content }}</p>
        </div>
      </article>
    </main>

    <footer class="backlog-footer" v-if="filteredMessages.length > 0">
      <span class="entry-count">{{ t('backlog.entries', '{count} entries', { count: filteredMessages.length }) }}</span>
      <button class="btn btn-secondary btn-sm" @click="scrollToBottom">
        <ArrowDown :size="14" />
        {{ t('backlog.jump-latest', 'Jump to latest') }}
      </button>
    </footer>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { ArrowDown, ArrowLeft, BookOpen, MessageCircle } from '@lucide/vue'
import { useI18n } from '../lib/i18n'
import { invokeCommand } from '../lib/tauri'

const { locale, t } = useI18n()

interface ChatMessage {
  role: string
  content: string
  emotion: string | null
  timestamp: string
}

interface CharacterInfo {
  id: string
  name: string
}

const characters = ref<CharacterInfo[]>([])
const selectedCharId = ref<string | null>(null)
const messages = ref<ChatMessage[]>([])
const filter = ref<'all' | 'player' | 'character'>('all')
const bodyRef = ref<HTMLDivElement>()

const avatarColors: Record<string, string> = {}
const palette = ['#6366f1', '#ec4899', '#14b8a6', '#f59e0b', '#8b5cf6', '#ef4444', '#06b6d4', '#84cc16']

function colorFor(id: string): string {
  if (avatarColors[id]) return avatarColors[id]
  let h = 0
  for (const ch of id) h = ((h << 5) - h) + ch.charCodeAt(0)
  avatarColors[id] = palette[Math.abs(h) % palette.length]
  return avatarColors[id]
}

function initials(name: string): string {
  return name.trim().slice(0, 2).toUpperCase() || '??'
}

const filteredMessages = computed(() => {
  let result = messages.value
  if (filter.value === 'player') result = result.filter(m => m.role === 'player')
  if (filter.value === 'character') result = result.filter(m => m.role !== 'player' && m.role !== 'system')
  return result
})

function avatarColor(msg: ChatMessage): string {
  if (msg.role === 'player') return '#f59e0b'
  if (msg.role === 'system') return '#64748b'
  return colorFor(msg.role)
}

function avatarLabel(msg: ChatMessage): string {
  if (msg.role === 'player') return 'P'
  if (msg.role === 'system') return 'S'
  return initials(msg.role)
}

function roleName(msg: ChatMessage): string {
  if (msg.role === 'player') return t('backlog.player', 'Player')
  if (msg.role === 'system') return t('backlog.system', 'System')
  const char = characters.value.find(c => c.id === msg.role || c.name === msg.role)
  return char?.name || msg.role
}

function formatTime(ts: string): string {
  try { return new Date(ts).toLocaleTimeString(locale.value, { hour: '2-digit', minute: '2-digit' }) }
  catch { return '' }
}

async function selectCharacter(id: string) {
  selectedCharId.value = id
  try {
    messages.value = await invokeCommand<ChatMessage[]>('get_chat_history', { characterId: id }, [])
  } catch {
    messages.value = []
  }
  nextTick(scrollToBottom)
}

function scrollToBottom() {
  nextTick(() => {
    if (bodyRef.value) bodyRef.value.scrollTop = bodyRef.value.scrollHeight
  })
}

onMounted(async () => {
  try {
    characters.value = await invokeCommand<CharacterInfo[]>('get_characters', undefined, [])
    if (characters.value.length > 0) await selectCharacter(characters.value[0].id)
  } catch {}
})

watch(filter, () => nextTick(scrollToBottom))
</script>

<style scoped>
.backlog-view {
  display: flex;
  flex-direction: column;
  height: calc(100dvh - 54px);
  background: var(--surface-0);
}

.backlog-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  padding: 24px 32px 16px;
  border-bottom: 1px solid var(--border);
}

.backlog-header h1 {
  font-size: 24px;
  font-weight: 800;
  color: var(--text-primary);
  margin: 4px 0 0;
}

.eyebrow {
  display: block;
  color: var(--text-tertiary);
  font-size: 11px;
  font-weight: 800;
  text-transform: uppercase;
}

.header-actions {
  display: flex;
  gap: 12px;
  align-items: center;
}

.filter-group {
  display: flex;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  overflow: hidden;
}

.filter-btn {
  padding: 6px 14px;
  border: none;
  border-right: 1px solid var(--border);
  background: var(--surface-1);
  color: var(--text-secondary);
  cursor: pointer;
  font: inherit;
  font-size: 12px;
  font-weight: 600;
  transition: all 0.15s;
}

.filter-btn:last-child { border-right: none; }
.filter-btn:hover { background: var(--surface-2); }
.filter-btn.active { background: var(--surface-3); color: var(--brand-light); }

.character-select {
  display: flex;
  gap: 8px;
  padding: 12px 32px;
  border-bottom: 1px solid var(--border);
  overflow-x: auto;
}

.char-chip {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 14px;
  border: 1px solid var(--border);
  border-radius: 100px;
  background: var(--surface-1);
  color: var(--text-secondary);
  cursor: pointer;
  font: inherit;
  font-size: 13px;
  font-weight: 600;
  white-space: nowrap;
  transition: all 0.15s;
}

.char-chip:hover { border-color: var(--brand); }
.char-chip.active { border-color: var(--brand); background: rgba(45,212,191,0.1); color: var(--brand-light); }

.chip-avatar {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 9px;
  font-weight: 800;
}

.backlog-body {
  flex: 1;
  overflow-y: auto;
  padding: 24px 32px;
}

.empty-backlog {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 80px 20px;
  text-align: center;
  color: var(--text-tertiary);
}

.empty-icon { font-size: 48px; }

.backlog-entry {
  display: flex;
  gap: 14px;
  padding: 14px 0;
  border-bottom: 1px solid rgba(255,255,255,0.04);
  animation: slideIn 0.2s ease;
}

@keyframes slideIn {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}

.entry-player { flex-direction: row-reverse; }
.entry-player .entry-content { text-align: right; }

.entry-avatar {
  width: 36px;
  height: 36px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 12px;
  font-weight: 800;
  flex-shrink: 0;
}

.entry-content { min-width: 0; flex: 1; }

.entry-head {
  display: flex;
  gap: 8px;
  align-items: baseline;
  margin-bottom: 4px;
}

.entry-player .entry-head { flex-direction: row-reverse; }

.entry-name {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-primary);
}

.entry-emotion {
  font-size: 11px;
  color: var(--brand-light);
  padding: 1px 8px;
  border: 1px solid rgba(45,212,191,0.3);
  border-radius: 100px;
}

.entry-time {
  font-size: 11px;
  color: var(--text-tertiary);
}

.entry-text {
  color: var(--text-secondary);
  font-size: 14px;
  line-height: 1.65;
  margin: 0;
  white-space: pre-wrap;
}

.backlog-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 32px;
  border-top: 1px solid var(--border);
}

.entry-count { font-size: 12px; color: var(--text-tertiary); }

.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 7px;
  min-height: 34px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  color: var(--text-secondary);
  cursor: pointer;
  font: inherit;
  font-weight: 700;
  font-size: 13px;
  padding: 6px 14px;
  transition: all 0.15s;
}

.btn:hover { border-color: var(--brand); color: var(--brand-light); }
.btn-primary { background: var(--brand); color: var(--surface-0); border-color: var(--brand); }
.btn-primary:hover { background: var(--brand-light); }
.btn-sm { min-height: 30px; padding: 4px 12px; font-size: 12px; }

@media (max-width: 640px) {
  .backlog-view { height: calc(100dvh - 112px); }
  .backlog-header, .backlog-body, .backlog-footer { padding-left: 16px; padding-right: 16px; }
  .backlog-header { align-items: stretch; flex-direction: column; gap: 14px; }
  .character-select { padding: 10px 16px; }
  .header-actions { align-items: stretch; gap: 8px; overflow-x: auto; }
  .filter-group { flex: 1 0 auto; }
  .filter-btn { flex: 1; padding-inline: 10px; white-space: nowrap; }
}
</style>
