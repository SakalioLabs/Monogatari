export type WebGpuDType = 'q4' | 'q4f16' | 'q8' | 'fp16' | 'fp32'

export interface WebGpuRuntimeConfig {
  modelId: string
  dtype: WebGpuDType
  maxNewTokens: number
  temperature: number
  topP: number
}

export interface WebGpuSupport {
  available: boolean
  reason: 'available' | 'server' | 'insecure-context' | 'webgpu-unavailable'
}

export interface WebGpuChatMessage {
  role: 'system' | 'user' | 'assistant'
  content: string
}

export interface WebGpuGenerationOptions {
  maxNewTokens?: number
  temperature?: number
  topP?: number
  onChunk?: (chunk: string) => void
}

export const DEFAULT_WEBGPU_RUNTIME_CONFIG: WebGpuRuntimeConfig = {
  modelId: 'onnx-community/Qwen2.5-0.5B-Instruct',
  dtype: 'q4',
  maxNewTokens: 256,
  temperature: 0.7,
  topP: 0.9,
}

const PACKAGED_ORT_WASM_URL = new URL(
  '../../node_modules/onnxruntime-web/dist/ort-wasm-simd-threaded.jsep.wasm',
  import.meta.url,
).href

type WebGpuNavigator = Navigator & {
  gpu?: {
    requestAdapter: () => Promise<unknown | null>
  }
}

let activeConfig = { ...DEFAULT_WEBGPU_RUNTIME_CONFIG }
let loadedGeneratorKey = ''
let generatorPromise: Promise<any> | null = null

export function detectWebGpuSupport(): WebGpuSupport {
  if (typeof window === 'undefined' || typeof navigator === 'undefined') {
    return { available: false, reason: 'server' }
  }

  const localHost = ['localhost', '127.0.0.1', '::1'].includes(window.location.hostname)
  if (!globalThis.isSecureContext && !localHost) {
    return { available: false, reason: 'insecure-context' }
  }

  if (!(navigator as WebGpuNavigator).gpu?.requestAdapter) {
    return { available: false, reason: 'webgpu-unavailable' }
  }

  return { available: true, reason: 'available' }
}

export function getWebGpuRuntimeConfig(): WebGpuRuntimeConfig {
  return { ...activeConfig }
}

export function configureWebGpuRuntime(config: Partial<WebGpuRuntimeConfig>): WebGpuRuntimeConfig {
  const next = normalizeConfig({ ...activeConfig, ...config })
  const nextKey = generatorKey(next)
  if (loadedGeneratorKey && loadedGeneratorKey !== nextKey) {
    generatorPromise = null
    loadedGeneratorKey = ''
  }
  activeConfig = next
  return getWebGpuRuntimeConfig()
}

export async function loadPackagedWebGpuConfig(): Promise<WebGpuRuntimeConfig> {
  if (typeof fetch === 'undefined') return getWebGpuRuntimeConfig()

  try {
    const response = await fetch(packagedAssetUrl('inference-runtime.json'), { cache: 'no-store' })
    if (!response.ok) return getWebGpuRuntimeConfig()
    const document = await response.json() as Record<string, unknown>
    if (document.backend !== 'webgpu') return getWebGpuRuntimeConfig()
    return configureWebGpuRuntime({
      modelId: stringValue(document.model_id, activeConfig.modelId),
      dtype: dtypeValue(document.dtype, activeConfig.dtype),
      maxNewTokens: numberValue(document.max_new_tokens, activeConfig.maxNewTokens),
      temperature: numberValue(document.temperature, activeConfig.temperature),
      topP: numberValue(document.top_p, activeConfig.topP),
    })
  } catch {
    return getWebGpuRuntimeConfig()
  }
}

export async function initializeWebGpuRuntime(
  config: Partial<WebGpuRuntimeConfig> = {},
  onProgress?: (progress: unknown) => void,
): Promise<WebGpuRuntimeConfig> {
  const support = detectWebGpuSupport()
  if (!support.available) throw new Error(webGpuSupportMessage(support.reason))

  const adapter = await (navigator as WebGpuNavigator).gpu?.requestAdapter()
  if (!adapter) throw new Error('WebGPU is exposed, but no compatible GPU adapter is available.')

  const normalized = configureWebGpuRuntime(config)
  await generatorForConfig(normalized, onProgress)
  return normalized
}

export async function generateWebGpuChat(
  messages: WebGpuChatMessage[],
  options: WebGpuGenerationOptions = {},
): Promise<string> {
  if (messages.length === 0) throw new Error('WebGPU generation requires at least one chat message.')

  const support = detectWebGpuSupport()
  if (!support.available) throw new Error(webGpuSupportMessage(support.reason))

  const config = getWebGpuRuntimeConfig()
  const generator = await generatorForConfig(config)
  const transformers = await loadTransformersRuntime()
  let streamedText = ''
  const streamer = options.onChunk
    ? new transformers.TextStreamer(generator.tokenizer, {
        skip_prompt: true,
        skip_special_tokens: true,
        callback_function: (chunk: string) => {
          if (!chunk) return
          streamedText += chunk
          options.onChunk?.(chunk)
        },
      })
    : undefined

  const temperature = clamp(options.temperature ?? config.temperature, 0, 2)
  const output = await generator(messages, {
    max_new_tokens: Math.round(clamp(options.maxNewTokens ?? config.maxNewTokens, 1, 2048)),
    temperature,
    top_p: clamp(options.topP ?? config.topP, 0.01, 1),
    do_sample: temperature > 0,
    repetition_penalty: 1.08,
    streamer,
  })

  const text = extractGeneratedText(output) || streamedText
  if (!text.trim()) throw new Error('The WebGPU model completed without generating text.')
  if (options.onChunk && !streamedText) options.onChunk(text)
  return text
}

export function webGpuSupportMessage(reason: WebGpuSupport['reason']): string {
  if (reason === 'insecure-context') return 'WebGPU requires HTTPS or a localhost development origin.'
  if (reason === 'webgpu-unavailable') return 'This browser does not expose WebGPU.'
  if (reason === 'server') return 'WebGPU is only available in a browser runtime.'
  return 'WebGPU is available.'
}

async function generatorForConfig(
  config: WebGpuRuntimeConfig,
  onProgress?: (progress: unknown) => void,
): Promise<any> {
  const key = generatorKey(config)
  if (generatorPromise && loadedGeneratorKey === key) return generatorPromise

  loadedGeneratorKey = key
  generatorPromise = loadTransformersRuntime()
    .then(({ pipeline }) => pipeline('text-generation', config.modelId, {
      device: 'webgpu',
      dtype: config.dtype,
      progress_callback: onProgress,
    }))
    .catch((error) => {
      generatorPromise = null
      loadedGeneratorKey = ''
      throw error
    })
  return generatorPromise
}

async function loadTransformersRuntime(): Promise<any> {
  const transformers: any = await import('@huggingface/transformers')
  const wasm = transformers.env?.backends?.onnx?.wasm
  if (!wasm) throw new Error('The packaged ONNX WebAssembly runtime is unavailable.')

  wasm.wasmPaths = {
    mjs: packagedAssetUrl('ort/ort-wasm-simd-threaded.jsep.mjs'),
    wasm: PACKAGED_ORT_WASM_URL,
  }
  return transformers
}

function packagedAssetUrl(assetPath: string): string {
  const normalized = assetPath.replace(/^\/+/, '')
  if (typeof document === 'undefined' || typeof window === 'undefined') return `/${normalized}`

  const configuredBase = import.meta.env.BASE_URL
  const baseUrl = configuredBase === './'
    ? new URL('./', document.baseURI)
    : new URL(configuredBase, window.location.origin)
  return new URL(normalized, baseUrl).href
}

function normalizeConfig(config: WebGpuRuntimeConfig): WebGpuRuntimeConfig {
  return {
    modelId: config.modelId.trim() || DEFAULT_WEBGPU_RUNTIME_CONFIG.modelId,
    dtype: dtypeValue(config.dtype, DEFAULT_WEBGPU_RUNTIME_CONFIG.dtype),
    maxNewTokens: Math.round(clamp(config.maxNewTokens, 1, 2048)),
    temperature: clamp(config.temperature, 0, 2),
    topP: clamp(config.topP, 0.01, 1),
  }
}

function generatorKey(config: WebGpuRuntimeConfig): string {
  return `${config.modelId}:${config.dtype}`
}

function extractGeneratedText(output: any): string {
  const generated = output?.[0]?.generated_text
  if (typeof generated === 'string') return generated.trim()
  if (!Array.isArray(generated)) return ''

  for (let index = generated.length - 1; index >= 0; index -= 1) {
    const message = generated[index]
    if (message?.role === 'assistant' && typeof message.content === 'string') {
      return message.content.trim()
    }
  }
  return ''
}

function dtypeValue(value: unknown, fallback: WebGpuDType): WebGpuDType {
  return ['q4', 'q4f16', 'q8', 'fp16', 'fp32'].includes(String(value))
    ? String(value) as WebGpuDType
    : fallback
}

function stringValue(value: unknown, fallback: string): string {
  return typeof value === 'string' && value.trim() ? value.trim() : fallback
}

function numberValue(value: unknown, fallback: number): number {
  const parsed = Number(value)
  return Number.isFinite(parsed) ? parsed : fallback
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, Number.isFinite(value) ? value : min))
}
