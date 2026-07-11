import { describe, expect, it } from 'vitest'

import type { StoryEventDefinition } from '../storyEvents'
import {
  aggregatePresetMatrixCoverage,
  runWorkflowLocally,
  validateWorkflowLocally,
  workflowRunContextPayloadFromValues,
  type Workflow,
  type WorkflowNode,
  type WorkflowRunContextPreset,
} from '../workflowPreview'

function node(
  id: string,
  node_type: string,
  connections: string[] = [],
  config: Record<string, any> = {},
): WorkflowNode {
  return { id, node_type, label: id, x: 0, y: 0, config, connections }
}

function workflow(nodes: WorkflowNode[], startNodeId = 'start'): Workflow {
  return { id: 'preview', name: 'Preview', nodes, start_node_id: startNodeId }
}

function event(overrides: Partial<StoryEventDefinition> = {}): StoryEventDefinition {
  return {
    event_id: 'bond_gate',
    event_type: 'relationship_milestone',
    description: 'Bond gate',
    data: {},
    actions: [{ type: 'unlock_scene', scene_id: 'studio' }],
    source_path: 'events/story_events.json',
    rule: {
      event_id: 'bond_gate',
      event_type: 'relationship_milestone',
      min_relationship: 0.5,
      score_metric: 'engagement',
      min_score: 0.8,
      min_evaluation_count: 2,
      character_ids: ['aoi'],
      repeatable: false,
    },
    ...overrides,
  }
}

describe('browser workflow preview', () => {
  it('normalizes bounded run context and stable triggered-event ids', () => {
    expect(workflowRunContextPayloadFromValues({
      character_id: ' aoi ',
      friendliness: -2,
      engagement: 2,
      creativity: Number.NaN,
      overall_score: 0.75,
      relationship: 4,
      evaluation_count: -3.6,
      already_triggered_events: 'event_b, event_a\nevent_b',
    })).toMatchObject({
      character_id: 'aoi',
      relationship: 1,
      evaluation_count: 0,
      already_triggered_events: ['event_b', 'event_a'],
      evaluation: {
        friendliness: 0,
        engagement: 1,
        creativity: 0,
        overall_score: 0.75,
      },
    })
  })

  it('reports graph, state-key, and event-catalog errors before execution', () => {
    const result = validateWorkflowLocally(workflow([
      node('start', 'start', ['write']),
      node('write', 'set_variable', ['gate'], { variable_name: '../route', value: 'open' }),
      node('gate', 'trigger_event', ['missing'], { event_id: 'unknown_event' }),
    ]), [event()])

    expect(result.valid).toBe(false)
    expect(result.issues.map((issue) => issue.code)).toEqual(expect.arrayContaining([
      'end_node_missing',
      'node_state_key_invalid',
      'node_event_unknown',
      'connection_target_missing',
    ]))
  })

  it('mirrors local variable and flag writes through condition branches', () => {
    const current = workflow([
      node('start', 'start', ['set_route']),
      node('set_route', 'set_variable', ['set_ready'], { variable_name: 'preview.route', value: 'open' }),
      node('set_ready', 'set_flag', ['condition'], { flag_name: 'preview.ready', value: true }),
      node('condition', 'condition', ['success', 'failure'], {
        condition: 'getVariable("preview.route") == "open" && hasFlag("preview.ready")',
      }),
      node('success', 'end'),
      node('failure', 'end'),
    ])

    const report = runWorkflowLocally(current)

    expect(report.completed).toBe(true)
    expect(report.stopped_reason).toBe('completed')
    expect(report.executed_node_ids).toEqual(['start', 'set_route', 'set_ready', 'condition', 'success'])
    expect(report.steps.find((step) => step.node_id === 'condition')?.output).toMatchObject({
      result: true,
      condition_supported: true,
      condition_error: null,
    })
  })

  it('stops unsupported conditions instead of silently taking the false branch', () => {
    const report = runWorkflowLocally(workflow([
      node('start', 'start', ['condition']),
      node('condition', 'condition', ['success', 'failure'], { condition: 'unknownCall()' }),
      node('success', 'end'),
      node('failure', 'end'),
    ]))

    expect(report.completed).toBe(false)
    expect(report.stopped_reason).toBe('condition_unsupported')
    expect(report.executed_node_ids).toEqual(['start', 'condition'])
    expect(report.steps[1].output.condition_supported).toBe(false)
  })

  it('distinguishes a missing preview evaluation from an unknown metric', () => {
    const current = workflow([
      node('start', 'start', ['evaluation']),
      node('evaluation', 'evaluation', ['passed', 'failed'], { criteria: 'overall', threshold: 0.5 }),
      node('passed', 'end'),
      node('failed', 'end'),
    ])
    const fallback = runWorkflowLocally(current)
    expect(fallback.executed_node_ids).toEqual(['start', 'evaluation', 'failed'])
    expect(fallback.steps[1].output).toMatchObject({ metric_supported: true, score: 0, passed: false })

    current.nodes[1].config.criteria = 'mystery'
    const unsupported = runWorkflowLocally(current)
    expect(unsupported.stopped_reason).toBe('evaluation_metric_unsupported')
    expect(unsupported.executed_node_ids).toEqual(['start', 'evaluation'])
  })

  it('applies preview relationship state to score-gated event decisions without leaking between runs', () => {
    const current = workflow([
      node('start', 'start', ['relationship']),
      node('relationship', 'relationship', ['event'], { character_id: 'aoi', delta: 0.4 }),
      node('event', 'trigger_event', ['success', 'blocked'], {
        character_id: 'aoi',
        event_id: 'bond_gate',
        event_type: 'relationship_milestone',
      }),
      node('success', 'end'),
      node('blocked', 'end'),
    ])
    const context = workflowRunContextPayloadFromValues({
      character_id: 'aoi',
      friendliness: 0.7,
      engagement: 0.9,
      creativity: 0.6,
      overall_score: 0.8,
      relationship: 0.1,
      evaluation_count: 2,
      already_triggered_events: '',
    })

    const first = runWorkflowLocally(current, { context, storyEvents: [event()] })
    const second = runWorkflowLocally(current, { context, storyEvents: [event()] })

    for (const report of [first, second]) {
      expect(report.executed_node_ids).toEqual(['start', 'relationship', 'event', 'success'])
      expect(report.steps[1].output).toMatchObject({ previous: 0.1, current: 0.5 })
      expect(report.steps[2].output).toMatchObject({
        triggered: true,
        applied: false,
        actions: [{ type: 'unlock_scene', scene_id: 'studio' }],
      })
    }
  })

  it('uses an injectable random source and emits useful scene and narration previews', () => {
    const randomReport = runWorkflowLocally(workflow([
      node('start', 'start', ['random']),
      node('random', 'random_branch', ['first', 'second'], { weights: [1, 3] }),
      node('first', 'end'),
      node('second', 'end'),
    ]), { random: () => 0.9 })
    expect(randomReport.executed_node_ids).toEqual(['start', 'random', 'second'])
    expect(randomReport.steps[1].output).toMatchObject({ index: 1, weights: [1, 3] })

    const contentReport = runWorkflowLocally(workflow([
      node('start', 'start', ['scene']),
      node('scene', 'scene_change', ['narration'], {
        scene_id: 'studio',
        background_path: 'assets/backgrounds/studio.svg',
      }),
      node('narration', 'narration', ['end'], { speaker: 'Guide', text: 'Welcome.' }),
      node('end', 'end'),
    ]))
    expect(contentReport.steps[1].output).toMatchObject({
      action: 'scene_change',
      scene_id: 'studio',
      background_path: 'assets/backgrounds/studio.svg',
    })
    expect(contentReport.steps[2].output).toEqual({
      action: 'narration',
      speaker: 'Guide',
      text: 'Welcome.',
    })
  })

  it('merges preset coverage in workflow node order', () => {
    const current = workflow([
      node('start', 'start', ['choice']),
      node('choice', 'choice', ['left', 'right'], { choices: ['Left', 'Right'] }),
      node('left', 'end'),
      node('right', 'end'),
    ])
    const presets: WorkflowRunContextPreset[] = [
      { id: 'left', label: 'Left', values: presetValues() },
      { id: 'right', label: 'Right', values: presetValues() },
    ]
    const report = aggregatePresetMatrixCoverage(current, [
      { preset: presets[0], report: runWorkflowLocally(current, { selections: { choice: 0 } }) },
      { preset: presets[1], report: runWorkflowLocally(current, { selections: { choice: 1 } }) },
    ])

    expect(report.coverage_percent).toBe(100)
    expect(report.executed_node_ids).toEqual(['start', 'choice', 'left', 'right'])
    expect(report.unvisited_node_ids).toEqual([])
  })
})

function presetValues(): WorkflowRunContextPreset['values'] {
  return {
    character_id: 'aoi',
    friendliness: 0.5,
    engagement: 0.5,
    creativity: 0.5,
    overall_score: 0.5,
    relationship: 0,
    evaluation_count: 0,
    already_triggered_events: '',
  }
}
