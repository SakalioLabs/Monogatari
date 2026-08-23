import { describe, expect, it } from 'vitest'

import {
  connectWorkflowNodes,
  createDefaultWorkflowFlow,
  createDefaultWorkflowNodeTypes,
  createWorkflowNode,
  findOpenWorkflowCanvasPosition,
  isWorkflowBooleanField,
  isWorkflowDocument,
  isWorkflowNumericField,
  nextWorkflowNodeSequence,
  normalizeWorkflowPath,
  removeWorkflowNode,
  safeWorkflowFileName,
  synchronizeWorkflowDocument,
  workflowConnectionPath,
  workflowConnections,
  workflowInputPortAtPoint,
  workflowNodeHasInput,
  workflowNodeHeight,
  workflowNodeAtPoint,
  workflowOutputPorts,
  workflowNumericFieldStep,
  workflowStoryPreviewTarget,
  WORKFLOW_NODE_HEIGHT,
  WORKFLOW_NODE_WIDTH,
  type WorkflowNodeTypeInfo,
} from '../workflowAuthoring'
import type { Workflow, WorkflowNode } from '../workflowContract'

function node(id: string, overrides: Partial<WorkflowNode> = {}): WorkflowNode {
  return {
    id,
    node_type: id === 'start' ? 'start' : 'dialogue',
    label: id,
    x: 20,
    y: 30,
    config: {},
    connections: [],
    ...overrides,
  }
}

function nodeType(nodeType: string): WorkflowNodeTypeInfo {
  return {
    node_type: nodeType,
    label: nodeType,
    description: nodeType,
    category: 'flow',
    configurable_fields: [],
  }
}

describe('workflow authoring contracts', () => {
  it('mirrors the authoritative node catalog fields and returns isolated copies', () => {
    const first = createDefaultWorkflowNodeTypes()
    const second = createDefaultWorkflowNodeTypes()
    expect(first).toHaveLength(24)
    expect(new Set(first.map((entry) => entry.node_type)).size).toBe(first.length)
    expect(first.find((entry) => entry.node_type === 'dialogue')?.configurable_fields)
      .toEqual(['speaker', 'text', 'emotion', 'use_llm'])
    expect(first.find((entry) => entry.node_type === 'llm_generate')?.configurable_fields)
      .toEqual(['prompt', 'system_prompt', 'max_tokens'])
    first[0].configurable_fields.push('mutated')
    expect(second[0].configurable_fields).toEqual([])
  })

  it('treats boolean and integer-backed fields as typed controls', () => {
    expect(isWorkflowBooleanField('dialogue', 'use_llm')).toBe(true)
    expect(isWorkflowBooleanField('set_flag', 'value')).toBe(true)
    expect(isWorkflowBooleanField('dialogue', 'text')).toBe(false)
    expect(isWorkflowNumericField('max_tokens')).toBe(true)
    expect(workflowNumericFieldStep('max_tokens')).toBe('1')
    expect(workflowNumericFieldStep('threshold')).toBe('0.05')
  })

  it('derives stage-preview targets only from complete story-entry references', () => {
    expect(workflowStoryPreviewTarget(node('dialogue-entry', {
      node_type: 'dialogue_entry',
      config: { dialogue_id: 'chapter_one', entry_node_id: 'opening' },
    }))).toEqual({ kind: 'dialogue', dialogueId: 'chapter_one', entryNodeId: 'opening' })
    expect(workflowStoryPreviewTarget(node('roleplay-entry', {
      node_type: 'scene_roleplay_entry',
      config: { roleplay_id: 'guild_free_talk' },
    }))).toEqual({ kind: 'scene-roleplay', roleplayId: 'guild_free_talk' })
    expect(workflowStoryPreviewTarget(node('campaign-entry', {
      node_type: 'roleplay_campaign_entry',
      config: { campaign_id: 'volume_one' },
    }))).toEqual({ kind: 'roleplay-campaign', campaignId: 'volume_one' })
    expect(workflowStoryPreviewTarget(node('incomplete', {
      node_type: 'dialogue_entry',
      config: { dialogue_id: 'chapter_one' },
    }))).toBeNull()
    expect(workflowStoryPreviewTarget(node('plain-dialogue'))).toBeNull()
  })

  it('builds deterministic connection geometry and bezier paths', () => {
    const nodes = [
      node('start', { x: 10, y: 20, connections: ['end'] }),
      node('end', { node_type: 'end', x: 400, y: 80 }),
    ]
    const connections = workflowConnections(nodes)
    expect(connections).toEqual([{
      sourceNodeId: 'start',
      targetNodeId: 'end',
      sourcePortIndex: 0,
      x1: 10 + WORKFLOW_NODE_WIDTH,
      y1: 20 + WORKFLOW_NODE_HEIGHT / 2,
      x2: 400,
      y2: 80 + WORKFLOW_NODE_HEIGHT / 2,
    }])
    expect(workflowConnectionPath(connections[0])).toBe('M 224 66 C 312 66, 312 126, 400 126')
  })

  it('allocates collision-free node ids and clamps new nodes inside the canvas', () => {
    const nodes = [node('node_1'), node('node_3')]
    expect(nextWorkflowNodeSequence(nodes)).toBe(2)
    const result = createWorkflowNode(nodeType('dialogue'), 'Dialogue', 0, 0, nodes, 1)
    expect(result.node).toMatchObject({ id: 'node_2', x: 16, y: 16 })
    expect(result.nextNodeSequence).toBe(4)
    expect(createWorkflowNode(nodeType('dialogue'), 'Dialogue', Number.NaN, Number.NaN, [], 1).node)
      .toMatchObject({ x: 16, y: 16 })
  })

  it('lays out a connected default flow horizontally or vertically by available width', () => {
    const types = createDefaultWorkflowNodeTypes()
    const wide = createDefaultWorkflowFlow(types, (type) => type.toUpperCase(), 640)
    expect(wide.startNodeId).toBe('node_1')
    expect(wide.nodes[0].connections).toEqual(['node_2'])
    expect(wide.nodes[1].x).toBeGreaterThan(wide.nodes[0].x)
    expect(wide.nodes[1].y).toBe(wide.nodes[0].y)
    expect(wide.nextNodeSequence).toBe(3)

    const narrow = createDefaultWorkflowFlow(types, (type) => type, WORKFLOW_NODE_WIDTH + 40)
    expect(narrow.nodes[1].y).toBeGreaterThan(narrow.nodes[0].y)
    const invalidWidth = createDefaultWorkflowFlow(types, (type) => type, Number.NaN)
    expect(invalidWidth.nodes.every((entry) => Number.isFinite(entry.x) && Number.isFinite(entry.y)))
      .toBe(true)
  })

  it('finds open canvas positions and falls back deterministically when the grid is full', () => {
    expect(findOpenWorkflowCanvasPosition([], 640, 520)).toEqual({ x: 28, y: 28 })
    const occupied = node('occupied', { x: 28, y: 28 })
    expect(findOpenWorkflowCanvasPosition([occupied], 640, 520)).toEqual({
      x: 28 + WORKFLOW_NODE_WIDTH + 24,
      y: 28,
    })
    expect(findOpenWorkflowCanvasPosition([occupied], WORKFLOW_NODE_WIDTH + 32, WORKFLOW_NODE_HEIGHT + 32))
      .toEqual({ x: 46, y: 46 })
  })

  it('connects and removes nodes without mutating the source graph', () => {
    const source = [node('start'), node('end', { node_type: 'end' })]
    const connected = connectWorkflowNodes(source, 'start', 'end')
    expect(connected.changed).toBe(true)
    expect(connected.nodes[0].connections).toEqual(['end'])
    expect(source[0].connections).toEqual([])
    expect(connectWorkflowNodes(connected.nodes, 'start', 'end').changed).toBe(false)
    expect(connectWorkflowNodes(connected.nodes, 'start', 'start').changed).toBe(false)

    const removed = removeWorkflowNode(connected.nodes, 'end')
    expect(removed.map((entry) => entry.id)).toEqual(['start'])
    expect(removed[0].connections).toEqual([])
    expect(connected.nodes[0].connections).toEqual(['end'])

    const branched = [
      node('gate', { node_type: 'condition', connections: ['yes', 'no'] }),
      node('yes'),
      node('no'),
    ]
    expect(removeWorkflowNode(branched, 'yes')[0].connections).toEqual([])
    expect(removeWorkflowNode(branched, 'no')[0].connections).toEqual(['yes'])
  })

  it('models runtime branch ports and replaces only the selected output', () => {
    const condition = node('gate', {
      node_type: 'condition',
      config: { condition: 'score > 0.5' },
      connections: ['yes', 'no'],
    })
    expect(workflowOutputPorts(condition).map(port => [port.index, port.kind, port.label]))
      .toEqual([[0, 'true', 'True'], [1, 'false', 'False']])

    const choice = node('choice', {
      node_type: 'choice',
      config: { choices: ['Stay', 'Leave', 'Ask'] },
    })
    expect(workflowOutputPorts(choice).map(port => port.label)).toEqual(['Stay', 'Leave', 'Ask'])
    expect(workflowOutputPorts(node('random', { node_type: 'random_branch' }))).toHaveLength(2)
    expect(workflowNodeHeight(choice)).toBeGreaterThan(WORKFLOW_NODE_HEIGHT)

    const graph = [
      condition,
      node('yes'),
      node('no'),
      node('later'),
      node('start', { node_type: 'start' }),
    ]
    const replaced = connectWorkflowNodes(graph, 'gate', 'later', 1)
    expect(replaced.reason).toBe('replaced')
    expect(replaced.nodes[0].connections).toEqual(['yes', 'later'])
    expect(connectWorkflowNodes(graph, 'gate', 'start', 0).reason).toBe('target_has_no_input')
    expect(connectWorkflowNodes(graph, 'gate', 'later', 2).reason).toBe('invalid_output')
  })

  it('accepts connection drops only on input ports', () => {
    const start = node('start', { node_type: 'start', x: 10, y: 10 })
    const target = node('target', { x: 300, y: 100 })
    expect(workflowNodeHasInput(start)).toBe(false)
    expect(workflowNodeHasInput(target)).toBe(true)
    expect(workflowInputPortAtPoint([start, target], { x: 300, y: 146 })).toBe(target)
    expect(workflowInputPortAtPoint([start, target], { x: 350, y: 146 })).toBeUndefined()
  })

  it('finds nodes at inclusive canvas boundaries', () => {
    const target = node('target', { x: 100, y: 200 })
    expect(workflowNodeAtPoint([target], { x: 100 + WORKFLOW_NODE_WIDTH, y: 200 + WORKFLOW_NODE_HEIGHT }))
      .toBe(target)
    expect(workflowNodeAtPoint([target], { x: 99, y: 200 })).toBeUndefined()
  })

  it('repairs stale start ids while synchronizing the authored document', () => {
    const workflow: Workflow = {
      id: 'wf',
      name: 'Workflow',
      nodes: [],
      start_node_id: 'missing',
    }
    const nodes = [node('dialogue'), node('start', { node_type: 'start' })]
    const synchronized = synchronizeWorkflowDocument(workflow, nodes)
    expect(synchronized.start_node_id).toBe('start')
    expect(synchronized.nodes).toBe(nodes)
    expect(workflow.nodes).toEqual([])
  })

  it('normalizes paths, portable download names, and complete workflow shapes', () => {
    expect(normalizeWorkflowPath(' workflows\\intro ', true)).toBe('workflows/intro.json')
    expect(normalizeWorkflowPath('intro.JSON', true)).toBe('intro.JSON')
    expect(safeWorkflowFileName('  Opening / Route  ')).toBe('Opening-Route')
    expect(safeWorkflowFileName('***')).toBe('workflow')

    const valid: Workflow = {
      id: 'wf',
      name: 'Workflow',
      start_node_id: 'start',
      nodes: [node('start')],
    }
    expect(isWorkflowDocument(valid)).toBe(true)
    expect(isWorkflowDocument({ ...valid, nodes: [null] })).toBe(false)
    expect(isWorkflowDocument({ ...valid, nodes: [{ ...valid.nodes[0], x: '10' }] })).toBe(false)
    expect(isWorkflowDocument({ ...valid, nodes: [{ ...valid.nodes[0], connections: [1] }] })).toBe(false)
  })
})
