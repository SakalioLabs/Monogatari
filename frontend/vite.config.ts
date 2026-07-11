import { defineConfig, type Plugin } from 'vite'
import vue from '@vitejs/plugin-vue'
import { createReadStream, readdirSync, statSync } from 'node:fs'
import type { IncomingMessage, ServerResponse } from 'node:http'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const mobileDevHost = process.env.TAURI_DEV_HOST
const frontendDir = fileURLToPath(new URL('.', import.meta.url))
const projectDataDir = path.resolve(frontendDir, '..', 'data')
const projectDataRoots = {
  assets: path.join(projectDataDir, 'assets'),
  events: path.join(projectDataDir, 'events'),
  scenes: path.join(projectDataDir, 'scenes'),
  dialogue: path.join(projectDataDir, 'dialogue'),
  endings: path.join(projectDataDir, 'endings'),
  characters: path.join(projectDataDir, 'characters'),
  knowledge: path.join(projectDataDir, 'knowledge'),
} as const

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
      server.middlewares.use('/project-assets.json', (request, response, next) => {
        if ((request.url || '/').split('?')[0] !== '/') return next()
        const manifest = {
          schema: 'monogatari-web-project-assets/v1',
          generated_by: 'frontend/vite.config.ts',
          assets: projectFiles(projectDataRoots.assets, '/assets'),
          event_catalogs: projectFiles(projectDataRoots.events, '/events'),
          scene_files: projectFiles(projectDataRoots.scenes, '/scenes'),
          dialogue_files: projectFiles(projectDataRoots.dialogue, '/dialogue'),
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
