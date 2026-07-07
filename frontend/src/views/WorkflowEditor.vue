<template>
  <div class="workflow-editor">
    <header class="toolbar">
      <div class="toolbar-left">
        <span class="eyebrow">Authoring</span>
        <h1>Workflow Editor</h1>
        <span class="workflow-name">{{ workflow?.name || 'Untitled' }}</span>
      </div>
      <div class="toolbar-right">
        <button class="btn btn-secondary btn-sm" @click="newWorkflow">New</button>
        <button class="btn btn-secondary btn-sm" @click="loadWorkflow">Open</button>
        <button class="btn btn-secondary btn-sm" @click="validateCurrentWorkflow">Validate</button>
        <button class="btn btn-primary btn-sm" @click="saveWorkflow">Save</button>
        <button class="btn btn-primary btn-sm" @click="exportJSON">Export</button>
        <span class="validation-pill" :class="validationStatusClass">{{ validationStatusLabel }}</span>
      </div>
    </header>

    <main class="editor-body">
      <aside class="node-palette">
        <div class="panel-title">
          <span class="eyebrow">Nodes</span>
          <strong>{{ nodeTypes.length }}</strong>
        </div>
        <div v-for="category in nodeCategories" :key="category.name" class="palette-category">
          <h2>{{ category.name }}</h2>
          <button
            v-for="nodeType in category.nodes"
            :key="nodeType.node_type"
            class="palette-node"
            draggable="true"
            @dragstart="onDragStart($event, nodeType)"
          >
            <span class="node-icon">{{ getNodeIcon(nodeType.node_type) }}</span>
            <span class="palette-copy">
              <strong>{{ nodeType.label }}</strong>
              <small>{{ nodeType.description }}</small>
            </span>
          </button>
        </div>
      </aside>

      <section
        ref="canvasRef"
        class="canvas"
        @drop="onDrop"
        @dragover.prevent
        @mousedown="onCanvasMouseDown"
      >
        <svg class="canvas-grid" width="100%" height="100%">
          <defs>
            <pattern id="grid" width="24" height="24" patternUnits="userSpaceOnUse">
              <path d="M 24 0 L 0 0 0 24" fill="none" stroke="rgba(170,180,195,0.12)" stroke-width="1" />
            </pattern>
          </defs>
          <rect width="100%" height="100%" fill="url(#grid)" />
        </svg>

        <svg class="connections" width="100%" height="100%">
          <path
            v-for="(conn, i) in connections"
            :key="i"
            :d="connectionPath(conn)"
            fill="none"
            stroke="var(--brand)"
            stroke-width="2"
          />
        </svg>

        <article
          v-for="node in nodes"
          :key="node.id"
          class="workflow-node"
          :class="[{ selected: selectedNode?.id === node.id }, 'node-type-' + node.node_type]"
          :style="{ left: node.x + 'px', top: node.y + 'px' }"
          @mousedown.stop="onNodeMouseDown($event, node)"
          @click.stop="selectNode(node)"
        >
          <header class="node-header">
            <span class="node-icon">{{ getNodeIcon(node.node_type) }}</span>
            <strong>{{ node.label }}</strong>
          </header>
          <div class="node-body">
            <span>{{ node.node_type }}</span>
            <button class="node-port output" title="Connect" @mousedown.stop="startConnection($event, node)"></button>
          </div>
        </article>
      </section>

      <aside class="properties-panel">
        <template v-if="selectedNode">
          <div class="panel-title">
            <span class="eyebrow">Properties</span>
            <strong>{{ selectedNode.node_type }}</strong>
          </div>

          <label class="property-group">
            <span>Label</span>
            <input class="input" v-model="selectedNode.label" />
          </label>

          <div v-for="field in getConfigFields(selectedNode.node_type)" :key="field" class="property-group">
            <span>{{ formatFieldName(field) }}</span>
            <textarea
              v-if="isLongField(field)"
              class="input"
              rows="4"
              :value="selectedNode.config[field]"
              @input="updateConfig(field, ($event.target as HTMLTextAreaElement).value)"
            ></textarea>
            <label v-else-if="field === 'value' && selectedNode.node_type === 'set_flag'" class="check-row">
              <input
                type="checkbox"
                :checked="selectedNode.config[field]"
                @change="updateConfig(field, ($event.target as HTMLInputElement).checked)"
              />
              <span>Enabled</span>
            </label>
            <input
              v-else
              class="input"
              :value="selectedNode.config[field]"
              @input="updateConfig(field, ($event.target as HTMLInputElement).value)"
            />
          </div>

          <button class="btn btn-danger" @click="deleteNode">Delete Node</button>
        </template>

        <div v-else class="empty-properties">
          <span class="empty-mark">--</span>
          <strong>No node selected</strong>
          <span>{{ nodes.length }} nodes on canvas</span>
        </div>

        <section class="validation-panel">
          <div class="panel-title">
            <span class="eyebrow">Validation</span>
            <button class="link-btn" @click="validateCurrentWorkflow">Run</button>
          </div>

          <div v-if="validationResult" class="validation-summary" :class="{ invalid: !validationResult.valid }">
            <strong>{{ validationResult.valid ? 'Ready to export' : 'Needs attention' }}</strong>
            <span>{{ validationResult.error_count }} errors · {{ validationResult.warning_count }} warnings</span>
          </div>

          <div v-if="validationMessage" class="validation-message">{{ validationMessage }}</div>

          <div v-if="validationResult?.issues.length" class="issue-list">
            <div v-for="(issue, index) in validationResult.issues" :key="`${issue.code}-${index}`" class="issue-item" :class="issue.severity">
              <span>{{ issue.severity }}</span>
              <strong>{{ issue.code }}</strong>
              <p>{{ issue.node_id ? `${issue.node_id}: ` : '' }}{{ issue.message }}</p>
            </div>
          </div>

          <p v-else-if="!validationResult" class="muted-copy">Run validation before saving or exporting workflows.</p>
        </section>
      </aside>
    </main>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '../lib/i18n'
import { computed, onMounted, ref } from 'vue'
import { invokeCommand } from '../lib/tauri'

const { t } = useI18n()

interface WorkflowNode {
  id: string
  node_type: string
  label: string
  x: number
  y: number
  config: Record<string, any>
  connections: string[]
}

interface Workflow {
  id: string
  name: string
  nodes: WorkflowNode[]
  start_node_id: string
}

interface NodeTypeInfo {
  node_type: string
  label: string
  description: string
  category: string
  configurable_fields: string[]
}

interface WorkflowValidationIssue {
  severity: string
  code: string
  node_id: string | null
  message: string
}

interface WorkflowValidationResult {
  valid: boolean
  error_count: number
  warning_count: number
  issues: WorkflowValidationIssue[]
}

const NODE_WIDTH = 214
const NODE_HEIGHT = 92

const workflow = ref<Workflow | null>(null)
const nodes = ref<WorkflowNode[]>([])
const selectedNode = ref<WorkflowNode | null>(null)
const nodeTypes = ref<NodeTypeInfo[]>([])
const canvasRef = ref<HTMLDivElement>()
const validationResult = ref<WorkflowValidationResult | null>(null)
const validationMessage = ref('')

const previewNodeTypes: NodeTypeInfo[] = [
  { node_type: 'start', label: 'Start', description: 'Workflow entry point', category: 'flow', configurable_fields: [] },
  { node_type: 'dialogue', label: 'Dialogue', description: 'Show character dialogue', category: 'content', configurable_fields: ['speaker_id', 'text'] },
  { node_type: 'choice', label: 'Choice', description: 'Present player choices', category: 'content', configurable_fields: ['choices'] },
  { node_type: 'condition', label: 'Condition', description: 'Branch by expression', category: 'flow', configurable_fields: ['condition'] },
  { node_type: 'llm_generate', label: 'LLM Generate', description: 'Generate text with the active model', category: 'ai', configurable_fields: ['prompt', 'system_prompt'] },
  { node_type: 'evaluation', label: 'Evaluation', description: 'Score conversation quality', category: 'ai', configurable_fields: ['criteria'] },
  { node_type: 'scene_change', label: 'Scene Change', description: 'Switch background scene', category: 'content', configurable_fields: ['scene_id'] },
  { node_type: 'relationship', label: 'Relationship', description: 'Modify relationship score', category: 'character', configurable_fields: ['character_id', 'delta'] },
  { node_type: 'end', label: 'End', description: 'Workflow exit', category: 'flow', configurable_fields: [] },
]

let nextNodeId = 1
let draggingNode: WorkflowNode | null = null
let connectingFrom: WorkflowNode | null = null
let dragOffset = { x: 0, y: 0 }

const nodeCategories = computed(() => {
  const categories: Record<string, NodeTypeInfo[]> = {}
  for (const nt of nodeTypes.value) {
    if (!categories[nt.category]) categories[nt.category] = []
    categories[nt.category].push(nt)
  }
  return Object.entries(categories).map(([name, nodes]) => ({
    name: name.charAt(0).toUpperCase() + name.slice(1),
    nodes,
  }))
})

const connections = computed(() => {
  const conns: { x1: number; y1: number; x2: number; y2: number }[] = []
  for (const node of nodes.value) {
    for (const targetId of node.connections) {
      const target = nodes.value.find((n) => n.id === targetId)
      if (target) {
        conns.push({
          x1: node.x + NODE_WIDTH,
          y1: node.y + NODE_HEIGHT / 2,
          x2: target.x,
          y2: target.y + NODE_HEIGHT / 2,
        })
      }
    }
  }
  return conns
})

const validationStatusLabel = computed(() => {
  if (!validationResult.value) return 'Not checked'
  if (validationResult.value.valid) return validationResult.value.warning_count > 0 ? 'Warnings' : 'Valid'
  return `${validationResult.value.error_count} errors`
})

const validationStatusClass = computed(() => {
  if (!validationResult.value) return 'neutral'
  if (validationResult.value.valid) return validationResult.value.warning_count > 0 ? 'warning' : 'valid'
  return 'invalid'
})

function getNodeIcon(type: string): string {
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
  return icons[type] || 'ND'
}

function connectionPath(conn: { x1: number; y1: number; x2: number; y2: number }): string {
  const mid = Math.max(60, Math.abs(conn.x2 - conn.x1) / 2)
  return `M ${conn.x1} ${conn.y1} C ${conn.x1 + mid} ${conn.y1}, ${conn.x2 - mid} ${conn.y2}, ${conn.x2} ${conn.y2}`
}

function getConfigFields(nodeType: string): string[] {
  const type = nodeTypes.value.find((t) => t.node_type === nodeType)
  return type?.configurable_fields || []
}

function isLongField(field: string): boolean {
  return field === 'text' || field === 'prompt' || field === 'system_prompt'
}

function formatFieldName(field: string): string {
  return field
    .split('_')
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ')
}

function updateConfig(field: string, value: any) {
  if (selectedNode.value) {
    selectedNode.value.config[field] = value
    markWorkflowDirty()
  }
}

function markWorkflowDirty() {
  validationResult.value = null
  validationMessage.value = ''
}

function syncWorkflowFromCanvas(): Workflow | null {
  if (!workflow.value) return null
  workflow.value.nodes = nodes.value
  if (!workflow.value.start_node_id || !nodes.value.some((node) => node.id === workflow.value?.start_node_id)) {
    workflow.value.start_node_id = nodes.value.find((node) => node.node_type === 'start')?.id || nodes.value[0]?.id || ''
  }
  return workflow.value
}

async function validateCurrentWorkflow(): Promise<WorkflowValidationResult | null> {
  const currentWorkflow = syncWorkflowFromCanvas()
  if (!currentWorkflow) return null

  validationMessage.value = ''
  try {
    const result = await invokeCommand<WorkflowValidationResult>(
      'validate_workflow',
      { workflow: currentWorkflow },
      () => validateWorkflowLocally(currentWorkflow)
    )
    validationResult.value = result
    return result
  } catch (e) {
    validationMessage.value = String(e)
    return null
  }
}

async function ensureWorkflowIsValid(): Promise<boolean> {
  const result = await validateCurrentWorkflow()
  if (!result?.valid) {
    validationMessage.value = 'Fix validation errors before saving or exporting.'
    return false
  }
  return true
}

function validateWorkflowLocally(currentWorkflow: Workflow): WorkflowValidationResult {
  const issues: WorkflowValidationIssue[] = []
  const ids = new Set<string>()
  const lookup = new Map<string, WorkflowNode>()
  const knownTypes = new Set(previewNodeTypes.map((type) => type.node_type).concat(['set_variable', 'set_flag', 'emotion_change', 'trigger_event']))
  const requiredFields: Record<string, string[]> = {
    dialogue: ['text'],
    choice: ['choices'],
    condition: ['condition'],
    set_variable: ['variable_name', 'value'],
    set_flag: ['flag_name', 'value'],
    llm_generate: ['prompt'],
    evaluation: ['criteria'],
    scene_change: ['scene_id'],
    trigger_event: ['event_id'],
    emotion_change: ['character_id', 'emotion'],
    relationship: ['character_id', 'delta'],
  }

  const addIssue = (severity: string, code: string, node_id: string | null, message: string) => {
    issues.push({ severity, code, node_id, message })
  }

  if (!currentWorkflow.id.trim()) addIssue('error', 'workflow_id_empty', null, 'Workflow id is required.')
  if (!currentWorkflow.name.trim()) addIssue('error', 'workflow_name_empty', null, 'Workflow name is required.')
  if (currentWorkflow.nodes.length === 0) addIssue('error', 'workflow_empty', null, 'Workflow must contain at least one node.')

  for (const node of currentWorkflow.nodes) {
    if (!node.id.trim()) {
      addIssue('error', 'node_id_empty', null, 'Every node must have a non-empty id.')
      continue
    }
    if (ids.has(node.id)) addIssue('error', 'node_id_duplicate', node.id, 'Node ids must be unique.')
    ids.add(node.id)
    lookup.set(node.id, node)
  }

  const startNodes = currentWorkflow.nodes.filter((node) => node.node_type === 'start')
  if (currentWorkflow.nodes.length > 0 && startNodes.length === 0) addIssue('error', 'start_node_missing', null, 'Workflow must include a start node.')
  if (startNodes.length > 1) addIssue('warning', 'start_node_multiple', null, 'Multiple start nodes found; only the configured start node is used.')
  if (currentWorkflow.nodes.length > 0 && !currentWorkflow.start_node_id.trim()) addIssue('error', 'start_node_id_empty', null, 'Workflow start_node_id is required.')
  const startNode = lookup.get(currentWorkflow.start_node_id)
  if (currentWorkflow.start_node_id && !startNode) addIssue('error', 'start_node_not_found', currentWorkflow.start_node_id, 'start_node_id does not match any node.')
  if (startNode && startNode.node_type !== 'start') addIssue('error', 'start_node_type_invalid', startNode.id, 'start_node_id must reference a start node.')
  if (currentWorkflow.nodes.length > 0 && !currentWorkflow.nodes.some((node) => node.node_type === 'end')) addIssue('warning', 'end_node_missing', null, 'Workflow has no end node.')

  for (const node of currentWorkflow.nodes) {
    if (!node.label.trim()) addIssue('warning', 'node_label_empty', node.id, 'Node label is empty.')
    if (!knownTypes.has(node.node_type)) {
      addIssue('error', 'node_type_unknown', node.id, `Unknown node type: ${node.node_type}`)
      continue
    }

    for (const field of requiredFields[node.node_type] || []) {
      if (!isConfigFieldPresent(node.config, field)) addIssue('error', 'node_config_missing', node.id, `Required field \`${field}\` is missing.`)
    }

    const localTargets = new Set<string>()
    for (const targetId of node.connections) {
      if (!targetId.trim()) {
        addIssue('error', 'connection_empty', node.id, 'Connection target id is empty.')
        continue
      }
      if (targetId === node.id) addIssue('error', 'connection_self', node.id, 'Node cannot connect to itself.')
      if (!ids.has(targetId)) addIssue('error', 'connection_target_missing', node.id, `Connection target \`${targetId}\` does not exist.`)
      if (localTargets.has(targetId)) addIssue('warning', 'connection_duplicate', node.id, `Duplicate connection to \`${targetId}\`.`)
      localTargets.add(targetId)
    }
  }

  if (startNode) {
    const visited = new Set<string>()
    const queue = [startNode.id]
    while (queue.length > 0) {
      const id = queue.shift()!
      if (visited.has(id)) continue
      visited.add(id)
      const node = lookup.get(id)
      if (node) {
        for (const targetId of node.connections) {
          if (lookup.has(targetId)) queue.push(targetId)
        }
      }
    }
    for (const node of currentWorkflow.nodes) {
      if (!visited.has(node.id)) addIssue('warning', 'node_unreachable', node.id, 'Node is not reachable from the configured start node.')
    }
  }

  const error_count = issues.filter((issue) => issue.severity === 'error').length
  const warning_count = issues.filter((issue) => issue.severity === 'warning').length
  return { valid: error_count === 0, error_count, warning_count, issues }
}

function isConfigFieldPresent(config: Record<string, any>, field: string): boolean {
  const value = config[field]
  if (value === null || value === undefined) return false
  if (typeof value === 'string') return value.trim().length > 0
  if (Array.isArray(value)) return value.length > 0
  if (typeof value === 'object') return Object.keys(value).length > 0
  return true
}

function createNode(type: NodeTypeInfo, x: number, y: number): WorkflowNode {
  return {
    id: `node_${nextNodeId++}`,
    node_type: type.node_type,
    label: type.label,
    x: Math.max(16, x - NODE_WIDTH / 2),
    y: Math.max(16, y - 28),
    config: {},
    connections: [],
  }
}

function selectNode(node: WorkflowNode) {
  selectedNode.value = node
}

function deleteNode() {
  if (!selectedNode.value) return
  const id = selectedNode.value.id
  nodes.value = nodes.value.filter((node) => node.id !== id)
  for (const node of nodes.value) {
    node.connections = node.connections.filter((targetId) => targetId !== id)
  }
  selectedNode.value = null
  markWorkflowDirty()
}

function onDragStart(event: DragEvent, nodeType: NodeTypeInfo) {
  event.dataTransfer?.setData('nodeType', JSON.stringify(nodeType))
}

function onDrop(event: DragEvent) {
  if (!event.dataTransfer || !canvasRef.value) return
  const rect = canvasRef.value.getBoundingClientRect()
  const x = event.clientX - rect.left
  const y = event.clientY - rect.top

  try {
    const nodeType = JSON.parse(event.dataTransfer.getData('nodeType')) as NodeTypeInfo
    const node = createNode(nodeType, x, y)
    nodes.value.push(node)
    selectNode(node)
    markWorkflowDirty()
  } catch (e) {
    console.error('Failed to create node:', e)
  }
}

function onNodeMouseDown(event: MouseEvent, node: WorkflowNode) {
  draggingNode = node
  dragOffset.x = event.clientX - node.x
  dragOffset.y = event.clientY - node.y

  const onMouseMove = (e: MouseEvent) => {
    if (!draggingNode || !canvasRef.value) return
    const rect = canvasRef.value.getBoundingClientRect()
    draggingNode.x = Math.max(0, e.clientX - rect.left - (dragOffset.x - rect.left))
    draggingNode.y = Math.max(0, e.clientY - rect.top - (dragOffset.y - rect.top))
    markWorkflowDirty()
  }

  const onMouseUp = () => {
    draggingNode = null
    window.removeEventListener('mousemove', onMouseMove)
    window.removeEventListener('mouseup', onMouseUp)
  }

  window.addEventListener('mousemove', onMouseMove)
  window.addEventListener('mouseup', onMouseUp)
}

function onCanvasMouseDown() {
  selectedNode.value = null
}

function getNodeAtClientPoint(event: MouseEvent): WorkflowNode | undefined {
  if (!canvasRef.value) return undefined
  const rect = canvasRef.value.getBoundingClientRect()
  const x = event.clientX - rect.left
  const y = event.clientY - rect.top
  return nodes.value.find((node) =>
    x >= node.x &&
    x <= node.x + NODE_WIDTH &&
    y >= node.y &&
    y <= node.y + NODE_HEIGHT
  )
}

function startConnection(event: MouseEvent, node: WorkflowNode) {
  event.preventDefault()
  connectingFrom = node

  const onMouseUp = (e: MouseEvent) => {
    const target = getNodeAtClientPoint(e)
    if (connectingFrom && target && target.id !== connectingFrom.id && !connectingFrom.connections.includes(target.id)) {
      connectingFrom.connections.push(target.id)
      markWorkflowDirty()
    }
    connectingFrom = null
    window.removeEventListener('mouseup', onMouseUp)
  }

  window.addEventListener('mouseup', onMouseUp)
}

async function newWorkflow() {
  workflow.value = {
    id: `wf_${Date.now()}`,
    name: 'New Workflow',
    nodes: [],
    start_node_id: '',
  }
  nodes.value = []
  selectedNode.value = null
  nextNodeId = 1
  markWorkflowDirty()
}

async function loadWorkflow() {
  try {
    const path = prompt('Enter workflow file path:')
    if (!path) return
    const wf = await invokeCommand<Workflow>('load_workflow', { path })
    workflow.value = wf
    nodes.value = wf.nodes
    selectedNode.value = null
    nextNodeId = wf.nodes.length + 1
    await validateCurrentWorkflow()
  } catch (e) {
    console.error('Failed to load workflow:', e)
  }
}

async function saveWorkflow() {
  if (!workflow.value) return
  if (!(await ensureWorkflowIsValid())) return
  syncWorkflowFromCanvas()
  try {
    const path = prompt('Enter save path:', 'workflow.json')
    if (!path) return
    await invokeCommand<void>('save_workflow', { workflow: workflow.value, path })
  } catch (e) {
    console.error('Failed to save workflow:', e)
  }
}

async function exportJSON() {
  if (!workflow.value) return
  if (!(await ensureWorkflowIsValid())) return
  syncWorkflowFromCanvas()
  const json = JSON.stringify(workflow.value, null, 2)
  const blob = new Blob([json], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `${workflow.value.name}.json`
  a.click()
  URL.revokeObjectURL(url)
}

onMounted(async () => {
  try {
    nodeTypes.value = await invokeCommand<NodeTypeInfo[]>('get_workflow_nodes', undefined, previewNodeTypes)
  } catch (e) {
    console.error('Failed to load node types:', e)
  }
  newWorkflow()
})
</script>

<style scoped>
.workflow-editor {
  height: 100vh;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: var(--surface-0);
}

.toolbar {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: center;
  padding: 14px 18px;
  border-bottom: 1px solid var(--border);
  background: var(--surface-1);
}

.toolbar-left {
  min-width: 0;
  display: flex;
  align-items: baseline;
  gap: 12px;
}

.toolbar-left h1 {
  color: var(--text-primary);
  font-size: 18px;
  line-height: 1.2;
}

.workflow-name {
  overflow: hidden;
  color: var(--text-tertiary);
  font-size: 13px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.validation-pill {
  display: inline-flex;
  align-items: center;
  min-height: 26px;
  padding: 3px 9px;
  border: 1px solid var(--border);
  border-radius: 999px;
  color: var(--text-tertiary);
  background: var(--surface-2);
  font-size: 11px;
  font-weight: 800;
}

.validation-pill.valid {
  color: var(--success);
  border-color: rgba(34,197,94,0.35);
}

.validation-pill.warning {
  color: var(--warning);
  border-color: rgba(245,158,11,0.38);
}

.validation-pill.invalid {
  color: var(--danger);
  border-color: rgba(239,68,68,0.4);
}

.eyebrow {
  color: var(--text-tertiary);
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0;
  text-transform: uppercase;
}

.editor-body {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 260px minmax(0, 1fr) 300px;
}

.node-palette,
.properties-panel {
  min-height: 0;
  overflow-y: auto;
  background: var(--surface-1);
}

.node-palette {
  border-right: 1px solid var(--border);
  padding: 14px;
}

.properties-panel {
  border-left: 1px solid var(--border);
  padding: 14px;
}

.panel-title {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: center;
  margin-bottom: 14px;
}

.panel-title strong {
  color: var(--brand-light);
  font-size: 12px;
}

.palette-category {
  margin-bottom: 18px;
}

.palette-category h2 {
  margin-bottom: 8px;
  color: var(--text-secondary);
  font-size: 12px;
}

.palette-node {
  width: 100%;
  display: grid;
  grid-template-columns: 34px minmax(0, 1fr);
  gap: 10px;
  align-items: center;
  margin-bottom: 6px;
  padding: 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-2);
  color: var(--text-primary);
  cursor: grab;
  text-align: left;
}

.palette-node:hover {
  border-color: var(--brand);
  background: var(--surface-3);
}

.node-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 28px;
  border-radius: var(--radius-sm);
  background: var(--surface-3);
  color: var(--brand-light);
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 900;
}

.palette-copy {
  min-width: 0;
  display: grid;
  gap: 2px;
}

.palette-copy strong {
  color: var(--text-primary);
  font-size: 13px;
}

.palette-copy small {
  overflow: hidden;
  color: var(--text-tertiary);
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.canvas {
  position: relative;
  overflow: hidden;
  background: var(--surface-0);
}

.canvas-grid,
.connections {
  position: absolute;
  inset: 0;
  pointer-events: none;
}

.workflow-node {
  position: absolute;
  z-index: 2;
  width: 214px;
  height: 92px;
  overflow: hidden;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-1);
  box-shadow: var(--shadow);
  cursor: move;
}

.workflow-node.selected {
  border-color: var(--brand);
  box-shadow: var(--shadow-brand), var(--shadow);
}

.node-header {
  display: flex;
  gap: 9px;
  align-items: center;
  min-height: 44px;
  padding: 9px 10px;
  border-bottom: 1px solid var(--border);
}

.node-header strong {
  overflow: hidden;
  color: var(--text-primary);
  font-size: 13px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.node-body {
  position: relative;
  display: flex;
  align-items: center;
  height: 47px;
  padding: 0 12px;
  color: var(--text-tertiary);
  font-size: 11px;
  font-family: var(--font-mono);
}

.node-port {
  position: absolute;
  right: -7px;
  top: 50%;
  width: 14px;
  height: 14px;
  border: 2px solid var(--surface-0);
  border-radius: 50%;
  background: var(--brand);
  cursor: crosshair;
  transform: translateY(-50%);
}

.property-group {
  display: grid;
  gap: 6px;
  margin-bottom: 14px;
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 700;
}

.check-row {
  display: flex;
  gap: 8px;
  align-items: center;
  color: var(--text-secondary);
}

.empty-properties {
  display: grid;
  place-items: center;
  align-content: center;
  gap: 8px;
  min-height: 60%;
  color: var(--text-tertiary);
  text-align: center;
}

.empty-properties strong {
  color: var(--text-primary);
}

.empty-mark {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 42px;
  height: 42px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-2);
  color: var(--brand-light);
  font-family: var(--font-mono);
  font-weight: 900;
}

.validation-panel {
  margin-top: 18px;
  padding-top: 14px;
  border-top: 1px solid var(--border);
}

.link-btn {
  border: none;
  background: transparent;
  color: var(--brand-light);
  cursor: pointer;
  font: inherit;
  font-size: 12px;
  font-weight: 800;
}

.validation-summary {
  display: grid;
  gap: 3px;
  padding: 12px;
  border: 1px solid rgba(34,197,94,0.28);
  border-radius: var(--radius);
  background: rgba(34,197,94,0.08);
}

.validation-summary.invalid {
  border-color: rgba(239,68,68,0.35);
  background: rgba(239,68,68,0.08);
}

.validation-summary strong {
  color: var(--text-primary);
  font-size: 13px;
}

.validation-summary span,
.muted-copy,
.validation-message {
  color: var(--text-tertiary);
  font-size: 12px;
}

.validation-message {
  margin-top: 10px;
  color: var(--warning);
}

.issue-list {
  display: grid;
  gap: 8px;
  margin-top: 12px;
}

.issue-item {
  display: grid;
  gap: 3px;
  padding: 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-2);
}

.issue-item span {
  color: var(--text-tertiary);
  font-size: 10px;
  font-weight: 900;
  text-transform: uppercase;
}

.issue-item.error span {
  color: var(--danger);
}

.issue-item.warning span {
  color: var(--warning);
}

.issue-item strong {
  color: var(--text-primary);
  font-size: 12px;
}

.issue-item p {
  color: var(--text-secondary);
  font-size: 12px;
  line-height: 1.35;
}

.node-type-start .node-icon { color: var(--success); }
.node-type-choice .node-icon,
.node-type-condition .node-icon { color: var(--warning); }
.node-type-llm_generate .node-icon,
.node-type-evaluation .node-icon { color: var(--info); }
.node-type-relationship .node-icon,
.node-type-emotion_change .node-icon { color: var(--narrative); }
.node-type-end .node-icon { color: var(--danger); }

@media (max-width: 1120px) {
  .editor-body {
    grid-template-columns: 220px minmax(0, 1fr);
  }

  .properties-panel {
    display: none;
  }
}

@media (max-width: 760px) {
  .toolbar {
    align-items: flex-start;
    flex-direction: column;
  }

  .editor-body {
    grid-template-columns: 1fr;
  }

  .node-palette {
    max-height: 220px;
    border-right: none;
    border-bottom: 1px solid var(--border);
  }
}
</style>

.node-type-narration { border-left: 3px solid #a78bfa; }
.node-type-bgm { border-left: 3px solid #f472b6; }
.node-type-sfx { border-left: 3px solid #fb923c; }
.node-type-wait { border-left: 3px solid #94a3b8; }
.node-type-random_branch { border-left: 3px solid #4ade80; }
.node-type-sub_workflow { border-left: 3px solid #60a5fa; }
.node-type-camera { border-left: 3px solid #38bdf8; }
.node-type-shake { border-left: 3px solid #fbbf24; }