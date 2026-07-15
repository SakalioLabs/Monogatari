import { describe, expect, it } from 'vitest'

import type { WorkflowExecutionReport, WorkflowExecutionStep } from '../workflowContract'
import {
  formatWorkflowCoverage,
  formatWorkflowScore,
  formatWorkflowThreshold,
  isWorkflowEvaluationStep,
  isWorkflowTriggerEventStep,
  isWorkflowTriggerEventTriggered,
  lastWorkflowExecutionStep,
  workflowChoiceOptions,
  workflowEventActualScore,
  workflowEventBlockers,
  workflowEventDecision,
  workflowEventMetric,
  workflowExecutionStepsByNode,
  workflowNodeRunClasses,
  workflowNodeRunDetail,
  workflowNodeRunOutcome,
  workflowNumericValue,
  workflowScorePercent,
  workflowStepCanChoose,
  workflowStringValue,
} from '../workflowExecutionPresentation'

function step(overrides: Partial<WorkflowExecutionStep> = {}): WorkflowExecutionStep {
  return {
    step_index: 0,
    node_id: 'node-1',
    node_type: 'dialogue',
    label: 'Node 1',
    output: {},
    next_node_id: null,
    stopped_reason: null,
    ...overrides,
  }
}

function report(steps: WorkflowExecutionStep[]): WorkflowExecutionReport {
  return {
    workflow_id: 'workflow',
    workflow_name: 'Workflow',
    completed: false,
    stopped_reason: null,
    node_count: 2,
    executed_node_count: steps.length,
    coverage_percent: 50,
    executed_node_ids: steps.map(({ node_id }) => node_id),
    unvisited_node_ids: [],
    steps,
    validation: { valid: true, error_count: 0, warning_count: 0, issues: [] },
  }
}

const detailOptions = {
  reasonLabel: (reason: string) => `Reason: ${reason}`,
  nextNodeLabel: (nodeId: string) => `Next: ${nodeId}`,
}

describe('Workflow execution evidence presentation', () => {
  it('indexes the latest visit per node and reports the final trace step', () => {
    const first = step({ step_index: 0, node_id: 'loop', label: 'First' })
    const second = step({ step_index: 1, node_id: 'next' })
    const third = step({ step_index: 2, node_id: 'loop', label: 'Last' })
    const execution = report([first, second, third])

    expect(workflowExecutionStepsByNode(execution).get('loop')).toBe(third)
    expect(lastWorkflowExecutionStep(execution)).toBe(third)
    expect(workflowExecutionStepsByNode(null).size).toBe(0)
    expect(lastWorkflowExecutionStep(null)).toBeNull()
  })

  it('accepts only scalar choice labels and requires an awaiting choice step', () => {
    const choice = step({
      node_type: 'choice',
      stopped_reason: 'awaiting_choice',
      output: { choices: ['Stay', 2, true, null, { text: 'opaque' }] },
    })
    expect(workflowChoiceOptions(choice)).toEqual(['Stay', '2', 'true'])
    expect(workflowStepCanChoose(choice)).toBe(true)
    expect(workflowStepCanChoose({ ...choice, stopped_reason: 'completed' })).toBe(false)
    expect(workflowChoiceOptions(step())).toEqual([])
  })

  it('does not turn missing or blank numeric evidence into a real zero score', () => {
    expect(workflowNumericValue(null)).toBeNull()
    expect(workflowNumericValue(undefined)).toBeNull()
    expect(workflowNumericValue('  ')).toBeNull()
    expect(workflowNumericValue(false)).toBeNull()
    expect(workflowNumericValue([])).toBeNull()
    expect(workflowNumericValue({ value: 0 })).toBeNull()
    expect(workflowNumericValue('0')).toBe(0)
    expect(workflowNumericValue('0.75')).toBe(0.75)
    expect(workflowNumericValue(Number.POSITIVE_INFINITY)).toBeNull()
    expect(formatWorkflowScore(null)).toBe('-')
    expect(formatWorkflowScore(0)).toBe('0.00')
    expect(formatWorkflowThreshold(null, 'None')).toBe('None')
  })

  it('formats bounded percentages and preserves explicit coverage evidence', () => {
    expect(formatWorkflowCoverage(null)).toBe('0%')
    expect(formatWorkflowCoverage(66.6)).toBe('67%')
    expect(workflowScorePercent(-1)).toBe('0%')
    expect(workflowScorePercent(0.456)).toBe('46%')
    expect(workflowScorePercent(2)).toBe('100%')
  })

  it('normalizes string evidence without leaking object coercion into choices', () => {
    expect(workflowStringValue(null, 'fallback')).toBe('fallback')
    expect(workflowStringValue('  value  ')).toBe('value')
    expect(workflowStringValue(' ', 'fallback')).toBe('fallback')
    expect(workflowStringValue({ value: 'opaque' }, 'fallback')).toBe('fallback')
  })
})

describe('Workflow node run states', () => {
  it('classifies evaluation steps and formats score details', () => {
    const passed = step({
      node_type: 'evaluation',
      output: { passed: true, metric: 'engagement', score: 0.8, threshold: 0.7 },
    })
    const failed = step({ node_type: 'evaluation', output: { passed: false } })
    const scoreOnly = step({ node_type: 'evaluation', output: { score: null } })

    expect(isWorkflowEvaluationStep(passed)).toBe(true)
    expect(workflowNodeRunOutcome(passed)).toBe('pass')
    expect(workflowNodeRunOutcome(failed)).toBe('fail')
    expect(workflowNodeRunOutcome(scoreOnly)).toBe('score')
    expect(workflowNodeRunDetail(passed, detailOptions)).toBe('engagement 0.80/0.70')
    expect(workflowNodeRunDetail(scoreOnly, detailOptions)).toBe('overall -')
  })

  it('reads typed event decisions, nested rule metrics, blockers, and scores defensively', () => {
    const triggered = step({
      node_type: 'trigger_event',
      output: {
        triggered: true,
        event_id: 'first_friend',
        decision: {
          actual_score: 0.82,
          rule: { score_metric: 'engagement' },
          blocked_reasons: [],
        },
      },
    })
    const blocked = step({
      node_type: 'trigger_event',
      output: {
        event_id: 'first_friend',
        decision: { actual_score_metric: 'overall', blocked_reasons: [' score_below_threshold ', null, {}] },
      },
    })

    expect(isWorkflowTriggerEventStep(triggered)).toBe(true)
    expect(isWorkflowTriggerEventTriggered(triggered)).toBe(true)
    expect(workflowNodeRunOutcome(triggered)).toBe('triggered')
    expect(workflowEventMetric(triggered)).toBe('engagement')
    expect(workflowEventActualScore(triggered)).toBe(0.82)
    expect(workflowEventBlockers(blocked)).toEqual(['score_below_threshold'])
    expect(workflowNodeRunOutcome(blocked)).toBe('blocked')
    expect(workflowNodeRunDetail(blocked, detailOptions)).toBe('score_below_threshold')
    expect(workflowEventDecision(step({ output: { decision: [] } }))).toEqual({})
  })

  it('classifies choice, end, blocked, and ordinary transitions with stable classes', () => {
    const waiting = step({ stopped_reason: 'awaiting_choice' })
    const ending = step({ node_type: 'end' })
    const blocked = step({ stopped_reason: 'step_limit' })
    const ran = step({ next_node_id: 'node-2' })

    expect(workflowNodeRunOutcome(null)).toBe('')
    expect(workflowNodeRunOutcome(waiting)).toBe('wait')
    expect(workflowNodeRunOutcome(ending)).toBe('done')
    expect(workflowNodeRunOutcome(blocked)).toBe('blocked')
    expect(workflowNodeRunOutcome(ran)).toBe('ran')
    expect(workflowNodeRunDetail(blocked, detailOptions)).toBe('Reason: step_limit')
    expect(workflowNodeRunDetail(ran, detailOptions)).toBe('Next: node-2')
    expect(workflowNodeRunClasses('node-1', waiting, waiting)).toEqual({
      'run-executed': true,
      'run-current': true,
      'run-pass': false,
      'run-fail': false,
      'run-wait': true,
      'run-score': false,
    })
  })
})
