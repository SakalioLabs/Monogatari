import { defineConfig, type Plugin } from 'vite'
import vue from '@vitejs/plugin-vue'
import {
  classifyAuthoringApiRuntimeHealth,
  resolveAuthoringApiKey,
  resolveAuthoringApiRuntime,
  type AuthoringApiRuntimeHealth,
} from './src/lib/authoringRuntimeConfig'
import { createReadStream, readFileSync, readdirSync, statSync } from 'node:fs'
import { createHash } from 'node:crypto'
import type { IncomingMessage, ServerResponse } from 'node:http'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const mobileDevHost = process.env.TAURI_DEV_HOST
const frontendDir = fileURLToPath(new URL('.', import.meta.url))
const defaultProjectDataDir = path.resolve(frontendDir, '..', 'data')
const projectDataDir = process.env.MONOGATARI_PROJECT_ROOT
  ? path.resolve(frontendDir, process.env.MONOGATARI_PROJECT_ROOT)
  : defaultProjectDataDir
const projectDataRoots = {
  assets: path.join(projectDataDir, 'assets'),
  events: path.join(projectDataDir, 'events'),
  scenes: path.join(projectDataDir, 'scenes'),
  dialogue: path.join(projectDataDir, 'dialogue'),
  roleplays: path.join(projectDataDir, 'roleplays'),
  campaigns: path.join(projectDataDir, 'campaigns'),
  endings: path.join(projectDataDir, 'endings'),
  characters: path.join(projectDataDir, 'characters'),
  knowledge: path.join(projectDataDir, 'knowledge'),
} as const
const projectSettingsPath = path.join(projectDataDir, 'settings.json')
const projectScope = `project-${createHash('sha256')
  .update(path.normalize(projectDataDir).toLowerCase())
  .digest('hex')}`
const authoringRuntimeHealthTtlMs = 10_000
const authoringRuntimeHealthTimeoutMs = 4_000
let authoringRuntimeHealthCache: {
  key: string
  checkedAt: number
  health: AuthoringApiRuntimeHealth
} | null = null

const assetContentTypes: Record<string, string> = {
  '.gif': 'image/gif',
  '.glb': 'model/gltf-binary',
  '.gltf': 'model/gltf+json',
  '.jpeg': 'image/jpeg',
  '.jpg': 'image/jpeg',
  '.json': 'application/json; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.mp3': 'audio/mpeg',
  '.ogg': 'audio/ogg',
  '.png': 'image/png',
  '.svg': 'image/svg+xml; charset=utf-8',
  '.wasm': 'application/wasm',
  '.wav': 'audio/wav',
  '.webp': 'image/webp',
}

function normalizeBasePath(value: string | undefined) {
  if (!value) return '/'
  if (value === './' || value.startsWith('http://') || value.startsWith('https://')) return value

  const withLeadingSlash = value.startsWith('/') ? value : `/${value}`
  return withLeadingSlash.endsWith('/') ? withLeadingSlash : `${withLeadingSlash}/`
}

function projectFiles(directory: string, routePrefix: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const entryPath = path.join(directory, entry.name)
    if (entry.isDirectory()) return projectFiles(entryPath, `${routePrefix}/${entry.name}`)
    return entry.isFile() ? [`${routePrefix}/${entry.name}`] : []
  }).sort()
}

function serveProjectFile(rootDir: string) {
  return (request: IncomingMessage, response: ServerResponse, next: (error?: unknown) => void) => {
    const requestPath = decodeURIComponent((request.url || '/').split('?')[0]).replace(/^\/+/, '')
    const filePath = path.resolve(rootDir, requestPath)
    const staysInsideRoot = filePath === rootDir || filePath.startsWith(`${rootDir}${path.sep}`)
    if (!staysInsideRoot) return next()

    try {
      if (!statSync(filePath).isFile()) return next()
      response.setHeader('Content-Type', assetContentTypes[path.extname(filePath).toLowerCase()] || 'application/octet-stream')
      response.setHeader('Cache-Control', 'no-cache')
      createReadStream(filePath).on('error', next).pipe(response)
    } catch {
      next()
    }
  }
}

function projectDataDevPlugin(): Plugin {
  let sessionApiKey = ''

  return {
    name: 'monogatari-project-data-dev',
    apply: 'serve' as const,
    configureServer(server) {
      server.middlewares.use('/authoring-inference-runtime.json', async (request, response, next) => {
        if ((request.url || '/').split('?')[0] !== '/') return next()
        const runtime = authoringApiRuntime(sessionApiKey)
        const health = runtime
          ? await authoringApiRuntimeHealth(runtime)
          : { ready: false, issue: 'upstream_unreachable' as const }
        response.setHeader('Content-Type', 'application/json; charset=utf-8')
        response.setHeader('Cache-Control', 'no-store')
        response.end(JSON.stringify(runtime ? {
          ...runtime.public,
          ...health,
        } : {
          schema: 'monogatari-authoring-inference-runtime/v1',
          provider: 'webgpu',
          ready: true,
          issue: null,
        }))
      })

      server.middlewares.use('/authoring-api/session', async (request, response, next) => {
        if ((request.url || '/').split('?')[0] !== '/' || request.method !== 'POST') return next()
        response.setHeader('Content-Type', 'application/json; charset=utf-8')
        response.setHeader('Cache-Control', 'no-store')
        if (!isTrustedAuthoringSessionRequest(request)) {
          response.statusCode = 403
          response.end(JSON.stringify({ error: { message: 'Authoring session request rejected.' } }))
          return
        }

        try {
          const body = JSON.parse((await readRequestBody(request, 16_384)).toString('utf8')) as {
            api_key?: unknown
          }
          const apiKey = typeof body.api_key === 'string' ? body.api_key.trim() : ''
          if (!apiKey || apiKey.length > 4_096) {
            response.statusCode = 400
            response.end(JSON.stringify({ error: { message: 'A bounded runtime credential is required.' } }))
            return
          }
          const runtime = authoringApiRuntime(apiKey)
          if (!runtime) {
            response.statusCode = 409
            response.end(JSON.stringify({ error: { message: 'The project API runtime is not configured.' } }))
            return
          }
          sessionApiKey = apiKey
          authoringRuntimeHealthCache = null
          const health = await authoringApiRuntimeHealth(runtime)
          response.statusCode = 200
          response.end(JSON.stringify({ ...runtime.public, ...health }))
        } catch {
          response.statusCode = 400
          response.end(JSON.stringify({ error: { message: 'Invalid authoring session request.' } }))
        }
      })

      server.middlewares.use('/authoring-api/chat/completions', async (request, response, next) => {
        if ((request.url || '/').split('?')[0] !== '/' || request.method !== 'POST') return next()
        const runtime = authoringApiRuntime(sessionApiKey)
        if (!runtime) {
          response.statusCode = 404
          response.end()
          return
        }
        try {
          const body = await readRequestBody(request, 1_000_000)
          const headers: Record<string, string> = { 'Content-Type': 'application/json' }
          if (runtime.apiKey) headers.Authorization = `Bearer ${runtime.apiKey}`
          const upstream = await fetch(`${runtime.baseUrl}/chat/completions`, {
            method: 'POST',
            headers,
            body,
          })
          response.setHeader('Cache-Control', 'no-store')
          if (upstream.ok) {
            response.statusCode = upstream.status
            response.setHeader('Content-Type', upstream.headers.get('content-type') || 'application/json; charset=utf-8')
            response.end(Buffer.from(await upstream.arrayBuffer()))
          } else {
            response.statusCode = upstream.status
            response.setHeader('Content-Type', 'application/json; charset=utf-8')
            response.end(JSON.stringify({
              error: { message: authoringProviderErrorMessage(upstream.status) },
            }))
            await upstream.body?.cancel().catch(() => undefined)
          }
        } catch (error) {
          response.statusCode = 502
          response.setHeader('Content-Type', 'application/json; charset=utf-8')
          response.end(JSON.stringify({ error: { message: `Authoring API proxy failed: ${safeErrorMessage(error)}` } }))
        }
      })

      server.middlewares.use('/project-assets.json', (request, response, next) => {
        if ((request.url || '/').split('?')[0] !== '/') return next()
        const manifest = {
          schema: 'monogatari-web-project-assets/v1',
          generated_by: 'frontend/vite.config.ts',
          project_scope: projectScope,
          assets: projectFiles(projectDataRoots.assets, '/assets'),
          event_catalogs: projectFiles(projectDataRoots.events, '/events'),
          scene_files: projectFiles(projectDataRoots.scenes, '/scenes'),
          dialogue_files: projectFiles(projectDataRoots.dialogue, '/dialogue'),
          roleplay_files: projectFiles(projectDataRoots.roleplays, '/roleplays'),
          campaign_files: projectFiles(projectDataRoots.campaigns, '/campaigns'),
          ending_files: projectFiles(projectDataRoots.endings, '/endings'),
          character_files: projectFiles(projectDataRoots.characters, '/characters'),
          knowledge_files: projectFiles(projectDataRoots.knowledge, '/knowledge'),
        }
        response.setHeader('Content-Type', 'application/json; charset=utf-8')
        response.setHeader('Cache-Control', 'no-cache')
        response.end(JSON.stringify(manifest))
      })

      for (const [route, rootDir] of Object.entries(projectDataRoots)) {
        server.middlewares.use(`/${route}`, serveProjectFile(rootDir))
      }
    },
  }
}

function authoringApiRuntime(sessionApiKey = '') {
  try {
    const settings = JSON.parse(readFileSync(projectSettingsPath, 'utf8')) as Record<string, any>
    const configured = resolveAuthoringApiRuntime(process.env, settings.ai || {})
    if (!configured) return null
    const configuredBaseUrl = configured.baseUrl.replace(/\/+$/, '')
    const parsed = new URL(configuredBaseUrl)
    if (!['http:', 'https:'].includes(parsed.protocol) || parsed.username || parsed.password) return null
    const baseUrl = parsed.pathname === '/' ? `${configuredBaseUrl}/v1` : configuredBaseUrl
    return {
      apiKey: sessionApiKey || resolveAuthoringApiKey(process.env),
      baseUrl,
      public: {
        schema: 'monogatari-authoring-inference-runtime/v1',
        provider: 'api',
        endpoint: '/authoring-api/chat/completions',
        model: configured.model,
        max_new_tokens: configured.maxNewTokens,
        temperature: configured.temperature,
        top_p: configured.topP,
      },
    }
  } catch {
    return null
  }
}

async function authoringApiRuntimeHealth(
  runtime: NonNullable<ReturnType<typeof authoringApiRuntime>>,
): Promise<AuthoringApiRuntimeHealth> {
  if (!runtime.apiKey) return classifyAuthoringApiRuntimeHealth('', null)

  const key = `${runtime.baseUrl}\n${runtime.public.model}\ncredential`
  const now = Date.now()
  if (authoringRuntimeHealthCache
    && authoringRuntimeHealthCache.key === key
    && now - authoringRuntimeHealthCache.checkedAt < authoringRuntimeHealthTtlMs) {
    return authoringRuntimeHealthCache.health
  }

  const controller = new AbortController()
  const timeout = setTimeout(() => controller.abort(), authoringRuntimeHealthTimeoutMs)
  let health = classifyAuthoringApiRuntimeHealth(runtime.apiKey, null)
  try {
    const headers: Record<string, string> = {}
    if (runtime.apiKey) headers.Authorization = `Bearer ${runtime.apiKey}`
    const response = await fetch(`${runtime.baseUrl}/models`, {
      method: 'GET',
      headers,
      signal: controller.signal,
    })
    health = classifyAuthoringApiRuntimeHealth(runtime.apiKey, response.status)
    try {
      await response.body?.cancel()
    } catch {
      // Readiness is determined by the upstream status, not body disposal.
    }
  } catch {
    health = classifyAuthoringApiRuntimeHealth(runtime.apiKey, null)
  } finally {
    clearTimeout(timeout)
  }
  authoringRuntimeHealthCache = { key, checkedAt: now, health }
  return health
}

function isTrustedAuthoringSessionRequest(request: IncomingMessage): boolean {
  if (request.headers['x-monogatari-authoring-session'] !== '1') return false
  const fetchSite = request.headers['sec-fetch-site']
  if (fetchSite && fetchSite !== 'same-origin') return false
  const origin = request.headers.origin
  const host = request.headers.host
  if (!origin || !host) return true
  try {
    return new URL(origin).host === host
  } catch {
    return false
  }
}

function authoringProviderErrorMessage(status: number): string {
  if (status === 401 || status === 403) return 'The authoring API rejected its runtime credential.'
  if (status === 404) return 'The authoring API route or model is unavailable.'
  if (status === 408 || status === 429) return 'The authoring API is temporarily unavailable.'
  return `The authoring API request failed with HTTP ${status}.`
}

function readRequestBody(request: IncomingMessage, maxBytes: number): Promise<Buffer> {
  return new Promise((resolve, reject) => {
    const chunks: Buffer[] = []
    let total = 0
    request.on('data', (chunk: Buffer) => {
      total += chunk.length
      if (total > maxBytes) {
        reject(new Error('request body exceeds 1000000 bytes'))
        request.destroy()
        return
      }
      chunks.push(chunk)
    })
    request.on('end', () => resolve(Buffer.concat(chunks)))
    request.on('error', reject)
  })
}

function safeErrorMessage(error: unknown): string {
  const message = error instanceof Error ? error.message : String(error)
  return message.replace(/Bearer\s+\S+/gi, 'Bearer <redacted>').slice(0, 500)
}

export default defineConfig({
  base: normalizeBasePath(process.env.VITE_BASE_PATH),
  plugins: [projectDataDevPlugin(), vue()],
  clearScreen: false,
  server: {
    host: mobileDevHost || false,
    port: 5173,
    strictPort: true,
    hmr: mobileDevHost
      ? {
          protocol: 'ws',
          host: mobileDevHost,
          port: 5174,
        }
      : undefined,
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: 'esnext',
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    chunkSizeWarningLimit: 900,
  },
})
