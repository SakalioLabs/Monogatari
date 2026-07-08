import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

const mobileDevHost = process.env.TAURI_DEV_HOST

function normalizeBasePath(value: string | undefined) {
  if (!value) return '/'
  if (value === './' || value.startsWith('http://') || value.startsWith('https://')) return value

  const withLeadingSlash = value.startsWith('/') ? value : `/${value}`
  return withLeadingSlash.endsWith('/') ? withLeadingSlash : `${withLeadingSlash}/`
}

export default defineConfig({
  base: normalizeBasePath(process.env.VITE_BASE_PATH),
  plugins: [vue()],
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
