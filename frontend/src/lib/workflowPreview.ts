import type { StoryEventDefinition } from './storyEvents'
import {
  evaluateLocalCondition as evaluateCondition,
  type LocalConditionScope,
  type LocalConditionValue,
} from './localCondition'
import type {
  Workflow,
  WorkflowExecutionReport,
  WorkflowExecutionStep,
  WorkflowNode,
  WorkflowPresetMatrixReport,
  WorkflowRunContextForm,
  WorkflowRunContextPayload,
  WorkflowRunContextPreset,
  WorkflowValidationIssue,
  WorkflowValidationResult,
} from './workflowContract'

export type {
  Workflow,
  WorkflowExecutionReport,
  WorkflowExecutionStep,
  WorkflowNode,
  WorkflowPresetMatrixReport,
  WorkflowRunContextForm,
  WorkflowRunContextPayload,
  WorkflowRunContextPreset,
  WorkflowValidationIssue,
  WorkflowValidationResult,
} from './workflowContract'

export interface WorkflowPreviewOptions {
  maxSteps?: number
  selections?: Record<string, number>
  context?: WorkflowRunContextPayload | null
  storyEvents?: readonly StoryEventDefinition[]
  random?: () => number
}

interface LocalWorkflowState {
  variables: Record<string, LocalConditionValue>
  flags: Record<string, boolean>
  relationships: Record<string, Record<string, number>>
  emotions: Record<string, string>
}

const WORKFLOW_PREVIEW_NODE_TYPES = new Set([
  'start',
  'dialogue',
  'choice',
  'condition',
  'set_variable',
  'set_flag',
  'llm_generate',
  'evaluation',
  'scene_change',
  'trigger_event',
  'emotion_change',
  'relationship',
  'narration',
  'bgm',
  'sfx',
  'wait',
  'random_branch',
  'sub_workflow',
  'camera',
  'shake',
  'end',
])

const WORKFLOW_REQUIRED_FIELDS: Record<string, string[]> = {
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

const WORKFLOW_STATE_KEY_MAX_CHARS = 128
export const WORKFLOW_STATE_KEY_PATTERN = /^[A-Za-z0-9_.-]+$/
export const WORKFLOW_CONDITION_MAX_CHARS = 2000
const WORKFLOW_CONDITION_CONTROL_PATTERN = /[\u0000-\u0008\u000B\u000C\u000E-\u001F\u007F-\u009F]/u

export function clampScore(value: unknown): number {
  const number = numericValue(value)
  return number === null ? 0 : Math.min(1, Math.max(0, number))
}

export function clampRelationship(value: unknown): number {
  const number = numericValue(value)
  return number === null ? 0 : Math.min(1, Math.max(-1, number))
}

export function workflowRunContextPayloadFromValues(
  values: Omit<WorkflowRunContextForm, 'enabled'> | WorkflowRunContextForm,
): WorkflowRunContextPayload {
  const alreadyTriggeredEvents = values.already_triggered_events
    .split(/[,\n]/)
    .map((eventId) => eventId.trim())
    .filter(Boolean)

  return {
    enabled: true,
    character_id: values.character_id.trim() || null,
    relationship: clampRelationship(values.relationship),
    evaluation_count: Math.max(0, Math.round(numericValue(values.evaluation_count) ?? 0)),
    already_triggered_events: [...new Set(alreadyTriggeredEvents)],
    evaluation: {
      friendliness: clampScore(values.friendliness),
      engagement: clampScore(values.engagement),
      creativity: clampScore(values.creativity),
      overall_score: clampScore(values.overall_score),
      summary: 'Workflow author preview context.',
    },
  }
}

export function aggregatePresetMatrixCoverage(
  currentWorkflow: Workflow,
  matrixRuns: { preset: WorkflowRunContextPreset; report: WorkflowExecutionReport }[],
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

export function validateWorkflowStateKey(value: unknown): string | null {
  if (typeof value !== 'string') return 'State key must be a string.'
  const key = value.trim()
  if (!key) return null
  if (key.length > WORKFLOW_STATE_KEY_MAX_CHARS) {
    return `State key must be ${WORKFLOW_STATE_KEY_MAX_CHARS} characters or fewer.`
  }
  if (key === '.' || key === '..') return 'State key cannot be a current or parent directory marker.'
  if (!WORKFLOW_STATE_KEY_PATTERN.test(key)) {
    return 'State key can contain only ASCII letters, numbers, dots, underscores, or hyphens.'
  }
  return null
}

export function validateWorkflowCondition(value: unknown): string | null {
  if (typeof value !== 'string') return 'Condition must be a string.'
  if (!value.trim()) return null
  if (Array.from(value).length > WORKFLOW_CONDITION_MAX_CHARS) {
    return `Condition must be ${WORKFLOW_CONDITION_MAX_CHARS} characters or fewer.`
  }
  if (WORKFLOW_CONDITION_CONTROL_PATTERN.test(value)) return 'Condition cannot contain control characters.'
  return null
}

export function validateWorkflowLocally(
  currentWorkflow: Workflow,
  storyEvents: readonly StoryEventDefinition[] = [],
): WorkflowValidationResult {
  const issues: WorkflowValidationIssue[] = []
  const ids = new Set<string>()
  const lookup = new Map<string, WorkflowNode>()

  const addIssue = (severity: string, code: string, node_id: string | null, message: string) => {
    issues.push({ severity, code, node_id, message })
  }

  if (!currentWorkflow.id.trim()) addIssue('error', 'workflow_id_empty', null, 'Workflow id is required.')
  if (!currentWorkflow.name.trim()) addIssue('error', 'workflow_name_empty', null, 'Workflow name is required.')
  if (currentWorkflow.nodes.length === 0) {
    addIssue('error', 'workflow_empty', null, 'Workflow must contain at least one node.')
  }

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
  if (currentWorkflow.nodes.length > 0 && startNodes.length === 0) {
    addIssue('error', 'start_node_missing', null, 'Workflow must include a start node.')
  }
  if (startNodes.length > 1) {
    addIssue('warning', 'start_node_multiple', null, 'Multiple start nodes found; only the configured start node is used.')
  }
  if (currentWorkflow.nodes.length > 0 && !currentWorkflow.start_node_id.trim()) {
    addIssue('error', 'start_node_id_empty', null, 'Workflow start_node_id is required.')
  }
  const startNode = lookup.get(currentWorkflow.start_node_id)
  if (currentWorkflow.start_node_id && !startNode) {
    addIssue('error', 'start_node_not_found', currentWorkflow.start_node_id, 'start_node_id does not match any node.')
  }
  if (startNode && startNode.node_type !== 'start') {
    addIssue('error', 'start_node_type_invalid', startNode.id, 'start_node_id must reference a start node.')
  }
  if (currentWorkflow.nodes.length > 0 && !currentWorkflow.nodes.some((node) => node.node_type === 'end')) {
    addIssue('warning', 'end_node_missing', null, 'Workflow has no end node.')
  }

  for (const node of currentWorkflow.nodes) {
    if (!node.label.trim()) addIssue('warning', 'node_label_empty', node.id, 'Node label is empty.')
    if (!WORKFLOW_PREVIEW_NODE_TYPES.has(node.node_type)) {
      addIssue('error', 'node_type_unknown', node.id, `Unknown node type: ${node.node_type}`)
      continue
    }

    for (const field of WORKFLOW_REQUIRED_FIELDS[node.node_type] || []) {
      if (!isConfigFieldPresentForNode(node.node_type, node.config, field)) {
        addIssue('error', 'node_config_missing', node.id, `Required field \`${field}\` is missing.`)
      }
    }

    if (node.node_type === 'trigger_event' && String(node.config.event_id || '').trim()) {
      const definition = localEventDefinition(storyEvents, String(node.config.event_id).trim())
      if (!definition) {
        addIssue('error', 'node_event_unknown', node.id, `Story event \`${node.config.event_id}\` is not in the active project catalog.`)
      } else if (node.config.event_type && String(node.config.event_type).trim() !== definition.event_type) {
        addIssue('error', 'node_event_unknown', node.id, `Story event \`${definition.event_id}\` does not use type \`${node.config.event_type}\`.`)
      } else if (
        definition.rule.character_ids?.length
        && node.config.character_id
        && !definition.rule.character_ids.includes(String(node.config.character_id).trim())
      ) {
        addIssue('error', 'node_event_character_mismatch', node.id, `Story event \`${definition.event_id}\` is not available for character \`${node.config.character_id}\`.`)
      }
    }

    for (const field of workflowStateKeyFields(node.node_type)) {
      const value = node.config[field]
      if (value === null || value === undefined || (typeof value === 'string' && !value.trim())) continue
      const error = validateWorkflowStateKey(value)
      if (error) {
        addIssue('error', 'node_state_key_invalid', node.id, `State key field \`${field}\` is invalid: ${error}`)
      }
    }

    if (node.node_type === 'condition') {
      const value = node.config.condition
      if (value !== null && value !== undefined && !(typeof value === 'string' && !value.trim())) {
        const error = validateWorkflowCondition(value)
        if (error) {
          addIssue('error', 'node_condition_invalid', node.id, `Condition field \`condition\` is invalid: ${error}`)
        }
      }
    }

    const localTargets = new Set<string>()
    for (const targetId of node.connections) {
      if (!targetId.trim()) {
        addIssue('error', 'connection_empty', node.id, 'Connection target id is empty.')
        continue
      }
      if (targetId === node.id) addIssue('error', 'connection_self', node.id, 'Node cannot connect to itself.')
      if (!ids.has(targetId)) {
        addIssue('error', 'connection_target_missing', node.id, `Connection target \`${targetId}\` does not exist.`)
      }
      if (localTargets.has(targetId)) {
        addIssue('warning', 'connection_duplicate', node.id, `Duplicate connection to \`${targetId}\`.`)
      }
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
      if (!visited.has(node.id)) {
        addIssue('warning', 'node_unreachable', node.id, 'Node is not reachable from the configured start node.')
      }
    }
  }

  const error_count = issues.filter((issue) => issue.severity === 'error').length
  const warning_count = issues.filter((issue) => issue.severity === 'warning').length
  return { valid: error_count === 0, error_count, warning_count, issues }
}

export function runWorkflowLocally(
  currentWorkflow: Workflow,
  options: WorkflowPreviewOptions = {},
): WorkflowExecutionReport {
  const requestedMaxSteps = numericValue(options.maxSteps)
  const maxSteps = Math.max(1, Math.min(Math.round(requestedMaxSteps ?? 64), 256))
  const selections = options.selections ?? {}
  const context = options.context ?? null
  const storyEvents = options.storyEvents ?? []
  const random = options.random ?? Math.random
  const validation = validateWorkflowLocally(currentWorkflow, storyEvents)

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

  for (let stepIndex = 0; stepIndex < maxSteps; stepIndex += 1) {
    const node = lookup.get(currentNodeId)
    if (!node) {
      stopped_reason = `missing_node:${currentNodeId}`
      break
    }
    const output = localNodeOutput(node, context, localState, storyEvents, random)
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

  if (!completed && !stopped_reason && steps.length >= maxSteps) stopped_reason = 'max_steps_reached'

  return {
    workflow_id: currentWorkflow.id,
    workflow_name: currentWorkflow.name,
    completed,
    stopped_reason,
    ...workflowCoverage(currentWorkflow, steps),
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
  return { node_count, executed_node_count, coverage_percent, executed_node_ids, unvisited_node_ids }
}

function createLocalWorkflowState(): LocalWorkflowState {
  return { variables: {}, flags: {}, relationships: {}, emotions: {} }
}

function localNodeOutput(
  node: WorkflowNode,
  context: WorkflowRunContextPayload | null,
  localState: LocalWorkflowState,
  storyEvents: readonly StoryEventDefinition[],
  random: () => number,
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
      const metricSupported = ['friendliness', 'engagement', 'creativity', 'overall'].includes(metric)
      const metricScore = metricSupported ? workflowMetricScore(context?.evaluation, metric) ?? 0 : null
      const score = metricScore ?? 0
      const threshold = Number(node.config.threshold)
      const passed = metricScore !== null && Number.isFinite(threshold) ? score >= threshold : null
      const variableName = localStateKey(node.config.variable_name)
      if (variableName && metricScore !== null) {
        localState.variables[variableName] = score
        if (typeof passed === 'boolean') localState.flags[`${variableName}_passed`] = passed
      }
      return {
        action: 'evaluation',
        character_id: node.config.character_id || context?.character_id || null,
        metric,
        metric_supported: metricSupported,
        score,
        threshold: Number.isFinite(threshold) ? threshold : null,
        passed,
        source: context?.enabled ? 'run_context_evaluation' : 'local_preview',
        evaluation: context?.evaluation || null,
      }
    }
    case 'trigger_event': {
      const decision = localEventDecision(node, context, localState, storyEvents)
      const definition = localEventDefinition(storyEvents, String(node.config.event_id || ''))
      return {
        action: 'trigger_event',
        event_id: node.config.event_id || '',
        event_type: node.config.event_type || '',
        triggered: decision.triggered,
        applied: false,
        actions: definition?.actions || [],
        application: null,
        evaluation_source: context?.enabled ? 'run_context_evaluation' : 'local_preview',
        decision,
      }
    }
    case 'scene_change':
      return {
        action: 'scene_change',
        scene_id: String(node.config.scene_id || ''),
        name: optionalText(node.config.name),
        background_path: optionalText(node.config.background_path ?? node.config.background),
        bgm_path: optionalText(node.config.bgm_path ?? node.config.bgm),
        access: null,
      }
    case 'llm_generate':
      return {
        action: 'llm_generate',
        prompt: String(node.config.prompt || ''),
        system_prompt: optionalText(node.config.system_prompt),
        simulated: true,
      }
    case 'narration':
      return {
        action: 'narration',
        speaker: node.config.speaker || 'Narrator',
        text: node.config.text || '',
      }
    case 'emotion_change': {
      const characterId = String(node.config.character_id || '').trim()
      const emotion = String(node.config.emotion || '').trim()
      const previousEmotion = characterId ? localState.emotions[characterId] || 'neutral' : 'neutral'
      if (characterId && emotion) localState.emotions[characterId] = emotion
      return { action: 'emotion_change', character_id: characterId, previous_emotion: previousEmotion, emotion }
    }
    case 'relationship': {
      const characterId = String(node.config.character_id || '').trim()
      const targetId = String(node.config.target_id || node.config.other_id || 'player').trim() || 'player'
      const delta = signedNumericConfig(node.config.delta) ?? 0
      const previous = localRelationshipValue(
        localState,
        characterId,
        targetId,
        localRelationshipFallback(context, characterId, targetId),
      )
      const current = Math.min(1, Math.max(-1, previous + delta))
      if (characterId) {
        localState.relationships[characterId] = localState.relationships[characterId] || {}
        localState.relationships[characterId][targetId] = current
      }
      return { action: 'relationship', character_id: characterId, target_id: targetId, delta, previous, current }
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
      const weights = workflowBranchWeights(node.config, node.connections.length)
      const index = selectWeightedBranchIndex(weights, random)
      const chosen = node.connections[index] || ''
      return { action: 'random_branch', chosen_connection: chosen, index, weights }
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
        x: signedNumericConfig(node.config.target_x) ?? 0,
        y: signedNumericConfig(node.config.target_y) ?? 0,
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
  localState: LocalWorkflowState,
  config: Record<string, any>,
) {
  const condition = String(value ?? '').trim()
  if (!condition) return { result: false, supported: false, error: 'condition_empty' }
  return evaluateCondition(condition, localConditionScope(context, localState, config))
}

function localConditionScope(
  context: WorkflowRunContextPayload | null,
  localState: LocalWorkflowState,
  config: Record<string, any>,
): LocalConditionScope {
  const evaluation = context?.evaluation
  const characterId = String(config.character_id || config.speaker_id || config.speaker || context?.character_id || '').trim()
  const relationship = localRelationshipValue(
    localState,
    characterId,
    'player',
    localRelationshipFallback(context, characterId, 'player'),
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

function localNextNode(node: WorkflowNode, output: Record<string, any>, selections: Record<string, number>) {
  if (node.node_type === 'end') return { nextNodeId: null, stoppedReason: 'completed' }
  if (node.node_type === 'choice') {
    const index = selections[node.id] ?? numericConfig(node.config.selected_index ?? node.config.default_choice_index)
    if (index !== null) {
      return {
        nextNodeId: node.connections[index] || null,
        stoppedReason: node.connections[index] ? null : 'choice_index_out_of_range',
      }
    }
    return { nextNodeId: null, stoppedReason: 'awaiting_choice' }
  }
  if (node.node_type === 'condition') {
    if (output.condition_supported !== true) return { nextNodeId: null, stoppedReason: 'condition_unsupported' }
    return branchByBool(node.connections, Boolean(output.result), 'condition_result_missing')
  }
  if (node.node_type === 'evaluation') {
    if (output.metric_supported === false) return { nextNodeId: null, stoppedReason: 'evaluation_metric_unsupported' }
    return branchByBool(
      node.connections,
      typeof output.passed === 'boolean' ? output.passed : null,
      'evaluation_threshold_missing',
    )
  }
  if (node.node_type === 'trigger_event') {
    return branchByBool(node.connections, Boolean(output.triggered), 'event_trigger_result_missing')
  }
  if (node.node_type === 'random_branch') {
    return {
      nextNodeId: String(output.chosen_connection || '') || null,
      stoppedReason: output.chosen_connection ? null : 'random_branch_has_no_choice',
    }
  }
  return {
    nextNodeId: node.connections[0] || null,
    stoppedReason: node.connections[0] ? null : 'no_next_node',
  }
}

function branchByBool(connections: string[], value: boolean | null, missingReason: string) {
  if (value === true) {
    return { nextNodeId: connections[0] || null, stoppedReason: connections[0] ? null : 'true_branch_missing' }
  }
  if (value === false) {
    return { nextNodeId: connections[1] || null, stoppedReason: connections[1] ? null : 'false_branch_missing' }
  }
  return { nextNodeId: connections[0] || null, stoppedReason: connections[0] ? null : missingReason }
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
  targetId = 'player',
): number {
  if (!context?.enabled || targetId !== 'player') return 0
  if (context.character_id && characterId && context.character_id.toLowerCase() !== characterId.toLowerCase()) return 0
  return context.relationship
}

function localRelationshipValue(
  localState: LocalWorkflowState,
  characterId: string,
  targetId = 'player',
  fallback = 0,
): number {
  if (!characterId) return fallback
  return localState.relationships[characterId]?.[targetId] ?? fallback
}

function arrayConfig(value: unknown): string[] {
  if (Array.isArray(value)) return value.map(String).filter(Boolean)
  if (typeof value === 'string') return value.split('\n').map((item) => item.trim()).filter(Boolean)
  return []
}

function numericValue(value: unknown): number | null {
  const number = Number(value)
  return Number.isFinite(number) ? number : null
}

function numericConfig(value: unknown): number | null {
  const number = numericValue(value)
  return number !== null && number >= 0 ? number : null
}

function signedNumericConfig(value: unknown): number | null {
  return numericValue(value)
}

function durationMsConfig(config: Record<string, any>, fallback: number): number {
  const durationMs = numericConfig(config.duration_ms)
  if (durationMs !== null) return Math.round(durationMs)
  const duration = numericConfig(config.duration)
  if (duration !== null) return Math.round(duration * 1000)
  return fallback
}

function workflowBranchWeights(config: Record<string, any>, connectionCount: number): number[] {
  if (connectionCount <= 0) return []
  const rawWeights = Array.isArray(config.weights)
    ? config.weights.map((value: unknown) => branchWeightConfig(value))
    : typeof config.weights === 'string'
      ? config.weights.split('\n').map((value: string) => branchWeightConfig(value))
      : []
  const weights = Array.from({ length: connectionCount }, (_, index) => rawWeights[index] ?? 1)
    .map((weight) => Number.isFinite(weight) && weight > 0 ? weight : 0)
  return weights.reduce((sum, weight) => sum + weight, 0) > 0 ? weights : weights.map(() => 1)
}

function branchWeightConfig(value: unknown): number {
  const text = String(value ?? '').trim()
  if (!text) return 1
  const number = Number(text)
  return Number.isFinite(number) ? number : 1
}

function selectWeightedBranchIndex(weights: number[], random: () => number): number {
  const total = weights.reduce((sum, weight) => sum + weight, 0)
  if (total <= 0) return 0
  const randomValue = numericValue(random()) ?? 0
  const roll = Math.min(1 - Number.EPSILON, Math.max(0, randomValue)) * total
  let acc = 0
  for (let index = 0; index < weights.length; index += 1) {
    acc += weights[index]
    if (roll < acc) return index
  }
  return Math.max(0, weights.length - 1)
}

function normalizeMetric(metric: unknown): string {
  const value = String(metric || '').trim().toLowerCase()
  if (!value || value === 'overall_score' || value === 'overall score' || value === 'total') return 'overall'
  if (value === 'friendliness_score' || value === 'friendliness score' || value === 'friendly') return 'friendliness'
  if (value === 'engagement_score' || value === 'engagement score' || value === 'engaged') return 'engagement'
  if (value === 'creativity_score' || value === 'creativity score' || value === 'creative') return 'creativity'
  return value
}

function workflowMetricScore(
  evaluation: WorkflowRunContextPayload['evaluation'] | null | undefined,
  metric: string,
): number | null {
  if (!evaluation) return null
  if (metric === 'friendliness') return evaluation.friendliness
  if (metric === 'engagement') return evaluation.engagement
  if (metric === 'creativity') return evaluation.creativity
  if (metric === 'overall') return evaluation.overall_score
  return null
}

function localEventDefinition(
  storyEvents: readonly StoryEventDefinition[],
  eventId: string,
): StoryEventDefinition | null {
  return storyEvents.find((event) => event.event_id === eventId) || null
}

function localEventDecision(
  node: WorkflowNode,
  context: WorkflowRunContextPayload | null,
  localState: LocalWorkflowState,
  storyEvents: readonly StoryEventDefinition[],
) {
  const eventId = String(node.config.event_id || '')
  const characterId = String(node.config.character_id || context?.character_id || '').trim()
  const configuredType = String(node.config.event_type || '')
  const definition = localEventDefinition(storyEvents, eventId)
  const rule = definition?.rule || null
  const relationship = localRelationshipValue(
    localState,
    characterId,
    'player',
    localRelationshipFallback(context, characterId, 'player'),
  )
  const evaluationCount = context?.enabled ? context.evaluation_count : 0
  const alreadyTriggered = Boolean(context?.already_triggered_events.includes(eventId))
  const actualMetric = rule?.score_metric || null
  const actualScore = actualMetric ? workflowMetricScore(context?.evaluation, actualMetric) : null
  const blocked_reasons: string[] = []

  if (!context?.enabled) blocked_reasons.push('local_preview_no_chat_session')
  if (!rule) blocked_reasons.push('event_rule_missing')
  if (definition && configuredType && configuredType !== definition.event_type) blocked_reasons.push('event_type_mismatch')
  if (rule?.character_ids?.length && !rule.character_ids.includes(characterId)) blocked_reasons.push('character_not_allowed')
  if (alreadyTriggered && !rule?.repeatable) blocked_reasons.push('already_triggered')
  if (rule?.min_relationship != null && relationship < rule.min_relationship) {
    blocked_reasons.push(`relationship ${relationship.toFixed(2)} < ${Number(rule.min_relationship).toFixed(2)}`)
  }
  if (rule?.min_evaluation_count != null && evaluationCount < rule.min_evaluation_count) {
    blocked_reasons.push(`evaluation_count ${evaluationCount} < ${rule.min_evaluation_count}`)
  }
  if (rule?.min_score != null && (actualScore === null || actualScore < rule.min_score)) {
    blocked_reasons.push(`${actualMetric || 'score'} ${formatPreviewScore(actualScore)} < ${Number(rule.min_score).toFixed(2)}`)
  }

  return {
    event_id: eventId,
    event_type: configuredType || definition?.event_type || '',
    description: definition?.description || eventId,
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

function workflowStateKeyFields(nodeType: string): string[] {
  if (nodeType === 'set_variable' || nodeType === 'evaluation') return ['variable_name']
  if (nodeType === 'set_flag') return ['flag_name']
  return []
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

function optionalText(value: unknown): string | null {
  const text = String(value ?? '').trim()
  return text || null
}

function formatPreviewScore(value: unknown): string {
  const number = numericValue(value)
  return number === null ? '-' : number.toFixed(2)
}
