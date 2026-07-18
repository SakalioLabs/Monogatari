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
  maxContextCharacters?: number
  recoveryMaxContextCharacters?: number
  onChunk?: (chunk: string) => void
  onReset?: () => void
}

export const WEBGPU_CONTEXT_CHARACTER_LIMIT = 6_000
export const WEBGPU_RECOVERY_CONTEXT_CHARACTER_LIMIT = 3_000

export const DEFAULT_WEBGPU_RUNTIME_CONFIG: WebGpuRuntimeConfig = {
  modelId: 'onnx-community/Qwen3.5-0.8B-Text-ONNX',
  dtype: 'q4',
  maxNewTokens: 96,
  temperature: 0.7,
  topP: 0.9,
}

const PACKAGED_ORT_WASM_URL = new URL(
  '../../node_modules/onnxruntime-web/dist/ort-wasm-simd-threaded.asyncify.wasm',
  import.meta.url,
).href

type WebGpuNavigator = Navigator & {
  gpu?: {
    requestAdapter: () => Promise<unknown | null>
  }
}

let activeConfig = { ...DEFAULT_WEBGPU_RUNTIME_CONFIG }
let loadedGeneratorKey = ''
let verifiedGeneratorKey = ''
let generatorPromise: Promise<any> | null = null
let generationQueue: Promise<void> = Promise.resolve()

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

export function hasVerifiedWebGpuGeneration(): boolean {
  return verifiedGeneratorKey === generatorKey(activeConfig)
}

export function configureWebGpuRuntime(config: Partial<WebGpuRuntimeConfig>): WebGpuRuntimeConfig {
  const next = normalizeConfig({ ...activeConfig, ...config })
  const nextKey = generatorKey(next)
  if (loadedGeneratorKey && loadedGeneratorKey !== nextKey) {
    generatorPromise = null
    loadedGeneratorKey = ''
  }
  if (verifiedGeneratorKey && verifiedGeneratorKey !== nextKey) verifiedGeneratorKey = ''
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

export function generateWebGpuChat(
  messages: WebGpuChatMessage[],
  options: WebGpuGenerationOptions = {},
): Promise<string> {
  const task = generationQueue.then(() => generateWebGpuChatNow(messages, options))
  generationQueue = task.then(() => undefined, () => undefined)
  return task
}

async function generateWebGpuChatNow(
  messages: WebGpuChatMessage[],
  options: WebGpuGenerationOptions,
): Promise<string> {
  if (messages.length === 0) throw new Error('WebGPU generation requires at least one chat message.')

  const support = detectWebGpuSupport()
  if (!support.available) throw new Error(webGpuSupportMessage(support.reason))

  const config = getWebGpuRuntimeConfig()
  let generator = await generatorForConfig(config)
  const transformers = await loadTransformersRuntime()
  let streamedText = ''
  const temperature = clamp(options.temperature ?? config.temperature, 0, 2)
  const requestedTokens = Math.round(clamp(options.maxNewTokens ?? config.maxNewTokens, 1, 512))
  const contextLimit = Math.round(clamp(
    options.maxContextCharacters ?? WEBGPU_CONTEXT_CHARACTER_LIMIT,
    1_024,
    32_000,
  ))
  const runGeneration = async (
    promptMessages: WebGpuChatMessage[],
    maxNewTokens: number,
  ): Promise<string> => {
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
    const output = await generator(promptMessages, {
      max_new_tokens: maxNewTokens,
      temperature,
      top_p: clamp(options.topP ?? config.topP, 0.01, 1),
      do_sample: temperature > 0,
      repetition_penalty: 1.08,
      streamer,
    })
    return extractGeneratedText(output) || streamedText
  }

  let text: string
  try {
    text = await runGeneration(compactWebGpuChatMessages(messages, contextLimit), requestedTokens)
  } catch (error) {
    if (!isWebGpuMemoryError(error)) throw error
    streamedText = ''
    options.onReset?.()
    await disposeWebGpuGenerator()
    const recoveryLimit = Math.round(clamp(
      options.recoveryMaxContextCharacters ?? WEBGPU_RECOVERY_CONTEXT_CHARACTER_LIMIT,
      1_024,
      contextLimit,
    ))
    try {
      generator = await generatorForConfig(config)
      text = await runGeneration(
        compactWebGpuChatMessages(messages, recoveryLimit),
        Math.min(requestedTokens, 48),
      )
    } catch (recoveryError) {
      if (!isWebGpuMemoryError(recoveryError)) throw recoveryError
      throw new Error(
        'WebGPU memory was exhausted after a reduced-context retry. Close other GPU-heavy tabs or select a smaller local model.',
      )
    }
  }

  if (!text.trim()) throw new Error('The WebGPU model completed without generating text.')
  verifiedGeneratorKey = generatorKey(config)
  if (options.onChunk && !streamedText) options.onChunk(text)
  return text
}

async function disposeWebGpuGenerator(): Promise<void> {
  const staleGenerator = generatorPromise
  generatorPromise = null
  loadedGeneratorKey = ''
  verifiedGeneratorKey = ''
  if (!staleGenerator) return
  try {
    const generator = await staleGenerator
    if (typeof generator?.dispose === 'function') await generator.dispose()
  } catch {
    // A failed pipeline has no reusable resources and is safe to discard.
  }
}

export function compactWebGpuChatMessages(
  messages: WebGpuChatMessage[],
  characterLimit = WEBGPU_CONTEXT_CHARACTER_LIMIT,
): WebGpuChatMessage[] {
  const limit = Math.round(clamp(characterLimit, 1_024, 32_000))
  const normalized = messages
    .filter((message) => ['system', 'user', 'assistant'].includes(message.role) && message.content.trim())
    .map((message) => ({ ...message, content: message.content.trim() }))
  if (normalized.length === 0) return []

  const firstSystem = normalized.find((message) => message.role === 'system')
  const conversation = normalized.filter((message) => message !== firstSystem)
  const compacted: WebGpuChatMessage[] = []
  let used = 0
  if (firstSystem) {
    const content = prefixCharacters(firstSystem.content, Math.floor(limit * 0.58))
    compacted.push({ role: 'system', content })
    used += [...content].length
  }

  const latest = conversation.at(-1)
  if (!latest) return compacted
  const latestContent = prefixCharacters(latest.content, Math.max(256, Math.floor(limit * 0.34)))
  const latestMessage = { ...latest, content: latestContent }
  used += [...latestContent].length

  const history = conversation.slice(0, -1)
  const selected: WebGpuChatMessage[] = []
  for (let end = history.length; end > 0;) {
    const start = Math.max(0, end - 2)
    const pair = history.slice(start, end).map((message) => ({
      ...message,
      content: prefixCharacters(message.content, 1_000),
    }))
    const pairCharacters = pair.reduce((total, message) => total + [...message.content].length, 0)
    if (used + pairCharacters > limit) break
    selected.unshift(...pair)
    used += pairCharacters
    end = start
  }
  compacted.push(...selected, latestMessage)
  return compacted
}

export function isWebGpuMemoryError(error: unknown): boolean {
  const message = error instanceof Error ? error.message : String(error)
  const normalized = message.toLocaleLowerCase()
  return normalized.includes('std::bad_alloc')
    || normalized.includes('out of memory')
    || normalized.includes('memory allocation')
    || normalized.includes('failed to allocate')
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
      console.error('[webgpu-inference] Failed to initialize the model runtime.', error)
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

  // Keep the module factory bundled by onnxruntime-web. Overriding the MJS path
  // makes ORT bootstrap it through a blob URL, which conflicts with the web CSP.
  wasm.numThreads = 1
  wasm.wasmPaths = {
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

function prefixCharacters(value: string, limit: number): string {
  return [...value].slice(0, limit).join('')
}
