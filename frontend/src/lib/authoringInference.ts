import type { WebGpuChatMessage, WebGpuGenerationOptions } from './webgpuInference'

interface AuthoringApiRuntime {
  schema: 'monogatari-authoring-inference-runtime/v1'
  provider: 'api'
  endpoint: string
  model: string
  max_new_tokens: number
  temperature: number
  top_p: number
}

interface AuthoringApiResponse {
  choices?: Array<{
    message?: {
      content?: string
    }
  }>
  error?: {
    message?: string
  }
}

let runtimePromise: Promise<AuthoringApiRuntime | null> | null = null
const AUTHORING_API_TIMEOUT_MS = 45_000

export async function loadAuthoringApiRuntime(): Promise<AuthoringApiRuntime | null> {
  if (runtimePromise) return runtimePromise
  runtimePromise = fetchAuthoringApiRuntime()
  return runtimePromise
}

export async function generateAuthoringApiChat(
  messages: WebGpuChatMessage[],
  options: WebGpuGenerationOptions = {},
): Promise<string> {
  const runtime = await loadAuthoringApiRuntime()
  if (!runtime) throw new Error('The authoring API runtime is unavailable.')
  if (!messages.length) throw new Error('API generation requires at least one chat message.')

  const controller = new AbortController()
  const timeout = globalThis.setTimeout(() => controller.abort(), AUTHORING_API_TIMEOUT_MS)
  let response: Response
  try {
    response = await fetch(runtime.endpoint, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      signal: controller.signal,
      body: JSON.stringify({
        model: runtime.model,
        messages,
        max_tokens: positiveInteger(options.maxNewTokens, runtime.max_new_tokens, 2_048),
        temperature: finiteNumber(options.temperature, runtime.temperature, 0, 2),
        top_p: finiteNumber(options.topP, runtime.top_p, 0.01, 1),
        stream: false,
      }),
    })
  } catch (error) {
    if (controller.signal.aborted) {
      throw new Error('The authoring API timed out before producing a reply.')
    }
    throw error
  } finally {
    globalThis.clearTimeout(timeout)
  }
  const document = await response.json().catch(() => ({})) as AuthoringApiResponse
  if (!response.ok) {
    throw new Error(document.error?.message || `Authoring API returned HTTP ${response.status}.`)
  }
  const content = document.choices?.[0]?.message?.content?.trim() || ''
  if (!content) throw new Error('The authoring API completed without generating text.')
  options.onChunk?.(content)
  return content
}

async function fetchAuthoringApiRuntime(): Promise<AuthoringApiRuntime | null> {
  if (typeof fetch === 'undefined') return null
  try {
    const response = await fetch('/authoring-inference-runtime.json', { cache: 'no-store' })
    if (!response.ok) return null
    const document = await response.json() as Record<string, unknown>
    if (document.schema !== 'monogatari-authoring-inference-runtime/v1'
      || document.provider !== 'api'
      || typeof document.endpoint !== 'string'
      || typeof document.model !== 'string'
      || !document.endpoint.trim()
      || !document.model.trim()) {
      return null
    }
    return {
      schema: 'monogatari-authoring-inference-runtime/v1',
      provider: 'api',
      endpoint: document.endpoint,
      model: document.model,
      max_new_tokens: positiveInteger(document.max_new_tokens, 256, 2_048),
      temperature: finiteNumber(document.temperature, 0.7, 0, 2),
      top_p: finiteNumber(document.top_p, 0.9, 0.01, 1),
    }
  } catch {
    return null
  }
}

function positiveInteger(value: unknown, fallback: number, max: number): number {
  const number = Number(value)
  return Number.isInteger(number) && number > 0 ? Math.min(number, max) : fallback
}

function finiteNumber(value: unknown, fallback: number, min: number, max: number): number {
  const number = Number(value)
  return Number.isFinite(number) ? Math.min(max, Math.max(min, number)) : fallback
}
