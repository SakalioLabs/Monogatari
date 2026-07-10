<template>
  <div class="analytics-page">
    <header class="page-header">
      <div class="header-copy">
        <span class="eyebrow"><Activity :size="13" />{{ t('analytics.eyebrow', 'Runtime telemetry') }}</span>
        <div class="title-row">
          <h1>{{ t('analytics.title', 'Analytics') }}</h1>
          <span class="source-chip" :class="dataSource">
            <Database :size="12" />{{ dataSourceLabel }}
          </span>
        </div>
        <p>{{ t('analytics.description', 'Review player engagement, conversation activity, and character interaction signals.') }}</p>
        <small v-if="lastUpdated" class="last-updated">
          {{ t('analytics.last-updated', 'Updated {time}', { time: formattedLastUpdated }) }}
        </small>
      </div>

      <div class="header-actions">
        <button
          class="icon-command"
          :class="{ spinning: loading }"
          :disabled="loading"
          :title="t('common.refresh', 'Refresh')"
          :aria-label="t('common.refresh', 'Refresh')"
          @click="refresh(true)"
        >
          <RefreshCw :size="16" />
        </button>
        <button class="btn btn-primary btn-sm" :disabled="exporting" @click="exportData">
          <Download :size="15" />
          {{ exporting ? t('analytics.exporting', 'Exporting') : t('analytics.export', 'Export JSON') }}
        </button>
      </div>
    </header>

    <div v-if="dataSource === 'sample'" class="source-notice">
      <Info :size="15" />
      <span>{{ t('analytics.sample-notice', 'Web preview uses a localized sample dataset. Desktop projects read analytics.json.') }}</span>
    </div>

    <section class="metrics-strip" :aria-label="t('analytics.metrics-label', 'Analytics summary')">
      <article v-for="metric in metrics" :key="metric.key" class="metric-card">
        <div class="metric-head">
          <component :is="metric.icon" :size="16" />
          <span>{{ metric.label }}</span>
        </div>
        <strong>{{ metric.value }}</strong>
        <small>{{ metric.detail }}</small>
      </article>
    </section>

    <section class="analytics-grid">
      <section class="panel" aria-labelledby="character-ranking-title">
        <header class="panel-head">
          <div>
            <span class="eyebrow"><UsersRound :size="13" />{{ t('analytics.characters', 'Characters') }}</span>
            <h2 id="character-ranking-title">{{ t('analytics.top-interactions', 'Top interactions') }}</h2>
          </div>
          <span class="panel-total">{{ summary.top_characters.length }}</span>
        </header>

        <div v-if="summary.top_characters.length" class="ranking-list">
          <div v-for="(item, index) in summary.top_characters" :key="item[0]" class="ranking-item">
            <span class="rank">{{ index + 1 }}</span>
            <div class="rank-main">
              <div class="rank-copy"><strong>{{ item[0] }}</strong><span>{{ item[1] }}</span></div>
              <div class="rank-track" role="progressbar" aria-valuemin="0" :aria-valuemax="topCharacterMax" :aria-valuenow="item[1]">
                <span :style="{ width: rankingWidth(item[1], topCharacterMax) }"></span>
              </div>
            </div>
          </div>
        </div>
        <div v-else class="inline-empty">
          <UsersRound :size="22" />
          <span>{{ t('analytics.characters-empty', 'Character interactions will appear after the first conversation.') }}</span>
        </div>
      </section>

      <section class="panel" aria-labelledby="choice-ranking-title">
        <header class="panel-head">
          <div>
            <span class="eyebrow"><ListChecks :size="13" />{{ t('analytics.choices', 'Choices') }}</span>
            <h2 id="choice-ranking-title">{{ t('analytics.most-popular', 'Most selected') }}</h2>
          </div>
          <span class="panel-total">{{ summary.top_choices.length }}</span>
        </header>

        <div v-if="summary.top_choices.length" class="ranking-list">
          <div v-for="(item, index) in summary.top_choices" :key="item[0]" class="ranking-item">
            <span class="rank">{{ index + 1 }}</span>
            <div class="rank-main">
              <div class="rank-copy"><strong>{{ item[0] }}</strong><span>{{ item[1] }}</span></div>
              <div class="rank-track choice" role="progressbar" aria-valuemin="0" :aria-valuemax="topChoiceMax" :aria-valuenow="item[1]">
                <span :style="{ width: rankingWidth(item[1], topChoiceMax) }"></span>
              </div>
            </div>
          </div>
        </div>
        <div v-else class="inline-empty">
          <ListChecks :size="22" />
          <span>{{ t('analytics.choices-empty', 'Player choice counts will appear after Story Mode branches are selected.') }}</span>
        </div>
      </section>

      <section class="panel session-panel" aria-labelledby="session-overview-title">
        <header class="panel-head">
          <div>
            <span class="eyebrow"><Gauge :size="13" />{{ t('analytics.engagement', 'Engagement') }}</span>
            <h2 id="session-overview-title">{{ t('analytics.session-overview', 'Session overview') }}</h2>
          </div>
          <span class="panel-total">{{ formatDuration(summary.avg_session_duration_ms) }}</span>
        </header>

        <div class="session-table">
          <div class="session-row">
            <span><Clock3 :size="14" />{{ t('analytics.avg-session-duration', 'Average session duration') }}</span>
            <strong>{{ formatDuration(summary.avg_session_duration_ms) }}</strong>
          </div>
          <div class="session-row">
            <span><MessagesSquare :size="14" />{{ t('analytics.conversations', 'Conversations') }}</span>
            <strong>{{ summary.conversation_count }}</strong>
          </div>
          <div class="session-row">
            <span><HeartHandshake :size="14" />{{ t('analytics.avg-relationship-score', 'Average relationship score') }}</span>
            <strong>{{ formatRelationship(summary.avg_relationship_score) }}</strong>
          </div>
        </div>

        <div class="relationship-scale">
          <div class="scale-copy">
            <span>{{ t('analytics.relationship-hostile', 'Hostile') }}</span>
            <strong>{{ t('analytics.relationship-range', 'Relationship range') }}</strong>
            <span>{{ t('analytics.relationship-close', 'Close') }}</span>
          </div>
          <div class="scale-track"><span class="scale-marker" :style="{ left: relationshipPosition }"></span></div>
          <div class="scale-values"><span>-1.0</span><span>0</span><span>1.0</span></div>
        </div>
      </section>
    </section>

    <Transition name="toast">
      <div v-if="notice" class="analytics-toast" :class="notice.type" role="status">
        <CheckCircle2 v-if="notice.type === 'success'" :size="16" />
        <CircleAlert v-else :size="16" />
        <span>{{ notice.message }}</span>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import {
  Activity,
  CheckCircle2,
  CircleAlert,
  Clock3,
  Database,
  Download,
  Gauge,
  HeartHandshake,
  Info,
  ListChecks,
  MessageCircleMore,
  MessagesSquare,
  RefreshCw,
  Sparkles,
  UsersRound,
} from '@lucide/vue'
import { hasTauriRuntime, invokeCommand } from '../lib/tauri'
import { useI18n } from '../lib/i18n'

const { locale, t } = useI18n()

interface AnalyticsSummary {
  total_events: number
  session_count: number
  avg_session_duration_ms: number
  top_choices: [string, number][]
  top_characters: [string, number][]
  conversation_count: number
  avg_relationship_score: number
}

type DataSource = 'project' | 'sample' | 'unavailable'
type Notice = { type: 'success' | 'error'; message: string }

const emptySummary = (): AnalyticsSummary => ({
  total_events: 0,
  session_count: 0,
  avg_session_duration_ms: 0,
  top_choices: [],
  top_characters: [],
  conversation_count: 0,
  avg_relationship_score: 0,
})

const summary = ref<AnalyticsSummary>(emptySummary())
const dataSource = ref<DataSource>('unavailable')
const loading = ref(false)
const exporting = ref(false)
const lastUpdated = ref<Date | null>(null)
const notice = ref<Notice | null>(null)
let noticeTimer: number | undefined

const previewSummary = computed<AnalyticsSummary>(() => ({
  total_events: 24,
  session_count: 3,
  avg_session_duration_ms: 180000,
  top_choices: [
    [t('analytics.preview.choice-1', 'Tell me about the cherry blossoms'), 8],
    [t('analytics.preview.choice-2', 'What is your favorite season?'), 6],
    [t('analytics.preview.choice-3', 'Let us explore the park'), 5],
  ],
  top_characters: [['Sakura', 15], ['Luna', 8], ['Kenji', 4]],
  conversation_count: 12,
  avg_relationship_score: 0.45,
}))

const metrics = computed(() => [
  {
    key: 'events',
    icon: Sparkles,
    label: t('analytics.events', 'Events'),
    value: summary.value.total_events,
    detail: t('analytics.events-detail', 'Recorded runtime actions'),
  },
  {
    key: 'sessions',
    icon: Activity,
    label: t('analytics.sessions', 'Sessions'),
    value: summary.value.session_count,
    detail: t('analytics.sessions-detail', 'Completed player sessions'),
  },
  {
    key: 'conversations',
    icon: MessageCircleMore,
    label: t('analytics.conversations', 'Conversations'),
    value: summary.value.conversation_count,
    detail: t('analytics.conversations-detail', 'Character conversations'),
  },
  {
    key: 'relationship',
    icon: HeartHandshake,
    label: t('analytics.avg-relationship', 'Average relationship'),
    value: formatRelationship(summary.value.avg_relationship_score),
    detail: t('analytics.relationship-detail', 'Normalized from -1.0 to 1.0'),
  },
])

const dataSourceLabel = computed(() => ({
  project: t('analytics.source-project', 'Project data'),
  sample: t('analytics.source-sample', 'Sample data'),
  unavailable: t('analytics.source-unavailable', 'Unavailable'),
})[dataSource.value])

const formattedLastUpdated = computed(() => lastUpdated.value
  ? new Intl.DateTimeFormat(locale.value, { hour: '2-digit', minute: '2-digit', second: '2-digit' }).format(lastUpdated.value)
  : '')
const topCharacterMax = computed(() => Math.max(1, ...summary.value.top_characters.map(item => item[1])))
const topChoiceMax = computed(() => Math.max(1, ...summary.value.top_choices.map(item => item[1])))
const relationshipPosition = computed(() => `${Math.max(0, Math.min(100, ((summary.value.avg_relationship_score + 1) / 2) * 100))}%`)

function rankingWidth(value: number, maximum: number): string {
  return `${Math.max(4, Math.min(100, (value / Math.max(1, maximum)) * 100))}%`
}

function formatRelationship(value: number): string {
  return Number.isFinite(value) ? value.toFixed(2) : '0.00'
}

function formatDuration(ms: number): string {
  if (!Number.isFinite(ms) || ms <= 0) return t('analytics.duration-zero', '0 seconds')
  const minutes = Math.floor(ms / 60000)
  const seconds = Math.floor((ms % 60000) / 1000)
  return minutes > 0
    ? t('analytics.duration-minutes', '{minutes}m {seconds}s', { minutes, seconds })
    : t('analytics.duration-seconds', '{seconds}s', { seconds })
}

async function refresh(announce = false) {
  if (loading.value) return
  loading.value = true
  try {
    if (hasTauriRuntime()) {
      summary.value = await invokeCommand<AnalyticsSummary>('get_analytics_summary')
      dataSource.value = 'project'
    } else {
      summary.value = previewSummary.value
      dataSource.value = 'sample'
    }
    lastUpdated.value = new Date()
    if (announce) notify('success', t('analytics.notice.refreshed', 'Analytics refreshed.'))
  } catch (error) {
    summary.value = emptySummary()
    dataSource.value = 'unavailable'
    notify('error', t('analytics.notice.load-failed', 'Analytics could not be loaded: {error}', { error: String(error) }))
  } finally {
    loading.value = false
  }
}

async function exportData() {
  if (exporting.value) return
  exporting.value = true
  try {
    const data = hasTauriRuntime()
      ? await invokeCommand<string>('export_analytics', { format: 'json' })
      : JSON.stringify({
        schema_version: 1,
        generated_at: new Date().toISOString(),
        source: 'sample',
        summary: summary.value,
      }, null, 2)
    const blob = new Blob([data], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const anchor = document.createElement('a')
    anchor.href = url
    anchor.download = 'analytics-export.json'
    anchor.click()
    URL.revokeObjectURL(url)
    notify('success', t('analytics.notice.exported', 'Analytics JSON exported.'))
  } catch (error) {
    notify('error', t('analytics.notice.export-failed', 'Analytics could not be exported: {error}', { error: String(error) }))
  } finally {
    exporting.value = false
  }
}

function notify(type: Notice['type'], message: string) {
  notice.value = { type, message }
  if (noticeTimer) window.clearTimeout(noticeTimer)
  noticeTimer = window.setTimeout(() => { notice.value = null }, 3200)
}

watch(locale, () => {
  if (dataSource.value === 'sample') summary.value = previewSummary.value
})

onMounted(() => refresh())
onUnmounted(() => {
  if (noticeTimer) window.clearTimeout(noticeTimer)
})
</script>

<style scoped>
.analytics-page { width: min(1180px, 100%); margin: 0 auto; padding: 28px 34px 42px; }
.page-header { display: flex; align-items: flex-start; justify-content: space-between; gap: 22px; margin-bottom: 16px; }
.header-copy { min-width: 0; }
.eyebrow { display: flex; align-items: center; gap: 6px; color: var(--text-tertiary); font-size: 10px; font-weight: 800; text-transform: uppercase; }
.title-row { display: flex; flex-wrap: wrap; align-items: center; gap: 10px; margin-top: 4px; }
.title-row h1 { margin: 0; color: var(--text-primary); font-size: 26px; line-height: 1.15; }
.source-chip { display: inline-flex; min-height: 24px; align-items: center; gap: 5px; padding: 3px 7px; border: 1px solid var(--border); border-radius: 999px; background: var(--surface-1); color: var(--text-secondary); font-size: 9px; font-weight: 800; }
.source-chip.project { border-color: color-mix(in srgb, var(--success) 45%, var(--border)); color: var(--success); }
.source-chip.sample { border-color: color-mix(in srgb, var(--warning) 45%, var(--border)); color: var(--warning); }
.source-chip.unavailable { color: var(--danger); }
.page-header p { max-width: 720px; margin: 7px 0 0; color: var(--text-tertiary); font-size: 12px; line-height: 1.5; }
.last-updated { display: block; margin-top: 7px; color: var(--text-tertiary); font-size: 9px; }
.header-actions { display: flex; flex: 0 0 auto; align-items: center; gap: 8px; }
.icon-command { display: inline-grid; width: 34px; height: 34px; place-items: center; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface-2); color: var(--text-secondary); cursor: pointer; }
.icon-command:hover:not(:disabled) { border-color: var(--border-strong); color: var(--text-primary); }
.icon-command:disabled, .btn:disabled { cursor: not-allowed; opacity: 0.55; }
.icon-command.spinning svg { animation: spin 0.8s linear infinite; }
.btn { display: inline-flex; min-height: 34px; align-items: center; justify-content: center; gap: 7px; }
.source-notice { display: flex; align-items: flex-start; gap: 8px; margin-bottom: 14px; padding: 9px 11px; border-left: 2px solid var(--warning); background: color-mix(in srgb, var(--warning) 8%, transparent); color: var(--text-secondary); font-size: 10px; line-height: 1.45; }
.source-notice svg { flex: 0 0 auto; margin-top: 1px; color: var(--warning); }
.metrics-strip { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 10px; margin-bottom: 14px; }
.metric-card { display: grid; min-width: 0; min-height: 104px; align-content: start; gap: 7px; padding: 13px 14px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-1); }
.metric-head { display: flex; min-width: 0; align-items: center; gap: 7px; color: var(--text-tertiary); font-size: 9px; font-weight: 800; text-transform: uppercase; }
.metric-head svg { flex: 0 0 auto; color: var(--brand-light); }
.metric-head span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.metric-card strong { overflow: hidden; color: var(--text-primary); font-size: 22px; line-height: 1.1; text-overflow: ellipsis; white-space: nowrap; }
.metric-card small { color: var(--text-tertiary); font-size: 9px; line-height: 1.4; }
.analytics-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; }
.panel { display: grid; min-width: 0; align-content: start; gap: 13px; padding: 15px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-1); }
.panel-head { display: flex; min-width: 0; align-items: flex-start; justify-content: space-between; gap: 14px; padding-bottom: 11px; border-bottom: 1px solid var(--border); }
.panel-head h2 { margin: 3px 0 0; color: var(--text-primary); font-size: 14px; line-height: 1.2; }
.panel-total { display: inline-flex; min-width: 28px; height: 24px; align-items: center; justify-content: center; padding: 0 7px; border: 1px solid var(--border); border-radius: 999px; color: var(--text-secondary); font-size: 10px; font-weight: 800; }
.ranking-list { display: grid; gap: 2px; }
.ranking-item { display: grid; min-width: 0; grid-template-columns: 26px minmax(0, 1fr); align-items: center; gap: 10px; padding: 9px 2px; border-bottom: 1px solid color-mix(in srgb, var(--border) 70%, transparent); }
.ranking-item:last-child { border-bottom: 0; }
.rank { display: grid; width: 24px; height: 24px; place-items: center; border: 1px solid var(--border); border-radius: 50%; color: var(--brand-light); font-size: 9px; font-weight: 850; }
.rank-main { display: grid; min-width: 0; gap: 6px; }
.rank-copy { display: flex; min-width: 0; align-items: baseline; justify-content: space-between; gap: 10px; }
.rank-copy strong { overflow: hidden; color: var(--text-primary); font-size: 11px; font-weight: 700; text-overflow: ellipsis; white-space: nowrap; }
.rank-copy span { flex: 0 0 auto; color: var(--text-secondary); font-family: var(--font-mono); font-size: 9px; }
.rank-track { height: 3px; overflow: hidden; border-radius: 999px; background: var(--surface-3); }
.rank-track span { display: block; height: 100%; border-radius: inherit; background: var(--brand); }
.rank-track.choice span { background: var(--info); }
.inline-empty { display: grid; min-height: 154px; place-items: center; align-content: center; gap: 9px; padding: 20px; color: var(--text-tertiary); text-align: center; }
.inline-empty span { max-width: 320px; font-size: 10px; line-height: 1.5; }
.session-panel { grid-column: 1 / -1; grid-template-columns: minmax(0, 1fr) minmax(260px, 0.75fr); }
.session-panel .panel-head { grid-column: 1 / -1; }
.session-table { display: grid; align-content: start; }
.session-row { display: flex; min-height: 42px; align-items: center; justify-content: space-between; gap: 14px; border-bottom: 1px solid var(--border); }
.session-row span { display: flex; min-width: 0; align-items: center; gap: 7px; color: var(--text-secondary); font-size: 10px; }
.session-row span svg { flex: 0 0 auto; color: var(--text-tertiary); }
.session-row strong { flex: 0 0 auto; color: var(--text-primary); font-size: 12px; }
.relationship-scale { display: grid; align-content: center; gap: 8px; padding: 8px 2px; }
.scale-copy, .scale-values { display: flex; align-items: center; justify-content: space-between; gap: 8px; color: var(--text-tertiary); font-size: 8px; }
.scale-copy strong { color: var(--text-secondary); font-size: 9px; }
.scale-track { position: relative; height: 5px; border-radius: 999px; background: linear-gradient(90deg, var(--danger), var(--surface-3) 50%, var(--success)); }
.scale-marker { position: absolute; top: 50%; width: 10px; height: 10px; border: 2px solid var(--surface-1); border-radius: 50%; background: var(--text-primary); transform: translate(-50%, -50%); }
.scale-values { font-family: var(--font-mono); }
.analytics-toast { position: fixed; right: 20px; bottom: 20px; z-index: 80; display: flex; max-width: min(420px, calc(100vw - 32px)); align-items: center; gap: 8px; padding: 10px 12px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface-2); color: var(--text-primary); box-shadow: var(--shadow-lg); font-size: 11px; }
.analytics-toast.success svg { color: var(--success); }
.analytics-toast.error svg { color: var(--danger); }
.toast-enter-active, .toast-leave-active { transition: opacity 0.16s ease, transform 0.16s ease; }
.toast-enter-from, .toast-leave-to { opacity: 0; transform: translateY(5px); }
@keyframes spin { to { transform: rotate(360deg); } }
@media (max-width: 900px) {
  .analytics-page { padding: 24px 22px 38px; }
  .metrics-strip { grid-template-columns: repeat(2, minmax(0, 1fr)); }
}
@media (max-width: 680px) {
  .analytics-page { padding: 18px 14px calc(86px + env(safe-area-inset-bottom)); }
  .page-header { align-items: flex-end; }
  .title-row h1 { font-size: 22px; }
  .page-header p { font-size: 11px; }
  .header-actions .btn { width: 34px; padding: 0; }
  .header-actions .btn svg { margin: 0; }
  .header-actions .btn { font-size: 0; }
  .metrics-strip, .analytics-grid { grid-template-columns: 1fr; }
  .metric-card { min-height: 90px; }
  .session-panel { grid-column: auto; grid-template-columns: 1fr; }
  .session-panel .panel-head { grid-column: auto; }
  .analytics-toast { right: 14px; bottom: calc(72px + env(safe-area-inset-bottom)); }
}
</style>
