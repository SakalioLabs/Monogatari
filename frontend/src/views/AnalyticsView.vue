<template>
  <div class="analytics-page">
    <header class="page-header">
      <div>
        <span class="eyebrow">Insights</span>
        <h1>Analytics Dashboard</h1>
        <p>Player engagement metrics, conversation patterns, and character interaction data.</p>
      </div>
      <div class="header-actions">
        <button class="btn btn-secondary btn-sm" @click="refresh">Refresh</button>
        <button class="btn btn-primary btn-sm" @click="exportData">{{ t("analytics.export", "Export JSON") }}</button>
      </div>
    </header>

    <section class="metrics-strip">
      <div class="metric-card" v-for="m in metrics" :key="m.label">
        <span class="metric-value">{{ m.value }}</span>
        <span class="metric-label">{{ m.label }}</span>
      </div>
    </section>

    <section class="analytics-grid">
      <div class="panel">
        <div class="panel-head">
          <span class="eyebrow">Characters</span>
          <strong>Top Interactions</strong>
        </div>
        <div v-if="summary.top_characters.length" class="ranking-list">
          <div v-for="(item, i) in summary.top_characters" :key="item[0]" class="ranking-item">
            <span class="rank">{{ i + 1 }}</span>
            <span class="rank-name">{{ item[0] }}</span>
            <span class="rank-count">{{ item[1] }}</span>
          </div>
        </div>
        <p v-else class="muted">No character interaction data yet. Start chatting to see metrics.</p>
      </div>

      <div class="panel">
        <div class="panel-head">
          <span class="eyebrow">Choices</span>
          <strong>Most Popular</strong>
        </div>
        <div v-if="summary.top_choices.length" class="ranking-list">
          <div v-for="(item, i) in summary.top_choices" :key="item[0]" class="ranking-item">
            <span class="rank">{{ i + 1 }}</span>
            <span class="rank-name">{{ item[0] }}</span>
            <span class="rank-count">{{ item[1] }}</span>
          </div>
        </div>
        <p v-else class="muted">No choice data recorded yet.</p>
      </div>

      <div class="panel full-width">
        <div class="panel-head">
          <span class="eyebrow">Engagement</span>
          <strong>Session Overview</strong>
        </div>
        <div class="engagement-grid">
          <div class="engagement-item">
            <span class="eng-label">Total Sessions</span>
            <span class="eng-value">{{ summary.session_count }}</span>
          </div>
          <div class="engagement-item">
            <span class="eng-label">Avg Session Duration</span>
            <span class="eng-value">{{ formatDuration(summary.avg_session_duration_ms) }}</span>
          </div>
          <div class="engagement-item">
            <span class="eng-label">Conversations</span>
            <span class="eng-value">{{ summary.conversation_count }}</span>
          </div>
          <div class="engagement-item">
            <span class="eng-label">Avg Relationship Score</span>
            <span class="eng-value">{{ summary.avg_relationship_score.toFixed(2) }}</span>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { invokeCommand } from '../lib/tauri'
import { useI18n } from '../lib/i18n'

const { t } = useI18n()

interface AnalyticsSummary {
  total_events: number
  session_count: number
  avg_session_duration_ms: number
  top_choices: [string, number][]
  top_characters: [string, number][]
  conversation_count: number
  avg_relationship_score: number
}

const summary = ref<AnalyticsSummary>({
  total_events: 0,
  session_count: 0,
  avg_session_duration_ms: 0,
  top_choices: [],
  top_characters: [],
  conversation_count: 0,
  avg_relationship_score: 0,
})

const previewSummary: AnalyticsSummary = {
  total_events: 24,
  session_count: 3,
  avg_session_duration_ms: 180000,
  top_choices: [['Tell me about the cherry blossoms', 8], ['What is your favorite season?', 6], ['Let us explore the park', 5]],
  top_characters: [['Sakura', 15], ['Luna', 8], ['Kenji', 4]],
  conversation_count: 12,
  avg_relationship_score: 0.45,
}

const metrics = computed(() => [
  { label: 'Total Events', value: summary.value.total_events },
  { label: 'Sessions', value: summary.value.session_count },
  { label: 'Conversations', value: summary.value.conversation_count },
  { label: 'Avg Relationship', value: summary.value.avg_relationship_score.toFixed(2) },
])

function formatDuration(ms: number): string {
  if (ms === 0) return '0m'
  const minutes = Math.floor(ms / 60000)
  const seconds = Math.floor((ms % 60000) / 1000)
  return minutes > 0 ? `${minutes}m ${seconds}s` : `${seconds}s`
}

async function refresh() {
  try {
    summary.value = await invokeCommand<AnalyticsSummary>('get_analytics_summary', {}, previewSummary)
  } catch {
    summary.value = previewSummary
  }
}

async function exportData() {
  try {
    const data = await invokeCommand<string>('export_analytics', { format: 'json' }, '{}')
    const blob = new Blob([data], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = 'analytics-export.json'
    a.click()
    URL.revokeObjectURL(url)
  } catch (e) { console.error(e) }
}

onMounted(refresh)
</script>

<style scoped>
.analytics-page { max-width: 1180px; margin: 0 auto; padding: 34px 40px; }
.page-header { display: flex; justify-content: space-between; gap: 18px; align-items: flex-start; margin-bottom: 22px; }
.page-header h1 { color: var(--text-primary); font-size: 28px; line-height: 1.15; }
.page-header p { color: var(--text-tertiary); font-size: 13px; margin-top: 4px; }
.eyebrow { display: block; color: var(--text-tertiary); font-size: 11px; font-weight: 800; letter-spacing: 0; text-transform: uppercase; }
.header-actions { display: flex; gap: 8px; flex-shrink: 0; }
.metrics-strip { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 12px; margin-bottom: 18px; }
.metric-card { display: grid; gap: 4px; min-height: 78px; padding: 15px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-1); }
.metric-value { color: var(--brand-light); font-size: 22px; font-weight: 800; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.metric-label { color: var(--text-tertiary); font-size: 11px; font-weight: 800; text-transform: uppercase; }
.analytics-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 14px; }
.full-width { grid-column: 1 / -1; }
.panel { border: 1px solid var(--border); border-radius: var(--radius); background: var(--surface-1); padding: 16px; display: grid; gap: 14px; }
.panel-head { display: flex; justify-content: space-between; align-items: baseline; }
.panel-head strong { color: var(--text-primary); font-size: 15px; }
.ranking-list { display: grid; gap: 8px; }
.ranking-item { display: flex; align-items: center; gap: 12px; padding: 10px 12px; border-radius: var(--radius-sm); background: var(--surface-2); }
.rank { width: 24px; height: 24px; border-radius: 50%; background: var(--surface-3); display: flex; align-items: center; justify-content: center; font-size: 11px; font-weight: 700; color: var(--brand-light); flex-shrink: 0; }
.rank-name { flex: 1; font-size: 13px; color: var(--text-primary); min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.rank-count { font-size: 13px; font-weight: 700; color: var(--text-secondary); }
.engagement-grid { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 10px; }
.engagement-item { display: grid; gap: 4px; padding: 14px; border-radius: var(--radius-sm); background: var(--surface-2); }
.eng-label { color: var(--text-tertiary); font-size: 11px; font-weight: 700; text-transform: uppercase; }
.eng-value { color: var(--text-primary); font-size: 20px; font-weight: 800; }
.muted { color: var(--text-tertiary); font-size: 13px; }
@media (max-width: 720px) {
  .analytics-page { padding: 22px 16px; }
  .metrics-strip, .analytics-grid, .engagement-grid { grid-template-columns: 1fr; }
}
</style>
