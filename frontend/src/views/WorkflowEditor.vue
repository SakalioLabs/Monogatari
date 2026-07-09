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
        <button class="btn btn-secondary btn-sm" @click="runCurrentWorkflow" :disabled="runningWorkflow">
          {{ runningWorkflow ? 'Running' : 'Run' }}
        </button>
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
          :class="[{ selected: selectedNode?.id === node.id }, nodeRunClass(node), 'node-type-' + node.node_type]"
          :style="{ left: node.x + 'px', top: node.y + 'px' }"
          @mousedown.stop="onNodeMouseDown($event, node)"
          @click.stop="selectNode(node)"
        >
          <header class="node-header">
            <span class="node-icon">{{ getNodeIcon(node.node_type) }}</span>
            <strong>{{ node.label }}</strong>
            <span
              v-if="nodeRunBadge(node)"
              class="node-run-badge"
              :class="nodeRunOutcome(node)"
              :title="nodeRunTooltip(node)"
            >
              {{ nodeRunBadge(node) }}
            </span>
          </header>
          <div class="node-body">
            <span>{{ node.node_type }}</span>
            <small v-if="nodeRunDetail(node)" class="node-run-detail">{{ nodeRunDetail(node) }}</small>
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
            <span>{{ validationResult.error_count }} errors - {{ validationResult.warning_count }} warnings</span>
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

        <section class="execution-panel">
          <div class="panel-title">
            <span class="eyebrow">Execution</span>
            <button class="link-btn" @click="runCurrentWorkflow" :disabled="runningWorkflow">
              {{ runningWorkflow ? 'Running' : 'Run' }}
            </button>
          </div>

          <div class="run-context-panel" :class="{ enabled: runContext.enabled }">
            <label class="run-context-toggle">
              <input v-model="runContext.enabled" type="checkbox" />
              <span>Preview Context</span>
            </label>
            <div class="run-context-presets">
              <button
                v-for="preset in runContextPresets"
                :key="preset.id"
                class="context-preset-btn"
                type="button"
                @click="applyRunContextPreset(preset)"
              >
                {{ preset.label }}
              </button>
              <button
                class="context-preset-btn matrix"
                type="button"
                :disabled="runningWorkflow"
                @click="runPresetMatrix"
              >
                Run Matrix
              </button>
            </div>
            <div class="run-context-grid">
              <label>
                <span>Character</span>
                <input class="input" v-model="runContext.character_id" />
              </label>
              <label>
                <span>Eval Count</span>
                <input class="input" v-model.number="runContext.evaluation_count" type="number" min="0" step="1" />
              </label>
              <label>
                <span>Relationship</span>
                <input class="input" v-model.number="runContext.relationship" type="number" min="-1" max="1" step="0.05" />
              </label>
              <label>
                <span>Friendliness</span>
                <input class="input" v-model.number="runContext.friendliness" type="number" min="0" max="1" step="0.05" />
              </label>
              <label>
                <span>Engagement</span>
                <input class="input" v-model.number="runContext.engagement" type="number" min="0" max="1" step="0.05" />
              </label>
              <label>
                <span>Creativity</span>
                <input class="input" v-model.number="runContext.creativity" type="number" min="0" max="1" step="0.05" />
              </label>
              <label>
                <span>Overall</span>
                <input class="input" v-model.number="runContext.overall_score" type="number" min="0" max="1" step="0.05" />
              </label>
              <label class="run-context-wide">
                <span>Already Triggered</span>
                <input class="input" v-model="runContext.already_triggered_events" placeholder="high_engagement" />
              </label>
            </div>
          </div>

          <div v-if="executionReport" class="execution-summary" :class="{ complete: executionReport.completed }">
            <strong>{{ executionReport.completed ? 'Completed' : 'Stopped' }}</strong>
            <span>{{ executionReport.steps.length }} steps - {{ executionReport.stopped_reason || 'ready' }}</span>
            <div class="coverage-row">
              <span>Coverage</span>
              <strong>{{ formatCoverage(executionReport.coverage_percent) }}</strong>
              <small>{{ executionReport.executed_node_count }}/{{ executionReport.node_count }} nodes</small>
            </div>
            <div v-if="executionReport.unvisited_node_ids.length" class="unvisited-node-list">
              <span v-for="nodeId in executionReport.unvisited_node_ids" :key="nodeId">{{ nodeId }}</span>
            </div>
          </div>

          <div v-if="presetMatrixReport" class="matrix-coverage-panel" :class="{ complete: presetMatrixReport.unvisited_node_ids.length === 0 }">
            <strong>Preset Matrix</strong>
            <span>{{ formatCoverage(presetMatrixReport.coverage_percent) }} - {{ presetMatrixReport.executed_node_count }}/{{ presetMatrixReport.node_count }} nodes</span>
            <div class="matrix-run-list">
              <span v-for="run in presetMatrixReport.runs" :key="run.preset_id">
                {{ run.label }} {{ formatCoverage(run.coverage_percent) }}
              </span>
            </div>
            <div v-if="presetMatrixReport.unvisited_node_ids.length" class="unvisited-node-list">
              <span v-for="nodeId in presetMatrixReport.unvisited_node_ids" :key="nodeId">{{ nodeId }}</span>
            </div>
          </div>

          <div v-if="executionMessage" class="validation-message">{{ executionMessage }}</div>

          <div v-if="executionReport?.steps.length" class="trace-list">
            <div
              v-for="step in executionReport.steps"
              :key="`${step.step_index}-${step.node_id}`"
              class="trace-item"
              :class="{
                'trace-score': isEvaluationStep(step),
                'trace-event': isTriggerEventStep(step),
                triggered: isTriggerEventTriggered(step),
              }"
            >
              <span>{{ step.step_index + 1 }}</span>
              <strong>{{ step.label || step.node_id }}</strong>
              <small>{{ step.node_type }}{{ step.next_node_id ? ` -> ${step.next_node_id}` : '' }}</small>
              <em v-if="step.stopped_reason">{{ step.stopped_reason }}</em>
              <div v-if="isEvaluationStep(step)" class="trace-diagnostics score-diagnostics">
                <div class="diagnostic-row">
                  <span>Metric</span>
                  <strong>{{ stringValue(step.output.metric, 'overall') }}</strong>
                </div>
                <div class="diagnostic-row">
                  <span>Score</span>
                  <strong>{{ formatScore(step.output.score) }}</strong>
                </div>
                <div class="score-meter" aria-hidden="true">
                  <i :style="{ width: scorePercent(step.output.score) }"></i>
                </div>
                <div class="diagnostic-row">
                  <span>Threshold</span>
                  <strong>{{ formatThreshold(step.output.threshold) }}</strong>
                </div>
                <div class="diagnostic-row">
                  <span>Source</span>
                  <strong>{{ stringValue(step.output.source, 'unknown') }}</strong>
                </div>
                <span class="gate-pill" :class="{ pass: step.output.passed === true, fail: step.output.passed === false }">
                  {{ step.output.passed === true ? 'Pass' : step.output.passed === false ? 'Fail' : 'No threshold' }}
                </span>
              </div>
              <div v-if="isTriggerEventStep(step)" class="trace-diagnostics event-diagnostics">
                <div class="event-state-row">
                  <span class="event-status" :class="{ active: isTriggerEventTriggered(step) }">
                    {{ isTriggerEventTriggered(step) ? 'Triggered' : 'Blocked' }}
                  </span>
                  <strong>{{ stringValue(step.output.event_id, 'event') }}</strong>
                </div>
                <div class="diagnostic-row">
                  <span>Type</span>
                  <strong>{{ stringValue(step.output.event_type, 'story_event') }}</strong>
                </div>
                <div class="diagnostic-row">
                  <span>Metric</span>
                  <strong>{{ eventMetric(step) }}</strong>
                </div>
                <div class="diagnostic-row">
                  <span>Actual</span>
                  <strong>{{ formatScore(eventActualScore(step)) }}</strong>
                </div>
                <div v-if="eventBlockers(step).length" class="blocker-list">
                  <span v-for="reason in eventBlockers(step)" :key="reason">{{ reason }}</span>
                </div>
              </div>
              <div v-if="canChooseStep(step)" class="choice-debug">
                <button
                  v-for="(choice, index) in choiceOptionsFor(step)"
                  :key="`${step.node_id}-${index}`"
                  class="choice-debug-btn"
                  @click="chooseWorkflowOption(step.node_id, index)"
                >
                  {{ choice }}
                </button>
              </div>
            </div>
          </div>

          <p v-else class="muted-copy">Run workflow to inspect node trace.</p>
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

interface WorkflowExecutionStep {
  step_index: number
  node_id: string
  node_type: string
  label: string
  output: Record<string, any>
  next_node_id: string | null
  stopped_reason: string | null
}

interface WorkflowExecutionReport {
  workflow_id: string
  workflow_name: string
  completed: boolean
  stopped_reason: string | null
  node_count: number
  executed_node_count: number
  coverage_percent: number
  executed_node_ids: string[]
  unvisited_node_ids: string[]
  steps: WorkflowExecutionStep[]
  validation: WorkflowValidationResult
}

interface WorkflowRunContextForm {
  enabled: boolean
  character_id: string
  friendliness: number
  engagement: number
  creativity: number
  overall_score: number
  relationship: number
  evaluation_count: number
  already_triggered_events: string
}

interface WorkflowRunContextPayload {
  enabled: boolean
  character_id: string | null
  relationship: number
  evaluation_count: number
  already_triggered_events: string[]
  evaluation: {
    friendliness: number
    engagement: number
    creativity: number
    overall_score: number
    summary: string
  }
}

interface WorkflowRunContextPreset {
  id: string
  label: string
  values: Omit<WorkflowRunContextForm, 'enabled'>
}

interface WorkflowPresetMatrixReport {
  node_count: number
  executed_node_count: number
  coverage_percent: number
  executed_node_ids: string[]
  unvisited_node_ids: string[]
  runs: {
    preset_id: string
    label: string
    coverage_percent: number
    executed_node_count: number
    unvisited_node_ids: string[]
  }[]
}

type LocalConditionValue = number | string | boolean
interface LocalWorkflowState {
  variables: Record<string, LocalConditionValue>
  flags: Record<string, boolean>
  relationships: Record<string, Record<string, number>>
  emotions: Record<string, string>
}
interface LocalConditionScope {
  context: Record<string, LocalConditionValue>
  variables: Record<string, LocalConditionValue>
  flags: Record<string, boolean>
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
const executionReport = ref<WorkflowExecutionReport | null>(null)
const executionMessage = ref('')
const runningWorkflow = ref(false)
const choiceSelections = ref<Record<string, number>>({})
const presetMatrixReport = ref<WorkflowPresetMatrixReport | null>(null)
const runContext = ref<WorkflowRunContextForm>({
  enabled: false,
  character_id: 'sakura',
  friendliness: 0.65,
  engagement: 0.85,
  creativity: 0.6,
  overall_score: 0.7,
  relationship: 0,
  evaluation_count: 2,
  already_triggered_events: '',
})

const runContextPresets: WorkflowRunContextPreset[] = [
  {
    id: 'unlock',
    label: 'Unlock',
    values: {
      character_id: 'sakura',
      friendliness: 0.72,
      engagement: 0.9,
      creativity: 0.62,
      overall_score: 0.75,
      relationship: 0.2,
      evaluation_count: 2,
      already_triggered_events: '',
    },
  },
  {
    id: 'low-score',
    label: 'Low Score',
    values: {
      character_id: 'sakura',
      friendliness: 0.45,
      engagement: 0.45,
      creativity: 0.35,
      overall_score: 0.42,
      relationship: 0,
      evaluation_count: 2,
      already_triggered_events: '',
    },
  },
  {
    id: 'repeat-block',
    label: 'Repeat Block',
    values: {
      character_id: 'sakura',
      friendliness: 0.72,
      engagement: 0.92,
      creativity: 0.65,
      overall_score: 0.76,
      relationship: 0.2,
      evaluation_count: 2,
      already_triggered_events: 'high_engagement',
    },
  },
]

const previewNodeTypes: NodeTypeInfo[] = [
  { node_type: 'start', label: 'Start', description: 'Workflow entry point', category: 'flow', configurable_fields: [] },
  { node_type: 'dialogue', label: 'Dialogue', description: 'Show character dialogue', category: 'content', configurable_fields: ['speaker_id', 'text'] },
  { node_type: 'choice', label: 'Choice', description: 'Present player choices', category: 'content', configurable_fields: ['choices'] },
  { node_type: 'condition', label: 'Condition', description: 'Branch by expression', category: 'flow', configurable_fields: ['condition'] },
  { node_type: 'set_variable', label: 'Set Variable', description: 'Set a game variable', category: 'logic', configurable_fields: ['variable_name', 'value'] },
  { node_type: 'set_flag', label: 'Set Flag', description: 'Set a game flag', category: 'logic', configurable_fields: ['flag_name', 'value'] },
  { node_type: 'llm_generate', label: 'LLM Generate', description: 'Generate text with the active model', category: 'ai', configurable_fields: ['prompt', 'system_prompt'] },
  { node_type: 'evaluation', label: 'Evaluation', description: 'Read latest conversation score', category: 'ai', configurable_fields: ['character_id', 'criteria', 'threshold', 'variable_name'] },
  { node_type: 'trigger_event', label: 'Trigger Event', description: 'Trigger score-aware story event', category: 'flow', configurable_fields: ['character_id', 'event_id', 'event_type'] },
  { node_type: 'scene_change', label: 'Scene Change', description: 'Switch background scene', category: 'content', configurable_fields: ['scene_id'] },
  { node_type: 'emotion_change', label: 'Change Emotion', description: 'Change character emotion', category: 'character', configurable_fields: ['character_id', 'emotion'] },
  { node_type: 'relationship', label: 'Relationship', description: 'Modify relationship score', category: 'character', configurable_fields: ['character_id', 'delta'] },
  { node_type: 'narration', label: 'Narration', description: 'Display narrator text', category: 'content', configurable_fields: ['text', 'speaker'] },
  { node_type: 'bgm', label: 'BGM', description: 'Control background music', category: 'media', configurable_fields: ['track_path', 'action', 'volume'] },
  { node_type: 'sfx', label: 'SFX', description: 'Play a sound effect', category: 'media', configurable_fields: ['sound_path', 'volume'] },
  { node_type: 'wait', label: 'Wait', description: 'Pause workflow execution', category: 'flow', configurable_fields: ['duration_ms'] },
  { node_type: 'random_branch', label: 'Random Branch', description: 'Randomly pick a branch', category: 'flow', configurable_fields: ['weights'] },
  { node_type: 'sub_workflow', label: 'Sub Workflow', description: 'Delegate to another workflow', category: 'flow', configurable_fields: ['workflow_id', 'workflow_path'] },
  { node_type: 'camera', label: 'Camera', description: 'Control camera motion', category: 'media', configurable_fields: ['action', 'target_x', 'target_y', 'zoom', 'duration_ms'] },
  { node_type: 'shake', label: 'Shake', description: 'Screen shake effect', category: 'media', configurable_fields: ['intensity', 'duration_ms'] },
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

const executionStepsByNode = computed(() => {
  const map = new Map<string, WorkflowExecutionStep>()
  for (const step of executionReport.value?.steps || []) {
    map.set(step.node_id, step)
  }
  return map
})

const lastExecutionStep = computed(() => {
  const steps = executionReport.value?.steps || []
  return steps.length ? steps[steps.length - 1] : null
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
  executionReport.value = null
  executionMessage.value = ''
  presetMatrixReport.value = null
  choiceSelections.value = {}
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

async function runCurrentWorkflow() {
  choiceSelections.value = {}
  await executeWorkflowWithSelections()
}

async function chooseWorkflowOption(nodeId: string, index: number) {
  choiceSelections.value = { ...choiceSelections.value, [nodeId]: index }
  await executeWorkflowWithSelections()
}

async function executeWorkflowWithSelections() {
  const currentWorkflow = syncWorkflowFromCanvas()
  if (!currentWorkflow) return

  executionMessage.value = ''
  const validation = await validateCurrentWorkflow()
  if (!validation?.valid) {
    executionMessage.value = 'Fix validation errors before running.'
    return
  }

  runningWorkflow.value = true
  try {
    const runContextPayload = workflowRunContextPayload()
    executionReport.value = await invokeCommand<WorkflowExecutionReport>(
      'execute_workflow',
      { workflow: currentWorkflow, maxSteps: 64, choiceSelections: choiceSelections.value, runContext: runContextPayload },
      () => runWorkflowLocally(currentWorkflow, 64, choiceSelections.value, runContextPayload)
    )
  } catch (e) {
    executionMessage.value = String(e)
  } finally {
    runningWorkflow.value = false
  }
}

async function runPresetMatrix() {
  const currentWorkflow = syncWorkflowFromCanvas()
  if (!currentWorkflow) return

  choiceSelections.value = {}
  executionMessage.value = ''
  const validation = await validateCurrentWorkflow()
  if (!validation?.valid) {
    executionMessage.value = 'Fix validation errors before running.'
    return
  }

  runningWorkflow.value = true
  try {
    const matrixRuns = []
    for (const preset of runContextPresets) {
      const runContextPayload = workflowRunContextPayloadFromValues(preset.values)
      const report = await invokeCommand<WorkflowExecutionReport>(
        'execute_workflow',
        { workflow: currentWorkflow, maxSteps: 64, choiceSelections: {}, runContext: runContextPayload },
        () => runWorkflowLocally(currentWorkflow, 64, {}, runContextPayload)
      )
      matrixRuns.push({ preset, report })
    }
    executionReport.value = matrixRuns[matrixRuns.length - 1]?.report || null
    presetMatrixReport.value = aggregatePresetMatrixCoverage(currentWorkflow, matrixRuns)
  } catch (e) {
    executionMessage.value = String(e)
  } finally {
    runningWorkflow.value = false
  }
}

function canChooseStep(step: WorkflowExecutionStep): boolean {
  return step.node_type === 'choice' && step.stopped_reason === 'awaiting_choice' && choiceOptionsFor(step).length > 0
}

function choiceOptionsFor(step: WorkflowExecutionStep): string[] {
  const choices = step.output?.choices
  return Array.isArray(choices) ? choices.map(String) : []
}

function isEvaluationStep(step: WorkflowExecutionStep): boolean {
  return step.node_type === 'evaluation'
}

function isTriggerEventStep(step: WorkflowExecutionStep): boolean {
  return step.node_type === 'trigger_event'
}

function isTriggerEventTriggered(step: WorkflowExecutionStep): boolean {
  return isTriggerEventStep(step) && step.output?.triggered === true
}

function stringValue(value: any, fallback = '-'): string {
  if (value === null || value === undefined) return fallback
  const text = String(value).trim()
  return text || fallback
}

function numericValue(value: any): number | null {
  const number = Number(value)
  return Number.isFinite(number) ? number : null
}

function formatScore(value: any): string {
  const number = numericValue(value)
  return number === null ? '-' : number.toFixed(2)
}

function formatThreshold(value: any): string {
  if (value === null || value === undefined) return 'None'
  return formatScore(value)
}

function formatCoverage(value: any): string {
  const number = numericValue(value)
  return number === null ? '0%' : `${number.toFixed(0)}%`
}

function scorePercent(value: any): string {
  const number = numericValue(value)
  if (number === null) return '0%'
  return `${Math.round(Math.min(1, Math.max(0, number)) * 100)}%`
}

function eventDecision(step: WorkflowExecutionStep): Record<string, any> {
  const decision = step.output?.decision
  return decision && typeof decision === 'object' && !Array.isArray(decision) ? decision : {}
}

function eventBlockers(step: WorkflowExecutionStep): string[] {
  const reasons = eventDecision(step).blocked_reasons
  return Array.isArray(reasons) ? reasons.map(String).filter(Boolean) : []
}

function eventMetric(step: WorkflowExecutionStep): string {
  const decision = eventDecision(step)
  return stringValue(decision.actual_score_metric ?? decision.rule?.score_metric, '-')
}

function eventActualScore(step: WorkflowExecutionStep): number | null {
  return numericValue(eventDecision(step).actual_score)
}

function clampScore(value: any): number {
  const number = numericValue(value)
  return number === null ? 0 : Math.min(1, Math.max(0, number))
}

function clampRelationship(value: any): number {
  const number = numericValue(value)
  return number === null ? 0 : Math.min(1, Math.max(-1, number))
}

function workflowRunContextPayload(): WorkflowRunContextPayload | null {
  if (!runContext.value.enabled) return null
  return workflowRunContextPayloadFromValues(runContext.value)
}

function workflowRunContextPayloadFromValues(values: Omit<WorkflowRunContextForm, 'enabled'> | WorkflowRunContextForm): WorkflowRunContextPayload {
  return {
    enabled: true,
    character_id: values.character_id.trim() || null,
    relationship: clampRelationship(values.relationship),
    evaluation_count: Math.max(0, Math.round(numericValue(values.evaluation_count) ?? 0)),
    already_triggered_events: values.already_triggered_events
      .split(/[,\n]/)
      .map((eventId) => eventId.trim())
      .filter(Boolean),
    evaluation: {
      friendliness: clampScore(values.friendliness),
      engagement: clampScore(values.engagement),
      creativity: clampScore(values.creativity),
      overall_score: clampScore(values.overall_score),
      summary: 'Workflow author preview context.',
    },
  }
}

function localConditionScope(
  context: WorkflowRunContextPayload | null,
  localState: LocalWorkflowState = createLocalWorkflowState(),
  config: Record<string, any> = {}
): LocalConditionScope {
  const evaluation = context?.evaluation
  const characterId = String(config.character_id || config.speaker_id || config.speaker || context?.character_id || '').trim()
  const relationship = localRelationshipValue(
    localState,
    characterId,
    'player',
    localRelationshipFallback(context, characterId, 'player')
  )
  const evaluationCount = context?.evaluation_count ?? 0
  const friendliness = evaluation?.friendliness ?? 0.5
  const engagement = evaluation?.engagement ?? 0.5
  const creativity = evaluation?.creativity ?? 0.5
  const overall = evaluation?.overall_score ?? 0.5
  return {
    context: {
      character_id: characterId,
      relationship,
      relationship_score: relationship,
      evaluation_count: evaluationCount,
      friendliness,
      friendliness_score: friendliness,
      engagement,
      engagement_score: engagement,
      creativity,
      creativity_score: creativity,
      overall,
      overall_score: overall,
      evaluation_source: context?.enabled ? 'run_context_evaluation' : 'local_preview',
    },
    variables: localState.variables,
    flags: localState.flags,
  }
}

function aggregatePresetMatrixCoverage(
  currentWorkflow: Workflow,
  matrixRuns: { preset: WorkflowRunContextPreset; report: WorkflowExecutionReport }[]
): WorkflowPresetMatrixReport {
  const seen = new Set<string>()
  for (const { report } of matrixRuns) {
    for (const nodeId of report.executed_node_ids) seen.add(nodeId)
  }
  const executed_node_ids = currentWorkflow.nodes
    .map((node) => node.id)
    .filter((nodeId) => seen.has(nodeId))
  const unvisited_node_ids = currentWorkflow.nodes
    .map((node) => node.id)
    .filter((nodeId) => !seen.has(nodeId))
  const node_count = currentWorkflow.nodes.length
  const executed_node_count = executed_node_ids.length
  const coverage_percent = node_count > 0 ? (executed_node_count / node_count) * 100 : 0
  return {
    node_count,
    executed_node_count,
    coverage_percent,
    executed_node_ids,
    unvisited_node_ids,
    runs: matrixRuns.map(({ preset, report }) => ({
      preset_id: preset.id,
      label: preset.label,
      coverage_percent: report.coverage_percent,
      executed_node_count: report.executed_node_count,
      unvisited_node_ids: report.unvisited_node_ids,
    })),
  }
}

function applyRunContextPreset(preset: WorkflowRunContextPreset) {
  runContext.value = {
    enabled: true,
    ...preset.values,
  }
}

function nodeExecutionStep(node: WorkflowNode): WorkflowExecutionStep | null {
  return executionStepsByNode.value.get(node.id) || null
}

function nodeRunOutcome(node: WorkflowNode): string {
  const step = nodeExecutionStep(node)
  if (!step) return ''
  if (step.stopped_reason === 'awaiting_choice') return 'wait'
  if (step.node_type === 'end') return 'done'
  if (isEvaluationStep(step)) {
    if (step.output?.passed === true) return 'pass'
    if (step.output?.passed === false) return 'fail'
    return 'score'
  }
  if (isTriggerEventStep(step)) {
    return isTriggerEventTriggered(step) ? 'triggered' : 'blocked'
  }
  if (step.stopped_reason && step.stopped_reason !== 'completed') return 'blocked'
  return 'ran'
}

function nodeRunClass(node: WorkflowNode): Record<string, boolean> {
  const step = nodeExecutionStep(node)
  const outcome = nodeRunOutcome(node)
  return {
    'run-executed': Boolean(step),
    'run-current': lastExecutionStep.value?.node_id === node.id,
    'run-pass': outcome === 'pass' || outcome === 'triggered' || outcome === 'done',
    'run-fail': outcome === 'fail' || outcome === 'blocked',
    'run-wait': outcome === 'wait',
    'run-score': outcome === 'score',
  }
}

function nodeRunBadge(node: WorkflowNode): string {
  const outcome = nodeRunOutcome(node)
  const labels: Record<string, string> = {
    pass: 'Pass',
    fail: 'Fail',
    score: 'Score',
    triggered: 'Event',
    blocked: 'Blocked',
    wait: 'Choice',
    done: 'Done',
    ran: 'Ran',
  }
  return labels[outcome] || ''
}

function nodeRunDetail(node: WorkflowNode): string {
  const step = nodeExecutionStep(node)
  if (!step) return ''
  if (isEvaluationStep(step)) {
    const metric = stringValue(step.output?.metric, 'overall')
    const threshold = step.output?.threshold === null || step.output?.threshold === undefined
      ? ''
      : `/${formatThreshold(step.output.threshold)}`
    return `${metric} ${formatScore(step.output?.score)}${threshold}`
  }
  if (isTriggerEventStep(step)) {
    const blockers = eventBlockers(step)
    return blockers[0] || stringValue(step.output?.event_id, 'event')
  }
  if (step.stopped_reason && step.stopped_reason !== 'completed') return step.stopped_reason
  return step.next_node_id ? `next ${step.next_node_id}` : ''
}

function nodeRunTooltip(node: WorkflowNode): string {
  const badge = nodeRunBadge(node)
  const detail = nodeRunDetail(node)
  return detail ? `${badge}: ${detail}` : badge
}

const WORKFLOW_STATE_KEY_MAX_CHARS = 128
const WORKFLOW_STATE_KEY_PATTERN = /^[A-Za-z0-9_.-]+$/
const WORKFLOW_CONDITION_MAX_CHARS = 2000
const WORKFLOW_CONDITION_CONTROL_PATTERN = /[\u0000-\u0008\u000B\u000C\u000E-\u001F\u007F-\u009F]/u

function validateWorkflowStateKey(value: unknown): string | null {
  if (typeof value !== 'string') return 'State key must be a string.'
  const key = value.trim()
  if (!key) return null
  if (key.length > WORKFLOW_STATE_KEY_MAX_CHARS) return `State key must be ${WORKFLOW_STATE_KEY_MAX_CHARS} characters or fewer.`
  if (key === '.' || key === '..') return 'State key cannot be a current or parent directory marker.'
  if (!WORKFLOW_STATE_KEY_PATTERN.test(key)) return 'State key can contain only ASCII letters, numbers, dots, underscores, or hyphens.'
  return null
}

function validateWorkflowCondition(value: unknown): string | null {
  if (typeof value !== 'string') return 'Condition must be a string.'
  if (!value.trim()) return null
  if (Array.from(value).length > WORKFLOW_CONDITION_MAX_CHARS) return `Condition must be ${WORKFLOW_CONDITION_MAX_CHARS} characters or fewer.`
  if (WORKFLOW_CONDITION_CONTROL_PATTERN.test(value)) return 'Condition cannot contain control characters.'
  return null
}

function workflowStateKeyFields(nodeType: string): string[] {
  if (nodeType === 'set_variable' || nodeType === 'evaluation') return ['variable_name']
  if (nodeType === 'set_flag') return ['flag_name']
  return []
}

function validateWorkflowLocally(currentWorkflow: Workflow): WorkflowValidationResult {
  const issues: WorkflowValidationIssue[] = []
  const ids = new Set<string>()
  const lookup = new Map<string, WorkflowNode>()
  const knownTypes = new Set(previewNodeTypes.map((type) => type.node_type))
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
    narration: ['text'],
    bgm: ['track_path'],
    sfx: ['sound_path'],
    wait: ['duration_ms'],
    sub_workflow: ['workflow_id'],
    shake: ['duration_ms'],
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
      if (!isConfigFieldPresentForNode(node.node_type, node.config, field)) addIssue('error', 'node_config_missing', node.id, `Required field \`${field}\` is missing.`)
    }
    for (const field of workflowStateKeyFields(node.node_type)) {
      const value = node.config[field]
      if (value === null || value === undefined || (typeof value === 'string' && !value.trim())) continue
      const error = validateWorkflowStateKey(value)
      if (error) addIssue('error', 'node_state_key_invalid', node.id, `State key field \`${field}\` is invalid: ${error}`)
    }
    if (node.node_type === 'condition') {
      const value = node.config.condition
      if (value !== null && value !== undefined && !(typeof value === 'string' && !value.trim())) {
        const error = validateWorkflowCondition(value)
        if (error) addIssue('error', 'node_condition_invalid', node.id, `Condition field \`condition\` is invalid: ${error}`)
      }
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

function isConfigFieldPresentForNode(nodeType: string, config: Record<string, any>, field: string): boolean {
  const aliases: Record<string, string[]> = {
    'bgm:track_path': ['track_path', 'track'],
    'sfx:sound_path': ['sound_path', 'sound'],
    'wait:duration_ms': ['duration_ms', 'duration'],
    'shake:duration_ms': ['duration_ms', 'duration'],
  }
  return (aliases[`${nodeType}:${field}`] || [field]).some((alias) => isConfigFieldPresent(config, alias))
}

function runWorkflowLocally(
  currentWorkflow: Workflow,
  maxSteps: number,
  selections: Record<string, number> = {},
  context: WorkflowRunContextPayload | null = null
): WorkflowExecutionReport {
  const validation = validateWorkflowLocally(currentWorkflow)
  if (!validation.valid) {
    return {
      workflow_id: currentWorkflow.id,
      workflow_name: currentWorkflow.name,
      completed: false,
      stopped_reason: 'validation_failed',
      ...workflowCoverage(currentWorkflow, []),
      steps: [],
      validation,
    }
  }

  const lookup = new Map(currentWorkflow.nodes.map((node) => [node.id, node]))
  const steps: WorkflowExecutionStep[] = []
  const localState = createLocalWorkflowState()
  let currentNodeId = currentWorkflow.start_node_id
  let completed = false
  let stopped_reason: string | null = null

  for (let stepIndex = 0; stepIndex < Math.max(1, Math.min(maxSteps, 256)); stepIndex += 1) {
    const node = lookup.get(currentNodeId)
    if (!node) {
      stopped_reason = `missing_node:${currentNodeId}`
      break
    }
    const output = localNodeOutput(node, context, localState)
    const next = localNextNode(node, output, selections)
    if (node.node_type === 'end') completed = true

    steps.push({
      step_index: stepIndex,
      node_id: node.id,
      node_type: node.node_type,
      label: node.label,
      output,
      next_node_id: next.nextNodeId,
      stopped_reason: next.stoppedReason,
    })

    if (completed) {
      stopped_reason = 'completed'
      break
    }
    if (next.stoppedReason) {
      stopped_reason = next.stoppedReason
      break
    }
    if (!next.nextNodeId) {
      stopped_reason = 'no_next_node'
      break
    }
    currentNodeId = next.nextNodeId
  }

  if (!completed && !stopped_reason && steps.length >= Math.max(1, Math.min(maxSteps, 256))) {
    stopped_reason = 'max_steps_reached'
  }

  const coverage = workflowCoverage(currentWorkflow, steps)

  return {
    workflow_id: currentWorkflow.id,
    workflow_name: currentWorkflow.name,
    completed,
    stopped_reason,
    ...coverage,
    steps,
    validation,
  }
}

function workflowCoverage(currentWorkflow: Workflow, steps: WorkflowExecutionStep[]) {
  const executed_node_ids: string[] = []
  const seen = new Set<string>()
  for (const step of steps) {
    if (!seen.has(step.node_id)) {
      seen.add(step.node_id)
      executed_node_ids.push(step.node_id)
    }
  }
  const unvisited_node_ids = currentWorkflow.nodes
    .filter((node) => !seen.has(node.id))
    .map((node) => node.id)
  const node_count = currentWorkflow.nodes.length
  const executed_node_count = executed_node_ids.length
  const coverage_percent = node_count > 0 ? (executed_node_count / node_count) * 100 : 0
  return {
    node_count,
    executed_node_count,
    coverage_percent,
    executed_node_ids,
    unvisited_node_ids,
  }
}

function createLocalWorkflowState(): LocalWorkflowState {
  return { variables: {}, flags: {}, relationships: {}, emotions: {} }
}

function localNodeOutput(
  node: WorkflowNode,
  context: WorkflowRunContextPayload | null = null,
  localState: LocalWorkflowState = createLocalWorkflowState()
): Record<string, any> {
  switch (node.node_type) {
    case 'start':
      return { action: 'start', node_id: node.id, next_connections: node.connections }
    case 'end':
      return { action: 'end', node_id: node.id, complete: true }
    case 'dialogue':
      return {
        action: 'dialogue',
        speaker: node.config.speaker_id || node.config.speaker || 'Narrator',
        text: node.config.text || '',
        emotion: node.config.emotion || null,
      }
    case 'choice':
      return { action: 'choice', choices: arrayConfig(node.config.choices), connection_count: node.connections.length }
    case 'set_variable': {
      const name = localStateKey(node.config.variable_name)
      const value = localVariableValue(node.config.value)
      if (name) localState.variables[name] = value
      return { status: 'ok', variable_name: name, value }
    }
    case 'set_flag': {
      const name = localStateKey(node.config.flag_name)
      const value = Boolean(node.config.value ?? true)
      if (name) localState.flags[name] = value
      return { status: 'ok', flag_name: name, value }
    }
    case 'condition': {
      const condition = evaluateLocalCondition(node.config.condition ?? 'true', context, localState, node.config)
      return {
        result: condition.result,
        condition_supported: condition.supported,
        condition_error: condition.error,
      }
    }
    case 'evaluation': {
      const metric = normalizeMetric(node.config.criteria || node.config.metric || 'overall')
      const score = workflowMetricScore(context?.evaluation, metric) ?? 0
      const threshold = Number(node.config.threshold)
      const passed = Number.isFinite(threshold) ? score >= threshold : null
      const variableName = localStateKey(node.config.variable_name)
      if (variableName) {
        localState.variables[variableName] = score
        if (typeof passed === 'boolean') localState.flags[`${variableName}_passed`] = passed
      }
      return {
        action: 'evaluation',
        character_id: node.config.character_id || context?.character_id || null,
        metric,
        score,
        threshold: Number.isFinite(threshold) ? threshold : null,
        passed,
        source: context?.enabled ? 'run_context_evaluation' : 'local_preview',
        evaluation: context?.evaluation || null,
      }
    }
    case 'trigger_event':
      return {
        action: 'trigger_event',
        event_id: node.config.event_id || '',
        event_type: node.config.event_type || '',
        triggered: localEventDecision(node, context, localState).triggered,
        evaluation_source: context?.enabled ? 'run_context_evaluation' : 'local_preview',
        decision: localEventDecision(node, context, localState),
      }
    case 'emotion_change': {
      const characterId = String(node.config.character_id || '').trim()
      const emotion = String(node.config.emotion || '').trim()
      const previousEmotion = characterId ? localState.emotions[characterId] || 'neutral' : 'neutral'
      if (characterId && emotion) localState.emotions[characterId] = emotion
      return {
        action: 'emotion_change',
        character_id: characterId,
        previous_emotion: previousEmotion,
        emotion,
      }
    }
    case 'relationship': {
      const characterId = String(node.config.character_id || '').trim()
      const targetId = String(node.config.target_id || node.config.other_id || 'player').trim() || 'player'
      const delta = numericConfig(node.config.delta) ?? 0
      const previous = localRelationshipValue(localState, characterId, targetId, localRelationshipFallback(context, characterId, targetId))
      const current = Math.min(1, Math.max(-1, previous + delta))
      if (characterId) {
        localState.relationships[characterId] = localState.relationships[characterId] || {}
        localState.relationships[characterId][targetId] = current
      }
      return {
        action: 'relationship',
        character_id: characterId,
        target_id: targetId,
        delta,
        previous,
        current,
      }
    }
    case 'bgm':
      return {
        action: 'bgm',
        track: node.config.track_path || node.config.track || '',
        play_action: node.config.action || 'play',
        volume: numericConfig(node.config.volume) ?? 1,
      }
    case 'sfx':
      return {
        action: 'sfx',
        sound: node.config.sound_path || node.config.sound || '',
        volume: numericConfig(node.config.volume) ?? 1,
      }
    case 'wait':
      return { action: 'wait', duration_ms: durationMsConfig(node.config, 1000) }
    case 'random_branch': {
      const chosen = node.connections[0] || ''
      return { action: 'random_branch', chosen_connection: chosen, index: 0 }
    }
    case 'sub_workflow':
      return {
        action: 'sub_workflow',
        workflow_id: node.config.workflow_id || '',
        workflow_path: node.config.workflow_path || null,
        status: 'delegated',
      }
    case 'camera':
      return {
        action: 'camera',
        camera_action: node.config.action || 'move',
        x: numericConfig(node.config.target_x) ?? 0,
        y: numericConfig(node.config.target_y) ?? 0,
        zoom: numericConfig(node.config.zoom) ?? 1,
        duration_ms: durationMsConfig(node.config, 500),
      }
    case 'shake':
      return {
        action: 'shake',
        intensity: numericConfig(node.config.intensity) ?? 5,
        duration_ms: durationMsConfig(node.config, 300),
      }
    default:
      return { action: node.node_type }
  }
}

function evaluateLocalCondition(
  value: unknown,
  context: WorkflowRunContextPayload | null,
  localState: LocalWorkflowState = createLocalWorkflowState(),
  config: Record<string, any> = {}
) {
  const condition = String(value ?? '').trim()
  if (!condition) return { result: false, supported: false, error: 'condition_empty' }
  try {
    return {
      result: evaluateLocalConditionExpression(condition, localConditionScope(context, localState, config)),
      supported: true,
      error: null,
    }
  } catch (error) {
    return { result: false, supported: false, error: String(error) }
  }
}

function evaluateLocalConditionExpression(expression: string, scope: LocalConditionScope): boolean {
  const text = stripOuterParens(expression.trim())
  const orParts = splitConditionExpression(text, '||')
  if (orParts.length > 1) return orParts.some((part) => evaluateLocalConditionExpression(part, scope))
  const andParts = splitConditionExpression(text, '&&')
  if (andParts.length > 1) return andParts.every((part) => evaluateLocalConditionExpression(part, scope))
  if (text.startsWith('!')) return !evaluateLocalConditionExpression(text.slice(1), scope)

  const comparison = findConditionComparison(text)
  if (comparison) {
    const left = localConditionValue(comparison.left, scope)
    const right = localConditionValue(comparison.right, scope)
    return compareLocalConditionValues(left, right, comparison.operator)
  }

  return Boolean(localConditionValue(text, scope))
}

function splitConditionExpression(expression: string, operator: '&&' | '||'): string[] {
  const parts: string[] = []
  let depth = 0
  let quote = ''
  let start = 0
  for (let index = 0; index < expression.length; index += 1) {
    const char = expression[index]
    if (quote) {
      if (char === quote && expression[index - 1] !== '\\') quote = ''
      continue
    }
    if (char === '"' || char === "'") {
      quote = char
      continue
    }
    if (char === '(') depth += 1
    if (char === ')') depth = Math.max(0, depth - 1)
    if (depth === 0 && expression.startsWith(operator, index)) {
      parts.push(expression.slice(start, index).trim())
      start = index + operator.length
      index += operator.length - 1
    }
  }
  if (parts.length === 0) return [expression.trim()]
  parts.push(expression.slice(start).trim())
  return parts.filter(Boolean)
}

function findConditionComparison(expression: string): { left: string; operator: string; right: string } | null {
  let depth = 0
  let quote = ''
  const operators = ['>=', '<=', '==', '!=', '>', '<']
  for (let index = 0; index < expression.length; index += 1) {
    const char = expression[index]
    if (quote) {
      if (char === quote && expression[index - 1] !== '\\') quote = ''
      continue
    }
    if (char === '"' || char === "'") {
      quote = char
      continue
    }
    if (char === '(') depth += 1
    if (char === ')') depth = Math.max(0, depth - 1)
    if (depth !== 0) continue
    const operator = operators.find((candidate) => expression.startsWith(candidate, index))
    if (!operator) continue
    return {
      left: expression.slice(0, index).trim(),
      operator,
      right: expression.slice(index + operator.length).trim(),
    }
  }
  return null
}

function localConditionValue(raw: string, scope: LocalConditionScope): LocalConditionValue {
  const text = stripOuterParens(raw.trim())
  if (/^true$/i.test(text)) return true
  if (/^false$/i.test(text)) return false
  if (/^-?\d+(?:\.\d+)?$/.test(text)) return Number(text)
  const quoted = text.match(/^(['"])(.*)\1$/)
  if (quoted) return quoted[2].replace(/\\(['"\\])/g, '$1')
  const variable = text.match(/^[A-Za-z_][A-Za-z0-9_]*$/)
  if (variable && Object.prototype.hasOwnProperty.call(scope.context, text)) return scope.context[text]
  const getVariable = text.match(/^getVariable\((['"])([A-Za-z0-9_.-]+)\1\)$/)
  if (getVariable && Object.prototype.hasOwnProperty.call(scope.variables, getVariable[2])) return scope.variables[getVariable[2]]
  const hasFlag = text.match(/^hasFlag\((['"])([A-Za-z0-9_.-]+)\1\)$/)
  if (hasFlag) return Boolean(scope.flags[hasFlag[2]])
  throw new Error(`unsupported_condition:${text}`)
}

function compareLocalConditionValues(left: LocalConditionValue, right: LocalConditionValue, operator: string): boolean {
  if (operator === '==' || operator === '!=') {
    const equal = left === right || String(left) === String(right)
    return operator === '==' ? equal : !equal
  }
  const leftNumber = numericValue(left)
  const rightNumber = numericValue(right)
  if (leftNumber === null || rightNumber === null) throw new Error('unsupported_non_numeric_comparison')
  if (operator === '>=') return leftNumber >= rightNumber
  if (operator === '<=') return leftNumber <= rightNumber
  if (operator === '>') return leftNumber > rightNumber
  if (operator === '<') return leftNumber < rightNumber
  return false
}

function stripOuterParens(value: string): string {
  let text = value.trim()
  while (text.startsWith('(') && text.endsWith(')') && outerParensWrapExpression(text)) {
    text = text.slice(1, -1).trim()
  }
  return text
}

function outerParensWrapExpression(value: string): boolean {
  let depth = 0
  let quote = ''
  for (let index = 0; index < value.length; index += 1) {
    const char = value[index]
    if (quote) {
      if (char === quote && value[index - 1] !== '\\') quote = ''
      continue
    }
    if (char === '"' || char === "'") {
      quote = char
      continue
    }
    if (char === '(') depth += 1
    if (char === ')') depth -= 1
    if (depth === 0 && index < value.length - 1) return false
  }
  return depth === 0
}

function localStateKey(value: unknown): string {
  return String(value ?? '').trim()
}

function localVariableValue(value: unknown): LocalConditionValue {
  if (typeof value === 'boolean') return value
  if (typeof value === 'number' && Number.isFinite(value)) return value
  return String(value ?? '')
}

function localRelationshipFallback(
  context: WorkflowRunContextPayload | null,
  characterId: string,
  targetId = 'player'
): number {
  if (!context?.enabled) return 0
  if (targetId !== 'player') return 0
  if (context.character_id && characterId && context.character_id !== characterId) return 0
  return context.relationship
}

function localRelationshipValue(
  localState: LocalWorkflowState,
  characterId: string,
  targetId = 'player',
  fallback = 0
): number {
  if (!characterId) return fallback
  return localState.relationships[characterId]?.[targetId] ?? fallback
}

function localNextNode(node: WorkflowNode, output: Record<string, any>, selections: Record<string, number>) {
  if (node.node_type === 'end') return { nextNodeId: null, stoppedReason: 'completed' }
  if (node.node_type === 'choice') {
    const index = selections[node.id] ?? numericConfig(node.config.selected_index ?? node.config.default_choice_index)
    if (index !== null) return { nextNodeId: node.connections[index] || null, stoppedReason: node.connections[index] ? null : 'choice_index_out_of_range' }
    return { nextNodeId: null, stoppedReason: 'awaiting_choice' }
  }
  if (node.node_type === 'condition') return branchByBool(node.connections, Boolean(output.result), 'condition_result_missing')
  if (node.node_type === 'evaluation') return branchByBool(node.connections, typeof output.passed === 'boolean' ? output.passed : null, 'evaluation_threshold_missing')
  if (node.node_type === 'trigger_event') return branchByBool(node.connections, Boolean(output.triggered), 'event_trigger_result_missing')
  if (node.node_type === 'random_branch') return { nextNodeId: String(output.chosen_connection || '') || null, stoppedReason: output.chosen_connection ? null : 'random_branch_has_no_choice' }
  return { nextNodeId: node.connections[0] || null, stoppedReason: node.connections[0] ? null : 'no_next_node' }
}

function branchByBool(connections: string[], value: boolean | null, missingReason: string) {
  if (value === true) return { nextNodeId: connections[0] || null, stoppedReason: connections[0] ? null : 'true_branch_missing' }
  if (value === false) return { nextNodeId: connections[1] || null, stoppedReason: connections[1] ? null : 'false_branch_missing' }
  return { nextNodeId: connections[0] || null, stoppedReason: connections[0] ? null : missingReason }
}

function arrayConfig(value: any): string[] {
  if (Array.isArray(value)) return value.map(String).filter(Boolean)
  if (typeof value === 'string') return value.split('\n').map((item) => item.trim()).filter(Boolean)
  return []
}

function numericConfig(value: any): number | null {
  const number = Number(value)
  return Number.isFinite(number) && number >= 0 ? number : null
}

function durationMsConfig(config: Record<string, any>, fallback: number): number {
  const durationMs = numericConfig(config.duration_ms)
  if (durationMs !== null) return Math.round(durationMs)
  const duration = numericConfig(config.duration)
  if (duration !== null) return Math.round(duration * 1000)
  return fallback
}

function normalizeMetric(metric: any): string {
  const value = String(metric || '').trim().toLowerCase()
  if (!value || value === 'overall_score' || value === 'overall score' || value === 'total') return 'overall'
  if (value === 'friendliness_score' || value === 'friendliness score' || value === 'friendly') return 'friendliness'
  if (value === 'engagement_score' || value === 'engagement score' || value === 'engaged') return 'engagement'
  if (value === 'creativity_score' || value === 'creativity score' || value === 'creative') return 'creativity'
  return value
}

function workflowMetricScore(evaluation: WorkflowRunContextPayload['evaluation'] | null | undefined, metric: string): number | null {
  if (!evaluation) return null
  if (metric === 'friendliness') return evaluation.friendliness
  if (metric === 'engagement') return evaluation.engagement
  if (metric === 'creativity') return evaluation.creativity
  if (metric === 'overall') return evaluation.overall_score
  return null
}

function localEventRule(eventId: string) {
  const rules: Record<string, Record<string, any>> = {
    first_friend: { event_id: 'first_friend', event_type: 'relationship_milestone', min_relationship: 0.3 },
    close_friend: { event_id: 'close_friend', event_type: 'relationship_milestone', min_relationship: 0.6 },
    best_friend: { event_id: 'best_friend', event_type: 'relationship_milestone', min_relationship: 0.8 },
    high_engagement: { event_id: 'high_engagement', event_type: 'special_dialogue', score_metric: 'engagement', min_score: 0.8, min_evaluation_count: 2 },
    creative_talk: { event_id: 'creative_talk', event_type: 'special_dialogue', score_metric: 'creativity', min_score: 0.8, min_evaluation_count: 2 },
    dedicated_player: { event_id: 'dedicated_player', event_type: 'cumulative_achievement', min_evaluation_count: 5 },
    super_dedicated: { event_id: 'super_dedicated', event_type: 'cumulative_achievement', min_evaluation_count: 10 },
  }
  return rules[eventId] || null
}

function localEventDecision(
  node: WorkflowNode,
  context: WorkflowRunContextPayload | null,
  localState: LocalWorkflowState = createLocalWorkflowState()
) {
  const eventId = String(node.config.event_id || '')
  const characterId = String(node.config.character_id || context?.character_id || '').trim()
  const configuredType = String(node.config.event_type || '')
  const rule = localEventRule(eventId)
  const relationship = localRelationshipValue(
    localState,
    characterId,
    'player',
    localRelationshipFallback(context, characterId, 'player')
  )
  const evaluationCount = context?.enabled ? context.evaluation_count : 0
  const alreadyTriggered = Boolean(context?.already_triggered_events.includes(eventId))
  const actualMetric = rule?.score_metric || null
  const actualScore = actualMetric ? workflowMetricScore(context?.evaluation, actualMetric) : null
  const blocked_reasons: string[] = []

  if (!context?.enabled) blocked_reasons.push('local_preview_no_chat_session')
  if (!rule) blocked_reasons.push('event_rule_missing')
  if (alreadyTriggered) blocked_reasons.push('already_triggered')
  if (rule?.min_relationship !== undefined && relationship < rule.min_relationship) {
    blocked_reasons.push(`relationship ${relationship.toFixed(2)} < ${Number(rule.min_relationship).toFixed(2)}`)
  }
  if (rule?.min_evaluation_count !== undefined && evaluationCount < rule.min_evaluation_count) {
    blocked_reasons.push(`evaluation_count ${evaluationCount} < ${rule.min_evaluation_count}`)
  }
  if (rule?.min_score !== undefined && (actualScore === null || actualScore < rule.min_score)) {
    blocked_reasons.push(`${actualMetric || 'score'} ${formatScore(actualScore)} < ${Number(rule.min_score).toFixed(2)}`)
  }

  return {
    event_id: eventId,
    event_type: configuredType || rule?.event_type || '',
    description: eventId,
    triggered: blocked_reasons.length === 0,
    already_triggered: alreadyTriggered,
    actual_relationship: relationship,
    actual_evaluation_count: evaluationCount,
    actual_score_metric: actualMetric,
    actual_score: actualScore,
    rule,
    blocked_reasons,
  }
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

.workflow-node.run-executed {
  border-color: rgba(96,165,250,0.32);
}

.workflow-node.run-current {
  box-shadow: 0 0 0 2px rgba(96,165,250,0.2), var(--shadow);
}

.workflow-node.run-pass {
  border-color: rgba(34,197,94,0.42);
}

.workflow-node.run-fail {
  border-color: rgba(239,68,68,0.42);
}

.workflow-node.run-wait {
  border-color: rgba(245,158,11,0.45);
}

.workflow-node.run-score {
  border-color: rgba(96,165,250,0.38);
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

.node-run-badge {
  flex-shrink: 0;
  max-width: 72px;
  margin-left: auto;
  padding: 3px 6px;
  border-radius: 999px;
  background: rgba(148,163,184,0.14);
  color: var(--text-tertiary);
  font-size: 9px;
  font-weight: 900;
  line-height: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  text-transform: uppercase;
  white-space: nowrap;
}

.node-run-badge.pass,
.node-run-badge.triggered,
.node-run-badge.done {
  background: rgba(34,197,94,0.14);
  color: var(--success);
}

.node-run-badge.fail,
.node-run-badge.blocked {
  background: rgba(239,68,68,0.14);
  color: var(--danger);
}

.node-run-badge.wait {
  background: rgba(245,158,11,0.14);
  color: var(--warning);
}

.node-run-badge.score {
  background: rgba(96,165,250,0.14);
  color: var(--info);
}

.node-body {
  position: relative;
  display: grid;
  align-content: center;
  gap: 2px;
  height: 47px;
  padding: 0 30px 0 12px;
  color: var(--text-tertiary);
  font-size: 11px;
  font-family: var(--font-mono);
}

.node-body span,
.node-run-detail {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.node-run-detail {
  color: var(--text-secondary);
  font-size: 10px;
  font-style: normal;
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

.validation-panel,
.execution-panel {
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

.validation-summary,
.execution-summary {
  display: grid;
  gap: 3px;
  padding: 12px;
  border: 1px solid rgba(34,197,94,0.28);
  border-radius: var(--radius);
  background: rgba(34,197,94,0.08);
}

.validation-summary.invalid,
.execution-summary:not(.complete) {
  border-color: rgba(239,68,68,0.35);
  background: rgba(239,68,68,0.08);
}

.validation-summary strong,
.execution-summary strong {
  color: var(--text-primary);
  font-size: 13px;
}

.validation-summary span,
.execution-summary span,
.muted-copy,
.validation-message {
  color: var(--text-tertiary);
  font-size: 12px;
}

.coverage-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 3px 8px;
  align-items: center;
  min-width: 0;
  padding-top: 4px;
  border-top: 1px solid rgba(148,163,184,0.14);
}

.coverage-row span,
.coverage-row small {
  color: var(--text-tertiary);
  font-size: 10px;
  font-weight: 800;
  text-transform: uppercase;
}

.coverage-row strong {
  color: var(--brand-light);
  font-size: 12px;
}

.coverage-row small {
  grid-column: 1 / -1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.unvisited-node-list {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
  min-width: 0;
}

.unvisited-node-list span {
  max-width: 100%;
  padding: 3px 6px;
  border-radius: var(--radius-sm);
  background: rgba(148,163,184,0.14);
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: 10px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.run-context-panel {
  display: grid;
  gap: 10px;
  margin-bottom: 12px;
  padding: 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-2);
}

.run-context-panel.enabled {
  border-color: rgba(96,165,250,0.32);
}

.run-context-toggle {
  display: flex;
  gap: 8px;
  align-items: center;
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 800;
}

.run-context-presets {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  min-width: 0;
}

.context-preset-btn {
  min-width: 0;
  max-width: 100%;
  padding: 5px 7px;
  border: 1px solid rgba(96,165,250,0.28);
  border-radius: 999px;
  background: rgba(96,165,250,0.1);
  color: var(--brand-light);
  cursor: pointer;
  font-size: 10px;
  font-weight: 900;
  overflow: hidden;
  text-overflow: ellipsis;
  text-transform: uppercase;
  white-space: nowrap;
}

.context-preset-btn:hover {
  border-color: rgba(96,165,250,0.5);
  background: rgba(96,165,250,0.16);
}

.context-preset-btn.matrix {
  border-color: rgba(34,197,94,0.3);
  background: rgba(34,197,94,0.1);
  color: var(--success);
}

.context-preset-btn:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.run-context-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}

.run-context-grid label {
  min-width: 0;
  display: grid;
  gap: 4px;
}

.run-context-grid span {
  color: var(--text-tertiary);
  font-size: 10px;
  font-weight: 800;
  text-transform: uppercase;
}

.run-context-grid .input {
  min-width: 0;
  height: 30px;
  padding: 5px 7px;
  font-size: 11px;
}

.run-context-wide {
  grid-column: 1 / -1;
}

.validation-message {
  margin-top: 10px;
  color: var(--warning);
}

.matrix-coverage-panel {
  display: grid;
  gap: 6px;
  margin-top: 10px;
  padding: 10px;
  border: 1px solid rgba(245,158,11,0.3);
  border-radius: var(--radius);
  background: rgba(245,158,11,0.08);
}

.matrix-coverage-panel.complete {
  border-color: rgba(34,197,94,0.32);
  background: rgba(34,197,94,0.08);
}

.matrix-coverage-panel strong {
  color: var(--text-primary);
  font-size: 13px;
}

.matrix-coverage-panel > span {
  color: var(--text-tertiary);
  font-size: 12px;
}

.matrix-run-list {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
  min-width: 0;
}

.matrix-run-list span {
  max-width: 100%;
  padding: 3px 6px;
  border-radius: var(--radius-sm);
  background: rgba(96,165,250,0.12);
  color: var(--brand-light);
  font-size: 10px;
  font-weight: 800;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
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

.trace-list {
  display: grid;
  gap: 8px;
  margin-top: 12px;
}

.trace-item {
  display: grid;
  grid-template-columns: 24px minmax(0, 1fr);
  gap: 4px 8px;
  align-items: center;
  padding: 9px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-2);
}

.trace-item.trace-score {
  border-color: rgba(96,165,250,0.28);
}

.trace-item.trace-event {
  border-color: rgba(245,158,11,0.3);
}

.trace-item.trace-event.triggered {
  border-color: rgba(34,197,94,0.36);
}

.trace-item span {
  width: 22px;
  height: 22px;
  display: grid;
  place-items: center;
  border-radius: var(--radius-sm);
  background: var(--surface-3);
  color: var(--text-tertiary);
  font-size: 11px;
  font-weight: 900;
}

.trace-item strong,
.trace-item small,
.trace-item em {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.trace-item strong {
  color: var(--text-primary);
  font-size: 12px;
}

.trace-item small,
.trace-item em {
  grid-column: 2;
  color: var(--text-tertiary);
  font-size: 11px;
  font-style: normal;
}

.trace-diagnostics {
  grid-column: 1 / -1;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 6px;
  min-width: 0;
  padding: 8px;
  border-radius: var(--radius-sm);
  background: rgba(15,23,42,0.24);
}

.diagnostic-row {
  min-width: 0;
  display: grid;
  gap: 2px;
}

.diagnostic-row span {
  width: auto;
  height: auto;
  display: block;
  border-radius: 0;
  background: transparent;
  color: var(--text-tertiary);
  font-size: 10px;
  font-weight: 800;
  text-transform: uppercase;
}

.diagnostic-row strong {
  min-width: 0;
  overflow: hidden;
  color: var(--text-primary);
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.score-meter {
  grid-column: 1 / -1;
  height: 5px;
  overflow: hidden;
  border-radius: 999px;
  background: var(--surface-3);
}

.score-meter i {
  display: block;
  height: 100%;
  border-radius: inherit;
  background: var(--info);
}

.gate-pill,
.event-status {
  width: auto;
  height: auto;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 3px 7px;
  border-radius: 999px;
  background: rgba(148,163,184,0.14);
  color: var(--text-tertiary);
  font-size: 10px;
  font-weight: 900;
  text-transform: uppercase;
}

.gate-pill.pass,
.event-status.active {
  background: rgba(34,197,94,0.14);
  color: var(--success);
}

.gate-pill.fail {
  background: rgba(239,68,68,0.14);
  color: var(--danger);
}

.event-state-row {
  grid-column: 1 / -1;
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 7px;
  align-items: center;
  min-width: 0;
}

.event-state-row strong {
  min-width: 0;
  overflow: hidden;
  color: var(--text-primary);
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.blocker-list {
  grid-column: 1 / -1;
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
  min-width: 0;
}

.blocker-list span {
  width: auto;
  max-width: 100%;
  height: auto;
  display: inline-flex;
  padding: 3px 6px;
  border-radius: var(--radius-sm);
  background: rgba(239,68,68,0.12);
  color: var(--danger);
  font-size: 10px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.choice-debug {
  grid-column: 1 / -1;
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  min-width: 0;
}

.choice-debug-btn {
  min-width: 0;
  max-width: 100%;
  padding: 5px 8px;
  border: 1px solid rgba(96,165,250,0.35);
  border-radius: var(--radius-sm);
  background: rgba(96,165,250,0.12);
  color: var(--brand-light);
  cursor: pointer;
  font-size: 11px;
  font-weight: 800;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
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
