import type {
  SceneRoleplaySession,
  SceneRoleplayTurnOutcome,
} from './sceneRoleplay'

export interface QualitySuiteSummary {
  name: string
  version: string
  description: string
  scenario_count: number
  path: string
  suite_sha256: string
}

export interface ConversationEvaluation {
  friendliness: number
  engagement: number
  creativity: number
  overall_score: number
  summary: string
}

export interface ChatSafetyTrace {
  input_wrapped_as_untrusted: boolean
  mind_contract_applied: boolean
  knowledge_context_pinned: boolean
  pinned_knowledge_ref_count: number
  pinned_knowledge_ref_ids: string[]
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

export interface WorkflowCoverageRunReport {
  index: number
  choice_selections: Record<string, number>
  completed: boolean
  stopped_reason: string | null
  coverage_percent: number
  executed_node_ids: string[]
  unvisited_node_ids: string[]
}

export interface WorkflowCoverageReport {
  workflow_id: string
  workflow_name: string
  run_count: number
  node_count: number
  executed_node_count: number
  coverage_percent: number
  executed_node_ids: string[]
  unvisited_node_ids: string[]
  runs: WorkflowCoverageRunReport[]
}

export interface SceneRoleplayPreviewStep {
  turn_index: number
  outcome: SceneRoleplayTurnOutcome
}

export interface SceneRoleplayPreviewReport {
  schema: string
  roleplay_id: string
  title: string
  executed_turn_count: number
  completed: boolean
  ending_id: string | null
  coverage_percent: number
  intrusion_detected_count: number
  guarded_response_count: number
  unguarded_intrusion_count: number
  visited_node_ids: string[]
  unvisited_node_ids: string[]
  final_session: SceneRoleplaySession
  steps: SceneRoleplayPreviewStep[]
}

export interface ProjectSceneRoleplayPreview {
  source_path: string
  source_sha256: string
  report: SceneRoleplayPreviewReport
}

export interface QualityEventTriggerRule {
  event_id: string
  event_type: string
  rule_fingerprint?: string
  min_relationship: number | null
  score_metric: string | null
  min_score: number | null
  min_evaluation_count: number | null
  character_ids?: string[]
  repeatable?: boolean
}

export interface EventTriggerDecision {
  event_id: string
  event_type: string
  description: string
  triggered: boolean
  already_triggered: boolean
  actual_relationship: number
  actual_evaluation_count: number
  actual_score_metric: string | null
  actual_score: number | null
  rule_fingerprint: string | null
  rule: QualityEventTriggerRule | null
  blocked_reasons: string[]
}

export interface QualityScenarioReport {
  id: string
  category: string
  passed: boolean
  issues: string[]
  evaluation: ConversationEvaluation
  relationship_delta: number
  triggered_events: string[]
  event_trigger_decisions: EventTriggerDecision[]
  event_rules_verified: QualityEventTriggerRule[]
  prompt_injection_detected: boolean
  private_reasoning_leak_detected: boolean
  identity_drift_detected: boolean
  style_drift_detected: boolean
  evaluation_summary_leak_detected: boolean
  workflow_output_leak_detected: boolean
  workflow_output: string | null
  memory_prompt_leak_detected: boolean
  runtime_safety_trace: ChatSafetyTrace | null
  workflow_coverage: WorkflowCoverageReport | null
  roleplay_preview: ProjectSceneRoleplayPreview | null
  knowledge_anchor_missing_detected: boolean
  knowledge_boundary_violation_detected: boolean
  knowledge_refs_resolved: string[]
}

export interface QualitySuiteRunMetadata {
  generated_at: string
  engine_version: string
  git_commit: string
  git_short_commit: string
  suite_path: string
  suite_sha256: string
  scenario_count: number
  pass_rate: number
}

export interface QualityCategorySummary {
  category: string
  total: number
  passed: number
  failed: number
}

export interface QualitySafetySignalCounts {
  prompt_injection_detected: number
  private_reasoning_leak_detected: number
  identity_drift_detected: number
  style_drift_detected: number
  evaluation_summary_leak_detected: number
  workflow_output_leak_detected: number
  memory_prompt_leak_detected: number
  runtime_guard_interventions: number
  knowledge_anchor_missing_detected: number
  knowledge_boundary_violation_detected: number
}

export interface WorkflowCoverageSummary {
  scenario_id: string
  workflow_id: string
  workflow_name: string
  coverage_percent: number
  executed_node_count: number
  node_count: number
  unvisited_node_ids: string[]
}

export interface RoleplayCoverageSummary {
  scenario_id: string
  roleplay_id: string
  source_path: string
  source_sha256: string
  ending_id: string | null
  coverage_percent: number
  intrusion_detected_count: number
  guarded_response_count: number
  unguarded_intrusion_count: number
  visited_node_ids: string[]
  unvisited_node_ids: string[]
}

export interface QualitySuiteAuditSummary {
  failed_scenario_ids: string[]
  category_summary: QualityCategorySummary[]
  safety_signal_counts: QualitySafetySignalCounts
  workflow_coverage: WorkflowCoverageSummary[]
  roleplay_coverage: RoleplayCoverageSummary[]
}

export interface QualitySuiteReport {
  suite_name: string
  version: string
  total: number
  passed: number
  failed: number
  run_metadata: QualitySuiteRunMetadata
  audit_summary: QualitySuiteAuditSummary
  scenarios: QualityScenarioReport[]
}

export type ScenarioStatusFilter = 'all' | 'failed' | 'passed'
export type QualityViewMode = 'scenarios' | 'audit'
