<template>
  <div class="chat-workbench">
    <aside class="character-rail">
      <header class="rail-header">
        <div>
          <span class="eyebrow"><UsersRound :size="13" aria-hidden="true" />{{ t('chat.cast', 'Cast') }}</span>
          <h1>{{ t('chat.title', 'Character Test') }}</h1>
        </div>
        <strong>{{ filteredCharacters.length }}/{{ characters.length }}</strong>
      </header>

      <label class="character-search">
        <Search :size="14" aria-hidden="true" />
        <input v-model="characterSearch" :placeholder="t('chat.search-characters', 'Search characters')" :aria-label="t('chat.search-characters', 'Search characters')" />
      </label>

      <div class="character-list">
        <button
          v-for="char in filteredCharacters"
          :key="char.id"
          class="character-row"
          :class="{ selected: selectedCharacter?.id === char.id }"
          @click="selectCharacter(char)"
        >
          <span class="avatar">{{ initials(char.name) }}</span>
          <span class="character-copy">
            <span class="row-title">{{ char.name }}</span>
            <span class="row-subtitle">{{ emotionLabel(char.emotion) }}</span>
          </span>
          <ChevronRight :size="14" aria-hidden="true" />
        </button>

        <div v-if="charactersLoading" class="empty-rail">
          <LoaderCircle :size="20" class="spin" aria-hidden="true" />
          <span>{{ t('chat.loading-characters', 'Loading characters') }}</span>
        </div>
        <div v-else-if="!filteredCharacters.length" class="empty-rail">
          <SearchX v-if="characterSearch" :size="21" aria-hidden="true" />
          <UsersRound v-else :size="21" aria-hidden="true" />
          <span>{{ characterSearch ? t('chat.no-character-results', 'No matching characters') : t('chat.no-characters', 'No characters loaded') }}</span>
        </div>
      </div>
    </aside>

    <section class="conversation-panel">
      <header class="conversation-header">
        <button class="icon-command" :title="t('chat.back', 'Back to dashboard')" :aria-label="t('chat.back', 'Back to dashboard')" @click="$router.push('/')">
          <ArrowLeft :size="16" aria-hidden="true" />
        </button>
        <div class="conversation-title">
          <span class="eyebrow">{{ selectedCharacter ? t('chat.session', 'Session') : t('chat.ready', 'Ready') }}</span>
          <h2>{{ selectedCharacter?.name || t('chat.select-character', 'Select a character') }}</h2>
        </div>
        <span class="runtime-badge" :class="{ webgpu: !desktopRuntimeAvailable }">
          <MonitorCog :size="13" aria-hidden="true" />
          {{ desktopRuntimeAvailable ? t('chat.desktop-runtime', 'Windows DirectML') : t('chat.webgpu-runtime', 'WebGPU runtime') }}
        </span>
        <div v-if="selectedCharacter" class="session-metrics">
          <span class="metric-pill"><Smile :size="13" aria-hidden="true" />{{ emotionLabel(currentEmotion) }}</span>
          <span class="metric-pill" :class="relationshipClass"><HeartHandshake :size="13" aria-hidden="true" />{{ t('chat.relation', 'Relation') }} {{ relationshipScore.toFixed(2) }}</span>
        </div>
        <div class="header-actions">
          <button class="icon-command" :disabled="!selectedCharacter || isLoading" :title="t('chat.score', 'Score')" :aria-label="t('chat.score', 'Score')" @click="refreshEvaluation">
            <Gauge :size="16" aria-hidden="true" />
          </button>
          <button class="icon-command danger-command" :disabled="!selectedCharacter || isLoading || messages.length === 0" :title="t('chat.clear', 'Clear')" :aria-label="t('chat.clear', 'Clear')" @click="requestClearChat">
            <Trash2 :size="16" aria-hidden="true" />
          </button>
          <button class="icon-command insight-toggle" :title="t('chat.open-insights', 'Open insights')" :aria-label="t('chat.open-insights', 'Open insights')" @click="compactInsightOpen = true">
            <PanelRightOpen :size="16" aria-hidden="true" />
          </button>
        </div>
      </header>

      <main v-if="selectedCharacter" ref="messagesRef" class="messages-area" :aria-label="t('chat.conversation', 'Conversation')">
        <div v-if="messages.length === 0" class="conversation-empty">
          <MessageCircleMore :size="28" aria-hidden="true" />
          <h3>{{ selectedCharacter.name }}</h3>
          <p>{{ t('chat.character-ready', 'The character session is ready for a message.') }}</p>
        </div>

        <article
          v-for="(msg, i) in messages"
          :key="`${msg.timestamp}-${i}`"
          class="message"
          :class="msg.role === 'player' ? 'msg-player' : 'msg-character'"
        >
          <div class="msg-avatar">
            <UserRound v-if="msg.role === 'player'" :size="15" aria-hidden="true" />
            <span v-else>{{ initials(selectedCharacter.name) }}</span>
          </div>
          <div class="msg-stack">
            <div class="msg-bubble">
              <span v-if="msg.content">{{ msg.content }}</span>
              <span v-else class="stream-placeholder"><LoaderCircle :size="13" class="spin" aria-hidden="true" />{{ t('chat.generating', 'Generating') }}</span>
            </div>
            <div class="msg-meta">
              <span>{{ msg.role === 'player' ? t('chat.player', 'Player') : selectedCharacter.name }}</span>
              <span v-if="msg.emotion">{{ emotionLabel(msg.emotion) }}</span>
              <span>{{ formatTime(msg.timestamp) }}</span>
            </div>
          </div>
        </article>
      </main>

      <main v-else class="select-state">
        <div class="select-state-inner">
          <MessagesSquare :size="30" aria-hidden="true" />
          <h2>{{ t('chat.session-desk', 'Character session desk') }}</h2>
          <p>{{ t('chat.no-active-session', 'No active character session.') }}</p>
        </div>
      </main>

      <footer v-if="selectedCharacter" class="composer">
        <textarea
          ref="inputRef"
          v-model="inputText"
          :disabled="isLoading"
          :placeholder="t('chat.placeholder', 'Message')"
          :aria-label="t('chat.placeholder', 'Message')"
          rows="1"
          @input="resizeInput"
          @keydown.enter.exact.prevent="sendMessage"
        ></textarea>
        <button class="send-btn" :disabled="!inputText.trim() || isLoading" :title="t('chat.send', 'Send')" :aria-label="t('chat.send', 'Send')" @click="sendMessage">
          <LoaderCircle v-if="isLoading" :size="15" class="spin" aria-hidden="true" />
          <Send v-else :size="15" aria-hidden="true" />
          <span>{{ isLoading ? t('chat.generating', 'Generating') : t('chat.send', 'Send') }}</span>
        </button>
      </footer>
    </section>

    <aside class="insight-panel" :class="{ 'compact-open': compactInsightOpen }">
      <header class="insight-header">
        <div>
          <span class="eyebrow"><ScanSearch :size="13" aria-hidden="true" />{{ t('chat.insights', 'Insights') }}</span>
          <strong>{{ selectedCharacter?.name || t('chat.no-session', 'No session') }}</strong>
        </div>
        <button class="icon-command insight-close" :title="t('common.close', 'Close')" :aria-label="t('common.close', 'Close')" @click="compactInsightOpen = false"><X :size="15" aria-hidden="true" /></button>
      </header>

      <div class="insight-tabs" role="tablist" :aria-label="t('chat.insight-views', 'Insight views')">
        <button role="tab" :aria-selected="insightTab === 'evaluation'" :class="{ active: insightTab === 'evaluation' }" :title="t('chat.evaluation', 'Evaluation')" @click="insightTab = 'evaluation'"><Gauge :size="15" aria-hidden="true" /><span>{{ t('chat.evaluation', 'Evaluation') }}</span></button>
        <button role="tab" :aria-selected="insightTab === 'safety'" :class="{ active: insightTab === 'safety' }" :title="t('chat.safety-trace', 'Safety Trace')" @click="insightTab = 'safety'"><ShieldCheck :size="15" aria-hidden="true" /><span>{{ t('chat.safety', 'Safety') }}</span></button>
        <button role="tab" :aria-selected="insightTab === 'events'" :class="{ active: insightTab === 'events' }" :title="t('chat.story-events', 'Story Events')" @click="insightTab = 'events'"><Milestone :size="15" aria-hidden="true" /><span>{{ t('chat.events', 'Events') }}</span></button>
        <button role="tab" :aria-selected="insightTab === 'runtime'" :class="{ active: insightTab === 'runtime' }" :title="t('chat.runtime', 'Runtime')" @click="insightTab = 'runtime'"><Activity :size="15" aria-hidden="true" /><span>{{ t('chat.runtime', 'Runtime') }}</span></button>
      </div>

      <div v-if="selectedCharacter" class="insight-content">
        <section v-if="insightTab === 'evaluation'" class="insight-section">
          <div class="section-head">
            <span>{{ t('chat.evaluation', 'Evaluation') }}</span>
            <button class="text-command" :disabled="isLoading" @click="refreshEvaluation">{{ t('chat.refresh', 'Refresh') }}</button>
          </div>
          <div class="insight-grid">
            <div class="insight-item"><span class="insight-value">{{ messageCount }}</span><span class="insight-label">{{ t('chat.messages', 'Messages') }}</span></div>
            <div class="insight-item"><span class="insight-value">{{ playerMessageCount }}</span><span class="insight-label">{{ t('chat.player', 'Player') }}</span></div>
          </div>
          <div v-if="evaluation" class="score-stack">
            <div class="score-row"><span>{{ t('chat.friendliness', 'Friendliness') }}</span><strong>{{ percent(evaluation.friendliness) }}</strong><div class="bar-track"><div class="bar-fill" :style="{ width: percent(evaluation.friendliness) }"></div></div></div>
            <div class="score-row"><span>{{ t('chat.engagement', 'Engagement') }}</span><strong>{{ percent(evaluation.engagement) }}</strong><div class="bar-track"><div class="bar-fill engagement" :style="{ width: percent(evaluation.engagement) }"></div></div></div>
            <div class="score-row"><span>{{ t('chat.creativity', 'Creativity') }}</span><strong>{{ percent(evaluation.creativity) }}</strong><div class="bar-track"><div class="bar-fill creativity" :style="{ width: percent(evaluation.creativity) }"></div></div></div>
            <div class="score-row overall-row"><span>{{ t('quality.overall', 'Overall') }}</span><strong>{{ percent(evaluation.overall_score) }}</strong><div class="bar-track"><div class="bar-fill overall" :style="{ width: percent(evaluation.overall_score) }"></div></div></div>
            <p class="eval-summary">{{ evaluation.summary }}</p>
          </div>
          <div v-else class="compact-empty"><Gauge :size="21" aria-hidden="true" /><span>{{ t('chat.no-score', 'No score yet.') }}</span></div>
        </section>

        <section v-else-if="insightTab === 'safety'" class="insight-section safety-trace-panel">
          <div class="section-head"><span>{{ t('chat.safety-trace', 'Safety Trace') }}</span><strong>{{ activeSafetyFlagCount }}/{{ runtimeSafetyFlags.length }}</strong></div>
          <div class="safety-flag-list">
            <div v-for="flag in runtimeSafetyFlags" :key="flag.key" class="safety-pill" :class="{ active: flag.active }">
              <component :is="flag.active ? CircleAlert : CheckCircle2" :size="14" aria-hidden="true" />
              <span>{{ flag.label }}</span>
              <strong>{{ flag.active ? t('chat.active', 'Active') : t('chat.clear-state', 'Clear') }}</strong>
            </div>
          </div>
          <p class="trace-note">{{ safetyTraceSummary }}</p>
        </section>

        <section v-else-if="insightTab === 'events'" class="insight-section event-decision-panel">
          <div class="section-head"><span>{{ t('chat.story-events', 'Story Events') }}</span><strong>{{ eventDecisionSummary.length }}</strong></div>
          <div v-if="eventDecisionSummary.length" class="event-decision-list">
            <div v-for="decision in eventDecisionSummary" :key="decision.event_id" class="event-decision-row" :class="{ triggered: decision.triggered }">
              <span>{{ decision.event_id }}</span>
              <strong>{{ decision.triggered ? t('chat.event-ready', 'Ready') : t('chat.event-blocked', 'Blocked') }}</strong>
              <small>{{ eventDecisionReason(decision) }}</small>
              <code v-if="shortRuleFingerprint(decision)" class="rule-fingerprint">{{ t('chat.rule', 'Rule') }} {{ shortRuleFingerprint(decision) }}</code>
            </div>
          </div>
          <div v-else class="compact-empty"><Milestone :size="21" aria-hidden="true" /><span>{{ t('chat.no-event-decision', 'No event decision yet.') }}</span></div>
        </section>

        <section v-else class="insight-section">
          <div class="section-head"><span>{{ t('chat.runtime', 'Runtime') }}</span></div>
          <div class="runtime-list">
            <span><b>{{ t('chat.mode', 'Mode') }}</b>{{ isStreaming ? t('chat.streaming', 'Streaming') : t('chat.idle', 'Idle') }}</span>
            <span><b>{{ t('chat.runtime-source', 'Source') }}</b>{{ desktopRuntimeAvailable ? t('chat.desktop-runtime', 'Windows DirectML') : t('chat.webgpu-runtime', 'WebGPU runtime') }}</span>
            <span><b>{{ t('chat.emotion', 'Emotion') }}</b>{{ emotionLabel(currentEmotion) }}</span>
            <span><b>{{ t('chat.relation', 'Relation') }}</b>{{ relationshipScore.toFixed(2) }}</span>
            <span><b>{{ t('chat.unlocks', 'Unlocks') }}</b>{{ unlockedContentCount }}</span>
          </div>
        </section>
      </div>

      <div v-else class="insight-empty">
        <MousePointer2 :size="24" aria-hidden="true" />
        <strong>{{ t('chat.select-character', 'Select a character') }}</strong>
        <p>{{ t('chat.no-insight-data', 'No evaluation or runtime data is available.') }}</p>
      </div>
    </aside>

    <div v-if="clearConfirmationOpen" class="modal-backdrop" @mousedown.self="clearConfirmationOpen = false">
      <section class="confirm-dialog" role="alertdialog" aria-modal="true" aria-labelledby="chat-clear-title">
        <CircleAlert :size="24" aria-hidden="true" />
        <h2 id="chat-clear-title">{{ t('chat.clear-title', 'Clear this conversation?') }}</h2>
        <p>{{ t('chat.clear-copy', 'Messages and the current evaluation for this character will be removed.') }}</p>
        <footer><button class="btn btn-secondary btn-sm" @click="clearConfirmationOpen = false">{{ t('common.cancel', 'Cancel') }}</button><button class="btn btn-danger btn-sm" @click="confirmClearChat">{{ t('chat.clear', 'Clear') }}</button></footer>
      </section>
    </div>

    <Transition name="fade">
      <section v-if="activeEvent" class="event-toast" role="status">
        <div><strong>{{ activeEvent.description }}</strong><span v-for="(result, index) in activeEventActionResults" :key="`${activeEvent.event_id}-${index}`">{{ storyActionLabel(result) }}</span></div>
        <button class="icon-command" :title="t('common.close', 'Close')" :aria-label="t('common.close', 'Close')" @click="clearActiveEvent"><X :size="14" aria-hidden="true" /></button>
      </section>
    </Transition>

    <Transition name="fade">
      <section v-if="errorMessage" class="error-toast" role="status">
        <span>{{ errorMessage }}</span>
        <button class="icon-command" :title="t('common.close', 'Close')" :aria-label="t('common.close', 'Close')" @click="errorMessage = null"><X :size="14" aria-hidden="true" /></button>
      </section>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useRoute } from 'vue-router'
import {
  Activity,
  ArrowLeft,
  CheckCircle2,
  ChevronRight,
  CircleAlert,
  Gauge,
  HeartHandshake,
  LoaderCircle,
  MessageCircleMore,
  MessagesSquare,
  Milestone,
  MonitorCog,
  MousePointer2,
  PanelRightOpen,
  ScanSearch,
  Search,
  SearchX,
  Send,
  ShieldCheck,
  Smile,
  Trash2,
  UserRound,
  UsersRound,
  X,
} from '@lucide/vue'
import { hasTauriRuntime, invokeCommand } from '../lib/tauri'
import { useI18n } from '../lib/i18n'
import { loadStoryCharacters } from '../lib/storyContent'
import { generateWebGpuChat } from '../lib/webgpuInference'
import type { StoryEventAction } from '../lib/storyEvents'
import {
  loadStoryProgress,
  type StoryEventActionResult,
  type StoryEventApplication,
  type StoryProgressSnapshot,
} from '../lib/storyProgress'

const { locale, t } = useI18n()
const route = useRoute()

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

type InsightTab = 'evaluation' | 'safety' | 'events' | 'runtime'

const characters = ref<CharacterInfo[]>([])
const selectedCharacter = ref<CharacterInfo | null>(null)
const messages = ref<ChatMessage[]>([])
const inputText = ref('')
const characterSearch = ref('')
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
const insightTab = ref<InsightTab>('evaluation')
const compactInsightOpen = ref(false)
const clearConfirmationOpen = ref(false)
const messagesRef = ref<HTMLDivElement>()
const inputRef = ref<HTMLTextAreaElement>()
const STREAM_FAILURE_BUBBLE = 'Generation failed before the streamed reply completed.'
const desktopRuntimeAvailable = hasTauriRuntime()

let streamUnlisteners: UnlistenFn[] = []

const messageCount = computed(() => messages.value.length)
const playerMessageCount = computed(() => messages.value.filter((m) => m.role === 'player').length)
const filteredCharacters = computed(() => {
  const query = characterSearch.value.trim().toLocaleLowerCase()
  if (!query) return characters.value
  return characters.value.filter((character) => [character.name, character.id]
    .some((value) => value.toLocaleLowerCase().includes(query)))
})
const relationshipClass = computed(() => {
  if (relationshipScore.value >= 0.6) return 'rel-high'
  if (relationshipScore.value >= 0.3) return 'rel-mid'
  return 'rel-low'
})

const runtimeSafetyFlags = computed(() => {
  const trace = safetyTrace.value
  return [
    { key: 'mind', label: t('chat.safety.mind', 'Mind contract'), active: !!trace?.mind_contract_applied },
    { key: 'knowledge', label: t('chat.safety.knowledge', 'Knowledge context'), active: !!trace?.knowledge_context_pinned },
    { key: 'input', label: t('chat.safety.input', 'Input guard'), active: !!trace?.input_prompt_injection_detected || !!trace?.input_private_reasoning_request_detected },
    { key: 'response', label: t('chat.safety.response', 'Response guard'), active: !!trace?.response_guard_applied },
    { key: 'memory', label: t('chat.safety.memory', 'Memory guard'), active: !!trace?.memory_guard_applied },
    { key: 'relation', label: t('chat.safety.relation', 'Relation guard'), active: !!trace?.relationship_delta_blocked },
    { key: 'stream', label: t('chat.safety.stream', 'Stream guard'), active: !!trace?.stream_guard_applied },
  ]
})

const activeSafetyFlagCount = computed(() => runtimeSafetyFlags.value.filter((flag) => flag.active).length)

const safetyTraceSummary = computed(() => {
  const trace = safetyTrace.value
  if (!trace) return t('chat.no-runtime-trace', 'No runtime trace yet.')
  const notes = trace.guard_notes || []
  const refSummary = trace.pinned_knowledge_ref_ids?.length
    ? t('chat.trace-refs', 'Refs: {refs}', { refs: trace.pinned_knowledge_ref_ids.join(', ') })
    : ''
  if (!notes.length || notes.includes('no_runtime_safety_interventions')) {
    return refSummary
      ? t('chat.no-interventions-with-refs', 'No runtime safety interventions. {refs}', { refs: refSummary })
      : t('chat.no-interventions', 'No runtime safety interventions.')
  }
  return [...notes.map(formatSafetyNote), refSummary].filter(Boolean).join(' · ')
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

function emotionLabel(emotion: string | null | undefined): string {
  const value = (emotion || 'neutral').trim().toLocaleLowerCase()
  const labels: Record<string, string> = {
    neutral: t('chat.emotion.neutral', 'Neutral'),
    happy: t('chat.emotion.happy', 'Happy'),
    sad: t('chat.emotion.sad', 'Sad'),
    angry: t('chat.emotion.angry', 'Angry'),
    surprised: t('chat.emotion.surprised', 'Surprised'),
    thinking: t('chat.emotion.thinking', 'Thinking'),
    worried: t('chat.emotion.worried', 'Worried'),
    excited: t('chat.emotion.excited', 'Excited'),
    calm: t('chat.emotion.calm', 'Calm'),
  }
  return labels[value] || emotion || labels.neutral
}

function formatTime(ts: string): string {
  try {
    return new Date(ts).toLocaleTimeString(locale.value, { hour: '2-digit', minute: '2-digit' })
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
  const labels: Record<string, string> = {
    no_runtime_safety_interventions: t('quality.guard-note.no-interventions', 'No runtime interventions'),
    character_mind_contract_applied: t('quality.guard-note.mind-contract', 'Character mind contract applied'),
    pinned_knowledge_context_applied: t('quality.guard-note.pinned-knowledge', 'Pinned knowledge context applied'),
    input_prompt_injection_detected: t('quality.guard-note.injection-detected', 'Input prompt injection detected'),
    input_private_reasoning_request_detected: t('quality.guard-note.reasoning-request', 'Private reasoning request detected'),
    private_reasoning_blocked: t('quality.guard-note.reasoning-blocked', 'Private reasoning blocked'),
    identity_drift_blocked: t('quality.guard-note.identity-blocked', 'Identity drift blocked'),
    style_drift_blocked: t('quality.guard-note.style-blocked', 'Style drift blocked'),
    memory_guard_applied: t('quality.guard-note.memory-guard', 'Memory guard applied'),
    relationship_delta_blocked: t('quality.guard-note.relationship-blocked', 'Relationship delta blocked'),
    stream_guard_applied: t('quality.guard-note.stream-guard', 'Stream guard applied'),
  }
  return labels[note] || note.replace(/_/g, ' ')
}

function streamFailureBubble(): string {
  return t('chat.stream-failure', STREAM_FAILURE_BUBBLE)
}

function eventDecisionReason(decision: EventTriggerDecision): string {
  if (decision.triggered) {
    const metric = decision.actual_score_metric && decision.actual_score !== null && decision.actual_score !== undefined
      ? `${decision.actual_score_metric} ${percent(decision.actual_score)}`
      : t('chat.relation-value', 'Relation {value}', { value: decision.actual_relationship.toFixed(2) })
    return t('chat.event-ready-reason', '{metric} · {count} evaluations', { metric, count: decision.actual_evaluation_count })
  }
  return decision.blocked_reasons[0] || t('chat.waiting-trigger-rule', 'Waiting for trigger rule')
}

function shortRuleFingerprint(decision: EventTriggerDecision): string {
  const fingerprint = decision.rule_fingerprint || decision.rule?.rule_fingerprint || ''
  return fingerprint ? fingerprint.slice(0, 10) : ''
}

function storyActionLabel(result: StoryEventActionResult): string {
  const state = result.changed ? t('chat.action-unlocked', 'unlocked') : t('chat.action-already-unlocked', 'already unlocked')
  if (result.action.type === 'unlock_scene') return t('chat.action-scene', 'Scene {id} {state}', { id: result.action.scene_id, state })
  if (result.action.type === 'unlock_dialogue') return t('chat.action-dialogue', 'Dialogue {id} {state}', { id: result.action.dialogue_id, state })
  if (result.action.type === 'unlock_ending') return t('chat.action-ending', 'Ending {id} {state}', { id: result.action.ending_id, state })
  return t('chat.action-flag', 'Flag {id} {state}', {
    id: result.action.flag,
    state: result.changed ? t('chat.action-updated', 'updated') : t('chat.action-unchanged', 'unchanged'),
  })
}

function requestClearChat() {
  if (!selectedCharacter.value || messages.value.length === 0) return
  clearConfirmationOpen.value = true
}

async function confirmClearChat() {
  clearConfirmationOpen.value = false
  await clearChat()
}

function browserPreviewEvaluationReport(): ConversationEvaluationReport {
  const participation = Math.min(1, 0.42 + playerMessageCount.value * 0.04)
  return {
    evaluation: {
      friendliness: participation,
      engagement: Math.min(1, participation + 0.05),
      creativity: Math.max(0.35, participation - 0.04),
      overall_score: participation,
      summary: t('chat.preview-score-summary', 'Local test score based on message activity.'),
    },
    event_trigger_decisions: [],
    triggerable_events: [],
  }
}

async function completeWebGpuReply(assistantMessage: ChatMessage, character: CharacterInfo) {
  const conversation = messages.value
    .filter((message) => message !== assistantMessage && message.content.trim())
    .slice(-16)
    .map((message) => ({
      role: message.role === 'player' ? 'user' as const : 'assistant' as const,
      content: message.content,
    }))
  const generated = await generateWebGpuChat([
    {
      role: 'system',
      content: `You are ${character.name}. Stay in character and reply in ${locale.value}. Character context: ${character.description}`,
    },
    ...conversation,
  ], {
    onChunk: (chunk) => {
      assistantMessage.content += chunk
      scrollToBottom()
    },
  })
  if (!assistantMessage.content.trim()) assistantMessage.content = generated
  assistantMessage.emotion = character.emotion || 'neutral'
  currentEmotion.value = assistantMessage.emotion || 'neutral'
  isStreaming.value = false
  scrollToBottom()
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
  clearConfirmationOpen.value = false
  compactInsightOpen.value = false
  insightTab.value = 'evaluation'
  try {
    const [history, audit] = await Promise.all([
      invokeCommand<ChatMessage[]>('get_chat_history', { characterId: char.id }, []),
      invokeCommand<ChatSessionAuditReport | null>('get_chat_session_audit', { characterId: char.id }, null),
    ])
    messages.value = history
    relationshipScore.value = audit?.relationship_score || 0
    evaluation.value = audit?.last_evaluation || null
    safetyTrace.value = audit?.last_safety_trace || null
    eventDecisions.value = audit?.event_trigger_decisions || []
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
      errorMessage.value = event.payload || t('chat.generation-failed', 'Generation failed')
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
    if (desktopRuntimeAvailable) {
      await attachStreamListeners(assistantMessage)
      await invokeCommand<void>('send_chat_message_stream', {
        characterId: character.id,
        message: text,
      })
      await refreshRelationship()
    } else {
      await completeWebGpuReply(assistantMessage, character)
    }
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
  } catch (e) {
    console.error(e)
  }
}

async function refreshEvaluation() {
  if (!selectedCharacter.value) return
  try {
    const characterId = selectedCharacter.value.id
    const report = await invokeCommand<ConversationEvaluationReport>(
      'evaluate_conversation_report',
      { characterId },
      browserPreviewEvaluationReport,
    )
    evaluation.value = report.evaluation
    eventDecisions.value = report.event_trigger_decisions || []
  } catch (e) {
    errorMessage.value = String(e)
  }
}

async function clearChat() {
  if (!selectedCharacter.value) return
  try {
    await invokeCommand<void>('clear_chat_history', { characterId: selectedCharacter.value.id }, () => undefined)
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
      loadStoryCharacters(),
      loadStoryProgress(),
    ])
    characters.value = loadedCharacters.map(character => ({
      id: character.id,
      name: character.name,
      description: character.description,
      emotion: character.emotion,
      live2d_model_path: character.live2d_model_path ?? null,
    }))
    storyProgress.value = progress
    const requestedCharacter = typeof route.query.character === 'string' ? route.query.character : ''
    const initialCharacter = characters.value.find(character => character.id === requestedCharacter)
    if (initialCharacter) await selectCharacter(initialCharacter)
  } catch (e) {
    errorMessage.value = String(e)
  } finally {
    charactersLoading.value = false
  }
})

onUnmounted(cleanupStreamListeners)
</script>

<style scoped>
.chat-workbench {
  position: relative;
  display: grid;
  height: calc(100svh - 56px);
  min-height: 0;
  grid-template-columns: 230px minmax(0, 1fr) 320px;
  overflow: hidden;
  background: var(--surface-0);
}

.character-rail,
.conversation-panel,
.insight-panel {
  min-width: 0;
  min-height: 0;
  background: var(--surface-1);
}

.character-rail {
  display: grid;
  grid-template-rows: 52px 42px minmax(0, 1fr);
  gap: 0;
  border-right: 1px solid var(--border);
  overflow: hidden;
}

.rail-header,
.conversation-header,
.insight-header,
.eyebrow,
.character-search,
.session-metrics,
.header-actions,
.metric-pill,
.runtime-badge,
.msg-meta,
.stream-placeholder,
.insight-tabs,
.section-head,
.event-toast,
.error-toast {
  display: flex;
  align-items: center;
}

.rail-header {
  min-width: 0;
  justify-content: space-between;
  gap: 10px;
  padding: 8px 11px;
  border-bottom: 1px solid var(--border);
}

.rail-header > div { display: grid; min-width: 0; gap: 2px; }
.rail-header h1,
.conversation-title h2 {
  margin: 0;
  overflow: hidden;
  color: var(--text-primary);
  font-size: 14px;
  line-height: 1.2;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.rail-header > strong {
  color: var(--brand-light);
  font-family: var(--font-mono);
  font-size: 9px;
}

.eyebrow {
  gap: 5px;
  color: var(--text-tertiary);
  font-size: 8px;
  font-weight: 800;
  letter-spacing: 0;
  text-transform: uppercase;
}

.character-search {
  height: 30px;
  gap: 7px;
  margin: 6px 9px;
  padding: 0 8px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--surface-0);
  color: var(--text-tertiary);
}

.character-search:focus-within { border-color: var(--border-strong); color: var(--text-secondary); }
.character-search input { width: 100%; min-width: 0; border: 0; outline: 0; background: transparent; color: var(--text-primary); font-size: 9px; }
.character-search input::placeholder { color: var(--text-tertiary); }

.character-list {
  min-height: 0;
  padding: 6px;
  overflow-y: auto;
  scrollbar-width: none;
}

.character-list::-webkit-scrollbar,
.messages-area::-webkit-scrollbar,
.insight-content::-webkit-scrollbar { display: none; }

.character-row {
  display: grid;
  width: 100%;
  min-width: 0;
  min-height: 48px;
  grid-template-columns: 32px minmax(0, 1fr) auto;
  align-items: center;
  gap: 8px;
  margin-bottom: 3px;
  padding: 6px 7px;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  color: inherit;
  text-align: left;
  cursor: pointer;
}

.character-row:hover { background: var(--surface-2); }
.character-row.selected { border-color: color-mix(in srgb, var(--brand) 36%, var(--border)); background: color-mix(in srgb, var(--brand) 9%, var(--surface-1)); }
.character-row > svg { color: var(--text-tertiary); }
.character-row.selected > svg { color: var(--brand-light); }

.avatar,
.msg-avatar {
  display: inline-flex;
  flex: 0 0 auto;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  font-weight: 800;
}

.avatar {
  width: 32px;
  height: 32px;
  background: color-mix(in srgb, var(--brand) 17%, var(--surface-2));
  color: var(--brand-light);
  font-size: 10px;
}

.character-copy { display: grid; min-width: 0; gap: 2px; }
.row-title, .row-subtitle { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.row-title { color: var(--text-primary); font-size: 10px; font-weight: 800; }
.row-subtitle { color: var(--text-tertiary); font-size: 8px; }

.empty-rail,
.conversation-empty,
.select-state,
.insight-empty,
.compact-empty {
  display: grid;
  place-items: center;
  align-content: center;
  gap: 7px;
  color: var(--text-tertiary);
  text-align: center;
}

.empty-rail { min-height: 120px; font-size: 9px; }

.conversation-panel {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--surface-0);
}

.conversation-header {
  display: grid;
  min-height: 58px;
  flex: 0 0 auto;
  grid-template-columns: 34px minmax(120px, 1fr) auto auto auto;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border-bottom: 1px solid var(--border);
  background: var(--surface-1);
}

.icon-command {
  display: inline-grid;
  width: 34px;
  height: 34px;
  flex: 0 0 34px;
  place-items: center;
  padding: 0;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--surface-2);
  color: var(--text-secondary);
  cursor: pointer;
}

.icon-command:hover:not(:disabled) { border-color: var(--border-strong); color: var(--text-primary); }
.icon-command:disabled { cursor: not-allowed; opacity: 0.42; }
.danger-command:hover:not(:disabled) { border-color: color-mix(in srgb, var(--danger) 45%, var(--border)); color: var(--danger); }
.conversation-title { display: grid; min-width: 0; gap: 3px; }
.runtime-badge, .metric-pill { gap: 5px; border-radius: 999px; white-space: nowrap; }
.runtime-badge { padding: 4px 7px; border: 1px solid color-mix(in srgb, var(--success) 28%, var(--border)); color: var(--success); font-size: 8px; font-weight: 800; }
.runtime-badge.webgpu { border-color: var(--border-strong); color: var(--text-primary); }
.session-metrics, .header-actions { min-width: 0; gap: 5px; }
.metric-pill { min-height: 26px; padding: 3px 7px; border: 1px solid var(--border); background: var(--surface-0); color: var(--text-secondary); font-size: 8px; font-weight: 750; }
.metric-pill.rel-high { color: var(--success); }
.metric-pill.rel-mid { color: var(--warning); }
.metric-pill.rel-low { color: var(--text-secondary); }
.insight-toggle { display: none; }

.messages-area {
  min-height: 0;
  flex: 1;
  padding: 18px clamp(14px, 4vw, 48px);
  overflow-y: auto;
  scrollbar-width: none;
}

.message {
  display: flex;
  max-width: min(720px, 84%);
  gap: 9px;
  margin-bottom: 14px;
  animation: message-enter 0.16s ease;
}

.msg-player { flex-direction: row-reverse; margin-left: auto; }
.msg-avatar { width: 30px; height: 30px; background: color-mix(in srgb, var(--brand) 17%, var(--surface-2)); color: var(--brand-light); font-size: 9px; }
.msg-player .msg-avatar { background: color-mix(in srgb, var(--warning) 16%, var(--surface-2)); color: var(--warning); }
.msg-stack { min-width: 0; }
.msg-bubble { padding: 9px 11px; border: 1px solid var(--border); border-radius: 6px; background: var(--surface-1); color: var(--text-primary); font-size: 11px; line-height: 1.55; white-space: pre-wrap; overflow-wrap: anywhere; }
.msg-player .msg-bubble { border-color: color-mix(in srgb, var(--warning) 28%, var(--border)); background: color-mix(in srgb, var(--warning) 7%, var(--surface-1)); }
.msg-meta { min-width: 0; gap: 7px; padding: 4px 2px 0; color: var(--text-tertiary); font-size: 8px; }
.stream-placeholder { gap: 6px; color: var(--text-tertiary); }

.conversation-empty { min-height: 100%; padding: 24px; }
.conversation-empty h3, .select-state h2, .insight-empty strong { margin: 0; color: var(--text-primary); font-size: 13px; }
.conversation-empty p, .select-state p, .insight-empty p { max-width: 380px; margin: 0; color: var(--text-tertiary); font-size: 9px; line-height: 1.55; }
.select-state { min-height: 0; flex: 1; padding: 24px; }
.select-state-inner { display: grid; max-width: 390px; justify-items: center; gap: 8px; }

.composer {
  display: grid;
  min-height: 62px;
  flex: 0 0 auto;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: end;
  gap: 8px;
  padding: 9px 10px;
  border-top: 1px solid var(--border);
  background: var(--surface-1);
}

.composer textarea { width: 100%; min-height: 42px; max-height: 144px; resize: none; padding: 10px 11px; border: 1px solid var(--border); border-radius: 6px; outline: 0; background: var(--surface-0); color: var(--text-primary); font: inherit; font-size: 10px; line-height: 1.45; }
.composer textarea:focus { border-color: var(--border-strong); box-shadow: var(--shadow-brand); }
.send-btn { display: inline-flex; min-width: 86px; height: 42px; align-items: center; justify-content: center; gap: 6px; padding: 0 12px; border: 0; border-radius: 6px; background: var(--brand); color: var(--surface-0); font-size: 10px; font-weight: 800; cursor: pointer; }
.send-btn:hover:not(:disabled) { background: var(--brand-light); }
.send-btn:disabled { cursor: not-allowed; opacity: 0.48; }

.insight-panel {
  display: grid;
  grid-template-rows: 52px 42px minmax(0, 1fr);
  border-left: 1px solid var(--border);
  overflow: hidden;
}

.insight-header { min-width: 0; justify-content: space-between; gap: 8px; padding: 8px 10px; border-bottom: 1px solid var(--border); }
.insight-header > div { display: grid; min-width: 0; gap: 3px; }
.insight-header strong { overflow: hidden; color: var(--text-primary); font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
.insight-close { display: none; }
.insight-tabs { min-width: 0; gap: 2px; padding: 5px 6px; border-bottom: 1px solid var(--border); }
.insight-tabs button { display: inline-flex; min-width: 0; height: 30px; flex: 1; align-items: center; justify-content: center; gap: 4px; padding: 0 5px; border: 1px solid transparent; border-radius: 5px; background: transparent; color: var(--text-tertiary); font-size: 8px; cursor: pointer; }
.insight-tabs button span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.insight-tabs button.active { border-color: var(--border); background: var(--surface-2); color: var(--text-primary); }
.insight-content { min-height: 0; overflow-y: auto; scrollbar-width: none; }
.insight-section { display: grid; align-content: start; gap: 12px; padding: 12px; }
.section-head { min-width: 0; justify-content: space-between; gap: 8px; color: var(--text-tertiary); font-size: 8px; font-weight: 800; text-transform: uppercase; }
.section-head strong { color: var(--text-primary); font-family: var(--font-mono); font-size: 9px; }
.text-command { padding: 0; border: 0; background: transparent; color: var(--brand-light); font-size: 8px; cursor: pointer; }
.text-command:disabled { cursor: not-allowed; color: var(--text-tertiary); }
.insight-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 6px; }
.insight-item { display: grid; gap: 3px; padding: 8px; border: 1px solid var(--border); border-radius: 6px; background: var(--surface-2); }
.insight-value { color: var(--brand-light); font-family: var(--font-mono); font-size: 16px; font-weight: 800; }
.insight-label { color: var(--text-tertiary); font-size: 8px; text-transform: uppercase; }
.score-stack { display: grid; gap: 11px; padding-top: 2px; }
.score-row { display: grid; min-width: 0; grid-template-columns: minmax(0, 1fr) auto; gap: 5px 8px; color: var(--text-secondary); font-size: 9px; }
.score-row strong { color: var(--text-primary); font-family: var(--font-mono); font-size: 9px; }
.bar-track { height: 4px; grid-column: 1 / -1; border-radius: 999px; background: var(--surface-3); overflow: hidden; }
.bar-fill { height: 100%; max-width: 100%; border-radius: inherit; background: var(--brand); }
.bar-fill.engagement { background: var(--info); }
.bar-fill.creativity { background: var(--warning); }
.bar-fill.overall { background: var(--success); }
.overall-row { padding-top: 7px; border-top: 1px solid var(--border); }
.eval-summary, .trace-note { margin: 0; color: var(--text-tertiary); font-size: 9px; line-height: 1.55; overflow-wrap: anywhere; }
.compact-empty { min-height: 180px; font-size: 9px; }
.safety-flag-list { display: grid; gap: 4px; }
.safety-pill { display: grid; min-height: 31px; grid-template-columns: 16px minmax(0, 1fr) auto; align-items: center; gap: 6px; padding: 5px 7px; border-radius: 5px; color: var(--success); }
.safety-pill span { overflow: hidden; color: var(--text-secondary); font-size: 9px; text-overflow: ellipsis; white-space: nowrap; }
.safety-pill strong { color: var(--text-tertiary); font-size: 8px; }
.safety-pill.active { background: color-mix(in srgb, var(--warning) 8%, transparent); color: var(--warning); }
.safety-pill.active strong { color: var(--warning); }
.event-decision-list { display: grid; gap: 5px; }
.event-decision-row { display: grid; min-width: 0; grid-template-columns: minmax(0, 1fr) auto; gap: 4px 8px; padding: 8px; border: 1px solid color-mix(in srgb, var(--danger) 24%, var(--border)); border-radius: 6px; background: color-mix(in srgb, var(--danger) 6%, var(--surface-1)); }
.event-decision-row.triggered { border-color: color-mix(in srgb, var(--success) 26%, var(--border)); background: color-mix(in srgb, var(--success) 6%, var(--surface-1)); }
.event-decision-row span, .event-decision-row small, .rule-fingerprint { min-width: 0; overflow-wrap: anywhere; }
.event-decision-row span { color: var(--text-secondary); font-family: var(--font-mono); font-size: 8px; font-weight: 800; }
.event-decision-row strong { color: var(--danger); font-size: 8px; text-transform: uppercase; }
.event-decision-row.triggered strong { color: var(--success); }
.event-decision-row small, .rule-fingerprint { grid-column: 1 / -1; color: var(--text-tertiary); font-size: 8px; line-height: 1.4; }
.rule-fingerprint { font-family: var(--font-mono); }
.runtime-list { display: grid; gap: 4px; }
.runtime-list span { display: flex; min-height: 32px; align-items: center; justify-content: space-between; gap: 10px; padding: 5px 7px; border-bottom: 1px solid var(--border); color: var(--text-secondary); font-size: 9px; }
.runtime-list b { color: var(--text-tertiary); font-weight: 650; }
.insight-empty { min-height: 100%; padding: 20px; }

.modal-backdrop {
  position: absolute;
  z-index: 80;
  inset: 0;
  display: grid;
  place-items: center;
  padding: 16px;
  background: color-mix(in srgb, var(--surface-0) 78%, transparent);
}

.confirm-dialog { display: grid; width: min(360px, 100%); gap: 9px; padding: 16px; border: 1px solid var(--border-strong); border-radius: 6px; background: var(--surface-1); box-shadow: var(--shadow-lg); }
.confirm-dialog > svg { color: var(--danger); }
.confirm-dialog h2 { margin: 0; color: var(--text-primary); font-size: 14px; }
.confirm-dialog p { margin: 0; color: var(--text-secondary); font-size: 10px; line-height: 1.5; }
.confirm-dialog footer { display: flex; justify-content: flex-end; gap: 7px; margin-top: 4px; }

.event-toast, .error-toast { position: fixed; z-index: 100; left: 50%; width: min(440px, calc(100vw - 28px)); justify-content: space-between; gap: 10px; transform: translateX(-50%); padding: 9px 10px; border: 1px solid var(--border-strong); border-radius: 6px; box-shadow: var(--shadow-lg); }
.event-toast { top: 68px; background: color-mix(in srgb, var(--success) 14%, var(--surface-1)); color: var(--text-primary); }
.event-toast > div { display: grid; min-width: 0; gap: 2px; }
.event-toast strong { font-size: 10px; }
.event-toast span { color: var(--text-secondary); font-size: 8px; }
.error-toast { bottom: 18px; background: color-mix(in srgb, var(--danger) 14%, var(--surface-1)); color: var(--danger); font-size: 9px; }
.event-toast .icon-command, .error-toast .icon-command { width: 28px; height: 28px; flex-basis: 28px; background: transparent; }
.fade-enter-active, .fade-leave-active { transition: opacity 0.18s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
.spin { animation: chat-spin 0.9s linear infinite; }

@keyframes chat-spin { to { transform: rotate(360deg); } }
@keyframes message-enter { from { opacity: 0; transform: translateY(5px); } }

@media (max-width: 1450px) {
  .chat-workbench { grid-template-columns: 226px minmax(0, 1fr); }
  .insight-panel { position: absolute; z-index: 40; inset: 0 0 0 auto; display: none; width: min(360px, 100%); border-left: 1px solid var(--border-strong); box-shadow: var(--shadow-lg); }
  .insight-panel.compact-open { display: grid; }
  .insight-toggle, .insight-close { display: inline-grid; }
}

@media (min-width: 1200px) and (max-width: 1450px) {
  .chat-workbench:has(.insight-panel.compact-open) .conversation-panel { margin-right: 360px; }
  .chat-workbench:has(.insight-panel.compact-open) .runtime-badge,
  .chat-workbench:has(.insight-panel.compact-open) .session-metrics { display: none; }
}

@media (max-width: 900px) {
  .runtime-badge { display: none; }
  .conversation-header { grid-template-columns: 34px minmax(100px, 1fr) auto auto; }
}

@media (max-width: 760px) {
  .chat-workbench { height: calc(100svh - 56px - 60px - env(safe-area-inset-bottom, 0px)); grid-template-columns: 1fr; grid-template-rows: 132px minmax(0, 1fr); }
  .character-rail { grid-row: 1; grid-template-rows: 38px 34px 60px; border-right: 0; border-bottom: 1px solid var(--border); }
  .rail-header { padding: 5px 9px; }
  .rail-header .eyebrow { display: none; }
  .rail-header h1 { font-size: 13px; }
  .character-search { height: 28px; margin: 3px 9px; }
  .character-list { display: flex; gap: 5px; padding: 5px 9px; overflow-x: auto; overflow-y: hidden; }
  .character-row { min-width: 140px; height: 48px; margin: 0; }
  .character-row > svg { display: none; }
  .empty-rail { min-width: 100%; min-height: 48px; grid-auto-flow: column; }
  .conversation-panel { grid-row: 2; }
  .conversation-header { min-height: 82px; grid-template-columns: 34px minmax(80px, 1fr) auto; grid-template-rows: 34px 28px; align-content: center; gap: 5px 7px; padding: 7px 9px; }
  .conversation-title { grid-column: 2; }
  .header-actions { grid-column: 3; grid-row: 1; }
  .session-metrics { grid-column: 2 / -1; grid-row: 2; }
  .metric-pill { max-width: 150px; overflow: hidden; text-overflow: ellipsis; }
  .messages-area { padding: 12px 10px; }
  .message { max-width: 94%; }
  .composer { min-height: 58px; padding: 8px 9px; }
  .insight-panel { left: 0; width: 100%; }
  .event-toast { top: 62px; }
  .error-toast { bottom: calc(68px + env(safe-area-inset-bottom, 0px)); }
}

@media (max-width: 430px) {
  .send-btn { width: 42px; min-width: 42px; padding: 0; }
  .send-btn span { display: none; }
  .header-actions { gap: 3px; }
  .header-actions .icon-command { width: 32px; height: 32px; flex-basis: 32px; }
  .insight-tabs button span { display: none; }
}
</style>
