import type {
  WorkflowExecutionReport,
  WorkflowExecutionStep,
} from './workflowContract'

export type WorkflowNodeRunOutcome =
  | ''
  | 'wait'
  | 'done'
  | 'pass'
  | 'fail'
  | 'score'
  | 'triggered'
  | 'blocked'
  | 'ran'

export interface WorkflowNodeRunDetailOptions {
  reasonLabel: (reason: string) => string
  nextNodeLabel: (nodeId: string) => string
}

export function workflowExecutionStepsByNode(
  report: WorkflowExecutionReport | null | undefined,
): Map<string, WorkflowExecutionStep> {
  const steps = new Map<string, WorkflowExecutionStep>()
  for (const step of report?.steps ?? []) steps.set(step.node_id, step)
  return steps
}

export function lastWorkflowExecutionStep(
  report: WorkflowExecutionReport | null | undefined,
): WorkflowExecutionStep | null {
  const steps = report?.steps ?? []
  return steps.length > 0 ? steps[steps.length - 1] : null
}

export function workflowChoiceOptions(step: WorkflowExecutionStep): string[] {
  const choices = step.output?.choices
  if (!Array.isArray(choices)) return []
  return choices
    .filter((choice) => ['string', 'number', 'boolean'].includes(typeof choice))
    .map(String)
}

export function workflowStepCanChoose(step: WorkflowExecutionStep): boolean {
  return step.node_type === 'choice'
    && step.stopped_reason === 'awaiting_choice'
    && workflowChoiceOptions(step).length > 0
}

export function isWorkflowEvaluationStep(step: WorkflowExecutionStep): boolean {
  return step.node_type === 'evaluation'
}

export function isWorkflowTriggerEventStep(step: WorkflowExecutionStep): boolean {
  return step.node_type === 'trigger_event'
}

export function isWorkflowTriggerEventTriggered(step: WorkflowExecutionStep): boolean {
  return isWorkflowTriggerEventStep(step) && step.output?.triggered === true
}

export function workflowStringValue(value: unknown, fallback = '-'): string {
  if (value === null || value === undefined) return fallback
  if (!['string', 'number', 'boolean'].includes(typeof value)) return fallback
  const text = String(value).trim()
  return text || fallback
}

export function workflowNumericValue(value: unknown): number | null {
  if (value === null || value === undefined) return null
  if (typeof value !== 'number' && typeof value !== 'string') return null
  if (typeof value === 'string' && !value.trim()) return null
  const numeric = Number(value)
  return Number.isFinite(numeric) ? numeric : null
}

export function formatWorkflowScore(value: unknown): string {
  const numeric = workflowNumericValue(value)
  return numeric === null ? '-' : numeric.toFixed(2)
}

export function formatWorkflowThreshold(value: unknown, noneLabel: string): string {
  return value === null || value === undefined ? noneLabel : formatWorkflowScore(value)
}

export function formatWorkflowCoverage(value: unknown): string {
  const numeric = workflowNumericValue(value)
  return numeric === null ? '0%' : `${numeric.toFixed(0)}%`
}

export function workflowScorePercent(value: unknown): string {
  const numeric = workflowNumericValue(value)
  if (numeric === null) return '0%'
  return `${Math.round(Math.min(1, Math.max(0, numeric)) * 100)}%`
}

export function workflowEventDecision(step: WorkflowExecutionStep): Record<string, unknown> {
  return recordValue(step.output?.decision)
}

export function workflowEventBlockers(step: WorkflowExecutionStep): string[] {
  const reasons = workflowEventDecision(step).blocked_reasons
  if (!Array.isArray(reasons)) return []
  return reasons
    .filter((reason) => ['string', 'number', 'boolean'].includes(typeof reason))
    .map((reason) => workflowStringValue(reason, ''))
    .filter(Boolean)
}

export function workflowEventMetric(step: WorkflowExecutionStep): string {
  const decision = workflowEventDecision(step)
  const rule = recordValue(decision.rule)
  return workflowStringValue(decision.actual_score_metric ?? rule.score_metric, '-')
}

export function workflowEventActualScore(step: WorkflowExecutionStep): number | null {
  return workflowNumericValue(workflowEventDecision(step).actual_score)
}

export function workflowNodeRunOutcome(
  step: WorkflowExecutionStep | null | undefined,
): WorkflowNodeRunOutcome {
  if (!step) return ''
  if (step.stopped_reason === 'awaiting_choice') return 'wait'
  if (step.node_type === 'end') return 'done'
  if (isWorkflowEvaluationStep(step)) {
    if (step.output?.passed === true) return 'pass'
    if (step.output?.passed === false) return 'fail'
    return 'score'
  }
  if (isWorkflowTriggerEventStep(step)) {
    return isWorkflowTriggerEventTriggered(step) ? 'triggered' : 'blocked'
  }
  if (step.stopped_reason && step.stopped_reason !== 'completed') return 'blocked'
  return 'ran'
}

export function workflowNodeRunClasses(
  nodeId: string,
  step: WorkflowExecutionStep | null | undefined,
  lastStep: WorkflowExecutionStep | null | undefined,
): Record<string, boolean> {
  const outcome = workflowNodeRunOutcome(step)
  return {
    'run-executed': Boolean(step),
    'run-current': lastStep?.node_id === nodeId,
    'run-pass': outcome === 'pass' || outcome === 'triggered' || outcome === 'done',
    'run-fail': outcome === 'fail' || outcome === 'blocked',
    'run-wait': outcome === 'wait',
    'run-score': outcome === 'score',
  }
}

export function workflowNodeRunDetail(
  step: WorkflowExecutionStep | null | undefined,
  options: WorkflowNodeRunDetailOptions,
): string {
  if (!step) return ''
  if (isWorkflowEvaluationStep(step)) {
    const metric = workflowStringValue(step.output?.metric, 'overall')
    const threshold = step.output?.threshold === null || step.output?.threshold === undefined
      ? ''
      : `/${formatWorkflowScore(step.output.threshold)}`
    return `${metric} ${formatWorkflowScore(step.output?.score)}${threshold}`
  }
  if (isWorkflowTriggerEventStep(step)) {
    return workflowEventBlockers(step)[0] || workflowStringValue(step.output?.event_id, 'event')
  }
  if (step.stopped_reason && step.stopped_reason !== 'completed') {
    return options.reasonLabel(step.stopped_reason)
  }
  return step.next_node_id ? options.nextNodeLabel(step.next_node_id) : ''
}

function recordValue(value: unknown): Record<string, unknown> {
  return value && typeof value === 'object' && !Array.isArray(value)
    ? value as Record<string, unknown>
    : {}
}
