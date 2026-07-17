import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import {
  e2eHost,
  e2ePort,
  startE2eServer,
} from '../../frontend/e2e/server-lifecycle.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')

test('E2E server lifecycle starts Vite in-process and closes exactly once', async () => {
  const calls = []
  const lifecycle = await startE2eServer({
    host: '127.0.0.2',
    port: 4318,
    async createServer(config) {
      calls.push({ action: 'create', config })
      return {
        async listen() {
          calls.push({ action: 'listen' })
        },
        async close() {
          calls.push({ action: 'close' })
        },
      }
    },
  })

  assert.equal(lifecycle.origin, 'http://127.0.0.2:4318')
  assert.equal(calls[0].config.root, path.join(repositoryRoot, 'frontend'))
  assert.equal(calls[0].config.configFile, path.join(repositoryRoot, 'frontend', 'vite.config.ts'))
  assert.deepEqual(calls[0].config.server, {
    host: '127.0.0.2',
    port: 4318,
    strictPort: true,
  })
  assert.deepEqual(calls.slice(1), [{ action: 'listen' }])

  await lifecycle.close()
  await lifecycle.close()
  assert.deepEqual(calls.slice(1), [{ action: 'listen' }, { action: 'close' }])
})

test('E2E server lifecycle closes a partially started Vite server', async () => {
  let closeCount = 0
  await assert.rejects(
    startE2eServer({
      async createServer() {
        return {
          async listen() {
            throw new Error('port unavailable')
          },
          async close() {
            closeCount += 1
          },
        }
      },
    }),
    /port unavailable/,
  )
  assert.equal(closeCount, 1)
})

test('Playwright delegates server ownership without a shell child process', async () => {
  const config = await readFile(path.join(repositoryRoot, 'frontend', 'playwright.config.ts'), 'utf8')
  const setup = await readFile(path.join(repositoryRoot, 'frontend', 'e2e', 'global-setup.mjs'), 'utf8')
  const lifecycle = await readFile(path.join(repositoryRoot, 'frontend', 'e2e', 'server-lifecycle.mjs'), 'utf8')

  assert(config.includes("globalSetup: './e2e/global-setup.mjs'"))
  assert(!config.includes('webServer:'))
  assert(!config.includes('npm run dev'))
  assert(setup.includes('startE2eServer'))
  assert(lifecycle.includes("await import('vite')"))
  assert(!lifecycle.includes("from 'vite'"))
  assert.equal(e2eHost, '127.0.0.1')
  assert.equal(e2ePort, 4317)
})
