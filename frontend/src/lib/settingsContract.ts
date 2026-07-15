import type { WebGpuDType } from './webgpuInference'

export type SettingsSection = 'project' | 'ai' | 'voice' | 'sync' | 'diagnostics'
export type SettingsAiProvider = 'api' | 'onnx' | 'webgpu'

export interface TtsSettings {
  provider: 'system' | 'azure' | 'elevenlabs'
  api_url: string | null
  api_region: string
  api_voice_id: string
  api_key: string
  default_voice: string | null
  language: string
  speed: number
  pitch: number
}

export interface EngineStatus {
  initialized: boolean
  character_count: number
  dialogue_count: number
  knowledge_count: number
  story_event_count: number
  story_event_catalog_fingerprint: string
  applied_story_event_count: number
  unlocked_story_content_count: number
  story_progress_fingerprint: string
  ai_engines: string[]
  active_ai_engine: string | null
}

export type InferenceBackendId =
  | 'web_gpu'
  | 'llama_cpp'
  | 'win_ml_gen_ai'
  | 'direct_ml_onnx'
  | 'mlx_lm'
  | 'vllm'
  | 'sglang'
  | 'open_ai_compatible'

export type BackendReadiness =
  | 'ready'
  | 'probe_required'
  | 'setup_required'
  | 'blocked'
  | 'unavailable'

export type DeploymentTarget = 'web' | 'desktop' | 'server'
export type ModelProfile = 'qwen35_text08_b' | 'generic_causal_lm'
export type AccelerationKind = 'cpu' | 'cuda' | 'vulkan' | 'metal' | 'rocm' | 'musa' | 'sycl'

export interface HostCapabilities {
  os: string
  architecture: string
  is_wsl: boolean
  cuda_runtime_detected: boolean
  vulkan_probe_detected: boolean
  metal_runtime_detected: boolean
  rocm_runtime_detected: boolean
  musa_runtime_detected: boolean
  sycl_runtime_detected: boolean
  winml_runtime_detected: boolean
  directml_runtime_detected: boolean
  llama_cpp_detected: boolean
  mlx_lm_detected: boolean
  vllm_detected: boolean
  sglang_detected: boolean
  docker_detected: boolean
}

export interface BackendAssessment {
  backend: InferenceBackendId
  readiness: BackendReadiness
  reason_code: string
  summary: string
  accelerators: AccelerationKind[]
}

export interface InferenceBackendPlan {
  schema: string
  target: DeploymentTarget
  model_profile: ModelProfile
  host: HostCapabilities
  recommended_backend: InferenceBackendId | null
  next_probe: InferenceBackendId | null
  fallback_order: InferenceBackendId[]
  selection_summary: string
  backends: BackendAssessment[]
}

export interface ProjectPathStatus {
  key: string
  label: string
  relative_path: string
  absolute_path: string
  exists: boolean
  item_count: number
  required: boolean
}

export interface ProjectConfigIssue {
  severity: string
  code: string
  path: string | null
  message: string
}

export type SettingsDocument = Record<string, unknown>

export interface ProjectConfigState {
  project_path: string
  settings_path: string
  settings_exists: boolean
  valid: boolean
  error_count: number
  warning_count: number
  config: SettingsDocument
  paths: ProjectPathStatus[]
  issues: ProjectConfigIssue[]
}

export interface CloudSyncStatus {
  connected: boolean
  status: string
  last_sync: string | null
  file_count: number
  pending_uploads: number
  pending_downloads: number
  conflict_count: number
  provider: string
  endpoint_configured: boolean
  token_configured: boolean
}

export interface CloudSaveEntry {
  save_id: string
  device_id: string
  timestamp: string
  size_bytes: number
  checksum: string
}

export interface SettingsFormValues {
  projectTitle: string
  targetFps: number
  provider: SettingsAiProvider
  apiBaseUrl: string
  apiModel: string
  modelPath: string
  tokenizerPath: string
  webGpuModelId: string
  webGpuDtype: WebGpuDType
  webGpuMaxNewTokens: number
  paths: Readonly<Record<string, string>>
}
