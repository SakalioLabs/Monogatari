export interface WorkflowNode {
  id: string
  node_type: string
  label: string
  x: number
  y: number
  config: Record<string, any>
  connections: string[]
}

export interface Workflow {
  id: string
  name: string
  nodes: WorkflowNode[]
  start_node_id: string
}

export interface WorkflowNodeTypeInfo {
  node_type: string
  label: string
  description: string
  category: string
  configurable_fields: string[]
}

export interface WorkflowFileSummary {
  path: string
  workflow_id: string
  name: string
  node_count: number
}

export interface WorkflowValidationIssue {
  severity: string
  code: string
  node_id: string | null
  message: string
}

export interface WorkflowValidationResult {
  valid: boolean
  error_count: number
  warning_count: number
  issues: WorkflowValidationIssue[]
}

export interface WorkflowExecutionStep {
  step_index: number
  node_id: string
  node_type: string
  label: string
  output: Record<string, any>
  next_node_id: string | null
  stopped_reason: string | null
}

export interface WorkflowExecutionReport {
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

export interface WorkflowRunContextForm {
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

export interface WorkflowRunContextPayload {
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

export interface WorkflowRunContextPreset {
  id: string
  label: string
  values: Omit<WorkflowRunContextForm, 'enabled'>
}

export interface WorkflowPresetMatrixReport {
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
