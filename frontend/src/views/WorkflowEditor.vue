<template>
  <div class="workflow-editor">
    <!-- Toolbar -->
    <div class="toolbar">
      <div class="toolbar-left">
        <h2>🔧 Workflow Editor</h2>
        <span class="workflow-name">{{ workflow?.name || 'Untitled' }}</span>
      </div>
      <div class="toolbar-right">
        <button class="btn btn-secondary" @click="newWorkflow">New</button>
        <button class="btn btn-secondary" @click="loadWorkflow">Open</button>
        <button class="btn btn-primary" @click="saveWorkflow">Save</button>
        <button class="btn btn-primary" @click="exportJSON">Export</button>
      </div>
    </div>

    <div class="editor-body">
      <!-- Node Palette -->
      <div class="node-palette">
        <h3>Nodes</h3>
        <div v-for="category in nodeCategories" :key="category.name" class="palette-category">
          <h4>{{ category.name }}</h4>
          <div
            v-for="nodeType in category.nodes"
            :key="nodeType.node_type"
            class="palette-node"
            draggable="true"
            @dragstart="onDragStart($event, nodeType)"
          >
            <span class="node-icon">{{ getNodeIcon(nodeType.node_type) }}</span>
            <span>{{ nodeType.label }}</span>
          </div>
        </div>
      </div>

      <!-- Canvas -->
      <div
        class="canvas"
        ref="canvasRef"
        @drop="onDrop"
        @dragover.prevent
        @mousedown="onCanvasMouseDown"
      >
        <!-- Grid background -->
        <svg class="canvas-grid" width="100%" height="100%">
          <defs>
            <pattern id="grid" width="20" height="20" patternUnits="userSpaceOnUse">
              <path d="M 20 0 L 0 0 0 20" fill="none" stroke="#2a2a4a" stroke-width="0.5"/>
            </pattern>
          </defs>
          <rect width="100%" height="100%" fill="url(#grid)"/>
        </svg>

        <!-- Connections -->
        <svg class="connections" width="100%" height="100%">
          <line
            v-for="(conn, i) in connections"
            :key="i"
            :x1="conn.x1"
            :y1="conn.y1"
            :x2="conn.x2"
            :y2="conn.y2"
            stroke="#6c5ce7"
            stroke-width="2"
            stroke-dasharray="5,5"
          />
        </svg>

        <!-- Nodes -->
        <div
          v-for="node in nodes"
          :key="node.id"
          class="workflow-node"
          :class="{ selected: selectedNode?.id === node.id }"
          :style="{ left: node.x + 'px', top: node.y + 'px' }"
          @mousedown.stop="onNodeMouseDown($event, node)"
          @click.stop="selectNode(node)"
        >
          <div class="node-header" :class="'node-type-' + node.node_type">
            <span class="node-icon">{{ getNodeIcon(node.node_type) }}</span>
            <span>{{ node.label }}</span>
          </div>
          <div class="node-body">
            <div class="node-port output" @mousedown.stop="startConnection($event, node)"></div>
          </div>
        </div>
      </div>

      <!-- Properties Panel -->
      <div class="properties-panel" v-if="selectedNode">
        <h3>Properties</h3>
        <div class="property-group">
          <label>Label</label>
          <input class="input" v-model="selectedNode.label" />
        </div>
        <div class="property-group">
          <label>Type</label>
          <span class="type-badge">{{ selectedNode.node_type }}</span>
        </div>

        <!-- Dynamic config fields -->
        <div v-for="field in getConfigFields(selectedNode.node_type)" :key="field" class="property-group">
          <label>{{ formatFieldName(field) }}</label>
          <textarea
            v-if="field === 'text' || field === 'prompt' || field === 'system_prompt'"
            class="input"
            rows="3"
            :value="selectedNode.config[field]"
            @input="updateConfig(field, ($event.target as HTMLTextAreaElement).value)"
          ></textarea>
          <input
            v-else-if="field === 'value' && selectedNode.node_type === 'set_flag'"
            type="checkbox"
            :checked="selectedNode.config[field]"
            @change="updateConfig(field, ($event.target as HTMLInputElement).checked)"
          />
          <input
            v-else
            class="input"
            :value="selectedNode.config[field]"
            @input="updateConfig(field, ($event.target as HTMLInputElement).value)"
          />
        </div>

        <button class="btn btn-danger" @click="deleteNode" style="margin-top: 20px">
          Delete Node
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

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

const workflow = ref<Workflow | null>(null)
const nodes = ref<WorkflowNode[]>([])
const selectedNode = ref<WorkflowNode | null>(null)
const nodeTypes = ref<NodeTypeInfo[]>([])
const canvasRef = ref<HTMLDivElement>()

let nextNodeId = 1
let draggingNode: WorkflowNode | null = null
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
          x1: node.x + 100,
          y1: node.y + 40,
          x2: target.x + 100,
          y2: target.y,
        })
      }
    }
  }
  return conns
})

function getNodeIcon(type: string): string {
  const icons: Record<string, string> = {
    start: '▶️',
    dialogue: '💬',
    choice: '🔀',
    condition: '❓',
    set_variable: '📝',
    set_flag: '🚩',
    llm_generate: '🤖',
    emotion_change: '😊',
    relationship: '❤️',
    end: '⏹️',
  }
  return icons[type] || '📦'
}

function getConfigFields(nodeType: string): string[] {
  const type = nodeTypes.value.find((t) => t.node_type === nodeType)
  return type?.configurable_fields || []
}

function formatFieldName(field: string): string {
  return field
    .split('_')
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(' ')
}

function updateConfig(field: string, value: any) {
  if (selectedNode.value) {
    selectedNode.value.config[field] = value
  }
}

function createNode(type: NodeTypeInfo, x: number, y: number): WorkflowNode {
  return {
    id: `node_${nextNodeId++}`,
    node_type: type.node_type,
    label: type.label,
    x,
    y,
    config: {},
    connections: [],
  }
}

function selectNode(node: WorkflowNode) {
  selectedNode.value = node
}

function deleteNode() {
  if (selectedNode.value) {
    nodes.value = nodes.value.filter((n) => n.id !== selectedNode.value!.id)
    // Remove connections to this node
    for (const node of nodes.value) {
      node.connections = node.connections.filter((id) => id !== selectedNode.value!.id)
    }
    selectedNode.value = null
  }
}

function onDragStart(event: DragEvent, nodeType: NodeTypeInfo) {
  if (event.dataTransfer) {
    event.dataTransfer.setData('nodeType', JSON.stringify(nodeType))
  }
}

function onDrop(event: DragEvent) {
  if (!event.dataTransfer || !canvasRef.value) return
  const rect = canvasRef.value.getBoundingClientRect()
  const x = event.clientX - rect.left
  const y = event.clientY - rect.top

  try {
    const nodeType = JSON.parse(event.dataTransfer.getData('nodeType'))
    const node = createNode(nodeType, x, y)
    nodes.value.push(node)
    selectNode(node)
  } catch (e) {
    console.error('Failed to create node:', e)
  }
}

function onNodeMouseDown(event: MouseEvent, node: WorkflowNode) {
  draggingNode = node
  dragOffset.x = event.clientX - node.x
  dragOffset.y = event.clientY - node.y

  const onMouseMove = (e: MouseEvent) => {
    if (draggingNode) {
      draggingNode.x = e.clientX - dragOffset.x
      draggingNode.y = e.clientY - dragOffset.y
    }
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

let connectingFrom: WorkflowNode | null = null

function startConnection(event: MouseEvent, node: WorkflowNode) {
  connectingFrom = node
  const onMouseUp = (e: MouseEvent) => {
    // Find target node under cursor
    const target = nodes.value.find((n) => {
      return n.id !== connectingFrom?.id // Simplified hit test
    })
    if (connectingFrom && target && !connectingFrom.connections.includes(target.id)) {
      connectingFrom.connections.push(target.id)
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
}

async function loadWorkflow() {
  // In a real implementation, this would open a file dialog
  try {
    const path = prompt('Enter workflow file path:')
    if (path) {
      const wf = await invoke<Workflow>('load_workflow', { path })
      workflow.value = wf
      nodes.value = wf.nodes
    }
  } catch (e) {
    console.error('Failed to load workflow:', e)
  }
}

async function saveWorkflow() {
  if (!workflow.value) return
  workflow.value.nodes = nodes.value
  if (nodes.value.length > 0) {
    workflow.value.start_node_id = nodes.value[0].id
  }
  try {
    const path = prompt('Enter save path:', 'workflow.json')
    if (path) {
      await invoke('save_workflow', { workflow: workflow.value, path })
      alert('Workflow saved!')
    }
  } catch (e) {
    console.error('Failed to save workflow:', e)
  }
}

function exportJSON() {
  if (!workflow.value) return
  workflow.value.nodes = nodes.value
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
    nodeTypes.value = await invoke('get_workflow_nodes')
  } catch (e) {
    console.error('Failed to load node types:', e)
  }
  newWorkflow()
})
</script>

<style scoped>
.workflow-editor {
  height: 100vh;
  display: flex;
  flex-direction: column;
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 20px;
  background: var(--bg-card);
  border-bottom: 1px solid var(--border);
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 15px;
}

.toolbar-left h2 {
  font-size: 18px;
  color: var(--primary);
}

.workflow-name {
  color: var(--text-muted);
  font-size: 14px;
}

.toolbar-right {
  display: flex;
  gap: 10px;
}

.editor-body {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.node-palette {
  width: 200px;
  background: var(--bg-card);
  border-right: 1px solid var(--border);
  padding: 15px;
  overflow-y: auto;
}

.node-palette h3 {
  margin-bottom: 15px;
  color: var(--primary);
  font-size: 14px;
}

.palette-category {
  margin-bottom: 15px;
}

.palette-category h4 {
  font-size: 12px;
  color: var(--text-muted);
  margin-bottom: 8px;
  text-transform: uppercase;
}

.palette-node {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  background: var(--bg-input);
  border-radius: var(--radius);
  margin-bottom: 4px;
  cursor: grab;
  font-size: 13px;
  transition: background 0.2s;
}

.palette-node:hover {
  background: var(--primary);
}

.canvas {
  flex: 1;
  position: relative;
  overflow: hidden;
  background: var(--bg-dark);
}

.canvas-grid, .connections {
  position: absolute;
  top: 0;
  left: 0;
}

.workflow-node {
  position: absolute;
  width: 200px;
  background: var(--bg-card);
  border: 2px solid var(--border);
  border-radius: var(--radius);
  cursor: move;
  z-index: 10;
  transition: border-color 0.2s;
}

.workflow-node.selected {
  border-color: var(--primary);
  box-shadow: 0 0 10px rgba(108, 92, 231, 0.3);
}

.node-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: 6px 6px 0 0;
  font-size: 13px;
  font-weight: 500;
}

.node-type-start { background: #00b894; }
.node-type-dialogue { background: #6c5ce7; }
.node-type-choice { background: #e17055; }
.node-type-condition { background: #fdcb6e; color: #333; }
.node-type-set_variable, .node-type-set_flag { background: #0984e3; }
.node-type-llm_generate { background: #e84393; }
.node-type-emotion_change, .node-type-relationship { background: #00cec9; }
.node-type-end { background: #636e72; }

.node-body {
  padding: 10px 12px;
  min-height: 30px;
  position: relative;
}

.node-port {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: var(--primary);
  position: absolute;
  cursor: crosshair;
}

.node-port.output {
  right: -6px;
  top: 50%;
  transform: translateY(-50%);
}

.properties-panel {
  width: 280px;
  background: var(--bg-card);
  border-left: 1px solid var(--border);
  padding: 15px;
  overflow-y: auto;
}

.properties-panel h3 {
  margin-bottom: 15px;
  color: var(--primary);
  font-size: 14px;
}

.property-group {
  margin-bottom: 15px;
}

.property-group label {
  display: block;
  font-size: 12px;
  color: var(--text-muted);
  margin-bottom: 5px;
}

.type-badge {
  display: inline-block;
  padding: 4px 8px;
  background: var(--bg-input);
  border-radius: 4px;
  font-size: 12px;
  color: var(--secondary);
}
</style>
