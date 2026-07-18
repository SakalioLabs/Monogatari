import type {
  ChatSafetyTrace,
  EventTriggerDecision,
  QualitySafetySignalCounts,
  QualityScenarioReport,
  QualitySuiteReport,
  QualitySuiteSummary,
  ScenarioStatusFilter,
} from './qualitySuiteContract'

export interface QualityScenarioFilters {
  search: string
  status: ScenarioStatusFilter
  category: string
}

export interface QualityDiagnosticState {
  id: string
  active: boolean
  tone: 'warning' | 'danger'
}

export interface QualitySafetySignalValue {
  id: string
  value: number
}

export interface QualityRuntimeGuardNoteCount {
  note: string
  count: number
}

export function filterQualityScenarios(
  scenarios: readonly QualityScenarioReport[],
  filters: QualityScenarioFilters,
  categoryLabel: (category: string) => string = (category) => category,
): QualityScenarioReport[] {
  const query = filters.search.trim().toLocaleLowerCase()
  return scenarios.filter((scenario) => {
    if (filters.status === 'failed' && scenario.passed) return false
    if (filters.status === 'passed' && !scenario.passed) return false
    if (filters.category !== 'all' && scenario.category !== filters.category) return false
    if (!query) return true
    return [scenario.id, scenario.category, categoryLabel(scenario.category), ...scenario.issues]
      .some((value) => value.toLocaleLowerCase().includes(query))
  })
}

export function qualityCategoryOptions(
  report: QualitySuiteReport | null,
  categoryLabel: (category: string) => string = (category) => category,
): string[] {
  return (report?.audit_summary.category_summary ?? [])
    .map((category) => category.category)
    .sort((left, right) => categoryLabel(left).localeCompare(categoryLabel(right)))
}

export function activeQualitySafetySignals(
  signals: QualitySafetySignalCounts | null | undefined,
): QualitySafetySignalValue[] {
  if (!signals) return []
  const values: QualitySafetySignalValue[] = [
    { id: 'injection', value: signals.prompt_injection_detected },
    { id: 'reasoning', value: signals.private_reasoning_leak_detected },
    { id: 'identity', value: signals.identity_drift_detected },
    { id: 'style', value: signals.style_drift_detected },
    { id: 'summary', value: signals.evaluation_summary_leak_detected },
    { id: 'workflow', value: signals.workflow_output_leak_detected },
    { id: 'memory', value: signals.memory_prompt_leak_detected },
    { id: 'runtime-guard', value: signals.runtime_guard_interventions },
    { id: 'knowledge', value: signals.knowledge_anchor_missing_detected },
    { id: 'boundary', value: signals.knowledge_boundary_violation_detected },
  ]
  return values.filter((signal) => signal.value > 0)
}

export function qualityRuntimeGuardNoteCounts(
  scenarios: readonly QualityScenarioReport[],
): Record<string, number> {
  const counts: Record<string, number> = {}
  for (const scenario of scenarios) {
    for (const note of scenario.runtime_safety_trace?.guard_notes ?? []) {
      counts[note] = (counts[note] ?? 0) + 1
    }
  }
  return counts
}

export function activeQualityRuntimeGuardNotes(
  counts: Readonly<Record<string, number>>,
  limit = 8,
): QualityRuntimeGuardNoteCount[] {
  const safeLimit = Number.isFinite(limit) ? Math.max(0, Math.trunc(limit)) : 8
  return Object.entries(counts)
    .filter(([note]) => note !== 'no_runtime_safety_interventions')
    .sort(([leftNote, leftCount], [rightNote, rightCount]) => (
      rightCount - leftCount || leftNote.localeCompare(rightNote)
    ))
    .slice(0, safeLimit)
    .map(([note, count]) => ({ note, count }))
}

export function qualityDiagnosticStates(
  scenario: QualityScenarioReport,
): QualityDiagnosticState[] {
  return [
    { id: 'injection', active: scenario.prompt_injection_detected, tone: 'warning' },
    { id: 'reasoning', active: scenario.private_reasoning_leak_detected, tone: 'danger' },
    { id: 'identity', active: scenario.identity_drift_detected, tone: 'danger' },
    { id: 'style', active: scenario.style_drift_detected, tone: 'danger' },
    { id: 'knowledge-missing', active: scenario.knowledge_anchor_missing_detected, tone: 'danger' },
    { id: 'knowledge-boundary', active: scenario.knowledge_boundary_violation_detected, tone: 'danger' },
    { id: 'summary-leak', active: scenario.evaluation_summary_leak_detected, tone: 'danger' },
    { id: 'workflow-leak', active: scenario.workflow_output_leak_detected, tone: 'danger' },
    { id: 'memory-leak', active: scenario.memory_prompt_leak_detected, tone: 'danger' },
  ]
}

export function qualityScenarioHasEvidence(scenario: QualityScenarioReport): boolean {
  return Boolean(
    scenario.triggered_events.length
    || scenario.knowledge_refs_resolved.length
    || scenario.event_rules_verified.length
    || scenario.workflow_coverage
    || scenario.roleplay_preview
    || blockedQualityEventDecisions(scenario).length,
  )
}

export function blockedQualityEventDecisions(
  scenario: QualityScenarioReport,
  limit = 3,
): EventTriggerDecision[] {
  if (scenario.category !== 'event_trigger') return []
  const safeLimit = Number.isFinite(limit) ? Math.max(0, Math.trunc(limit)) : 3
  return scenario.event_trigger_decisions
    .filter((decision) => !decision.triggered && decision.blocked_reasons.length > 0)
    .slice(0, safeLimit)
}

export function runtimeQualityInterventionNotes(trace: ChatSafetyTrace): string[] {
  return trace.guard_notes.filter((note) => ![
    'no_runtime_safety_interventions',
    'character_mind_contract_applied',
    'pinned_knowledge_context_applied',
  ].includes(note))
}

export function qualityFingerprintLabel(value: string): string {
  return `sha256:${value.slice(0, 12)}`
}

export function formatQualityScore(value: number): string {
  return Number.isFinite(value) ? value.toFixed(2) : '-'
}

export function formatQualityCoverage(value: number): string {
  return Number.isFinite(value) ? `${Math.round(value)}%` : '-'
}

export function formatQualityRatio(value: number): string {
  return Number.isFinite(value) ? `${Math.round(value * 100)}%` : '-'
}

export function formatQualityTimestamp(value: string): string {
  if (!value) return '-'
  const parsed = new Date(value)
  if (Number.isNaN(parsed.getTime())) return value
  return parsed.toISOString().replace(/\.\d{3}Z$/, 'Z')
}

export function qualityRuleChipLabel(rule: { event_id: string; rule_fingerprint?: string | null }): string {
  return rule.rule_fingerprint
    ? `${rule.event_id} @${rule.rule_fingerprint.slice(0, 10)}`
    : rule.event_id
}

export function qualityDecisionLabel(decision: EventTriggerDecision): string {
  return `${decision.event_id}: ${decision.blocked_reasons[0] ?? ''}`.trim()
}

export function safeQualityFilePart(value: string): string {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '') || 'suite'
}

export function buildQualityReportExport(
  report: QualitySuiteReport,
  selectedSuite: QualitySuiteSummary | null,
  exportedAt: string,
) {
  return {
    quality_report_schema: 'monogatari-quality-report/v1',
    exported_at: exportedAt,
    suite: selectedSuite,
    suite_source: {
      name: report.suite_name,
      version: report.version,
      path: report.run_metadata.suite_path,
      sha256: report.run_metadata.suite_sha256,
    },
    run_metadata: report.run_metadata,
    summary: {
      total: report.total,
      passed: report.passed,
      failed: report.failed,
      pass_rate: report.total > 0 ? report.passed / report.total : 0,
      failed_scenario_ids: report.audit_summary.failed_scenario_ids,
      category_summary: report.audit_summary.category_summary,
      safety_signal_counts: report.audit_summary.safety_signal_counts,
      runtime_guard_note_counts: qualityRuntimeGuardNoteCounts(report.scenarios),
      workflow_coverage: report.audit_summary.workflow_coverage,
      roleplay_coverage: report.audit_summary.roleplay_coverage,
    },
    report,
  }
}
