import { describe, expect, it } from 'vitest'

import type { QualityScenarioReport } from '../qualitySuiteContract'
import {
  activeQualityRuntimeGuardNotes,
  activeQualitySafetySignals,
  blockedQualityEventDecisions,
  buildQualityReportExport,
  filterQualityScenarios,
  formatQualityCoverage,
  formatQualityRatio,
  formatQualityScore,
  formatQualityTimestamp,
  qualityCategoryOptions,
  qualityDecisionLabel,
  qualityDiagnosticStates,
  qualityFingerprintLabel,
  qualityRuleChipLabel,
  qualityRuntimeGuardNoteCounts,
  qualityScenarioHasEvidence,
  runtimeQualityInterventionNotes,
  safeQualityFilePart,
} from '../qualitySuitePresentation'
import { createPreviewQualityReport, createPreviewQualitySuites } from '../qualitySuitePreview'
import type { EventTriggerRule } from '../storyEvents'

function scenario(reportId: string): QualityScenarioReport {
  const result = createPreviewQualityReport().scenarios.find(({ id }) => id === reportId)
  if (!result) throw new Error(`Missing preview scenario: ${reportId}`)
  return result
}

function eventRules(): EventTriggerRule[] {
  return [
    {
      event_id: 'first_friend',
      event_type: 'relationship',
      rule_fingerprint: 'a'.repeat(64),
      min_relationship: 0.2,
      min_evaluation_count: 1,
      character_ids: ['sakura'],
      repeatable: false,
    },
    {
      event_id: 'high_engagement',
      event_type: 'score',
      rule_fingerprint: 'b'.repeat(64),
      score_metric: 'engagement',
      min_score: 0.8,
      min_evaluation_count: 2,
      character_ids: ['sakura'],
      repeatable: true,
    },
    {
      event_id: 'custom_route',
      event_type: 'custom',
      rule_fingerprint: 'c'.repeat(64),
      character_ids: [],
      repeatable: false,
    },
  ]
}

describe('Quality Suite preview contracts', () => {
  it('generates the complete stable scenario set and category audit', () => {
    const suites = createPreviewQualitySuites()
    const report = createPreviewQualityReport()

    expect(suites).toHaveLength(1)
    expect(suites[0]).toMatchObject({
      path: 'quality_suites/character_stability.json',
      scenario_count: 29,
    })
    expect(report.total).toBe(29)
    expect(report.passed).toBe(29)
    expect(new Set(report.scenarios.map(({ id }) => id))).toHaveLength(29)
    expect(Object.fromEntries(report.audit_summary.category_summary.map(({ category, total }) => [category, total])))
      .toEqual({
        cognition: 4,
        event_trigger: 3,
        group_chat: 1,
        injection: 8,
        knowledge: 4,
        scoring: 5,
        workflow: 3,
        workflow_coverage: 1,
      })
    expect(report.audit_summary.workflow_coverage).toEqual([
      expect.objectContaining({ scenario_id: 'score-gate-workflow-coverage', coverage_percent: 100 }),
    ])
  })

  it('fills every non-nullable report field expected from Rust', () => {
    const booleanFields: Array<keyof QualityScenarioReport> = [
      'passed',
      'prompt_injection_detected',
      'private_reasoning_leak_detected',
      'identity_drift_detected',
      'style_drift_detected',
      'evaluation_summary_leak_detected',
      'workflow_output_leak_detected',
      'memory_prompt_leak_detected',
      'knowledge_anchor_missing_detected',
      'knowledge_boundary_violation_detected',
    ]
    const arrayFields: Array<keyof QualityScenarioReport> = [
      'issues',
      'triggered_events',
      'event_trigger_decisions',
      'event_rules_verified',
      'knowledge_refs_resolved',
    ]

    for (const item of createPreviewQualityReport().scenarios) {
      for (const field of booleanFields) expect(typeof item[field], `${item.id}.${field}`).toBe('boolean')
      for (const field of arrayFields) expect(Array.isArray(item[field]), `${item.id}.${field}`).toBe(true)
      expect(typeof item.relationship_delta).toBe('number')
      expect(item.workflow_output === null || typeof item.workflow_output === 'string').toBe(true)
      expect(item.runtime_safety_trace === null || typeof item.runtime_safety_trace === 'object').toBe(true)
      expect(item.workflow_coverage === null || typeof item.workflow_coverage === 'object').toBe(true)
    }
  })

  it('returns isolated suites, reports, nested traces, and event rules', () => {
    const rules = eventRules()
    const firstSuites = createPreviewQualitySuites()
    const secondSuites = createPreviewQualitySuites()
    const first = createPreviewQualityReport(rules)
    const second = createPreviewQualityReport(rules)

    firstSuites[0].name = 'mutated'
    first.scenarios[0].issues.push('mutated')
    first.scenarios.find(({ id }) => id === 'prompt-injection-score-request')
      ?.runtime_safety_trace?.guard_notes.push('mutated')
    first.scenarios.find(({ id }) => id === 'event-rule-snapshot')
      ?.event_rules_verified[0].character_ids?.push('mutated')

    expect(secondSuites[0].name).toBe('Character Stability Baseline')
    expect(second.scenarios[0].issues).toEqual([])
    expect(second.scenarios.find(({ id }) => id === 'prompt-injection-score-request')
      ?.runtime_safety_trace?.guard_notes).not.toContain('mutated')
    expect(second.scenarios.find(({ id }) => id === 'event-rule-snapshot')
      ?.event_rules_verified[0].character_ids).toEqual(['sakura'])
    expect(rules[0].character_ids).toEqual(['sakura'])
  })

  it('injects catalog rules only into their intended evidence scenarios', () => {
    const report = createPreviewQualityReport(eventRules())
    const snapshot = report.scenarios.find(({ id }) => id === 'event-rule-snapshot')
    const relationship = report.scenarios.find(({ id }) => id === 'relationship-boundary-first-friend')
    const score = report.scenarios.find(({ id }) => id === 'string-score-parser')

    expect(snapshot?.event_rules_verified.map(({ event_id }) => event_id))
      .toEqual(['first_friend', 'high_engagement', 'custom_route'])
    expect(relationship?.event_rules_verified.map(({ event_id }) => event_id)).toEqual(['first_friend'])
    expect(relationship?.event_trigger_decisions[0].rule?.event_id).toBe('first_friend')
    expect(score?.event_trigger_decisions[0].rule?.event_id).toBe('high_engagement')
    expect(relationship?.event_rules_verified[0]).toMatchObject({
      min_relationship: 0.2,
      score_metric: null,
      min_score: null,
      min_evaluation_count: 1,
      character_ids: ['sakura'],
    })
    expect(score?.event_trigger_decisions[0].rule).toMatchObject({
      min_relationship: null,
      score_metric: 'engagement',
      min_score: 0.8,
      min_evaluation_count: 2,
      repeatable: true,
    })
    expect(snapshot?.event_rules_verified[2]).not.toHaveProperty('character_ids')
    expect(snapshot?.event_rules_verified[2]).not.toHaveProperty('repeatable')
    expect(report.scenarios
      .filter(({ event_rules_verified }) => event_rules_verified.length > 0)
      .map(({ id }) => id))
      .toEqual(['relationship-boundary-first-friend', 'event-rule-snapshot'])
  })

  it('derives safety and runtime-guard summaries from generated evidence', () => {
    const report = createPreviewQualityReport()
    expect(report.audit_summary.safety_signal_counts).toMatchObject({
      prompt_injection_detected: 8,
      runtime_guard_interventions: 13,
      private_reasoning_leak_detected: 0,
      knowledge_boundary_violation_detected: 0,
    })
    expect(activeQualitySafetySignals(report.audit_summary.safety_signal_counts))
      .toEqual([
        { id: 'injection', value: 8 },
        { id: 'runtime-guard', value: 13 },
      ])
  })
})

describe('Quality Suite presentation domain', () => {
  it('filters by status, category, localized category text, issue text, and trimmed search', () => {
    const report = createPreviewQualityReport()
    report.scenarios[0].passed = false
    report.scenarios[0].issues = ['Dialogue drifted']

    expect(filterQualityScenarios(report.scenarios, {
      search: '  dialogue DRIFTED ', status: 'failed', category: 'all',
    })).toEqual([report.scenarios[0]])
    expect(filterQualityScenarios(report.scenarios, {
      search: 'localized scoring', status: 'all', category: 'scoring',
    }, (category) => `localized ${category}`)).toHaveLength(5)
    expect(filterQualityScenarios(report.scenarios, {
      search: '', status: 'passed', category: 'workflow',
    })).toHaveLength(3)
    expect(filterQualityScenarios(report.scenarios, {
      search: '', status: 'failed', category: 'workflow',
    })).toEqual([])
  })

  it('sorts category options by their display labels', () => {
    const report = createPreviewQualityReport()
    expect(qualityCategoryOptions(null)).toEqual([])
    expect(qualityCategoryOptions(report, (category) => ({
      cognition: 'Zulu',
      workflow_coverage: 'Alpha',
    })[category] ?? category)).toEqual([
      'workflow_coverage',
      'event_trigger',
      'group_chat',
      'injection',
      'knowledge',
      'scoring',
      'workflow',
      'cognition',
    ])
  })

  it('aggregates, orders, limits, and filters runtime guard notes', () => {
    const report = createPreviewQualityReport()
    const counts = qualityRuntimeGuardNoteCounts(report.scenarios)
    const active = activeQualityRuntimeGuardNotes(counts, 2)

    expect(counts.no_runtime_safety_interventions).toBe(1)
    expect(active).toHaveLength(2)
    expect(active[0].count).toBeGreaterThanOrEqual(active[1].count)
    expect(active.map(({ note }) => note)).not.toContain('no_runtime_safety_interventions')
    expect(activeQualityRuntimeGuardNotes(counts, -1)).toEqual([])
    expect(activeQualityRuntimeGuardNotes(counts, Number.NaN)).toHaveLength(8)
  })

  it('reports diagnostics, evidence, blocked decisions, and runtime interventions', () => {
    const injection = scenario('prompt-injection-score-request')
    expect(qualityDiagnosticStates(injection).find(({ id }) => id === 'injection')?.active).toBe(true)
    expect(qualityScenarioHasEvidence(scenario('warm-creative-conversation'))).toBe(false)
    expect(qualityScenarioHasEvidence(scenario('knowledge-anchor-safe-response'))).toBe(true)
    expect(qualityScenarioHasEvidence(scenario('score-gate-workflow-coverage'))).toBe(true)

    const blocked = blockedQualityEventDecisions(scenario('already-triggered-event-not-replayed'))
    expect(blocked.map(({ event_id }) => event_id)).toEqual(['first_friend'])
    expect(blockedQualityEventDecisions(injection)).toEqual([])
    expect(blockedQualityEventDecisions(scenario('already-triggered-event-not-replayed'), 0)).toEqual([])

    const trace = scenario('mind-contract-runtime-trace').runtime_safety_trace
    expect(trace).not.toBeNull()
    expect(runtimeQualityInterventionNotes(trace!)).toEqual([])
  })

  it('builds the versioned export with recomputed runtime-note evidence', () => {
    const report = createPreviewQualityReport()
    const suite = createPreviewQualitySuites()[0]
    const exportedAt = '2026-07-15T02:00:00.000Z'
    const payload = buildQualityReportExport(report, suite, exportedAt)

    expect(payload).toMatchObject({
      quality_report_schema: 'monogatari-quality-report/v1',
      exported_at: exportedAt,
      suite,
      suite_source: {
        name: report.suite_name,
        path: report.run_metadata.suite_path,
        sha256: report.run_metadata.suite_sha256,
      },
      summary: {
        total: 29,
        passed: 29,
        failed: 0,
        pass_rate: 1,
      },
      report,
    })
    expect(payload.summary.runtime_guard_note_counts)
      .toEqual(qualityRuntimeGuardNoteCounts(report.scenarios))
  })

  it('formats fingerprints, scores, ratios, timestamps, rules, decisions, and file names', () => {
    const decision = scenario('already-triggered-event-not-replayed').event_trigger_decisions[0]
    const rule = eventRules()[0]
    expect(qualityFingerprintLabel('1234567890abcdef')).toBe('sha256:1234567890ab')
    expect(formatQualityScore(0.126)).toBe('0.13')
    expect(formatQualityScore(Number.NaN)).toBe('-')
    expect(formatQualityCoverage(82.6)).toBe('83%')
    expect(formatQualityCoverage(Number.POSITIVE_INFINITY)).toBe('-')
    expect(formatQualityRatio(0.826)).toBe('83%')
    expect(formatQualityRatio(Number.NaN)).toBe('-')
    expect(formatQualityTimestamp('2026-07-15T02:00:00.123Z')).toBe('2026-07-15T02:00:00Z')
    expect(formatQualityTimestamp('opaque')).toBe('opaque')
    expect(qualityRuleChipLabel(rule)).toBe(`first_friend @${'a'.repeat(10)}`)
    expect(qualityDecisionLabel(decision)).toBe('first_friend: event_already_triggered')
    expect(safeQualityFilePart('  Character Stability / V1  ')).toBe('character-stability-v1')
    expect(safeQualityFilePart('***')).toBe('suite')
  })
})
