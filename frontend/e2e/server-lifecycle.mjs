import path from 'node:path'
import { fileURLToPath } from 'node:url'

import { createServer as createViteServer } from 'vite'

export const e2eHost = '127.0.0.1'
export const e2ePort = 4317

const frontendDirectory = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')

export async function startE2eServer(options = {}) {
  const createServer = options.createServer ?? createViteServer
  const host = options.host ?? e2eHost
  const port = options.port ?? e2ePort
  const server = await createServer({
    configFile: path.join(frontendDirectory, 'vite.config.ts'),
    root: frontendDirectory,
    logLevel: 'error',
    server: {
      host,
      port,
      strictPort: true,
    },
  })

  try {
    await server.listen()
  } catch (error) {
    await server.close().catch(() => {})
    throw error
  }

  let closed = false
  return Object.freeze({
    origin: `http://${host}:${port}`,
    async close() {
      if (closed) return
      closed = true
      await server.close()
    },
  })
}
