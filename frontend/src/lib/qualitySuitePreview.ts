import type { EventTriggerRule } from './storyEvents'
import type {
  ChatSafetyTrace,
  EventTriggerDecision,
  QualityCategorySummary,
  QualityEventTriggerRule,
  QualitySafetySignalCounts,
  QualityScenarioReport,
  QualitySuiteReport,
  QualitySuiteSummary,
  WorkflowCoverageReport,
} from './qualitySuiteContract'

const PREVIEW_SUITE_PATH = 'quality_suites/character_stability.json'
const PREVIEW_SUITE_SHA256 = '50eb7994d9f2432b7b798a441610f2661c714370505ee836e80565b75377a11d'

const PREVIEW_SCENARIOS: ReadonlyArray<readonly [string, string]> = [
  ['warm-creative-conversation', 'scoring'],
  ['multilingual-warm-creative-conversation', 'scoring'],
  ['prompt-injection-score-request', 'injection'],
  ['fallback-injection-score-contained', 'injection'],
  ['tool-role-injection-contained', 'injection'],
  ['structured-role-injection-contained', 'injection'],
  ['block-body-prompt-injection-contained', 'injection'],
  ['relationship-injection-delta-contained', 'injection'],
  ['multilingual-prompt-injection-contained', 'injection'],
  ['unicode-obfuscated-injection-contained', 'injection'],
  ['group-chat-runtime-trace-contained', 'group_chat'],
  ['private-reasoning-safe-response', 'cognition'],
  ['identity-stability-safe-response', 'cognition'],
  ['style-drift-sanitized-response', 'cognition'],
  ['mind-contract-runtime-trace', 'cognition'],
  ['knowledge-anchor-safe-response', 'knowledge'],
  ['knowledge-boundary-safe-response', 'knowledge'],
  ['memory-poisoning-contained', 'knowledge'],
  ['memory-prompt-replay-sanitized', 'knowledge'],
  ['string-score-parser', 'scoring'],
  ['overrange-score-clamped', 'scoring'],
  ['evaluation-summary-sanitized', 'scoring'],
  ['workflow-output-sanitized', 'workflow'],
  ['workflow-tool-output-sanitized', 'workflow'],
  ['workflow-guard-only-output-fallback', 'workflow'],
  ['relationship-boundary-first-friend', 'event_trigger'],
  ['already-triggered-event-not-replayed', 'event_trigger'],
  ['score-gate-workflow-coverage', 'workflow_coverage'],
  ['event-rule-snapshot', 'event_trigger'],
]

export function createPreviewQualitySuites(): QualitySuiteSummary[] {
  return [{
    name: 'Character Stability Baseline',
    version: '0.1.0',
    description: 'Offline regression scenarios for character behavior, prompt safety, scoring, knowledge, story events, and Workflow coverage.',
    scenario_count: PREVIEW_SCENARIOS.length,
    path: PREVIEW_SUITE_PATH,
    suite_sha256: PREVIEW_SUITE_SHA256,
  }]
}

export function createPreviewQualityReport(
  eventRules: readonly EventTriggerRule[] = [],
): QualitySuiteReport {
  const scenarios = PREVIEW_SCENARIOS.map(([id, category]) => previewScenario(id, category))
  const byId = new Map(scenarios.map((scenario) => [scenario.id, scenario]))
  const knowledgeRefs = ['sakura_nature', 'sakura_art_knowledge']

  patchScenario(byId, 'warm-creative-conversation', {
    relationship_delta: 0.05,
    evaluation: previewEvaluation(0.82, 0.88, 0.86, 'Warm creative exchange stayed in character.'),
  })
  patchScenario(byId, 'multilingual-warm-creative-conversation', {
    relationship_delta: 0.05,
    evaluation: previewEvaluation(0.8, 0.86, 0.82, 'Multilingual warmth remained stable.'),
  })

  const injectionIds = PREVIEW_SCENARIOS
    .filter(([, category]) => category === 'injection')
    .map(([id]) => id)
  for (const id of injectionIds) {
    patchScenario(byId, id, {
      prompt_injection_detected: true,
      runtime_safety_trace: previewTrace({
        input_prompt_injection_detected: true,
        response_guard_applied: true,
        guard_notes: ['input_prompt_injection_detected', 'response_guard_applied'],
      }),
    })
  }
  patchScenario(byId, 'prompt-injection-score-request', {
    knowledge_refs_resolved: [...knowledgeRefs],
  })
  patchScenario(byId, 'relationship-injection-delta-contained', {
    runtime_safety_trace: previewTrace({
      input_prompt_injection_detected: true,
      relationship_delta_blocked: true,
      guard_notes: ['input_prompt_injection_detected', 'relationship_delta_blocked'],
    }),
  })
  patchScenario(byId, 'group-chat-runtime-trace-contained', {
    runtime_safety_trace: previewTrace({
      input_prompt_injection_detected: true,
      response_guard_applied: true,
      stream_guard_applied: true,
      guard_notes: ['input_prompt_injection_detected', 'response_guard_applied', 'stream_guard_applied'],
    }),
  })
  patchScenario(byId, 'private-reasoning-safe-response', {
    runtime_safety_trace: previewTrace({
      input_private_reasoning_request_detected: true,
      private_reasoning_blocked: true,
      guard_notes: ['input_private_reasoning_request_detected', 'private_reasoning_blocked'],
    }),
  })
  patchScenario(byId, 'identity-stability-safe-response', {
    runtime_safety_trace: previewTrace({
      identity_drift_blocked: true,
      response_guard_applied: true,
      guard_notes: ['identity_drift_blocked', 'response_guard_applied'],
    }),
  })
  patchScenario(byId, 'style-drift-sanitized-response', {
    runtime_safety_trace: previewTrace({
      style_drift_blocked: true,
      response_guard_applied: true,
      guard_notes: ['style_drift_blocked', 'response_guard_applied'],
    }),
  })
  patchScenario(byId, 'mind-contract-runtime-trace', {
    knowledge_refs_resolved: [...knowledgeRefs],
    runtime_safety_trace: previewTrace({
      mind_contract_applied: true,
      knowledge_context_pinned: true,
      pinned_knowledge_ref_count: knowledgeRefs.length,
      pinned_knowledge_ref_ids: [...knowledgeRefs],
      guard_notes: [
        'no_runtime_safety_interventions',
        'character_mind_contract_applied',
        'pinned_knowledge_context_applied',
      ],
    }),
  })
  for (const id of [
    'knowledge-anchor-safe-response',
    'knowledge-boundary-safe-response',
    'memory-poisoning-contained',
  ]) {
    patchScenario(byId, id, { knowledge_refs_resolved: [...knowledgeRefs] })
  }
  patchScenario(byId, 'memory-prompt-replay-sanitized', {
    runtime_safety_trace: previewTrace({
      memory_guard_applied: true,
      guard_notes: ['memory_guard_applied'],
    }),
  })

  const highEngagementRule = eventRules.find((rule) => rule.event_id === 'high_engagement') ?? null
  patchScenario(byId, 'string-score-parser', {
    triggered_events: ['high_engagement'],
    event_trigger_decisions: [previewDecision('high_engagement', true, highEngagementRule)],
  })
  for (const id of ['workflow-output-sanitized', 'workflow-tool-output-sanitized']) {
    patchScenario(byId, id, {
      workflow_output: 'Workflow output withheld because it referenced unsafe prompt-control text.',
    })
  }
  patchScenario(byId, 'workflow-guard-only-output-fallback', {
    workflow_output: 'Workflow generation failed before safe story text was produced.',
  })

  const firstFriendRule = eventRules.find((rule) => rule.event_id === 'first_friend') ?? null
  patchScenario(byId, 'relationship-boundary-first-friend', {
    triggered_events: ['first_friend'],
    event_trigger_decisions: [previewDecision('first_friend', true, firstFriendRule)],
    event_rules_verified: firstFriendRule ? [cloneEventRule(firstFriendRule)] : [],
  })
  patchScenario(byId, 'already-triggered-event-not-replayed', {
    event_trigger_decisions: [{
      ...previewDecision('first_friend', false, firstFriendRule),
      already_triggered: true,
      blocked_reasons: ['event_already_triggered'],
    }],
  })
  patchScenario(byId, 'score-gate-workflow-coverage', {
    workflow_coverage: previewWorkflowCoverage(),
  })
  patchScenario(byId, 'event-rule-snapshot', {
    event_rules_verified: eventRules.map(cloneEventRule),
  })

  return previewReport(scenarios)
}

function previewScenario(id: string, category: string): QualityScenarioReport {
  return {
    id,
    category,
    passed: true,
    issues: [],
    evaluation: previewEvaluation(0.72, 0.8, 0.68, 'Preview evidence passed.'),
    relationship_delta: 0,
    triggered_events: [],
    event_trigger_decisions: [],
    event_rules_verified: [],
    prompt_injection_detected: false,
    private_reasoning_leak_detected: false,
    identity_drift_detected: false,
    style_drift_detected: false,
    evaluation_summary_leak_detected: false,
    workflow_output_leak_detected: false,
    workflow_output: null,
    memory_prompt_leak_detected: false,
    runtime_safety_trace: null,
    workflow_coverage: null,
    knowledge_anchor_missing_detected: false,
    knowledge_boundary_violation_detected: false,
    knowledge_refs_resolved: [],
  }
}

function previewEvaluation(
  friendliness: number,
  engagement: number,
  creativity: number,
  summary: string,
) {
  return {
    friendliness,
    engagement,
    creativity,
    overall_score: (friendliness + engagement + creativity) / 3,
    summary,
  }
}

function previewTrace(overrides: Partial<ChatSafetyTrace> = {}): ChatSafetyTrace {
  const trace = {
    input_wrapped_as_untrusted: true,
    mind_contract_applied: true,
    knowledge_context_pinned: false,
    pinned_knowledge_ref_count: 0,
    pinned_knowledge_ref_ids: [],
    input_prompt_injection_detected: false,
    input_private_reasoning_request_detected: false,
    response_guard_applied: false,
    private_reasoning_blocked: false,
    identity_drift_blocked: false,
    style_drift_blocked: false,
    memory_guard_applied: false,
    relationship_delta_blocked: false,
    stream_guard_applied: false,
    guard_notes: ['no_runtime_safety_interventions'],
    ...overrides,
  }
  return {
    ...trace,
    pinned_knowledge_ref_ids: [...trace.pinned_knowledge_ref_ids],
    guard_notes: [...trace.guard_notes],
  }
}

function previewDecision(
  eventId: string,
  triggered: boolean,
  rule: EventTriggerRule | null,
): EventTriggerDecision {
  return {
    event_id: eventId,
    event_type: rule?.event_type ?? eventId,
    description: eventId.replace(/_/g, ' '),
    triggered,
    already_triggered: false,
    actual_relationship: 0.25,
    actual_evaluation_count: 3,
    actual_score_metric: rule?.score_metric ?? null,
    actual_score: rule?.min_score ?? null,
    rule_fingerprint: rule?.rule_fingerprint ?? null,
    rule: rule ? cloneEventRule(rule) : null,
    blocked_reasons: triggered ? [] : ['threshold_not_met'],
  }
}

function cloneEventRule(rule: EventTriggerRule): QualityEventTriggerRule {
  return {
    event_id: rule.event_id,
    event_type: rule.event_type,
    ...(rule.rule_fingerprint ? { rule_fingerprint: rule.rule_fingerprint } : {}),
    min_relationship: rule.min_relationship ?? null,
    score_metric: rule.score_metric ?? null,
    min_score: rule.min_score ?? null,
    min_evaluation_count: rule.min_evaluation_count ?? null,
    ...(rule.character_ids?.length ? { character_ids: [...rule.character_ids] } : {}),
    ...(rule.repeatable ? { repeatable: true } : {}),
  }
}

function previewWorkflowCoverage(): WorkflowCoverageReport {
  return {
    workflow_id: 'score_gate_demo',
    workflow_name: 'Score Gate Demo',
    run_count: 2,
    node_count: 4,
    executed_node_count: 4,
    coverage_percent: 100,
    executed_node_ids: ['start', 'condition', 'trigger', 'end'],
    unvisited_node_ids: [],
    runs: [
      {
        index: 0,
        choice_selections: {},
        completed: true,
        stopped_reason: null,
        coverage_percent: 75,
        executed_node_ids: ['start', 'condition', 'end'],
        unvisited_node_ids: ['trigger'],
      },
      {
        index: 1,
        choice_selections: {},
        completed: true,
        stopped_reason: null,
        coverage_percent: 100,
        executed_node_ids: ['start', 'condition', 'trigger', 'end'],
        unvisited_node_ids: [],
      },
    ],
  }
}

function patchScenario(
  scenarios: Map<string, QualityScenarioReport>,
  id: string,
  patch: Partial<QualityScenarioReport>,
) {
  const scenario = scenarios.get(id)
  if (!scenario) throw new Error(`Unknown preview Quality scenario: ${id}`)
  Object.assign(scenario, patch)
}

function previewReport(scenarios: QualityScenarioReport[]): QualitySuiteReport {
  const total = scenarios.length
  const passed = scenarios.filter((scenario) => scenario.passed).length
  const failed = total - passed
  return {
    suite_name: 'Character Stability Baseline',
    version: '0.1.0',
    total,
    passed,
    failed,
    run_metadata: {
      generated_at: '2026-07-09T00:00:00Z',
      engine_version: '0.9.5',
      git_commit: 'preview',
      git_short_commit: 'preview',
      suite_path: PREVIEW_SUITE_PATH,
      suite_sha256: PREVIEW_SUITE_SHA256,
      scenario_count: total,
      pass_rate: total > 0 ? passed / total : 0,
    },
    audit_summary: {
      failed_scenario_ids: scenarios.filter((scenario) => !scenario.passed).map((scenario) => scenario.id),
      category_summary: previewCategorySummary(scenarios),
      safety_signal_counts: previewSafetySignalCounts(scenarios),
      workflow_coverage: scenarios.flatMap((scenario) => scenario.workflow_coverage
        ? [{
            scenario_id: scenario.id,
            workflow_id: scenario.workflow_coverage.workflow_id,
            workflow_name: scenario.workflow_coverage.workflow_name,
            coverage_percent: scenario.workflow_coverage.coverage_percent,
            executed_node_count: scenario.workflow_coverage.executed_node_count,
            node_count: scenario.workflow_coverage.node_count,
            unvisited_node_ids: [...scenario.workflow_coverage.unvisited_node_ids],
          }]
        : []),
    },
    scenarios,
  }
}

function previewCategorySummary(
  scenarios: readonly QualityScenarioReport[],
): QualityCategorySummary[] {
  const categories = new Map<string, QualityCategorySummary>()
  for (const scenario of scenarios) {
    const summary = categories.get(scenario.category) ?? {
      category: scenario.category,
      total: 0,
      passed: 0,
      failed: 0,
    }
    summary.total += 1
    if (scenario.passed) summary.passed += 1
    else summary.failed += 1
    categories.set(scenario.category, summary)
  }
  return [...categories.values()].sort((left, right) => left.category.localeCompare(right.category))
}

function previewSafetySignalCounts(
  scenarios: readonly QualityScenarioReport[],
): QualitySafetySignalCounts {
  const count = (predicate: (scenario: QualityScenarioReport) => boolean) => (
    scenarios.filter(predicate).length
  )
  return {
    prompt_injection_detected: count((scenario) => scenario.prompt_injection_detected),
    private_reasoning_leak_detected: count((scenario) => scenario.private_reasoning_leak_detected),
    identity_drift_detected: count((scenario) => scenario.identity_drift_detected),
    style_drift_detected: count((scenario) => scenario.style_drift_detected),
    evaluation_summary_leak_detected: count((scenario) => scenario.evaluation_summary_leak_detected),
    workflow_output_leak_detected: count((scenario) => scenario.workflow_output_leak_detected),
    memory_prompt_leak_detected: count((scenario) => scenario.memory_prompt_leak_detected),
    runtime_guard_interventions: count((scenario) => Boolean(
      scenario.runtime_safety_trace
      && !scenario.runtime_safety_trace.guard_notes.includes('no_runtime_safety_interventions'),
    )),
    knowledge_anchor_missing_detected: count((scenario) => scenario.knowledge_anchor_missing_detected),
    knowledge_boundary_violation_detected: count((scenario) => scenario.knowledge_boundary_violation_detected),
  }
}
