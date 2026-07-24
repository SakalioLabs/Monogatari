import { defineConfig, type Plugin } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolveAuthoringApiKey } from './src/lib/authoringRuntimeConfig'
import { createReadStream, readFileSync, readdirSync, statSync } from 'node:fs'
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
  endings: path.join(projectDataDir, 'endings'),
  characters: path.join(projectDataDir, 'characters'),
  knowledge: path.join(projectDataDir, 'knowledge'),
} as const
const projectSettingsPath = path.join(projectDataDir, 'settings.json')

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
  return {
    name: 'monogatari-project-data-dev',
    apply: 'serve' as const,
    configureServer(server) {
      server.middlewares.use('/authoring-inference-runtime.json', (request, response, next) => {
        if ((request.url || '/').split('?')[0] !== '/') return next()
        const runtime = authoringApiRuntime()
        response.setHeader('Content-Type', 'application/json; charset=utf-8')
        response.setHeader('Cache-Control', 'no-store')
        response.end(JSON.stringify(runtime?.public || {
          schema: 'monogatari-authoring-inference-runtime/v1',
          provider: 'webgpu',
        }))
      })

      server.middlewares.use('/authoring-api/chat/completions', async (request, response, next) => {
        if ((request.url || '/').split('?')[0] !== '/' || request.method !== 'POST') return next()
        const runtime = authoringApiRuntime()
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
          response.statusCode = upstream.status
          response.setHeader('Content-Type', upstream.headers.get('content-type') || 'application/json; charset=utf-8')
          response.setHeader('Cache-Control', 'no-store')
          response.end(Buffer.from(await upstream.arrayBuffer()))
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
          assets: projectFiles(projectDataRoots.assets, '/assets'),
          event_catalogs: projectFiles(projectDataRoots.events, '/events'),
          scene_files: projectFiles(projectDataRoots.scenes, '/scenes'),
          dialogue_files: projectFiles(projectDataRoots.dialogue, '/dialogue'),
          roleplay_files: projectFiles(projectDataRoots.roleplays, '/roleplays'),
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

function authoringApiRuntime() {
  try {
    const settings = JSON.parse(readFileSync(projectSettingsPath, 'utf8')) as Record<string, any>
    if (settings.ai?.provider !== 'api') return null
    const api = settings.ai.api || {}
    const configuredBaseUrl = String(api.base_url || api.baseUrl || '').trim().replace(/\/+$/, '')
    const model = String(api.model || '').trim()
    if (!configuredBaseUrl || !model) return null
    const parsed = new URL(configuredBaseUrl)
    if (!['http:', 'https:'].includes(parsed.protocol) || parsed.username || parsed.password) return null
    const baseUrl = parsed.pathname === '/' ? `${configuredBaseUrl}/v1` : configuredBaseUrl
    return {
      apiKey: resolveAuthoringApiKey(process.env),
      baseUrl,
      public: {
        schema: 'monogatari-authoring-inference-runtime/v1',
        provider: 'api',
        endpoint: '/authoring-api/chat/completions',
        model,
        max_new_tokens: Number(api.max_tokens || api.maxTokens || 256),
        temperature: Number(api.temperature || 0.7),
        top_p: Number(api.top_p || api.topP || 0.9),
      },
    }
  } catch {
    return null
  }
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
