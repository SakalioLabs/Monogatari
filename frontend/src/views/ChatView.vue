<template>
  <div class="chat-workbench">
    <aside class="character-rail">
      <div class="rail-header">
        <div>
          <span class="eyebrow">{{ t('chat.cast', 'Cast') }}</span>
          <h1>{{ t('chat.title', 'AI Chat') }}</h1>
        </div>
        <span class="status-dot" :class="{ online: characters.length > 0 }"></span>
      </div>

      <div class="character-list">
        <button
          v-for="char in characters"
          :key="char.id"
          class="character-row"
          :class="{ selected: selectedCharacter?.id === char.id }"
          @click="selectCharacter(char)"
        >
          <span class="avatar">{{ initials(char.name) }}</span>
          <span class="character-copy">
            <span class="row-title">{{ char.name }}</span>
            <span class="row-subtitle">{{ char.description || 'No profile text' }}</span>
          </span>
        </button>

        
    <div v-if="characters.length === 0" class="empty-rail">
          <span class="empty-mark">--</span>
          <span>{{ t('chat.no-characters', 'No characters loaded') }}</span>
        </div>
      </div>
    </aside>

    <section class="conversation-panel">
      <header class="conversation-header">
        <button
          class="icon-btn"
          :title="t('chat.back', 'Back to dashboard')"
          :aria-label="t('chat.back', 'Back to dashboard')"
          @click="$router.push('/')"
        ><span aria-hidden="true">&larr;</span></button>
        <div class="conversation-title">
          <span class="eyebrow">{{ selectedCharacter ? t('chat.session', 'Session') : t('chat.ready', 'Ready') }}</span>
          <h2>{{ selectedCharacter?.name || t('chat.select-character', 'Select a character') }}</h2>
        </div>
        <div class="session-metrics" v-if="selectedCharacter">
          <span class="metric-pill">{{ currentEmotion }}</span>
          <span class="metric-pill" :class="relationshipClass">Rel {{ relationshipScore.toFixed(2) }}</span>
        </div>
        <div class="header-actions">
          <button class="btn btn-secondary btn-sm" :disabled="!selectedCharacter || isLoading" @click="refreshEvaluation">{{ t('chat.score', 'Score') }}</button>
          <button class="btn btn-secondary btn-sm" :disabled="!selectedCharacter || isLoading" @click="clearChat">{{ t('chat.clear', 'Clear') }}</button>
        </div>
      </header>

      <main v-if="selectedCharacter" ref="messagesRef" class="messages-area">
        <div v-if="messages.length === 0" class="conversation-empty">
          <span class="empty-mark">00</span>
          <h3>{{ selectedCharacter.name }}</h3>
          <p>{{ selectedCharacter.description || 'Character profile is ready.' }}</p>
        </div>

        <article
          v-for="(msg, i) in messages"
          :key="`${msg.timestamp}-${i}`"
          class="message"
          :class="msg.role === 'player' ? 'msg-player' : 'msg-character'"
        >
          <div class="msg-avatar">{{ msg.role === 'player' ? 'P' : initials(selectedCharacter.name) }}</div>
          <div class="msg-stack">
            <div class="msg-bubble">
              <span v-if="msg.content">{{ msg.content }}</span>
              <span v-else class="stream-placeholder">Generating</span>
            </div>
            <div class="msg-meta">
              <span v-if="msg.emotion">{{ msg.emotion }}</span>
              <span>{{ formatTime(msg.timestamp) }}</span>
            </div>
          </div>
        </article>
      </main>

      <main v-else class="select-state">
        <div class="select-state-inner">
          <span class="empty-mark">M</span>
          <h2>{{ t('chat.session-desk', 'Character session desk') }}</h2>
          <p>{{ t('chat.session-desc', 'Profiles, relationship state, streaming replies, and scoring stay in one focused surface.') }}</p>
        </div>
      </main>

      <footer v-if="selectedCharacter" class="composer">
        <textarea
          ref="inputRef"
          v-model="inputText"
          :disabled="isLoading"
          :placeholder="t('chat.placeholder', 'Message')"
          rows="1"
          @input="resizeInput"
          @keydown.enter.exact.prevent="sendMessage"
        ></textarea>
        <button class="send-btn" :disabled="!inputText.trim() || isLoading" title="Send" @click="sendMessage">
          <span v-if="isLoading" class="spinner"></span>
          <span v-else>{{ t('chat.send', 'Send') }}</span>
        </button>
      </footer>
    </section>

    <aside class="insight-panel">
      <section class="insight-section">
        <span class="eyebrow">{{ t('chat.signal', 'Signal') }}</span>
        <div class="insight-grid">
          <div class="insight-item">
            <span class="insight-value">{{ messageCount }}</span>
            <span class="insight-label">{{ t('chat.messages', 'Messages') }}</span>
          </div>
          <div class="insight-item">
            <span class="insight-value">{{ playerMessageCount }}</span>
            <span class="insight-label">{{ t('chat.player', 'Player') }}</span>
          </div>
        </div>
      </section>

      <section class="insight-section">
        <div class="section-head">
          <span class="eyebrow">{{ t('chat.evaluation', 'Evaluation') }}</span>
          <button class="link-btn" :disabled="!selectedCharacter || isLoading" @click="refreshEvaluation">{{ t('chat.refresh', 'Refresh') }}</button>
        </div>
        <div v-if="evaluation" class="score-stack">
          <div class="score-row">
            <span>{{ t('chat.friendliness', 'Friendliness') }}</span>
            <strong>{{ percent(evaluation.friendliness) }}</strong>
            <div class="bar-track"><div class="bar-fill" :style="{ width: percent(evaluation.friendliness) }"></div></div>
          </div>
          <div class="score-row">
            <span>{{ t('chat.engagement', 'Engagement') }}</span>
            <strong>{{ percent(evaluation.engagement) }}</strong>
            <div class="bar-track"><div class="bar-fill engagement" :style="{ width: percent(evaluation.engagement) }"></div></div>
          </div>
          <div class="score-row">
            <span>{{ t('chat.creativity', 'Creativity') }}</span>
            <strong>{{ percent(evaluation.creativity) }}</strong>
            <div class="bar-track"><div class="bar-fill creativity" :style="{ width: percent(evaluation.creativity) }"></div></div>
          </div>
          <p class="eval-summary">{{ evaluation.summary }}</p>
        </div>
        <p v-else class="muted-copy">No score yet.</p>
      </section>

      <section class="insight-section safety-trace-panel">
        <span class="eyebrow">{{ t('chat.safety-trace', 'Safety Trace') }}</span>
        <div class="safety-pill-grid">
          <span
            v-for="flag in runtimeSafetyFlags"
            :key="flag.key"
            class="safety-pill"
            :class="{ active: flag.active }"
          >{{ flag.label }}</span>
        </div>
        <p class="trace-note">{{ safetyTraceSummary }}</p>
      </section>

      <section class="insight-section event-decision-panel">
        <span class="eyebrow">{{ t('chat.story-events', 'Story Events') }}</span>
        <div v-if="eventDecisionSummary.length" class="event-decision-list">
          <div
            v-for="decision in eventDecisionSummary"
            :key="decision.event_id"
            class="event-decision-row"
            :class="{ triggered: decision.triggered }"
          >
            <span>{{ decision.event_id }}</span>
            <strong>{{ decision.triggered ? 'Ready' : 'Blocked' }}</strong>
            <small>{{ eventDecisionReason(decision) }}</small>
            <code v-if="shortRuleFingerprint(decision)" class="rule-fingerprint">rule {{ shortRuleFingerprint(decision) }}</code>
          </div>
        </div>
        <p v-else class="muted-copy">No event decision yet.</p>
      </section>

      <section class="insight-section">
        <span class="eyebrow">{{ t('chat.runtime', 'Runtime') }}</span>
        <div class="runtime-list">
          <span><b>{{ t('chat.mode', 'Mode') }}</b>{{ isStreaming ? 'Streaming' : 'Idle' }}</span>
          <span><b>{{ t('chat.emotion', 'Emotion') }}</b>{{ currentEmotion }}</span>
          <span><b>{{ t('chat.relation', 'Relation') }}</b>{{ relationshipScore.toFixed(2) }}</span>
          <span><b>{{ t('chat.unlocks', 'Unlocks') }}</b>{{ unlockedContentCount }}</span>
        </div>
      </section>
    </aside>

    <Transition name="fade">
      <div v-if="activeEvent" class="event-toast" @click="clearActiveEvent">
        <strong>{{ activeEvent.description }}</strong>
        <span v-for="(result, index) in activeEventActionResults" :key="`${activeEvent.event_id}-${index}`">
          {{ storyActionLabel(result) }}
        </span>
      </div>
    </Transition>

    <Transition name="fade">
      <div v-if="errorMessage" class="error-toast" @click="errorMessage = null">{{ errorMessage }}</div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { invokeCommand } from '../lib/tauri'
import { useI18n } from '../lib/i18n'
import type { StoryEventAction } from '../lib/storyEvents'
import {
  loadStoryProgress,
  type StoryEventActionResult,
  type StoryEventApplication,
  type StoryProgressSnapshot,
} from '../lib/storyProgress'
import LoadingSpinner from '../components/LoadingSpinner.vue'

const { t } = useI18n()

interface ChatMessage {
  role: string
  content: string
  emotion: string | null
  timestamp: string
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
  data: Record<string, unknown>
  actions: StoryEventAction[]
}

interface EventTriggerRule {
  event_id: string
  event_type: string
  rule_fingerprint?: string | null
  min_relationship?: number | null
  score_metric?: string | null
  min_score?: number | null
  min_evaluation_count?: number | null
}

interface EventTriggerDecision {
  event_id: string
  event_type: string
  description: string
  triggered: boolean
  already_triggered: boolean
  actual_relationship: number
  actual_evaluation_count: number
  actual_score_metric?: string | null
  actual_score?: number | null
  rule_fingerprint?: string | null
  rule?: EventTriggerRule | null
  blocked_reasons: string[]
}

interface ConversationEvaluationReport {
  evaluation: ConversationEvaluation
  event_trigger_decisions: EventTriggerDecision[]
  triggerable_events: TriggeredEvent[]
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

interface ChatSessionAuditReport {
  character_id: string
  message_count: number
  player_message_count: number
  cumulative_score: number
  evaluation_count: number
  relationship_score: number
  triggered_event_ids: string[]
  last_evaluation?: ConversationEvaluation | null
  last_safety_trace?: ChatSafetyTrace | null
  event_trigger_decisions: EventTriggerDecision[]
  triggerable_events: TriggeredEvent[]
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
const isStreaming = ref(false)
const currentEmotion = ref('neutral')
const relationshipScore = ref(0)
const evaluation = ref<ConversationEvaluation | null>(null)
const safetyTrace = ref<ChatSafetyTrace | null>(null)
const eventDecisions = ref<EventTriggerDecision[]>([])
const activeEvent = ref<TriggeredEvent | null>(null)
const activeEventApplication = ref<StoryEventApplication | null>(null)
const storyProgress = ref<StoryProgressSnapshot | null>(null)
const errorMessage = ref<string | null>(null)
const charactersLoading = ref(true)
const messagesRef = ref<HTMLDivElement>()
const inputRef = ref<HTMLTextAreaElement>()
const STREAM_FAILURE_BUBBLE = 'Generation failed before the streamed reply completed.'

let streamUnlisteners: UnlistenFn[] = []

const messageCount = computed(() => messages.value.length)
const playerMessageCount = computed(() => messages.value.filter((m) => m.role === 'player').length)
const relationshipClass = computed(() => {
  if (relationshipScore.value >= 0.6) return 'rel-high'
  if (relationshipScore.value >= 0.3) return 'rel-mid'
  return 'rel-low'
})

const runtimeSafetyFlags = computed(() => {
  const trace = safetyTrace.value
  return [
    { key: 'mind', label: 'Mind', active: !!trace?.mind_contract_applied },
    { key: 'knowledge', label: 'Knowledge', active: !!trace?.knowledge_context_pinned },
    { key: 'input', label: 'Input', active: !!trace?.input_prompt_injection_detected || !!trace?.input_private_reasoning_request_detected },
    { key: 'response', label: 'Response', active: !!trace?.response_guard_applied },
    { key: 'memory', label: 'Memory', active: !!trace?.memory_guard_applied },
    { key: 'relation', label: 'Relation', active: !!trace?.relationship_delta_blocked },
    { key: 'stream', label: 'Stream', active: !!trace?.stream_guard_applied },
  ]
})

const safetyTraceSummary = computed(() => {
  const trace = safetyTrace.value
  if (!trace) return 'No runtime trace yet.'
  const notes = trace.guard_notes || []
  const refSummary = trace.pinned_knowledge_ref_ids?.length
    ? `Refs ${trace.pinned_knowledge_ref_ids.join(', ')}`
    : ''
  if (!notes.length || notes.includes('no_runtime_safety_interventions')) {
    return refSummary ? `No runtime safety interventions. ${refSummary}` : 'No runtime safety interventions.'
  }
  return [...notes.map(formatSafetyNote), refSummary].filter(Boolean).join(' / ')
})

const eventDecisionSummary = computed(() => {
  return eventDecisions.value
    .slice()
    .sort((a, b) => Number(b.triggered) - Number(a.triggered))
    .slice(0, 5)
})

const unlockedContentCount = computed(() => {
  const progress = storyProgress.value
  if (!progress) return 0
  return progress.unlocked_scene_ids.length
    + progress.unlocked_dialogue_ids.length
    + progress.unlocked_ending_ids.length
})

const activeEventActionResults = computed<StoryEventActionResult[]>(() => {
  const application = activeEventApplication.value
  if (application && application.event_id === activeEvent.value?.event_id) {
    return application.action_results
  }
  return (activeEvent.value?.actions || []).map((action) => ({ action, changed: true }))
})

function initials(name: string): string {
  return name.trim().slice(0, 2).toUpperCase() || 'AI'
}

function percent(value: number): string {
  return `${Math.round(Math.max(0, Math.min(1, value)) * 100)}%`
}

function formatTime(ts: string): string {
  try {
    return new Date(ts).toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
  } catch {
    return ''
  }
}

function scrollToBottom() {
  nextTick(() => {
    if (messagesRef.value) messagesRef.value.scrollTop = messagesRef.value.scrollHeight
  })
}

function formatSafetyNote(note: string) {
  return note
    .replace(/_/g, ' ')
    .replace(/\b\w/g, (ch) => ch.toUpperCase())
}

function streamFailureBubble(): string {
  return STREAM_FAILURE_BUBBLE
}

function eventDecisionReason(decision: EventTriggerDecision): string {
  if (decision.triggered) {
    const metric = decision.actual_score_metric && decision.actual_score !== null && decision.actual_score !== undefined
      ? `${decision.actual_score_metric} ${percent(decision.actual_score)}`
      : `Rel ${decision.actual_relationship.toFixed(2)}`
    return `${metric} / Eval ${decision.actual_evaluation_count}`
  }
  return decision.blocked_reasons[0] || 'Waiting for trigger rule'
}

function shortRuleFingerprint(decision: EventTriggerDecision): string {
  const fingerprint = decision.rule_fingerprint || decision.rule?.rule_fingerprint || ''
  return fingerprint ? fingerprint.slice(0, 10) : ''
}

function storyActionLabel(result: StoryEventActionResult): string {
  const state = result.changed ? 'unlocked' : 'already unlocked'
  if (result.action.type === 'unlock_scene') return `Scene ${result.action.scene_id} ${state}`
  if (result.action.type === 'unlock_dialogue') return `Dialogue ${result.action.dialogue_id} ${state}`
  if (result.action.type === 'unlock_ending') return `Ending ${result.action.ending_id} ${state}`
  return `Flag ${result.action.flag} ${result.changed ? 'updated' : 'unchanged'}`
}

function clearActiveEvent() {
  activeEvent.value = null
  activeEventApplication.value = null
}

async function refreshStoryProgress() {
  storyProgress.value = await loadStoryProgress()
}

function resizeInput() {
  nextTick(() => {
    if (!inputRef.value) return
    inputRef.value.style.height = 'auto'
    inputRef.value.style.height = `${Math.min(inputRef.value.scrollHeight, 144)}px`
  })
}

async function selectCharacter(char: CharacterInfo) {
  selectedCharacter.value = char
  currentEmotion.value = char.emotion || 'neutral'
  evaluation.value = null
  safetyTrace.value = null
  eventDecisions.value = []
  errorMessage.value = null
  try {
    const [history, audit] = await Promise.all([
      invokeCommand<ChatMessage[]>('get_chat_history', { characterId: char.id }, []),
      invokeCommand<ChatSessionAuditReport>('get_chat_session_audit', { characterId: char.id }),
    ])
    messages.value = history
    relationshipScore.value = audit.relationship_score
    evaluation.value = audit.last_evaluation || null
    safetyTrace.value = audit.last_safety_trace || null
    eventDecisions.value = audit.event_trigger_decisions || []
  } catch (e) {
    errorMessage.value = String(e)
  }
  scrollToBottom()
  nextTick(() => inputRef.value?.focus())
}

async function attachStreamListeners(assistantMessage: ChatMessage) {
  cleanupStreamListeners()
  streamUnlisteners = await Promise.all([
    listen<string>('chat-chunk', (event) => {
      assistantMessage.content += event.payload
      isStreaming.value = true
      scrollToBottom()
    }),
    listen<string>('chat-replace', (event) => {
      assistantMessage.content = event.payload || ''
      isStreaming.value = true
      scrollToBottom()
    }),
    listen<string>('chat-complete', (event) => {
      if (event.payload) assistantMessage.content = event.payload
      isStreaming.value = false
      scrollToBottom()
    }),
    listen<string>('chat-emotion', (event) => {
      currentEmotion.value = event.payload || currentEmotion.value
      assistantMessage.emotion = currentEmotion.value
    }),
    listen<number>('chat-relationship', async () => {
      await refreshRelationship()
    }),
    listen<ConversationEvaluation>('chat-evaluation', (event) => {
      evaluation.value = event.payload
    }),
    listen<ChatSafetyTrace>('chat-safety-trace', (event) => {
      safetyTrace.value = event.payload
    }),
    listen<EventTriggerDecision[]>('chat-event-decisions', (event) => {
      eventDecisions.value = event.payload || []
    }),
    listen<TriggeredEvent[]>('chat-events', (event) => {
      if (event.payload.length > 0) {
        activeEvent.value = event.payload[0]
        activeEventApplication.value = null
      }
    }),
    listen<StoryEventApplication[]>('chat-event-applications', async (event) => {
      if (event.payload.length > 0) activeEventApplication.value = event.payload[0]
      await refreshStoryProgress()
    }),
    listen<string>('chat-error', (event) => {
      errorMessage.value = event.payload || 'Generation failed'
      assistantMessage.content = streamFailureBubble()
      isStreaming.value = false
    }),
  ])
}

function cleanupStreamListeners() {
  for (const unlisten of streamUnlisteners) unlisten()
  streamUnlisteners = []
}

async function sendMessage() {
  if (!inputText.value.trim() || !selectedCharacter.value || isLoading.value) return

  const character = selectedCharacter.value
  const text = inputText.value.trim()
  inputText.value = ''
  resizeInput()
  errorMessage.value = null
  safetyTrace.value = null
  eventDecisions.value = []

  messages.value.push({ role: 'player', content: text, emotion: null, timestamp: new Date().toISOString() })
    
    // Track message count for achievements
    const charKey = 'monogatari-chat-count-' + character.id
    const count = parseInt(localStorage.getItem(charKey) || '0') + 1
    localStorage.setItem(charKey, String(count))
    
    // Check achievement unlocks
    if (typeof (window as any).__monogatari_unlock === 'function') {
      const unlock = (window as any).__monogatari_unlock
      if (count === 1) unlock('first_chat')
      if (count >= 10) unlock('chat_10')
      
      // Check total messages across all characters
      let total = 0
      for (let i = 0; i < localStorage.length; i++) {
        const k = localStorage.key(i)
        if (k?.startsWith('monogatari-chat-count-')) total += parseInt(localStorage.getItem(k) || '0')
      }
      if (total >= 50) unlock('chat_50')
      if (total >= 10) unlock('all_characters')
    }
  const assistantMessage: ChatMessage = {
    role: 'character',
    content: '',
    emotion: null,
    timestamp: new Date().toISOString(),
  }
  messages.value.push(assistantMessage)
  scrollToBottom()

  isLoading.value = true
  isStreaming.value = true

  try {
    await attachStreamListeners(assistantMessage)
    await invokeCommand<void>('send_chat_message_stream', {
      characterId: character.id,
      message: text,
    })
    await refreshRelationship()
  } catch (e) {
    errorMessage.value = String(e)
    assistantMessage.content = streamFailureBubble()
  } finally {
    isLoading.value = false
    isStreaming.value = false
    cleanupStreamListeners()
    scrollToBottom()
    nextTick(() => inputRef.value?.focus())
  }
}

async function refreshRelationship() {
  if (!selectedCharacter.value) return
  try {
    relationshipScore.value = await invokeCommand<number>('get_relationship_score', { characterId: selectedCharacter.value.id }, 0)
    // Check relationship achievement
    if (relationshipScore.value >= 0.8 && typeof (window as any).__monogatari_unlock === 'function') {
      (window as any).__monogatari_unlock('high_relationship')
    }
  } catch (e) {
    console.error(e)
  }
}

async function refreshEvaluation() {
  if (!selectedCharacter.value) return
  try {
    const characterId = selectedCharacter.value.id
    const report = await invokeCommand<ConversationEvaluationReport>('evaluate_conversation_report', { characterId })
    evaluation.value = report.evaluation
    eventDecisions.value = report.event_trigger_decisions || []
    // Check evaluation achievement
    if (evaluation.value && evaluation.value.overall_score > 0.8 && typeof (window as any).__monogatari_unlock === 'function') {
      (window as any).__monogatari_unlock('eval_high')
    }
  } catch (e) {
    errorMessage.value = String(e)
  }
}

async function clearChat() {
  if (!selectedCharacter.value) return
  try {
    await invokeCommand<void>('clear_chat_history', { characterId: selectedCharacter.value.id }, undefined)
    messages.value = []
    evaluation.value = null
    safetyTrace.value = null
    eventDecisions.value = []
    await refreshRelationship()
  } catch (e) {
    errorMessage.value = String(e)
  }
}

onMounted(async () => {
  try {
    const [loadedCharacters, progress] = await Promise.all([
      invokeCommand<CharacterInfo[]>('get_characters', undefined, []),
      loadStoryProgress(),
    ])
    characters.value = loadedCharacters
    storyProgress.value = progress
  } catch (e) {
    errorMessage.value = String(e)
  }
})

onUnmounted(cleanupStreamListeners)
</script>

<style scoped>
.chat-workbench {
  display: grid;
  grid-template-columns: 280px minmax(0, 1fr) 300px;
  height: 100vh;
  min-height: 0;
  background: var(--surface-0);
}

.character-rail,
.insight-panel {
  min-width: 0;
  background: var(--surface-1);
  border-color: var(--border);
}

.character-rail {
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border);
}

.rail-header,
.conversation-header,
.insight-section {
  border-bottom: 1px solid var(--border);
}

.rail-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
}

.eyebrow {
  display: block;
  color: var(--text-tertiary);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0;
  text-transform: uppercase;
}

.rail-header h1,
.conversation-title h2,
.select-state h2 {
  color: var(--text-primary);
  font-size: 20px;
  font-weight: 750;
  line-height: 1.2;
}

.status-dot {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  background: var(--surface-4);
  box-shadow: 0 0 0 4px rgba(255,255,255,0.04);
}

.status-dot.online {
  background: var(--success);
}

.character-list {
  flex: 1;
  overflow-y: auto;
  padding: 10px;
}

.character-row {
  width: 100%;
  display: flex;
  gap: 12px;
  align-items: center;
  padding: 12px;
  margin-bottom: 6px;
  border: 1px solid transparent;
  border-radius: var(--radius);
  background: transparent;
  color: var(--text-primary);
  cursor: pointer;
  text-align: left;
  transition: background var(--transition-fast), border-color var(--transition-fast);
}

.character-row:hover,
.character-row.selected {
  background: var(--surface-2);
  border-color: var(--border-light);
}

.avatar,
.msg-avatar {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  color: var(--surface-0);
  background: var(--brand);
  font-weight: 800;
}

.avatar {
  width: 38px;
  height: 38px;
  border-radius: var(--radius-sm);
}

.character-copy {
  min-width: 0;
  display: grid;
  gap: 2px;
}

.row-title {
  color: var(--text-primary);
  font-size: 13px;
  font-weight: 700;
}

.row-subtitle {
  overflow: hidden;
  color: var(--text-tertiary);
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.conversation-panel {
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background:
    linear-gradient(180deg, rgba(255,255,255,0.025), transparent 220px),
    var(--surface-0);
}

.conversation-header {
  display: grid;
  grid-template-columns: 38px minmax(0, 1fr) auto auto;
  gap: 12px;
  align-items: center;
  padding: 14px 18px;
  background: rgba(15,17,21,0.86);
  backdrop-filter: blur(16px);
}

.icon-btn {
  display: grid;
  place-items: center;
  width: 34px;
  height: 34px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-1);
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 24px;
  line-height: 1;
  overflow: hidden;
}

.icon-btn:hover {
  border-color: var(--brand);
  color: var(--brand-light);
}

.session-metrics,
.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.metric-pill {
  display: inline-flex;
  align-items: center;
  min-height: 26px;
  padding: 4px 10px;
  border: 1px solid var(--border);
  border-radius: 999px;
  color: var(--text-secondary);
  background: var(--surface-1);
  font-size: 12px;
  font-weight: 700;
}

.metric-pill.rel-high { color: var(--success); }
.metric-pill.rel-mid { color: var(--warning); }
.metric-pill.rel-low { color: var(--danger); }

.messages-area {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 28px min(5vw, 56px);
}

.message {
  display: flex;
  gap: 12px;
  max-width: min(760px, 86%);
  margin-bottom: 18px;
  animation: slideInUp 0.18s ease;
}

.msg-player {
  flex-direction: row-reverse;
  margin-left: auto;
}

.msg-avatar {
  width: 34px;
  height: 34px;
  border-radius: var(--radius-sm);
  font-size: 12px;
}

.msg-player .msg-avatar {
  background: var(--accent);
}

.msg-stack {
  min-width: 0;
}

.msg-bubble {
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 12px 14px;
  color: var(--text-primary);
  background: var(--surface-1);
  box-shadow: var(--shadow-sm);
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.msg-player .msg-bubble {
  background: rgba(245,158,11,0.12);
  border-color: rgba(245,158,11,0.32);
}

.msg-meta {
  display: flex;
  gap: 8px;
  padding: 5px 2px 0;
  color: var(--text-tertiary);
  font-size: 11px;
}

.stream-placeholder {
  color: var(--text-tertiary);
}

.composer {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 92px;
  gap: 10px;
  align-items: end;
  padding: 14px 18px;
  border-top: 1px solid var(--border);
  background: var(--surface-1);
}

.composer textarea {
  width: 100%;
  max-height: 144px;
  min-height: 42px;
  resize: none;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-2);
  color: var(--text-primary);
  padding: 11px 12px;
  font: inherit;
  outline: none;
}

.composer textarea:focus {
  border-color: var(--brand);
  box-shadow: var(--shadow-brand);
}

.send-btn {
  min-height: 42px;
  border: none;
  border-radius: var(--radius);
  background: var(--brand);
  color: var(--surface-0);
  cursor: pointer;
  font-weight: 800;
}

.send-btn:hover:not(:disabled) {
  background: var(--brand-light);
}

.send-btn:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.insight-panel {
  border-left: 1px solid var(--border);
  overflow-y: auto;
}

.insight-section {
  padding: 18px;
}

.section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 14px;
}

.insight-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
  margin-top: 12px;
}

.insight-item {
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-2);
  padding: 12px;
}

.insight-value {
  display: block;
  color: var(--brand-light);
  font-size: 24px;
  font-weight: 800;
}

.insight-label {
  color: var(--text-tertiary);
  font-size: 11px;
}

.score-stack {
  display: grid;
  gap: 14px;
}

.score-row {
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 6px 10px;
  color: var(--text-secondary);
  font-size: 12px;
}

.score-row strong {
  color: var(--text-primary);
}

.bar-track {
  grid-column: 1 / -1;
  height: 7px;
  overflow: hidden;
  border-radius: 999px;
  background: var(--surface-3);
}

.bar-fill {
  height: 100%;
  border-radius: inherit;
  background: var(--brand);
  transition: width var(--transition);
}

.bar-fill.engagement { background: var(--info); }
.bar-fill.creativity { background: var(--accent); }

.eval-summary,
.muted-copy {
  color: var(--text-tertiary);
  font-size: 12px;
}

.runtime-list {
  display: grid;
  gap: 10px;
  margin-top: 12px;
}

.runtime-list span {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  color: var(--text-secondary);
  font-size: 12px;
}

.runtime-list b {
  color: var(--text-tertiary);
  font-weight: 600;
}

.safety-pill-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 12px;
}

.safety-pill {
  padding: 4px 8px;
  border-radius: 999px;
  background: var(--surface-3);
  color: var(--text-tertiary);
  font-size: 10px;
  font-weight: 800;
  text-transform: uppercase;
}

.safety-pill.active {
  background: rgba(245,158,11,0.14);
  color: var(--warning);
}

.trace-note {
  margin-top: 10px;
  color: var(--text-tertiary);
  font-size: 12px;
  line-height: 1.5;
  overflow-wrap: anywhere;
}

.event-decision-list {
  display: grid;
  gap: 7px;
  margin-top: 12px;
}

.event-decision-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 4px 8px;
  padding: 8px 10px;
  border: 1px solid rgba(239,68,68,0.22);
  border-radius: var(--radius-sm);
  background: rgba(239,68,68,0.08);
}

.event-decision-row.triggered {
  border-color: rgba(34,197,94,0.24);
  background: rgba(34,197,94,0.08);
}

.event-decision-row span,
.event-decision-row small,
.rule-fingerprint {
  min-width: 0;
  overflow-wrap: anywhere;
}

.event-decision-row span {
  color: var(--text-secondary);
  font-size: 11px;
  font-weight: 800;
}

.event-decision-row strong {
  color: var(--danger);
  font-size: 11px;
  text-transform: uppercase;
}

.event-decision-row.triggered strong {
  color: var(--success);
}

.event-decision-row small {
  grid-column: 1 / -1;
  color: var(--text-tertiary);
  font-size: 11px;
  line-height: 1.35;
}

.rule-fingerprint {
  grid-column: 1 / -1;
  color: var(--text-tertiary);
  font-size: 10px;
  font-weight: 800;
}

.link-btn {
  border: none;
  background: transparent;
  color: var(--brand-light);
  cursor: pointer;
  font-weight: 700;
}

.link-btn:disabled {
  cursor: not-allowed;
  color: var(--text-tertiary);
}

.conversation-empty,
.select-state,
.empty-rail {
  display: grid;
  place-items: center;
  align-content: center;
  text-align: center;
  color: var(--text-tertiary);
}

.conversation-empty {
  min-height: 60%;
}

.select-state {
  flex: 1;
  padding: 24px;
}

.select-state-inner {
  max-width: 420px;
}

.conversation-empty h3,
.select-state h2 {
  margin-top: 12px;
  color: var(--text-primary);
}

.conversation-empty p,
.select-state p {
  margin-top: 6px;
  color: var(--text-tertiary);
}

.empty-mark {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 42px;
  height: 42px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  color: var(--brand-light);
  background: var(--surface-2);
  font-family: var(--font-mono);
  font-weight: 800;
}

.event-toast,
.error-toast {
  position: fixed;
  z-index: 100;
  left: 50%;
  transform: translateX(-50%);
  display: grid;
  gap: 2px;
  min-width: min(420px, calc(100vw - 32px));
  padding: 12px 14px;
  border-radius: var(--radius);
  box-shadow: var(--shadow-lg);
}

.event-toast {
  top: 18px;
  border: 1px solid rgba(45,212,191,0.36);
  background: rgba(15,118,110,0.96);
  color: white;
}

.error-toast {
  bottom: 18px;
  border: 1px solid rgba(239,68,68,0.42);
  background: rgba(127,29,29,0.96);
  color: white;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

@media (max-width: 1120px) {
  .chat-workbench {
    grid-template-columns: 240px minmax(0, 1fr);
  }

  .insight-panel {
    display: none;
  }
}

@media (max-width: 760px) {
  .chat-workbench {
    grid-template-columns: 1fr;
  }

  .character-rail {
    max-height: 220px;
    border-right: none;
    border-bottom: 1px solid var(--border);
  }

  .conversation-header {
    grid-template-columns: 34px minmax(0, 1fr);
  }

  .session-metrics,
  .header-actions {
    grid-column: 2;
    justify-self: start;
    flex-wrap: wrap;
  }

  .message {
    max-width: 100%;
  }
}
</style>
