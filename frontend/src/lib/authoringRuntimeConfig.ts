export interface AuthoringRuntimeEnvironment {
  MONOGATARI_AI_API_KEY?: string
  MONOGATARI_API_KEY?: string
  MONOGATARI_AI_BASE_URL?: string
  MONOGATARI_AI_MODEL?: string
}

export function resolveAuthoringApiKey(environment: AuthoringRuntimeEnvironment): string {
  return String(
    environment.MONOGATARI_AI_API_KEY
      || environment.MONOGATARI_API_KEY
      || '',
  ).trim()
}

export interface AuthoringApiSettings {
  provider?: unknown
  api?: {
    base_url?: unknown
    baseUrl?: unknown
    model?: unknown
    max_tokens?: unknown
    maxTokens?: unknown
    temperature?: unknown
    top_p?: unknown
    topP?: unknown
  }
}

export interface ResolvedAuthoringApiRuntime {
  baseUrl: string
  model: string
  maxNewTokens: number
  temperature: number
  topP: number
}

export function resolveAuthoringApiRuntime(
  environment: AuthoringRuntimeEnvironment,
  settings: AuthoringApiSettings,
): ResolvedAuthoringApiRuntime | null {
  const api = settings.api || {}
  const environmentBaseUrl = String(environment.MONOGATARI_AI_BASE_URL || '').trim()
  const environmentModel = String(environment.MONOGATARI_AI_MODEL || '').trim()
  const environmentOverride = Boolean(environmentBaseUrl || environmentModel)
  if (!environmentOverride && settings.provider !== 'api') return null

  const baseUrl = environmentBaseUrl
    || String(api.base_url || api.baseUrl || '').trim()
  const model = environmentModel || String(api.model || '').trim()
  if (!baseUrl || !model) return null

  return {
    baseUrl,
    model,
    maxNewTokens: boundedNumber(api.max_tokens ?? api.maxTokens, 256, 1, 2_048),
    temperature: boundedNumber(api.temperature, 0.7, 0, 2),
    topP: boundedNumber(api.top_p ?? api.topP, 0.9, 0.01, 1),
  }
}

function boundedNumber(
  value: unknown,
  fallback: number,
  minimum: number,
  maximum: number,
): number {
  const parsed = Number(value)
  return Number.isFinite(parsed)
    ? Math.min(maximum, Math.max(minimum, parsed))
    : fallback
}
