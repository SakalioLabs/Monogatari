<template>
  <div class="quality-workbench">
    <header class="quality-toolbar">
      <div class="toolbar-title">
        <span class="eyebrow"><ShieldCheck :size="13" aria-hidden="true" />{{ t('quality.eyebrow', 'Release Gate') }}</span>
        <h1>{{ t('quality.title', 'Quality Suites') }}</h1>
        <span class="status-pill" :class="reportStatusClass">
          <LoaderCircle v-if="running" :size="13" class="spin" aria-hidden="true" />
          <CheckCircle2 v-else-if="report?.failed === 0" :size="13" aria-hidden="true" />
          <CircleAlert v-else :size="13" aria-hidden="true" />
          {{ running ? t('quality.running', 'Running') : reportStatus }}
        </span>
      </div>

      <div class="toolbar-metrics" :aria-label="t('quality.run-summary', 'Run summary')">
        <span><strong>{{ report?.total ?? '-' }}</strong>{{ t('quality.total', 'Total') }}</span>
        <span class="metric-pass"><strong>{{ report?.passed ?? '-' }}</strong>{{ t('quality.passed', 'Passed') }}</span>
        <span :class="{ 'metric-fail': (report?.failed ?? 0) > 0 }"><strong>{{ report?.failed ?? '-' }}</strong>{{ t('quality.failed', 'Failed') }}</span>
        <span><strong>{{ passRate }}</strong>{{ t('quality.pass-rate', 'Pass Rate') }}</span>
      </div>

      <div class="toolbar-actions">
        <button class="icon-command" :disabled="loading || running" :title="t('chat.refresh', 'Refresh')" :aria-label="t('chat.refresh', 'Refresh')" @click="refreshSuites">
          <RefreshCw :size="16" :class="{ spin: loading }" aria-hidden="true" />
        </button>
        <button class="btn btn-secondary btn-sm" :disabled="!report" @click="exportQualityReport">
          <Download :size="14" aria-hidden="true" />
          <span>{{ t('quality.export-json', 'Export JSON') }}</span>
        </button>
        <button class="btn btn-primary btn-sm" :disabled="running || !selectedSuite" @click="runSelectedSuite">
          <LoaderCircle v-if="running" :size="14" class="spin" aria-hidden="true" />
          <Play v-else :size="14" aria-hidden="true" />
          <span>{{ running ? t('quality.running', 'Running') : t('quality.run', 'Run Suite') }}</span>
        </button>
        <button class="icon-command inspector-toggle" :title="t('quality.open-diagnostics', 'Open diagnostics')" :aria-label="t('quality.open-diagnostics', 'Open diagnostics')" @click="compactInspectorOpen = true">
          <PanelRightOpen :size="16" aria-hidden="true" />
        </button>
      </div>
    </header>

    <section v-if="errorMessage" class="error-panel" role="status">
      <CircleAlert :size="16" aria-hidden="true" />
      <div><strong>{{ t('common.error', 'Error') }}</strong><p>{{ errorMessage }}</p></div>
      <button class="icon-command" :title="t('common.close', 'Close')" :aria-label="t('common.close', 'Close')" @click="errorMessage = null"><X :size="15" aria-hidden="true" /></button>
    </section>

    <main class="quality-body">
      <aside class="suite-browser">
        <div class="panel-heading">
          <span><ListChecks :size="15" aria-hidden="true" />{{ t('quality.suites', 'Suites') }}</span>
          <strong>{{ suites.length }}</strong>
        </div>

        <div class="suite-list">
          <button
            v-for="suite in suites"
            :key="suite.path"
            class="suite-row"
            :class="{ active: selectedSuite?.path === suite.path }"
            @click="selectSuite(suite)"
          >
            <span class="suite-icon"><ShieldCheck :size="15" aria-hidden="true" /></span>
            <span class="suite-copy">
              <strong>{{ suiteDisplayName(suite) }}</strong>
              <small>{{ suite.scenario_count }} {{ t('quality.scenarios', 'scenarios') }} · {{ suite.version }}</small>
            </span>
            <ChevronRight :size="14" aria-hidden="true" />
          </button>
          <div v-if="!suites.length && !loading" class="compact-empty">
            <FolderSearch :size="20" aria-hidden="true" />
            <span>{{ t('quality.empty', 'No quality suites found.') }}</span>
          </div>
        </div>

        <div v-if="selectedSuite" class="suite-provenance">
          <span>{{ t('quality.source', 'Source') }}</span>
          <strong :title="selectedSuite.path">{{ selectedSuite.path }}</strong>
          <span class="suite-fingerprint" :title="selectedSuite.suite_sha256">{{ fingerprintLabel(selectedSuite.suite_sha256) }}</span>
        </div>

        <div class="filter-panel">
          <div class="panel-heading compact">
            <span><SlidersHorizontal :size="14" aria-hidden="true" />{{ t('quality.filters', 'Filters') }}</span>
            <button v-if="filtersActive" class="text-command" @click="resetScenarioFilters">{{ t('quality.reset', 'Reset') }}</button>
          </div>
          <div class="segmented-control" :aria-label="t('quality.status-filter', 'Status filter')">
            <button :class="{ active: scenarioStatus === 'all' }" @click="scenarioStatus = 'all'">{{ t('quality.filter-all', 'All') }} <strong>{{ report?.total ?? 0 }}</strong></button>
            <button :class="{ active: scenarioStatus === 'failed' }" @click="scenarioStatus = 'failed'">{{ t('quality.failed', 'Failed') }} <strong>{{ report?.failed ?? 0 }}</strong></button>
            <button :class="{ active: scenarioStatus === 'passed' }" @click="scenarioStatus = 'passed'">{{ t('quality.passed', 'Passed') }} <strong>{{ report?.passed ?? 0 }}</strong></button>
          </div>
          <label class="filter-field">
            <span>{{ t('quality.category', 'Category') }}</span>
            <select v-model="selectedCategory" class="input" :aria-label="t('quality.category', 'Category')">
              <option value="all">{{ t('quality.all-categories', 'All categories') }}</option>
              <option v-for="category in categoryOptions" :key="category" :value="category">{{ categoryLabel(category) }}</option>
            </select>
          </label>
        </div>
      </aside>

      <section class="quality-content">
        <header class="content-toolbar">
          <div class="view-tabs" role="tablist" :aria-label="t('quality.report-views', 'Report views')">
            <button role="tab" :aria-selected="viewMode === 'scenarios'" :class="{ active: viewMode === 'scenarios' }" @click="viewMode = 'scenarios'">
              <ListChecks :size="14" aria-hidden="true" />{{ t('quality.scenario-results', 'Scenarios') }}
            </button>
            <button role="tab" :aria-selected="viewMode === 'audit'" :class="{ active: viewMode === 'audit' }" @click="viewMode = 'audit'">
              <ChartNoAxesColumnIncreasing :size="14" aria-hidden="true" />{{ t('quality.audit', 'Audit') }}
            </button>
          </div>
          <label v-if="viewMode === 'scenarios'" class="scenario-search">
            <Search :size="14" aria-hidden="true" />
            <input v-model="scenarioSearch" :placeholder="t('quality.search-scenarios', 'Search scenarios')" :aria-label="t('quality.search-scenarios', 'Search scenarios')" />
          </label>
          <span class="result-count">{{ filteredScenarios.length }}/{{ report?.total ?? 0 }}</span>
        </header>

        <div v-if="viewMode === 'scenarios'" class="scenario-browser">
          <button
            v-for="scenario in filteredScenarios"
            :key="scenario.id"
            class="scenario-row"
            :class="{ active: selectedScenario?.id === scenario.id, failed: !scenario.passed }"
            @click="selectScenario(scenario)"
          >
            <span class="scenario-state" :class="scenario.passed ? 'passed' : 'failed'">
              <Check v-if="scenario.passed" :size="14" aria-hidden="true" />
              <X v-else :size="14" aria-hidden="true" />
            </span>
            <span class="scenario-copy">
              <strong>{{ scenario.id }}</strong>
              <small>{{ categoryLabel(scenario.category) }}</small>
            </span>
            <span class="scenario-metric"><strong>{{ formatScore(scenario.evaluation.overall_score) }}</strong><small>{{ t('quality.overall', 'Overall') }}</small></span>
            <span v-if="scenario.runtime_safety_trace" class="scenario-metric"><strong>{{ runtimeInterventionNotes(scenario.runtime_safety_trace).length }}</strong><small>{{ t('quality.guards', 'Guards') }}</small></span>
            <span v-else class="scenario-metric"><strong>{{ scenario.issues.length }}</strong><small>{{ t('quality.issues', 'Issues') }}</small></span>
            <span v-if="scenario.roleplay_preview" class="roleplay-coverage-chip">{{ scenario.roleplay_preview.report.ending_id || formatCoverage(scenario.roleplay_preview.report.coverage_percent) }}</span>
            <span v-else-if="scenario.workflow_coverage" class="workflow-coverage-chip">{{ formatCoverage(scenario.workflow_coverage.coverage_percent) }}</span>
            <ChevronRight :size="15" class="row-chevron" aria-hidden="true" />
          </button>

          <div v-if="!report" class="empty-report">
            <ShieldCheck :size="28" aria-hidden="true" />
            <strong>{{ t('quality.ready', 'Ready to run') }}</strong>
            <p>{{ selectedSuite ? suiteDisplayDescription(selectedSuite) : t('quality.pick-suite', 'Select a suite to inspect release quality gates.') }}</p>
          </div>
          <div v-else-if="!filteredScenarios.length" class="empty-report">
            <SearchX :size="28" aria-hidden="true" />
            <strong>{{ t('quality.no-matching-scenarios', 'No matching scenarios') }}</strong>
            <p>{{ t('quality.adjust-filters', 'Adjust the search or filters to continue.') }}</p>
          </div>
        </div>

        <div v-else-if="report" class="audit-panel">
          <section class="audit-section">
            <div class="audit-head"><span>{{ t('quality.audit-categories', 'Categories') }}</span><strong>{{ report.audit_summary.category_summary.length }}</strong></div>
            <div class="category-audit-list">
              <div v-for="category in report.audit_summary.category_summary" :key="category.category" class="category-audit-row">
                <span>{{ categoryLabel(category.category) }}</span>
                <strong>{{ category.passed }}/{{ category.total }}</strong>
                <i :style="{ width: `${category.total ? Math.round((category.passed / category.total) * 100) : 0}%` }"></i>
              </div>
            </div>
          </section>

          <section v-if="report.audit_summary.roleplay_coverage.length" class="audit-section">
            <div class="audit-head"><span>{{ t('quality.roleplay-coverage', 'Roleplay Coverage') }}</span><strong>{{ report.audit_summary.roleplay_coverage.length }}</strong></div>
            <div class="roleplay-audit-list">
              <button
                v-for="coverage in report.audit_summary.roleplay_coverage"
                :key="coverage.scenario_id"
                class="roleplay-audit-row"
                :title="`${coverage.source_path} · ${fingerprintLabel(coverage.source_sha256)}`"
                @click="openScenarioById(coverage.scenario_id)"
              >
                <span>{{ coverage.roleplay_id }}</span>
                <strong>{{ formatCoverage(coverage.coverage_percent) }}</strong>
                <small>{{ coverage.ending_id || t('quality.in-progress', 'In progress') }}</small>
              </button>
            </div>
          </section>

          <section class="audit-section">
            <div class="audit-head"><span>{{ t('quality.audit-signals', 'Safety Signals') }}</span><strong>{{ activeSafetySignals.length }}</strong></div>
            <div class="safety-signal-list">
              <span v-for="signal in activeSafetySignals" :key="signal.id" class="audit-chip warning"><span>{{ signal.label }}</span><strong>{{ signal.value }}</strong></span>
              <span v-if="!activeSafetySignals.length" class="audit-chip ok"><span>{{ t('quality.no-active-signals', 'No active signals') }}</span><Check :size="13" aria-hidden="true" /></span>
            </div>
            <div v-if="activeRuntimeGuardNotes.length" class="runtime-guard-note-list">
              <span v-for="note in activeRuntimeGuardNotes" :key="note.note" class="guard-note-chip"><span>{{ formatGuardNote(note.note) }}</span><strong>{{ note.count }}</strong></span>
            </div>
          </section>

          <section class="audit-section">
            <div class="audit-head"><span>{{ t('quality.audit-failures', 'Failures') }}</span><strong>{{ report.audit_summary.failed_scenario_ids.length }}</strong></div>
            <div class="audit-chip-list">
              <button v-for="id in report.audit_summary.failed_scenario_ids" :key="id" class="audit-chip danger" @click="openScenarioById(id)">{{ id }}</button>
              <span v-if="!report.audit_summary.failed_scenario_ids.length" class="audit-chip ok"><span>{{ t('quality.no-failures', 'No failures') }}</span><Check :size="13" aria-hidden="true" /></span>
            </div>
            <div v-if="report.audit_summary.workflow_coverage.length" class="workflow-audit-list">
              <div v-for="coverage in report.audit_summary.workflow_coverage" :key="coverage.scenario_id" class="workflow-audit-row">
                <span>{{ coverage.workflow_name }}</span>
                <strong>{{ formatCoverage(coverage.coverage_percent) }}</strong>
                <small>{{ coverage.executed_node_count }}/{{ coverage.node_count }}</small>
              </div>
            </div>
          </section>

          <section class="audit-section run-section">
            <div class="audit-head"><span>{{ t('quality.audit-run', 'Run') }}</span><strong>{{ report.run_metadata.engine_version }}</strong></div>
            <div class="run-metadata-list">
              <div><span>{{ t('quality.source', 'Source') }}</span><strong :title="report.run_metadata.suite_path">{{ report.run_metadata.suite_path }}</strong></div>
              <div><span>{{ t('quality.fingerprint', 'Fingerprint') }}</span><strong :title="report.run_metadata.suite_sha256">{{ fingerprintLabel(report.run_metadata.suite_sha256) }}</strong></div>
              <div><span>{{ t('quality.commit', 'Commit') }}</span><strong>{{ report.run_metadata.git_short_commit }}</strong></div>
              <div><span>{{ t('quality.generated-at', 'Generated') }}</span><strong>{{ formatTimestamp(report.run_metadata.generated_at) }}</strong></div>
              <div><span>{{ t('quality.pass-rate', 'Pass Rate') }}</span><strong>{{ formatRatio(report.run_metadata.pass_rate) }}</strong></div>
            </div>
          </section>
        </div>

        <div v-else class="empty-report audit-empty">
          <ChartNoAxesColumnIncreasing :size="28" aria-hidden="true" />
          <strong>{{ t('quality.audit-awaiting-run', 'Audit awaits a run') }}</strong>
          <p>{{ t('quality.run-to-audit', 'Run the selected suite to generate audit evidence.') }}</p>
        </div>
      </section>

      <aside class="diagnostics-panel" :class="{ 'compact-open': compactInspectorOpen }">
        <header class="diagnostics-header">
          <div>
            <span class="eyebrow"><ScanSearch :size="13" aria-hidden="true" />{{ t('quality.diagnostics', 'Diagnostics') }}</span>
            <strong>{{ selectedScenario?.id || t('quality.no-selection', 'No selection') }}</strong>
          </div>
          <button class="icon-command diagnostics-close" :title="t('common.close', 'Close')" :aria-label="t('common.close', 'Close')" @click="compactInspectorOpen = false"><X :size="15" aria-hidden="true" /></button>
        </header>

        <div v-if="selectedScenario" class="diagnostics-content">
          <div class="diagnostic-summary">
            <span class="result" :class="{ ok: selectedScenario.passed }">{{ selectedScenario.passed ? t('quality.pass', 'Pass') : t('quality.fail', 'Fail') }}</span>
            <span class="category">{{ categoryLabel(selectedScenario.category) }}</span>
            <span>{{ selectedScenario.issues.length }} {{ t('quality.issues', 'issues') }}</span>
          </div>

          <section class="diagnostic-section">
            <h2>{{ t('quality.scores', 'Scores') }}</h2>
            <div class="score-grid">
              <div v-for="score in scoresFor(selectedScenario)" :key="score.label" class="score-item">
                <span>{{ score.label }}</span>
                <strong>{{ score.value.toFixed(2) }}</strong>
                <div class="score-bar"><i :style="{ width: `${Math.round(score.value * 100)}%` }"></i></div>
              </div>
            </div>
          </section>

          <section class="diagnostic-section">
            <h2>{{ t('quality.safety-checks', 'Safety checks') }}</h2>
            <div class="diagnostic-check-list">
              <div v-for="check in diagnosticChecks(selectedScenario)" :key="check.id" class="diagnostic-check" :class="[check.id, { active: check.active, warning: check.tone === 'warning' }]">
                <component :is="check.active ? CircleAlert : CheckCircle2" :size="14" aria-hidden="true" />
                <span>{{ check.label }}</span>
                <strong>{{ check.active ? check.activeLabel : check.clearLabel }}</strong>
              </div>
            </div>
          </section>

          <section v-if="selectedScenario.issues.length" class="diagnostic-section issue-section">
            <h2>{{ t('quality.issues', 'Issues') }}</h2>
            <ul class="issue-list"><li v-for="issue in selectedScenario.issues" :key="issue">{{ issue }}</li></ul>
          </section>

          <section v-if="selectedScenario.runtime_safety_trace" class="diagnostic-section runtime-trace-row">
            <h2>{{ t('quality.runtime-trace', 'Runtime Trace') }}</h2>
            <div class="trace-summary"><strong>{{ runtimeTraceLabel(selectedScenario.runtime_safety_trace) }}</strong><span>{{ runtimeTraceSummary(selectedScenario.runtime_safety_trace) }}</span></div>
            <div class="runtime-guard-note-list">
              <span v-for="note in selectedScenario.runtime_safety_trace.guard_notes" :key="note" class="guard-note-chip">{{ formatGuardNote(note) }}</span>
            </div>
          </section>

          <section v-if="selectedScenario.workflow_coverage" class="diagnostic-section workflow-coverage-row">
            <h2>{{ t('quality.workflow-coverage', 'Workflow Coverage') }}</h2>
            <div class="coverage-value"><strong>{{ formatCoverage(selectedScenario.workflow_coverage.coverage_percent) }}</strong><span>{{ selectedScenario.workflow_coverage.executed_node_count }}/{{ selectedScenario.workflow_coverage.node_count }} {{ t('quality.nodes', 'nodes') }} · {{ selectedScenario.workflow_coverage.run_count }} {{ t('quality.runs', 'runs') }}</span></div>
            <div class="coverage-bar"><i :style="{ width: `${selectedScenario.workflow_coverage.coverage_percent}%` }"></i></div>
          </section>

          <section v-if="selectedScenario.roleplay_preview" class="diagnostic-section roleplay-coverage-row">
            <h2>{{ t('quality.roleplay-coverage', 'Roleplay Coverage') }}</h2>
            <div class="coverage-value">
              <strong>{{ formatCoverage(selectedScenario.roleplay_preview.report.coverage_percent) }}</strong>
              <span>{{ selectedScenario.roleplay_preview.report.ending_id || t('quality.in-progress', 'In progress') }} · {{ selectedScenario.roleplay_preview.report.executed_turn_count }} {{ t('quality.turns', 'turns') }}</span>
            </div>
            <div class="coverage-bar"><i :style="{ width: `${selectedScenario.roleplay_preview.report.coverage_percent}%` }"></i></div>
            <div class="roleplay-score-list">
              <span v-for="score in roleplayScoresFor(selectedScenario)" :key="score.id" class="roleplay-score-chip"><span>{{ score.id }}</span><strong>{{ formatScore(score.value) }}</strong></span>
            </div>
            <div v-if="selectedScenario.roleplay_preview.report.final_session.observed_evidence.length" class="event-row">
              <span v-for="evidence in selectedScenario.roleplay_preview.report.final_session.observed_evidence" :key="evidence" class="knowledge-ref-chip">{{ evidence }}</span>
            </div>
          </section>

          <section v-if="selectedScenario.workflow_output" class="diagnostic-section workflow-output-row">
            <h2>{{ t('quality.workflow-output', 'Workflow Output') }}</h2>
            <p>{{ selectedScenario.workflow_output }}</p>
          </section>

          <section class="diagnostic-section">
            <h2>{{ t('quality.evidence', 'Evidence') }}</h2>
            <div class="event-row">
              <span v-for="event in selectedScenario.triggered_events" :key="event" class="event-chip">{{ event }}</span>
              <span v-for="ref in selectedScenario.knowledge_refs_resolved" :key="ref" class="knowledge-ref-chip">{{ ref }}</span>
              <span v-for="rule in selectedScenario.event_rules_verified" :key="rule.event_id" class="rule-chip" :title="rule.rule_fingerprint || rule.event_id">{{ ruleChipLabel(rule) }}</span>
              <span v-if="selectedScenario.workflow_coverage" class="workflow-coverage-chip">{{ formatCoverage(selectedScenario.workflow_coverage.coverage_percent) }}</span>
              <span v-if="selectedScenario.roleplay_preview" class="roleplay-coverage-chip">{{ selectedScenario.roleplay_preview.report.ending_id || formatCoverage(selectedScenario.roleplay_preview.report.coverage_percent) }}</span>
              <span v-for="decision in blockedEventDecisions(selectedScenario)" :key="`blocked-${decision.event_id}`" class="blocked-event-chip">{{ decisionLabel(decision) }}</span>
              <span v-if="!scenarioHasEvidence(selectedScenario)" class="muted small">{{ t('quality.no-events', 'No events') }}</span>
            </div>
          </section>
        </div>

        <div v-else class="diagnostics-empty">
          <MousePointer2 :size="24" aria-hidden="true" />
          <strong>{{ t('quality.select-scenario', 'Select a scenario') }}</strong>
          <p>{{ t('quality.select-scenario-copy', 'Choose a result to inspect scores, safety checks, and evidence.') }}</p>
        </div>
      </aside>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import {
  ChartNoAxesColumnIncreasing,
  Check,
  CheckCircle2,
  ChevronRight,
  CircleAlert,
  Download,
  FolderSearch,
  ListChecks,
  LoaderCircle,
  MousePointer2,
  PanelRightOpen,
  Play,
  RefreshCw,
  ScanSearch,
  Search,
  SearchX,
  ShieldCheck,
  SlidersHorizontal,
  X,
} from '@lucide/vue'
import { invokeCommand } from '../lib/tauri'
import { useI18n } from '../lib/i18n'
import { loadStoryEventCatalog } from '../lib/storyEvents'
import type {
  ChatSafetyTrace,
  QualityScenarioReport,
  QualitySuiteReport,
  QualitySuiteSummary,
  QualityViewMode,
  ScenarioStatusFilter,
} from '../lib/qualitySuiteContract'
import {
  activeQualityRuntimeGuardNotes,
  activeQualitySafetySignals,
  blockedQualityEventDecisions as blockedEventDecisions,
  buildQualityReportExport,
  filterQualityScenarios,
  formatQualityCoverage as formatCoverage,
  formatQualityRatio as formatRatio,
  formatQualityScore as formatScore,
  formatQualityTimestamp as formatTimestamp,
  qualityCategoryOptions,
  qualityDecisionLabel as decisionLabel,
  qualityDiagnosticStates,
  qualityFingerprintLabel as fingerprintLabel,
  qualityRuleChipLabel as ruleChipLabel,
  qualityRuntimeGuardNoteCounts,
  qualityScenarioHasEvidence as scenarioHasEvidence,
  runtimeQualityInterventionNotes as runtimeInterventionNotes,
  safeQualityFilePart,
} from '../lib/qualitySuitePresentation'
import { createPreviewQualityReport, createPreviewQualitySuites } from '../lib/qualitySuitePreview'

const { t } = useI18n()

interface DiagnosticCheck {
  id: string
  label: string
  active: boolean
  activeLabel: string
  clearLabel: string
  tone: 'warning' | 'danger'
}

const suites = ref<QualitySuiteSummary[]>([])
const selectedSuite = ref<QualitySuiteSummary | null>(null)
const report = ref<QualitySuiteReport | null>(null)
const loading = ref(false)
const running = ref(false)
const errorMessage = ref<string | null>(null)
const viewMode = ref<QualityViewMode>('scenarios')
const scenarioSearch = ref('')
const scenarioStatus = ref<ScenarioStatusFilter>('all')
const selectedCategory = ref('all')
const selectedScenarioId = ref('')
const compactInspectorOpen = ref(false)

const passRate = computed(() => {
  if (!report.value || report.value.total === 0) return '-'
  return `${Math.round((report.value.passed / report.value.total) * 100)}%`
})

const reportStatus = computed(() => {
  if (!report.value) return t('quality.idle', 'Idle')
  return report.value.failed === 0 ? t('quality.all-clear', 'All Clear') : t('quality.needs-work', 'Needs Work')
})

const reportStatusClass = computed(() => {
  if (!report.value) return 'idle'
  return report.value.failed === 0 ? 'ok' : 'bad'
})

const categoryOptions = computed(() => qualityCategoryOptions(report.value, categoryLabel))

const filteredScenarios = computed(() => filterQualityScenarios(
  report.value?.scenarios ?? [],
  {
    search: scenarioSearch.value,
    status: scenarioStatus.value,
    category: selectedCategory.value,
  },
  categoryLabel,
))

const selectedScenario = computed(() => report.value?.scenarios.find((scenario) => scenario.id === selectedScenarioId.value) ?? null)

const filtersActive = computed(() => Boolean(
  scenarioSearch.value.trim()
  || scenarioStatus.value !== 'all'
  || selectedCategory.value !== 'all',
))

const activeSafetySignals = computed(() => activeQualitySafetySignals(
  report.value?.audit_summary.safety_signal_counts,
).map((signal) => ({ ...signal, label: safetySignalLabel(signal.id) })))

const runtimeGuardNoteCounts = computed(() => qualityRuntimeGuardNoteCounts(report.value?.scenarios ?? []))

const activeRuntimeGuardNotes = computed(() => activeQualityRuntimeGuardNotes(runtimeGuardNoteCounts.value))

function suiteDisplayName(suite: QualitySuiteSummary) {
  if (suite.path.endsWith('character_stability.json')) {
    return t('quality.suite.character-stability', 'Character Stability Baseline')
  }
  if (suite.path.endsWith('blue_frame_roleplay.json')) {
    return t('quality.suite.blue-frame-roleplay', 'Blue Frame Roleplay Routes')
  }
  return suite.name
}

function suiteDisplayDescription(suite: QualitySuiteSummary) {
  if (suite.path.endsWith('character_stability.json')) {
    return t('quality.suite.character-stability-copy', 'Regression coverage for character behavior, safety, scoring, story events, knowledge, and workflows.')
  }
  if (suite.path.endsWith('blue_frame_roleplay.json')) {
    return t('quality.suite.blue-frame-roleplay-copy', 'Deterministic replay coverage for dynamic scene nodes, score evidence, and every authored ending.')
  }
  return suite.description
}

function categoryLabel(category: string) {
  const labels: Record<string, string> = {
    cognition: t('quality.category.cognition', 'Cognition'),
    event_trigger: t('quality.category.event-trigger', 'Event triggers'),
    group_chat: t('quality.category.group-chat', 'Group chat'),
    injection: t('quality.category.injection', 'Injection'),
    knowledge: t('quality.category.knowledge', 'Knowledge'),
    scoring: t('quality.category.scoring', 'Scoring'),
    workflow: t('quality.category.workflow', 'Workflow'),
    workflow_coverage: t('quality.category.workflow-coverage', 'Workflow coverage'),
    roleplay: t('quality.category.roleplay', 'Scene roleplay'),
  }
  return labels[category] || category.replace(/_/g, ' ')
}

function safetySignalLabel(signal: string) {
  const labels: Record<string, string> = {
    injection: t('quality.signal.injection', 'Input injection'),
    reasoning: t('quality.signal.reasoning', 'Reasoning leak'),
    identity: t('quality.signal.identity', 'Identity drift'),
    style: t('quality.signal.style', 'Style drift'),
    summary: t('quality.signal.summary', 'Summary leak'),
    workflow: t('quality.signal.workflow', 'Workflow leak'),
    memory: t('quality.signal.memory', 'Memory leak'),
    'runtime-guard': t('quality.signal.runtime-guard', 'Runtime guards'),
    knowledge: t('quality.signal.knowledge', 'Knowledge missing'),
    boundary: t('quality.signal.boundary', 'Knowledge boundary'),
  }
  return labels[signal] || signal.replace(/-/g, ' ')
}

function resetScenarioFilters() {
  scenarioSearch.value = ''
  scenarioStatus.value = 'all'
  selectedCategory.value = 'all'
}

function selectScenario(scenario: QualityScenarioReport) {
  selectedScenarioId.value = scenario.id
  compactInspectorOpen.value = true
}

function openScenarioById(id: string) {
  const scenario = report.value?.scenarios.find((candidate) => candidate.id === id)
  if (!scenario) return
  resetScenarioFilters()
  viewMode.value = 'scenarios'
  selectScenario(scenario)
}

function diagnosticChecks(scenario: QualityScenarioReport): DiagnosticCheck[] {
  const labels: Record<string, Pick<DiagnosticCheck, 'label' | 'activeLabel' | 'clearLabel'>> = {
    injection: {
      label: t('quality.check.injection', 'Input injection'),
      activeLabel: t('quality.detected', 'Detected'),
      clearLabel: t('quality.not-detected', 'Not detected'),
    },
    reasoning: {
      label: t('quality.check.reasoning', 'Private reasoning leak'),
      activeLabel: t('quality.detected', 'Detected'),
      clearLabel: t('quality.clear', 'Clear'),
    },
    identity: {
      label: t('quality.check.identity', 'Identity drift'),
      activeLabel: t('quality.detected', 'Detected'),
      clearLabel: t('quality.stable', 'Stable'),
    },
    style: {
      label: t('quality.check.style', 'Style drift'),
      activeLabel: t('quality.detected', 'Detected'),
      clearLabel: t('quality.stable', 'Stable'),
    },
    'knowledge-missing': {
      label: t('quality.check.knowledge', 'Knowledge anchor'),
      activeLabel: t('quality.missing', 'Missing'),
      clearLabel: t('quality.available', 'Available'),
    },
    'knowledge-boundary': {
      label: t('quality.check.boundary', 'Knowledge boundary'),
      activeLabel: t('quality.violated', 'Violated'),
      clearLabel: t('quality.contained', 'Contained'),
    },
    'summary-leak': {
      label: t('quality.check.summary', 'Evaluation summary leak'),
      activeLabel: t('quality.detected', 'Detected'),
      clearLabel: t('quality.clear', 'Clear'),
    },
    'workflow-leak': {
      label: t('quality.check.workflow', 'Workflow output leak'),
      activeLabel: t('quality.detected', 'Detected'),
      clearLabel: t('quality.clear', 'Clear'),
    },
    'memory-leak': {
      label: t('quality.check.memory', 'Memory prompt leak'),
      activeLabel: t('quality.detected', 'Detected'),
      clearLabel: t('quality.clear', 'Clear'),
    },
  }
  return qualityDiagnosticStates(scenario).map((state) => ({ ...state, ...labels[state.id] }))
}

function scoresFor(scenario: QualityScenarioReport) {
  return [
    { label: t('quality.friendliness', 'Friendliness'), value: scenario.evaluation.friendliness },
    { label: t('quality.engagement', 'Engagement'), value: scenario.evaluation.engagement },
    { label: t('quality.creativity', 'Creativity'), value: scenario.evaluation.creativity },
    { label: t('quality.overall', 'Overall'), value: scenario.evaluation.overall_score },
  ]
}

function roleplayScoresFor(scenario: QualityScenarioReport) {
  return Object.entries(scenario.roleplay_preview?.report.final_session.scores ?? {})
    .map(([id, value]) => ({ id, value }))
}

function runtimeTraceLabel(trace: ChatSafetyTrace) {
  const interventions = runtimeInterventionNotes(trace)
  return interventions.length === 0
    ? t('quality.trace-clean', 'Clean')
    : t('quality.trace-guards', '{count} guards', { count: interventions.length })
}

function runtimeTraceSummary(trace: ChatSafetyTrace) {
  const notes = trace.guard_notes
  const refSummary = trace.pinned_knowledge_ref_ids.length
    ? t('quality.trace-refs', 'Refs: {refs}', { refs: trace.pinned_knowledge_ref_ids.join(', ') })
    : ''
  if (!notes.length) return refSummary || t('quality.no-trace-notes', 'No trace notes')
  return [...notes.map(formatGuardNote), refSummary].filter(Boolean).join(' / ')
}

function formatGuardNote(note: string) {
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

function exportQualityReport() {
  if (!report.value) return
  const exportedAt = new Date().toISOString()
  const payload = buildQualityReportExport(report.value, selectedSuite.value, exportedAt)
  const blob = new Blob([JSON.stringify(payload, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `monogatari-quality-report-${safeQualityFilePart(report.value.suite_name)}-${exportedAt.replace(/[:.]/g, '-')}.json`
  a.click()
  URL.revokeObjectURL(url)
}

function selectSuite(suite: QualitySuiteSummary) {
  if (selectedSuite.value?.path === suite.path) return
  selectedSuite.value = suite
  report.value = null
  selectedScenarioId.value = ''
  compactInspectorOpen.value = false
  resetScenarioFilters()
  errorMessage.value = null
}

async function refreshSuites() {
  loading.value = true
  errorMessage.value = null
  try {
    const previousPath = selectedSuite.value?.path
    suites.value = await invokeCommand<QualitySuiteSummary[]>(
      'list_quality_suites',
      undefined,
      createPreviewQualitySuites(),
    )
    selectedSuite.value = suites.value.find((suite) => suite.path === previousPath) || suites.value[0] || null
    if (previousPath && selectedSuite.value?.path !== previousPath) {
      report.value = null
      selectedScenarioId.value = ''
    }
  } catch (e) {
    suites.value = []
    selectedSuite.value = null
    report.value = null
    selectedScenarioId.value = ''
    errorMessage.value = formatError(e)
  } finally {
    loading.value = false
  }
}

async function runSelectedSuite() {
  if (!selectedSuite.value) return
  running.value = true
  errorMessage.value = null
  try {
    const eventCatalog = await loadStoryEventCatalog()
    const previewReportWithCatalog = createPreviewQualityReport(eventCatalog.events.map((event) => event.rule))
    report.value = await invokeCommand<QualitySuiteReport>(
      'run_quality_suite',
      { suitePath: selectedSuite.value.path },
      previewReportWithCatalog,
    )
    resetScenarioFilters()
    viewMode.value = 'scenarios'
    const firstScenario = report.value.scenarios.find((scenario) => !scenario.passed) || report.value.scenarios[0]
    selectedScenarioId.value = firstScenario?.id || ''
  } catch (e) {
    report.value = null
    selectedScenarioId.value = ''
    errorMessage.value = formatError(e)
  } finally {
    running.value = false
  }
}

function formatError(error: unknown) {
  return String(error instanceof Error ? error.message : error)
}

onMounted(async () => {
  await refreshSuites()
  if (selectedSuite.value) await runSelectedSuite()
})
</script>

<style scoped>
.quality-workbench {
  display: flex;
  height: calc(100svh - 56px);
  min-height: 0;
  flex-direction: column;
  overflow: hidden;
  background: var(--surface-0);
}

.quality-toolbar {
  display: grid;
  min-height: 58px;
  flex: 0 0 auto;
  grid-template-columns: minmax(270px, 1fr) auto minmax(270px, 1fr);
  align-items: center;
  gap: 18px;
  padding: 9px 14px;
  border-bottom: 1px solid var(--border);
  background: var(--surface-1);
}

.toolbar-title,
.toolbar-actions,
.toolbar-metrics,
.panel-heading,
.panel-heading > span,
.eyebrow,
.status-pill,
.scenario-search,
.view-tabs,
.diagnostic-summary,
.audit-head,
.audit-chip,
.guard-note-chip {
  display: flex;
  align-items: center;
}

.toolbar-title {
  min-width: 0;
  gap: 10px;
}

.toolbar-title h1 {
  margin: 0;
  color: var(--text-primary);
  font-size: 16px;
  line-height: 1.2;
  white-space: nowrap;
}

.eyebrow,
.panel-heading > span {
  gap: 6px;
  color: var(--text-tertiary);
  font-size: 9px;
  font-weight: 800;
  letter-spacing: 0;
  text-transform: uppercase;
}

.toolbar-metrics {
  min-width: 0;
  justify-content: center;
  gap: 4px;
  padding: 3px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--surface-0);
}

.toolbar-metrics > span {
  display: grid;
  min-width: 68px;
  grid-template-columns: auto auto;
  align-items: baseline;
  justify-content: center;
  gap: 5px;
  padding: 4px 8px;
  border-right: 1px solid var(--border);
  color: var(--text-tertiary);
  font-size: 9px;
  font-weight: 700;
  white-space: nowrap;
}

.toolbar-metrics > span:last-child { border-right: 0; }
.toolbar-metrics strong { color: var(--text-primary); font-family: var(--font-mono); font-size: 12px; }
.toolbar-metrics .metric-pass strong { color: var(--success); }
.toolbar-metrics .metric-fail strong { color: var(--danger); }

.toolbar-actions {
  min-width: 0;
  justify-content: flex-end;
  gap: 7px;
}

.toolbar-actions .btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  white-space: nowrap;
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
.icon-command:disabled { cursor: not-allowed; opacity: 0.45; }
.inspector-toggle { display: none; }

.status-pill {
  min-width: 0;
  gap: 5px;
  padding: 4px 8px;
  border: 1px solid var(--border);
  border-radius: 999px;
  background: var(--surface-2);
  color: var(--text-tertiary);
  font-size: 9px;
  font-weight: 800;
  white-space: nowrap;
}

.status-pill.ok { border-color: color-mix(in srgb, var(--success) 35%, var(--border)); color: var(--success); }
.status-pill.bad { border-color: color-mix(in srgb, var(--danger) 35%, var(--border)); color: var(--danger); }
.spin { animation: quality-spin 0.9s linear infinite; }

.error-panel {
  display: grid;
  min-height: 46px;
  flex: 0 0 auto;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: center;
  gap: 10px;
  padding: 7px 12px;
  border-bottom: 1px solid color-mix(in srgb, var(--danger) 38%, var(--border));
  background: color-mix(in srgb, var(--danger) 9%, var(--surface-1));
  color: var(--danger);
}

.error-panel div { min-width: 0; }
.error-panel strong { display: block; font-size: 10px; text-transform: uppercase; }
.error-panel p { margin: 2px 0 0; color: var(--text-secondary); font-size: 11px; overflow-wrap: anywhere; }

.quality-body {
  position: relative;
  display: grid;
  min-height: 0;
  flex: 1;
  grid-template-columns: 238px minmax(0, 1fr) 340px;
  overflow: hidden;
}

.suite-browser,
.quality-content,
.diagnostics-panel {
  min-width: 0;
  min-height: 0;
  background: var(--surface-1);
}

.suite-browser {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr) auto auto;
  gap: 10px;
  padding: 12px;
  border-right: 1px solid var(--border);
  overflow: hidden;
}

.panel-heading {
  min-height: 24px;
  justify-content: space-between;
  gap: 10px;
}

.panel-heading strong,
.result-count {
  color: var(--brand-light);
  font-family: var(--font-mono);
  font-size: 10px;
}

.panel-heading.compact { min-height: 20px; }
.suite-list { display: grid; min-height: 0; align-content: start; gap: 5px; overflow-y: auto; scrollbar-width: none; }
.suite-list::-webkit-scrollbar { display: none; }

.suite-row {
  display: grid;
  width: 100%;
  min-width: 0;
  grid-template-columns: 30px minmax(0, 1fr) auto;
  align-items: center;
  gap: 8px;
  padding: 8px;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  color: inherit;
  text-align: left;
  cursor: pointer;
}

.suite-row:hover { background: var(--surface-2); }
.suite-row.active { border-color: color-mix(in srgb, var(--brand) 38%, var(--border)); background: color-mix(in srgb, var(--brand) 9%, var(--surface-1)); }
.suite-row > svg { color: var(--text-tertiary); }
.suite-row.active > svg { color: var(--brand-light); }

.suite-icon {
  display: grid;
  width: 30px;
  height: 30px;
  place-items: center;
  border-radius: 6px;
  background: var(--surface-3);
  color: var(--text-secondary);
}

.suite-row.active .suite-icon { background: color-mix(in srgb, var(--brand) 15%, var(--surface-2)); color: var(--brand-light); }
.suite-copy { display: grid; min-width: 0; gap: 2px; }
.suite-copy strong, .suite-copy small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.suite-copy strong { color: var(--text-primary); font-size: 11px; }
.suite-copy small { color: var(--text-tertiary); font-size: 9px; }

.compact-empty {
  display: grid;
  min-height: 110px;
  place-items: center;
  align-content: center;
  gap: 7px;
  color: var(--text-tertiary);
  font-size: 10px;
  text-align: center;
}

.suite-provenance {
  display: grid;
  min-width: 0;
  gap: 4px;
  padding: 9px 0;
  border-top: 1px solid var(--border);
  border-bottom: 1px solid var(--border);
}

.suite-provenance > span:first-child,
.filter-field > span {
  color: var(--text-tertiary);
  font-size: 8px;
  font-weight: 800;
  text-transform: uppercase;
}

.suite-provenance strong {
  overflow: hidden;
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: 9px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.suite-fingerprint {
  width: fit-content;
  max-width: 100%;
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  font-size: 8px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.filter-panel { display: grid; gap: 8px; }
.text-command { padding: 0; border: 0; background: transparent; color: var(--brand-light); font-size: 9px; cursor: pointer; }
.segmented-control { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 3px; padding: 3px; border-radius: 6px; background: var(--surface-0); }

.segmented-control button {
  display: flex;
  min-width: 0;
  align-items: center;
  justify-content: center;
  gap: 4px;
  padding: 6px 4px;
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: var(--text-tertiary);
  font-size: 8px;
  font-weight: 700;
  cursor: pointer;
}

.segmented-control button.active { background: var(--surface-2); color: var(--text-primary); box-shadow: var(--shadow); }
.segmented-control strong { color: inherit; font-family: var(--font-mono); font-size: 8px; }
.filter-field { display: grid; gap: 5px; }
.filter-field select { width: 100%; height: 30px; padding: 0 8px; font-size: 10px; }

.quality-content {
  display: grid;
  grid-template-rows: 48px minmax(0, 1fr);
  border-right: 1px solid var(--border);
  overflow: hidden;
}

.content-toolbar {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 10px;
  padding: 7px 10px;
  border-bottom: 1px solid var(--border);
  background: var(--surface-1);
}

.view-tabs { flex: 0 0 auto; gap: 3px; }
.view-tabs button {
  display: inline-flex;
  height: 32px;
  align-items: center;
  gap: 5px;
  padding: 0 9px;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  color: var(--text-tertiary);
  font-size: 10px;
  font-weight: 750;
  cursor: pointer;
}

.view-tabs button:hover { color: var(--text-secondary); }
.view-tabs button.active { border-color: var(--border); background: var(--surface-2); color: var(--text-primary); }

.scenario-search {
  min-width: 120px;
  max-width: 360px;
  height: 32px;
  flex: 1;
  gap: 7px;
  margin-left: auto;
  padding: 0 9px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--surface-0);
  color: var(--text-tertiary);
}

.scenario-search:focus-within { border-color: var(--border-strong); color: var(--text-secondary); }
.scenario-search input { width: 100%; min-width: 0; border: 0; outline: 0; background: transparent; color: var(--text-primary); font-size: 10px; }
.scenario-search input::placeholder { color: var(--text-tertiary); }
.result-count { flex: 0 0 auto; }

.scenario-browser {
  display: grid;
  min-height: 0;
  align-content: start;
  gap: 5px;
  padding: 9px;
  overflow-y: auto;
  scrollbar-width: none;
}

.scenario-browser::-webkit-scrollbar { display: none; }

.scenario-row {
  display: grid;
  width: 100%;
  min-width: 0;
  min-height: 58px;
  grid-template-columns: 28px minmax(150px, 1fr) 60px 60px auto 16px;
  align-items: center;
  gap: 9px;
  padding: 7px 9px;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  color: inherit;
  text-align: left;
  cursor: pointer;
}

.scenario-row:hover { background: var(--surface-2); }
.scenario-row.active { border-color: color-mix(in srgb, var(--brand) 36%, var(--border)); background: color-mix(in srgb, var(--brand) 8%, var(--surface-1)); }
.scenario-row.failed { border-left-color: var(--danger); }
.scenario-state { display: grid; width: 26px; height: 26px; place-items: center; border-radius: 999px; }
.scenario-state.passed { background: color-mix(in srgb, var(--success) 13%, transparent); color: var(--success); }
.scenario-state.failed { background: color-mix(in srgb, var(--danger) 13%, transparent); color: var(--danger); }
.scenario-copy { display: grid; min-width: 0; gap: 3px; }
.scenario-copy strong, .scenario-copy small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.scenario-copy strong { color: var(--text-primary); font-family: var(--font-mono); font-size: 10px; }
.scenario-copy small { color: var(--text-tertiary); font-size: 9px; }
.scenario-metric { display: grid; justify-items: end; gap: 2px; }
.scenario-metric strong { color: var(--text-secondary); font-family: var(--font-mono); font-size: 11px; }
.scenario-metric small { color: var(--text-tertiary); font-size: 8px; text-transform: uppercase; }
.row-chevron { color: var(--text-tertiary); }
.scenario-row.active .row-chevron { color: var(--brand-light); }

.category,
.result,
.workflow-coverage-chip,
.roleplay-coverage-chip,
.event-chip,
.knowledge-ref-chip,
.rule-chip,
.blocked-event-chip {
  flex: 0 0 auto;
  padding: 3px 6px;
  border-radius: 999px;
  background: var(--surface-3);
  color: var(--text-secondary);
  font-size: 8px;
  font-weight: 800;
  white-space: nowrap;
}

.result.ok { background: color-mix(in srgb, var(--success) 13%, transparent); color: var(--success); }
.result:not(.ok) { background: color-mix(in srgb, var(--danger) 13%, transparent); color: var(--danger); }
.workflow-coverage-chip { background: color-mix(in srgb, var(--success) 11%, transparent); color: var(--success); }
.roleplay-coverage-chip { max-width: 150px; overflow: hidden; background: color-mix(in srgb, var(--info) 12%, transparent); color: var(--info); text-overflow: ellipsis; }

.empty-report,
.diagnostics-empty {
  display: grid;
  min-height: 260px;
  place-items: center;
  align-content: center;
  gap: 8px;
  padding: 24px;
  color: var(--text-tertiary);
  text-align: center;
}

.empty-report strong,
.diagnostics-empty strong { color: var(--text-primary); font-size: 12px; }
.empty-report p,
.diagnostics-empty p { max-width: 360px; margin: 0; font-size: 10px; line-height: 1.55; }
.audit-empty { overflow: hidden; }

.audit-panel {
  display: grid;
  min-height: 0;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  align-content: start;
  overflow-y: auto;
  scrollbar-width: none;
}

.audit-panel::-webkit-scrollbar { display: none; }

.audit-section {
  display: grid;
  min-width: 0;
  align-content: start;
  gap: 10px;
  padding: 16px;
  border-right: 1px solid var(--border);
  border-bottom: 1px solid var(--border);
}

.audit-section:nth-child(2n) { border-right: 0; }
.audit-head { min-width: 0; justify-content: space-between; gap: 10px; color: var(--text-tertiary); font-size: 9px; font-weight: 800; text-transform: uppercase; }
.audit-head strong { color: var(--text-primary); font-family: var(--font-mono); font-size: 11px; }
.category-audit-list { display: grid; min-width: 0; gap: 5px; }
.category-audit-row { position: relative; display: grid; min-height: 30px; grid-template-columns: minmax(0, 1fr) auto; align-items: center; gap: 8px; padding: 6px 8px; border-radius: 5px; background: var(--surface-2); overflow: hidden; }
.category-audit-row span, .category-audit-row strong { position: relative; z-index: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.category-audit-row span { color: var(--text-secondary); font-size: 9px; }
.category-audit-row strong { color: var(--text-primary); font-family: var(--font-mono); font-size: 9px; }
.category-audit-row i { position: absolute; inset: 0 auto 0 0; max-width: 100%; background: color-mix(in srgb, var(--success) 10%, transparent); }
.audit-chip-list, .safety-signal-list, .runtime-guard-note-list { display: flex; min-width: 0; flex-wrap: wrap; gap: 5px; }
.safety-signal-list { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); }
.audit-chip, .guard-note-chip { min-width: 0; justify-content: space-between; gap: 8px; padding: 5px 7px; border: 1px solid var(--border); border-radius: 5px; background: var(--surface-2); color: var(--text-secondary); font-size: 8px; overflow: hidden; }
.audit-chip span, .guard-note-chip span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.audit-chip strong, .guard-note-chip strong { color: inherit; font-family: var(--font-mono); }
.audit-chip.ok { border-color: color-mix(in srgb, var(--success) 25%, var(--border)); color: var(--success); }
.audit-chip.warning { border-color: color-mix(in srgb, var(--warning) 22%, var(--border)); color: var(--warning); }
.audit-chip.danger { border-color: color-mix(in srgb, var(--danger) 25%, var(--border)); color: var(--danger); cursor: pointer; }
.guard-note-chip { border-color: color-mix(in srgb, var(--brand) 22%, var(--border)); color: var(--brand-light); }
.workflow-audit-list { display: grid; gap: 5px; padding-top: 2px; }
.roleplay-audit-list { display: grid; gap: 5px; }
.workflow-audit-row { display: grid; min-width: 0; grid-template-columns: minmax(0, 1fr) auto auto; align-items: center; gap: 8px; padding: 7px 8px; border-radius: 5px; background: color-mix(in srgb, var(--success) 7%, var(--surface-2)); }
.roleplay-audit-row { display: grid; min-width: 0; grid-template-columns: minmax(0, 1fr) auto minmax(80px, auto); align-items: center; gap: 8px; padding: 7px 8px; border: 0; border-radius: 5px; background: color-mix(in srgb, var(--info) 7%, var(--surface-2)); color: inherit; text-align: left; cursor: pointer; }
.workflow-audit-row span, .workflow-audit-row small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.roleplay-audit-row span, .roleplay-audit-row small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.workflow-audit-row span { color: var(--text-secondary); font-size: 9px; }
.roleplay-audit-row span { color: var(--text-secondary); font-size: 9px; }
.workflow-audit-row strong { color: var(--success); font-family: var(--font-mono); font-size: 10px; }
.roleplay-audit-row strong { color: var(--info); font-family: var(--font-mono); font-size: 10px; }
.workflow-audit-row small { color: var(--text-tertiary); font-size: 8px; }
.roleplay-audit-row small { color: var(--text-tertiary); font-size: 8px; text-align: right; }
.run-metadata-list { display: grid; min-width: 0; gap: 7px; }
.run-metadata-list > div { display: grid; min-width: 0; grid-template-columns: 88px minmax(0, 1fr); gap: 10px; }
.run-metadata-list span { color: var(--text-tertiary); font-size: 8px; text-transform: uppercase; }
.run-metadata-list strong { overflow: hidden; color: var(--text-secondary); font-family: var(--font-mono); font-size: 9px; font-weight: 600; text-overflow: ellipsis; white-space: nowrap; }

.diagnostics-panel {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  overflow: hidden;
}

.diagnostics-header {
  display: flex;
  min-width: 0;
  min-height: 48px;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 8px 10px;
  border-bottom: 1px solid var(--border);
}

.diagnostics-header > div { display: grid; min-width: 0; gap: 3px; }
.diagnostics-header strong { overflow: hidden; color: var(--text-primary); font-family: var(--font-mono); font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
.diagnostics-close { display: none; }
.diagnostics-content { min-height: 0; overflow-y: auto; scrollbar-width: none; }
.diagnostics-content::-webkit-scrollbar { display: none; }
.diagnostic-summary { min-width: 0; gap: 6px; padding: 10px 12px; border-bottom: 1px solid var(--border); }
.diagnostic-summary > span:last-child { margin-left: auto; color: var(--text-tertiary); font-size: 8px; }
.diagnostic-section { display: grid; min-width: 0; gap: 9px; padding: 12px; border-bottom: 1px solid var(--border); }
.diagnostic-section h2 { margin: 0; color: var(--text-tertiary); font-size: 9px; font-weight: 800; text-transform: uppercase; }
.score-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px; }
.score-item { display: grid; min-width: 0; grid-template-columns: minmax(0, 1fr) auto; gap: 5px; }
.score-item span { overflow: hidden; color: var(--text-tertiary); font-size: 8px; text-overflow: ellipsis; white-space: nowrap; }
.score-item strong { color: var(--text-primary); font-family: var(--font-mono); font-size: 10px; }
.score-bar { height: 3px; grid-column: 1 / -1; border-radius: 999px; background: var(--surface-3); overflow: hidden; }
.score-bar i { display: block; height: 100%; max-width: 100%; border-radius: inherit; background: var(--brand); }
.diagnostic-check-list { display: grid; gap: 3px; }
.diagnostic-check { display: grid; min-width: 0; min-height: 28px; grid-template-columns: 16px minmax(0, 1fr) auto; align-items: center; gap: 6px; padding: 4px 6px; border-radius: 5px; color: var(--success); }
.diagnostic-check span { overflow: hidden; color: var(--text-secondary); font-size: 9px; text-overflow: ellipsis; white-space: nowrap; }
.diagnostic-check strong { color: var(--text-tertiary); font-size: 8px; white-space: nowrap; }
.diagnostic-check.active { background: color-mix(in srgb, var(--danger) 8%, transparent); color: var(--danger); }
.diagnostic-check.active.warning { background: color-mix(in srgb, var(--warning) 8%, transparent); color: var(--warning); }
.diagnostic-check.active strong { color: inherit; }
.issue-section { background: color-mix(in srgb, var(--danger) 6%, var(--surface-1)); }
.issue-list { margin: 0; padding: 0 0 0 18px; color: var(--danger); font-family: var(--font-mono); font-size: 9px; line-height: 1.55; }
.runtime-trace-row { background: color-mix(in srgb, var(--warning) 5%, var(--surface-1)); }
.trace-summary { display: grid; gap: 4px; }
.trace-summary strong { color: var(--warning); font-size: 10px; }
.trace-summary span { color: var(--text-secondary); font-size: 9px; line-height: 1.45; overflow-wrap: anywhere; }
.workflow-coverage-row { background: color-mix(in srgb, var(--success) 5%, var(--surface-1)); }
.roleplay-coverage-row { background: color-mix(in srgb, var(--info) 5%, var(--surface-1)); }
.roleplay-coverage-row .coverage-value strong { color: var(--info); }
.roleplay-coverage-row .coverage-bar i { background: var(--info); }
.roleplay-score-list { display: flex; min-width: 0; flex-wrap: wrap; gap: 5px; }
.roleplay-score-chip { display: inline-flex; min-width: 0; align-items: center; gap: 6px; padding: 4px 6px; border: 1px solid color-mix(in srgb, var(--info) 20%, var(--border)); border-radius: 5px; background: var(--surface-2); color: var(--text-secondary); font-size: 8px; }
.roleplay-score-chip span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.roleplay-score-chip strong { color: var(--info); font-family: var(--font-mono); }
.coverage-value { display: flex; align-items: baseline; gap: 8px; }
.coverage-value strong { color: var(--success); font-family: var(--font-mono); font-size: 16px; }
.coverage-value span { color: var(--text-tertiary); font-size: 9px; }
.coverage-bar { height: 4px; border-radius: 999px; background: var(--surface-3); overflow: hidden; }
.coverage-bar i { display: block; height: 100%; max-width: 100%; background: var(--success); }
.workflow-output-row { background: color-mix(in srgb, var(--info) 6%, var(--surface-1)); }
.workflow-output-row p { margin: 0; color: var(--text-secondary); font-size: 9px; line-height: 1.5; overflow-wrap: anywhere; }
.event-row { display: flex; min-width: 0; flex-wrap: wrap; gap: 5px; }
.event-chip { background: color-mix(in srgb, var(--info) 12%, transparent); color: var(--info); }
.knowledge-ref-chip { background: color-mix(in srgb, var(--success) 11%, transparent); color: var(--success); }
.rule-chip { background: color-mix(in srgb, var(--warning) 12%, transparent); color: var(--warning); }
.blocked-event-chip { max-width: 100%; overflow: hidden; color: var(--text-secondary); text-overflow: ellipsis; }
.muted { color: var(--text-tertiary); }
.small { font-size: 8px; }

@keyframes quality-spin { to { transform: rotate(360deg); } }

@media (max-width: 1450px) {
  .quality-body { grid-template-columns: 226px minmax(0, 1fr); }
  .diagnostics-panel { position: absolute; z-index: 40; inset: 0 0 0 auto; display: none; width: min(360px, 100%); border-left: 1px solid var(--border-strong); box-shadow: var(--shadow-lg); }
  .diagnostics-panel.compact-open { display: grid; }
  .inspector-toggle, .diagnostics-close { display: inline-grid; }
}

@media (max-width: 1180px) {
  .quality-toolbar { grid-template-columns: minmax(260px, 1fr) auto; }
  .toolbar-metrics { display: none; }
}

@media (max-width: 760px) {
  .quality-workbench { height: calc(100svh - 56px - 60px - env(safe-area-inset-bottom, 0px)); }
  .quality-toolbar { min-height: 96px; grid-template-columns: 1fr; align-content: start; gap: 7px; padding: 8px 10px; }
  .toolbar-title { width: 100%; }
  .toolbar-title .eyebrow { display: none; }
  .toolbar-title h1 { font-size: 15px; }
  .toolbar-title .status-pill { margin-left: auto; }
  .toolbar-actions { width: 100%; justify-content: flex-start; overflow-x: auto; scrollbar-width: none; }
  .toolbar-actions::-webkit-scrollbar { display: none; }
  .toolbar-actions .inspector-toggle { margin-left: auto; }
  .quality-body { grid-template-columns: 1fr; grid-template-rows: 176px minmax(0, 1fr); }
  .suite-browser { grid-row: 1; grid-template-columns: auto minmax(0, 1fr); grid-template-rows: auto 58px minmax(0, 1fr); gap: 7px 10px; padding: 9px 10px; border-right: 0; border-bottom: 1px solid var(--border); }
  .suite-browser > .panel-heading { grid-column: 1 / -1; }
  .suite-list { grid-column: 1 / -1; display: flex; gap: 6px; overflow-x: auto; overflow-y: hidden; scrollbar-width: none; }
  .suite-list::-webkit-scrollbar { display: none; }
  .suite-row { min-width: 205px; }
  .suite-provenance { display: none; }
  .filter-panel { grid-column: 1 / -1; display: grid; grid-template-columns: auto minmax(200px, 1fr) minmax(120px, 0.55fr); align-items: end; gap: 7px; overflow-x: auto; scrollbar-width: none; }
  .filter-panel::-webkit-scrollbar { display: none; }
  .filter-panel .panel-heading { min-width: 58px; align-self: center; }
  .filter-panel .panel-heading > span { font-size: 8px; }
  .filter-panel .text-command { display: none; }
  .segmented-control { min-width: 200px; }
  .filter-field { min-width: 120px; }
  .quality-content { grid-row: 2; border-right: 0; }
  .diagnostics-panel { left: 0; width: 100%; border-left: 0; }
  .audit-panel { grid-template-columns: 1fr; }
  .audit-section, .audit-section:nth-child(2n) { border-right: 0; }
}

@media (max-width: 520px) {
  .toolbar-actions .btn { width: 34px; padding: 0; justify-content: center; }
  .toolbar-actions .btn span { display: none; }
  .content-toolbar { gap: 6px; }
  .view-tabs button { padding: 0 7px; }
  .scenario-search { min-width: 80px; }
  .scenario-row { grid-template-columns: 28px minmax(90px, 1fr) 48px 16px; gap: 7px; }
  .scenario-row .scenario-metric + .scenario-metric,
  .scenario-row > .workflow-coverage-chip,
  .scenario-row > .roleplay-coverage-chip { display: none; }
  .scenario-metric { grid-column: 3; }
  .row-chevron { grid-column: 4; }
  .result-count { display: none; }
}
</style>
