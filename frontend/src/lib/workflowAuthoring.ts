import type {
  Workflow,
  WorkflowFileSummary,
  WorkflowNode,
  WorkflowNodeTypeInfo,
} from './workflowContract'

export type { WorkflowFileSummary, WorkflowNodeTypeInfo } from './workflowContract'

export interface WorkflowConnectionGeometry {
  sourceNodeId: string
  targetNodeId: string
  x1: number
  y1: number
  x2: number
  y2: number
}

export interface WorkflowPoint {
  x: number
  y: number
}

export interface WorkflowNodeBuildResult {
  node: WorkflowNode
  nextNodeSequence: number
}

export interface WorkflowDefaultFlow {
  nodes: WorkflowNode[]
  startNodeId: string
  nextNodeSequence: number
}

export interface WorkflowConnectionUpdate {
  nodes: WorkflowNode[]
  changed: boolean
}

export const WORKFLOW_NODE_WIDTH = 214
export const WORKFLOW_NODE_HEIGHT = 92

const DEFAULT_WORKFLOW_NODE_TYPES: readonly WorkflowNodeTypeInfo[] = [
  { node_type: 'start', label: 'Start', description: 'Starting point of the workflow', category: 'flow', configurable_fields: [] },
  { node_type: 'dialogue', label: 'Dialogue', description: 'Show dialogue text from a character', category: 'content', configurable_fields: ['speaker', 'text', 'emotion', 'use_llm'] },
  { node_type: 'choice', label: 'Choice', description: 'Present choices to the player', category: 'content', configurable_fields: ['choices'] },
  { node_type: 'condition', label: 'Condition', description: 'Branch based on a condition', category: 'flow', configurable_fields: ['condition'] },
  { node_type: 'set_variable', label: 'Set Variable', description: 'Set a game variable', category: 'logic', configurable_fields: ['variable_name', 'value'] },
  { node_type: 'set_flag', label: 'Set Flag', description: 'Set a game flag', category: 'logic', configurable_fields: ['flag_name', 'value'] },
  { node_type: 'llm_generate', label: 'LLM Generate', description: 'Generate text using LLM', category: 'ai', configurable_fields: ['prompt', 'system_prompt', 'max_tokens'] },
  { node_type: 'evaluation', label: 'Evaluation', description: 'Read the latest LLM conversation score and compare a threshold', category: 'ai', configurable_fields: ['character_id', 'criteria', 'threshold', 'variable_name'] },
  { node_type: 'scene_change', label: 'Scene Change', description: 'Change the active background scene', category: 'content', configurable_fields: ['scene_id'] },
  { node_type: 'trigger_event', label: 'Trigger Event', description: 'Preview and trigger a score-aware story event', category: 'flow', configurable_fields: ['character_id', 'event_id', 'event_type'] },
  { node_type: 'emotion_change', label: 'Change Emotion', description: "Change a character's emotion", category: 'character', configurable_fields: ['character_id', 'emotion'] },
  { node_type: 'relationship', label: 'Relationship', description: 'Modify relationship score', category: 'character', configurable_fields: ['character_id', 'delta'] },
  { node_type: 'end', label: 'End', description: 'End of the workflow', category: 'flow', configurable_fields: [] },
  { node_type: 'narration', label: 'Narration', description: 'Display narrator text or inner monologue', category: 'content', configurable_fields: ['text', 'speaker'] },
  { node_type: 'bgm', label: 'BGM', description: 'Control background music playback', category: 'media', configurable_fields: ['track_path', 'action', 'volume'] },
  { node_type: 'sfx', label: 'SFX', description: 'Play a sound effect', category: 'media', configurable_fields: ['sound_path', 'volume'] },
  { node_type: 'wait', label: 'Wait', description: 'Pause workflow execution for a duration', category: 'flow', configurable_fields: ['duration_ms'] },
  { node_type: 'random_branch', label: 'Random Branch', description: 'Randomly select one of multiple branches', category: 'flow', configurable_fields: ['weights'] },
  { node_type: 'sub_workflow', label: 'Sub Workflow', description: 'Execute another workflow as a subroutine', category: 'flow', configurable_fields: ['workflow_id', 'workflow_path'] },
  { node_type: 'camera', label: 'Camera', description: 'Control camera position, zoom, and effects', category: 'media', configurable_fields: ['action', 'target_x', 'target_y', 'zoom', 'duration_ms'] },
  { node_type: 'shake', label: 'Shake', description: 'Screen shake effect for dramatic moments', category: 'media', configurable_fields: ['intensity', 'duration_ms'] },
]

export function createDefaultWorkflowNodeTypes(): WorkflowNodeTypeInfo[] {
  return DEFAULT_WORKFLOW_NODE_TYPES.map((nodeType) => ({
    ...nodeType,
    configurable_fields: [...nodeType.configurable_fields],
  }))
}

export function workflowConnections(nodes: readonly WorkflowNode[]): WorkflowConnectionGeometry[] {
  const lookup = new Map(nodes.map((node) => [node.id, node]))
  const connections: WorkflowConnectionGeometry[] = []
  for (const node of nodes) {
    for (const targetNodeId of node.connections) {
      const target = lookup.get(targetNodeId)
      if (!target) continue
      connections.push({
        sourceNodeId: node.id,
        targetNodeId,
        x1: node.x + WORKFLOW_NODE_WIDTH,
        y1: node.y + WORKFLOW_NODE_HEIGHT / 2,
        x2: target.x,
        y2: target.y + WORKFLOW_NODE_HEIGHT / 2,
      })
    }
  }
  return connections
}

export function workflowConnectionPath(connection: WorkflowConnectionGeometry): string {
  const middle = Math.max(60, Math.abs(connection.x2 - connection.x1) / 2)
  return `M ${connection.x1} ${connection.y1} C ${connection.x1 + middle} ${connection.y1}, ${connection.x2 - middle} ${connection.y2}, ${connection.x2} ${connection.y2}`
}

export function workflowNodeIcon(nodeType: string): string {
  const icons: Record<string, string> = {
    start: 'ST',
    dialogue: 'DG',
    choice: 'CH',
    condition: 'IF',
    set_variable: 'VR',
    set_flag: 'FL',
    llm_generate: 'AI',
    emotion_change: 'EM',
    relationship: 'RL',
    scene_change: 'SC',
    trigger_event: 'EV',
    evaluation: 'QA',
    end: 'EN',
  }
  return icons[nodeType] || 'ND'
}

export function workflowConfigFields(
  nodeTypes: readonly WorkflowNodeTypeInfo[],
  nodeType: string,
): string[] {
  const fields = nodeTypes.find((candidate) => candidate.node_type === nodeType)?.configurable_fields
  return fields ? [...fields] : []
}

export function isWorkflowLongField(field: string): boolean {
  return field === 'text' || field === 'prompt' || field === 'system_prompt'
}

export function isWorkflowBooleanField(nodeType: string, field: string): boolean {
  return (nodeType === 'set_flag' && field === 'value')
    || (nodeType === 'dialogue' && field === 'use_llm')
}

export function isWorkflowNumericField(field: string): boolean {
  return [
    'threshold',
    'delta',
    'volume',
    'duration_ms',
    'max_tokens',
    'target_x',
    'target_y',
    'zoom',
    'intensity',
  ].includes(field)
}

export function workflowNumericFieldStep(field: string): string | undefined {
  if (!isWorkflowNumericField(field)) return undefined
  return field === 'duration_ms' || field === 'max_tokens' ? '1' : '0.05'
}

export function nextWorkflowNodeSequence(
  nodes: readonly WorkflowNode[],
  minimum = 1,
): number {
  const ids = new Set(nodes.map((node) => node.id))
  let sequence = Number.isFinite(minimum) ? Math.max(1, Math.trunc(minimum)) : 1
  while (ids.has(`node_${sequence}`)) sequence += 1
  return sequence
}

export function createWorkflowNode(
  nodeType: WorkflowNodeTypeInfo,
  label: string,
  pointerX: number,
  pointerY: number,
  nodes: readonly WorkflowNode[],
  minimumSequence = 1,
): WorkflowNodeBuildResult {
  const sequence = nextWorkflowNodeSequence(nodes, minimumSequence)
  const safePointerX = Number.isFinite(pointerX) ? pointerX : WORKFLOW_NODE_WIDTH / 2 + 16
  const safePointerY = Number.isFinite(pointerY) ? pointerY : 44
  const node: WorkflowNode = {
    id: `node_${sequence}`,
    node_type: nodeType.node_type,
    label,
    x: Math.max(16, safePointerX - WORKFLOW_NODE_WIDTH / 2),
    y: Math.max(16, safePointerY - 28),
    config: {},
    connections: [],
  }
  return {
    node,
    nextNodeSequence: nextWorkflowNodeSequence([...nodes, node], sequence + 1),
  }
}

export function findOpenWorkflowCanvasPosition(
  nodes: readonly WorkflowNode[],
  canvasWidth = 640,
  canvasHeight = 520,
): WorkflowPoint {
  const width = Math.max(
    WORKFLOW_NODE_WIDTH + 32,
    Number.isFinite(canvasWidth) ? canvasWidth : 640,
  )
  const height = Math.max(
    WORKFLOW_NODE_HEIGHT + 32,
    Number.isFinite(canvasHeight) ? canvasHeight : 520,
  )
  for (let y = 28; y <= Math.max(28, height - WORKFLOW_NODE_HEIGHT - 24); y += WORKFLOW_NODE_HEIGHT + 24) {
    for (let x = 28; x <= Math.max(28, width - WORKFLOW_NODE_WIDTH - 24); x += WORKFLOW_NODE_WIDTH + 24) {
      const overlaps = nodes.some((node) => (
        x < node.x + WORKFLOW_NODE_WIDTH + 12
        && x + WORKFLOW_NODE_WIDTH + 12 > node.x
        && y < node.y + WORKFLOW_NODE_HEIGHT + 12
        && y + WORKFLOW_NODE_HEIGHT + 12 > node.y
      ))
      if (!overlaps) return { x, y }
    }
  }
  const offset = (nodes.length % 6) * 18
  return { x: 28 + offset, y: 28 + offset }
}

export function createDefaultWorkflowFlow(
  nodeTypes: readonly WorkflowNodeTypeInfo[],
  labelForNodeType: (nodeType: string) => string,
  canvasWidth = 640,
  minimumSequence = 1,
): WorkflowDefaultFlow {
  const startType = nodeTypes.find((nodeType) => nodeType.node_type === 'start')
  const endType = nodeTypes.find((nodeType) => nodeType.node_type === 'end')
  if (!startType || !endType) throw new Error('Workflow node catalog requires start and end nodes.')

  const width = Math.max(
    WORKFLOW_NODE_WIDTH + 40,
    Number.isFinite(canvasWidth) ? canvasWidth : 640,
  )
  const startX = 20
  const startY = 132
  const horizontalEndX = Math.max(startX, width - WORKFLOW_NODE_WIDTH - 20)
  const hasHorizontalRoom = horizontalEndX >= startX + WORKFLOW_NODE_WIDTH + 12
  const endX = hasHorizontalRoom ? horizontalEndX : Math.max(20, (width - WORKFLOW_NODE_WIDTH) / 2)
  const endY = hasHorizontalRoom ? startY : startY + WORKFLOW_NODE_HEIGHT + 34
  const startResult = createWorkflowNode(
    startType,
    labelForNodeType('start'),
    startX + WORKFLOW_NODE_WIDTH / 2,
    startY + 28,
    [],
    minimumSequence,
  )
  const endResult = createWorkflowNode(
    endType,
    labelForNodeType('end'),
    endX + WORKFLOW_NODE_WIDTH / 2,
    endY + 28,
    [startResult.node],
    startResult.nextNodeSequence,
  )
  return {
    nodes: [{ ...startResult.node, connections: [endResult.node.id] }, endResult.node],
    startNodeId: startResult.node.id,
    nextNodeSequence: endResult.nextNodeSequence,
  }
}

export function removeWorkflowNode(
  nodes: readonly WorkflowNode[],
  nodeId: string,
): WorkflowNode[] {
  return nodes
    .filter((node) => node.id !== nodeId)
    .map((node) => node.connections.includes(nodeId)
      ? { ...node, connections: node.connections.filter((targetId) => targetId !== nodeId) }
      : node)
}

export function connectWorkflowNodes(
  nodes: readonly WorkflowNode[],
  sourceNodeId: string,
  targetNodeId: string,
): WorkflowConnectionUpdate {
  const source = nodes.find((node) => node.id === sourceNodeId)
  const targetExists = nodes.some((node) => node.id === targetNodeId)
  if (!source || !targetExists || sourceNodeId === targetNodeId || source.connections.includes(targetNodeId)) {
    return { nodes: [...nodes], changed: false }
  }
  return {
    nodes: nodes.map((node) => node.id === sourceNodeId
      ? { ...node, connections: [...node.connections, targetNodeId] }
      : node),
    changed: true,
  }
}

export function workflowNodeAtPoint(
  nodes: readonly WorkflowNode[],
  point: WorkflowPoint,
): WorkflowNode | undefined {
  return nodes.find((node) => point.x >= node.x
    && point.x <= node.x + WORKFLOW_NODE_WIDTH
    && point.y >= node.y
    && point.y <= node.y + WORKFLOW_NODE_HEIGHT)
}

export function synchronizeWorkflowDocument(
  workflow: Workflow,
  nodes: WorkflowNode[],
): Workflow {
  const startNodeId = nodes.some((node) => node.id === workflow.start_node_id)
    ? workflow.start_node_id
    : nodes.find((node) => node.node_type === 'start')?.id || nodes[0]?.id || ''
  return {
    ...workflow,
    nodes,
    start_node_id: startNodeId,
  }
}

export function normalizeWorkflowPath(value: string, appendExtension: boolean): string {
  let path = value.trim().replace(/\\/g, '/')
  if (appendExtension && path && !path.toLowerCase().endsWith('.json')) path += '.json'
  return path
}

export function safeWorkflowFileName(value: string): string {
  return value.trim().replace(/[^a-z0-9._-]+/gi, '-').replace(/^-+|-+$/g, '') || 'workflow'
}

export function isWorkflowDocument(value: unknown): value is Workflow {
  if (!isRecord(value)
    || typeof value.id !== 'string'
    || typeof value.name !== 'string'
    || typeof value.start_node_id !== 'string'
    || !Array.isArray(value.nodes)) return false
  return value.nodes.every((node) => isRecord(node)
    && typeof node.id === 'string'
    && typeof node.node_type === 'string'
    && typeof node.label === 'string'
    && typeof node.x === 'number'
    && Number.isFinite(node.x)
    && typeof node.y === 'number'
    && Number.isFinite(node.y)
    && isRecord(node.config)
    && Array.isArray(node.connections)
    && node.connections.every((targetId) => typeof targetId === 'string'))
}

function isRecord(value: unknown): value is Record<string, any> {
  return Boolean(value) && typeof value === 'object' && !Array.isArray(value)
}
