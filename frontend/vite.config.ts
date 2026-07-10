import { defineConfig, type Plugin } from 'vite'
import vue from '@vitejs/plugin-vue'
import { createReadStream, statSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const mobileDevHost = process.env.TAURI_DEV_HOST
const frontendDir = fileURLToPath(new URL('.', import.meta.url))
const projectAssetDir = path.resolve(frontendDir, '..', 'data', 'assets')

const assetContentTypes: Record<string, string> = {
  '.gif': 'image/gif',
  '.glb': 'model/gltf-binary',
  '.gltf': 'model/gltf+json',
  '.jpeg': 'image/jpeg',
  '.jpg': 'image/jpeg',
  '.json': 'application/json; charset=utf-8',
  '.mp3': 'audio/mpeg',
  '.ogg': 'audio/ogg',
  '.png': 'image/png',
  '.svg': 'image/svg+xml; charset=utf-8',
  '.wav': 'audio/wav',
  '.webp': 'image/webp',
}

function normalizeBasePath(value: string | undefined) {
  if (!value) return '/'
  if (value === './' || value.startsWith('http://') || value.startsWith('https://')) return value

  const withLeadingSlash = value.startsWith('/') ? value : `/${value}`
  return withLeadingSlash.endsWith('/') ? withLeadingSlash : `${withLeadingSlash}/`
}

function projectAssetsDevPlugin(): Plugin {
  return {
    name: 'monogatari-project-assets-dev',
    apply: 'serve' as const,
    configureServer(server) {
      server.middlewares.use('/assets', (request, response, next) => {
        const requestPath = decodeURIComponent((request.url || '/').split('?')[0]).replace(/^\/+/, '')
        const filePath = path.resolve(projectAssetDir, requestPath)
        const staysInsideAssetRoot = filePath === projectAssetDir || filePath.startsWith(`${projectAssetDir}${path.sep}`)
        if (!staysInsideAssetRoot) return next()

        try {
          if (!statSync(filePath).isFile()) return next()
          response.setHeader('Content-Type', assetContentTypes[path.extname(filePath).toLowerCase()] || 'application/octet-stream')
          response.setHeader('Cache-Control', 'no-cache')
          createReadStream(filePath).on('error', next).pipe(response)
        } catch {
          next()
        }
      })
    },
  }
}

export default defineConfig({
  base: normalizeBasePath(process.env.VITE_BASE_PATH),
  plugins: [projectAssetsDevPlugin(), vue()],
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
