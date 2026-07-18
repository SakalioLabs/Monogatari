import type { WebGpuRuntimeConfig } from './webgpuInference'
import type {
  CloudSyncStatus,
  EngineStatus,
  ProjectConfigIssue,
  ProjectConfigState,
  ProjectPathStatus,
  SettingsDocument,
  SettingsFormValues,
  SettingsSection,
} from './settingsContract'

const SETTINGS_SECTIONS = new Set<SettingsSection>([
  'project',
  'ai',
  'voice',
  'sync',
  'diagnostics',
])

const RUNTIME_SECRET_SETTING_KEYS = new Set([
  'api_key',
  'apikey',
  'api-token',
  'api_token',
  'authorization',
  'x-api-key',
  'api-key',
  'token',
  'access_token',
  'accesstoken',
  'secret',
  'password',
])

const UNSAFE_PATH_SEGMENTS = new Set(['__proto__', 'prototype', 'constructor'])
const SETTINGS_PATH_SEGMENT = /^[A-Za-z0-9_-]+$/

const PREVIEW_PATHS: ReadonlyArray<Readonly<ProjectPathStatus>> = [
  { key: 'characters', label: 'Characters', relative_path: 'characters', absolute_path: '', exists: true, item_count: 1, required: true },
  { key: 'dialogue', label: 'Dialogue', relative_path: 'dialogue', absolute_path: '', exists: true, item_count: 1, required: true },
  { key: 'roleplays', label: 'Scene Roleplays', relative_path: 'roleplays', absolute_path: '', exists: true, item_count: 1, required: true },
  { key: 'knowledge', label: 'Knowledge', relative_path: 'knowledge', absolute_path: '', exists: true, item_count: 1, required: true },
  { key: 'scenes', label: 'Scenes', relative_path: 'scenes', absolute_path: '', exists: true, item_count: 2, required: false },
  { key: 'assets', label: 'Assets', relative_path: 'assets', absolute_path: '', exists: true, item_count: 1, required: true },
  { key: 'events', label: 'Story Events', relative_path: 'events', absolute_path: '', exists: true, item_count: 1, required: false },
  { key: 'endings', label: 'Story Endings', relative_path: 'endings', absolute_path: '', exists: true, item_count: 1, required: false },
  { key: 'saves', label: 'Saves', relative_path: 'saves', absolute_path: '', exists: false, item_count: 0, required: false },
  { key: 'quality_suites', label: 'Quality Suites', relative_path: 'quality_suites', absolute_path: '', exists: true, item_count: 1, required: false },
]

export function normalizeSettingsSection(value: string | null | undefined): SettingsSection {
  return value && SETTINGS_SECTIONS.has(value as SettingsSection)
    ? value as SettingsSection
    : 'project'
}

export function createEmptyEngineStatus(): EngineStatus {
  return {
    initialized: false,
    character_count: 0,
    dialogue_count: 0,
    knowledge_count: 0,
    story_event_count: 0,
    story_event_catalog_fingerprint: '',
    applied_story_event_count: 0,
    unlocked_story_content_count: 0,
    story_progress_fingerprint: '',
    ai_engines: [],
    active_ai_engine: null,
  }
}

export function createBrowserSyncStatus(
  provider: string,
  endpoint: string,
  token: string,
): CloudSyncStatus {
  const endpointConfigured = Boolean(endpoint.trim())
  const tokenConfigured = Boolean(token.trim())
  const status = provider === 'local'
    ? 'local_clean'
    : !endpointConfigured
      ? 'endpoint_missing'
      : !tokenConfigured
        ? 'token_missing'
        : 'remote_ready'
  return {
    connected: provider === 'custom' && endpointConfigured && tokenConfigured,
    status,
    last_sync: null,
    file_count: 0,
    pending_uploads: 0,
    pending_downloads: 0,
    conflict_count: 0,
    provider,
    endpoint_configured: endpointConfigured,
    token_configured: tokenConfigured,
  }
}

export function createPreviewProjectState(runtime: WebGpuRuntimeConfig): ProjectConfigState {
  const config: SettingsDocument = {
    render: { title: 'Monogatari Engine' },
    engine: { target_fps: 60 },
    ai: {
      provider: 'webgpu',
      api: { base_url: 'https://api.openai.com/v1', api_key: '', model: 'gpt-4o-mini' },
      onnx: { model_path: 'models/model.onnx', tokenizer_path: 'models/tokenizer.json', use_directml: true },
      webgpu: {
        model_id: runtime.modelId,
        dtype: runtime.dtype,
        max_new_tokens: runtime.maxNewTokens,
        temperature: runtime.temperature,
        top_p: runtime.topP,
      },
    },
    paths: Object.fromEntries(PREVIEW_PATHS.map((path) => [path.key, path.relative_path])),
  }
  return {
    project_path: 'Browser preview',
    settings_path: 'Browser preview/settings.json',
    settings_exists: true,
    valid: true,
    error_count: 0,
    warning_count: 0,
    config,
    paths: PREVIEW_PATHS.map((path) => ({ ...path })),
    issues: [],
  }
}

export function createBrowserProjectState(
  config: SettingsDocument,
  preview: ProjectConfigState,
): ProjectConfigState {
  const copiedConfig = cloneSettingsDocument(config)
  const issues: ProjectConfigIssue[] = []
  const configuredProvider = getSettingsConfigValue(copiedConfig, ['ai', 'provider']) || 'api'
  if (configuredProvider === 'api') {
    if (!String(getSettingsConfigValue(copiedConfig, ['ai', 'api', 'base_url']) || '').trim()) {
      issues.push({ severity: 'warning', code: 'api_base_url_missing', path: 'ai.api.base_url', message: 'API base URL is empty.' })
    }
    issues.push({ severity: 'warning', code: 'api_key_missing', path: 'ai.api.api_key', message: 'API key is not configured.' })
  } else if (configuredProvider === 'onnx'
    && !String(getSettingsConfigValue(copiedConfig, ['ai', 'onnx', 'model_path']) || '').trim()) {
    issues.push({ severity: 'warning', code: 'onnx_model_missing', path: 'ai.onnx.model_path', message: 'ONNX model path is empty.' })
  } else if (configuredProvider === 'webgpu'
    && !String(getSettingsConfigValue(copiedConfig, ['ai', 'webgpu', 'model_id']) || '').trim()) {
    issues.push({ severity: 'warning', code: 'webgpu_model_missing', path: 'ai.webgpu.model_id', message: 'WebGPU model ID is empty.' })
  }
  return {
    ...preview,
    config: copiedConfig,
    warning_count: issues.length,
    issues,
    paths: preview.paths.map((path) => ({
      ...path,
      relative_path: String(getSettingsConfigValue(copiedConfig, ['paths', path.key]) || path.relative_path),
    })),
  }
}

export function buildSettingsConfig(
  source: SettingsDocument,
  values: SettingsFormValues,
): SettingsDocument {
  const config = cloneSettingsDocument(source)
  setSettingsConfigValue(config, ['render', 'title'], values.projectTitle)
  setSettingsConfigValue(config, ['engine', 'target_fps'], finitePositive(values.targetFps, 60))
  setSettingsConfigValue(config, ['ai', 'provider'], values.provider)
  setSettingsConfigValue(config, ['ai', 'api', 'base_url'], values.apiBaseUrl)
  setSettingsConfigValue(config, ['ai', 'api', 'api_key'], '')
  setSettingsConfigValue(config, ['ai', 'api', 'model'], values.apiModel)
  setSettingsConfigValue(config, ['ai', 'onnx', 'model_path'], values.modelPath)
  setSettingsConfigValue(config, ['ai', 'onnx', 'tokenizer_path'], values.tokenizerPath)
  setSettingsConfigValue(config, ['ai', 'onnx', 'use_directml'], true)
  setSettingsConfigValue(config, ['ai', 'webgpu', 'model_id'], values.webGpuModelId)
  setSettingsConfigValue(config, ['ai', 'webgpu', 'dtype'], values.webGpuDtype)
  setSettingsConfigValue(config, ['ai', 'webgpu', 'max_new_tokens'], finitePositive(values.webGpuMaxNewTokens, 256))
  setSettingsConfigValue(config, ['ai', 'webgpu', 'temperature'], 0.7)
  setSettingsConfigValue(config, ['ai', 'webgpu', 'top_p'], 0.9)
  for (const [key, value] of Object.entries(values.paths)) {
    setSettingsConfigValue(config, ['paths', key], value)
  }
  return scrubRuntimeSecretSettings(config) as SettingsDocument
}

export function getSettingsConfigValue(
  config: SettingsDocument,
  path: readonly string[],
): unknown {
  return path.reduce<unknown>((current, key) => (
    isSettingsObject(current) ? current[key] : undefined
  ), config)
}

export function setSettingsConfigValue(
  config: SettingsDocument,
  path: readonly string[],
  value: unknown,
): void {
  if (path.length === 0) throw new Error('Settings config path cannot be empty')
  for (const segment of path) {
    if (!SETTINGS_PATH_SEGMENT.test(segment) || UNSAFE_PATH_SEGMENTS.has(segment.toLowerCase())) {
      throw new Error(`Unsafe settings config path segment: ${segment || '<empty>'}`)
    }
  }
  let current = config
  for (const key of path.slice(0, -1)) {
    const next = current[key]
    if (!isSettingsObject(next)) current[key] = {}
    current = current[key] as SettingsDocument
  }
  current[path[path.length - 1]] = value
}

export function scrubRuntimeSecretSettings(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(scrubRuntimeSecretSettings)
  if (typeof value === 'string') return scrubRuntimeSecretString(value)
  if (!isSettingsObject(value)) return value
  return Object.fromEntries(Object.entries(value).map(([key, entry]) => (
    isRuntimeSecretSettingKey(key)
      ? [key, '']
      : [key, scrubRuntimeSecretSettings(entry)]
  )))
}

export function sanitizeManifestSettings(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(sanitizeManifestSettings)
  if (typeof value === 'string') return scrubRuntimeSecretString(value)
  if (!isSettingsObject(value)) return value
  return Object.fromEntries(Object.entries(value).map(([key, entry]) => (
    isRuntimeSecretSettingKey(key)
      ? [key, '<redacted>']
      : [key, sanitizeManifestSettings(entry)]
  )))
}

export function scrubRuntimeSecretString(value: string): string {
  return scrubSecretAssignments(scrubTokenLikeValues(value))
}

export function safeSettingsFileName(value: string): string {
  return value.trim().replace(/[^a-z0-9._-]+/gi, '-').replace(/^-+|-+$/g, '') || 'monogatari-project'
}

export function formatSettingsBytes(value: number): string {
  if (!Number.isFinite(value) || value <= 0) return '0 B'
  const units = ['B', 'KiB', 'MiB', 'GiB']
  const unit = Math.min(Math.floor(Math.log(value) / Math.log(1024)), units.length - 1)
  const amount = value / (1024 ** unit)
  return `${amount >= 10 || unit === 0 ? amount.toFixed(0) : amount.toFixed(1)} ${units[unit]}`
}

export function createBrowserProjectManifest(options: {
  exportedAt: string
  projectPath: string
  settings: SettingsDocument
  engineVersion?: string
  gitCommit?: string
  gitShortCommit?: string
}) {
  return {
    format: 'monogatari-project',
    schema: 'monogatari-project-export@1',
    exported_at: options.exportedAt,
    export_metadata: {
      engine_version: options.engineVersion ?? '0.9.5',
      git_commit: options.gitCommit ?? 'preview',
      git_short_commit: options.gitShortCommit ?? 'preview',
    },
    project_path: options.projectPath,
    settings: sanitizeManifestSettings(options.settings),
    content: {},
    content_summary: {
      schema: 'monogatari-project-content-summary/v1',
      file_count: 0,
      json_file_count: 0,
      asset_file_count: 0,
      category_counts: {},
      category_bytes: {},
      category_fingerprint_algorithm: 'sha256:path-size-file-sha256-v1',
      category_fingerprints: {},
      exported_categories: ['characters', 'dialogue', 'roleplays', 'knowledge', 'scenes', 'assets', 'events', 'endings', 'locales', 'quality_suites', 'workflows'],
    },
    package: {
      file_count: 0,
      total_bytes: 0,
      fingerprint_algorithm: 'sha256:path-size-file-sha256-v1',
      content_sha256: 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
      directories: ['assets', 'characters', 'dialogue', 'endings', 'events', 'knowledge', 'locales', 'quality_suites', 'roleplays', 'scenes', 'workflows'],
      files: [],
    },
    archive: {
      format: 'zip',
      manifest_path: 'monogatari-project.json',
      extension: '.monogatari',
    },
  }
}

function cloneSettingsDocument(value: SettingsDocument): SettingsDocument {
  const cloned = JSON.parse(JSON.stringify(value ?? {})) as unknown
  return isSettingsObject(cloned) ? cloned : {}
}

function isSettingsObject(value: unknown): value is SettingsDocument {
  return Boolean(value && typeof value === 'object' && !Array.isArray(value))
}

function isRuntimeSecretSettingKey(key: string): boolean {
  return RUNTIME_SECRET_SETTING_KEYS.has(key.toLowerCase())
}

function scrubTokenLikeValues(value: string): string {
  return value
    .replace(/github_pat_[A-Za-z0-9_]{20,}|gh[pousr]_[A-Za-z0-9]{20,}|sk-[A-Za-z0-9_-]{20,}|glpat-[A-Za-z0-9_-]{20,}|xox[baprs]-[A-Za-z0-9-]{20,}|AIza[0-9A-Za-z_-]{30,}|AKIA[0-9A-Z]{16}/g, '<redacted>')
    .replace(/\bBearer\s+[A-Za-z0-9._~+/-]{20,}=?/gi, 'Bearer <redacted>')
    .replace(/\beyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b/g, '<redacted>')
}

function scrubSecretAssignments(value: string): string {
  return Array.from(RUNTIME_SECRET_SETTING_KEYS).reduce((current, key) => {
    const pattern = new RegExp(`(^|[^A-Za-z0-9_-])(${escapeRegExp(key)})(\\s*[:=]\\s*)(?:"([^"]*)"|'([^']*)'|([^&,}\\];\\r\\n]+))`, 'gi')
    return current.replace(pattern, (_match, prefix, matchedKey, separator, doubleQuoted, singleQuoted) => {
      if (doubleQuoted !== undefined) return `${prefix}${matchedKey}${separator}"<redacted>"`
      if (singleQuoted !== undefined) return `${prefix}${matchedKey}${separator}'<redacted>'`
      return `${prefix}${matchedKey}${separator}<redacted>`
    })
  }, value)
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

function finitePositive(value: number, fallback: number): number {
  return Number.isFinite(value) && value > 0 ? value : fallback
}
