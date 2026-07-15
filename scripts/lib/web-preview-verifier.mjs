import { spawn } from 'node:child_process'
import { createServer } from 'node:net'
import path from 'node:path'

import { expectedFrontendRoutes } from './frontend-route-verifier.mjs'
import { normalizeWebBasePath } from './web-base-path.mjs'

const previewContentChecks = Object.freeze([
  ['/scenes/sakura_park.json', 'id', 'sakura_park'],
  ['/dialogue/sakura_park_walk.json', 'id', 'sakura_park_walk'],
  ['/endings/best_friend_ending.json', 'schema', 'monogatari-story-ending/v1'],
])
const maxPreviewOutputCharacters = 64 * 1024

export function previewUrl(port, basePath, routePath) {
  const pathBase = basePath === '/' ? '' : basePath.replace(/\/$/, '')
  const normalizedRoute = routePath === '/'
    ? '/'
    : `/${String(routePath).replace(/^\/+/, '')}`
  return `http://127.0.0.1:${port}${pathBase}${normalizedRoute}`
}

export async function collectWebPreviewEvidence({
  port,
  basePath = '/',
  routes = expectedFrontendRoutes,
  fetchImpl = globalThis.fetch,
} = {}) {
  const normalizedBase = normalizeWebBasePath(basePath)
  const issues = []

  for (const route of routes) {
    const routeUrl = previewUrl(port, normalizedBase, route.path)
    let response
    let body
    try {
      response = await fetchImpl(routeUrl)
      body = await response.text()
    } catch (error) {
      issues.push(`${route.path}: preview request failed: ${error.message}`)
      continue
    }

    const contentType = response.headers.get('content-type') ?? ''
    if (response.status !== 200) {
      issues.push(`${route.path}: expected HTTP 200 from preview, got ${response.status}`)
    }
    if (!contentType.includes('text/html')) {
      issues.push(`${route.path}: expected text/html from preview, got ${contentType || '<missing content-type>'}`)
    }
    if (!body.includes('<div id="app">')) {
      issues.push(`${route.path}: preview response did not include the Vue app mount point`)
    }
  }

  try {
    const eventResponse = await fetchImpl(previewUrl(port, normalizedBase, '/events/story_events.json'))
    const eventContentType = eventResponse.headers.get('content-type') ?? ''
    const eventCatalog = await eventResponse.json()
    if (eventResponse.status !== 200 || !eventContentType.includes('application/json')) {
      issues.push(`story events: expected HTTP 200 application/json, got ${eventResponse.status} ${eventContentType}`)
    }
    if (eventCatalog?.schema !== 'monogatari-story-event-catalog/v1') {
      issues.push('story events: preview catalog schema is invalid')
    }
  } catch (error) {
    issues.push(`story events: preview request failed: ${error.message}`)
  }

  for (const [contentPath, field, expectedValue] of previewContentChecks) {
    try {
      const response = await fetchImpl(previewUrl(port, normalizedBase, contentPath))
      const payload = await response.json()
      if (response.status !== 200 || payload?.[field] !== expectedValue) {
        issues.push(`${contentPath}: project content preview response is invalid`)
      }
    } catch (error) {
      issues.push(`${contentPath}: preview request failed: ${error.message}`)
    }
  }

  return {
    normalizedBase,
    routeCount: routes.length,
    issues,
  }
}

export function createWebPreviewVerifier({
  frontendDirectory,
  logger = console,
  fetchImpl = globalThis.fetch,
  spawnProcess = spawn,
  nodeExecutable = process.execPath,
  environment = process.env,
} = {}) {
  if (typeof frontendDirectory !== 'string' || frontendDirectory.length === 0) {
    throw new Error('Web preview verifier requires a frontend directory.')
  }
  if (typeof fetchImpl !== 'function') {
    throw new Error('Web preview verifier requires a fetch implementation.')
  }

  return async function verifyWebPreview({ basePath = '/', env = {} } = {}) {
    const normalizedBase = normalizeWebBasePath(basePath)
    const port = await findOpenPort()
    const viteBin = path.join(frontendDirectory, 'node_modules', 'vite', 'bin', 'vite.js')
    const child = spawnProcess(nodeExecutable, [
      viteBin,
      'preview',
      '--host',
      '127.0.0.1',
      '--port',
      String(port),
      '--strictPort',
    ], {
      cwd: frontendDirectory,
      env: { ...environment, ...env },
      stdio: ['ignore', 'pipe', 'pipe'],
    })

    let output = ''
    let startupError = null
    const appendOutput = (chunk) => {
      output = `${output}${chunk.toString()}`.slice(-maxPreviewOutputCharacters)
    }
    child.stdout?.on('data', appendOutput)
    child.stderr?.on('data', appendOutput)
    child.once('error', (error) => { startupError = error })

    try {
      await waitForPreview({
        port,
        basePath: normalizedBase,
        fetchImpl,
        output: () => output,
        startupError: () => startupError,
        processStatus: () => ({
          exitCode: child.exitCode,
          signalCode: child.signalCode,
        }),
      })
      const evidence = await collectWebPreviewEvidence({
        port,
        basePath: normalizedBase,
        fetchImpl,
      })
      if (evidence.issues.length > 0) {
        throw new Error(
          `Web/PWA preview smoke failed (${normalizedBase} base):\n${evidence.issues.join('\n')}\n${output}`,
        )
      }

      logger.log(
        `[release] Web/PWA preview smoke OK (${normalizedBase} base, ${evidence.routeCount} route(s))`,
      )
      return evidence
    } finally {
      await stopPreview(child)
    }
  }
}

async function findOpenPort(start = 4187) {
  for (let port = start; port < start + 100; port += 1) {
    if (await canListen(port)) return port
  }
  throw new Error(`Could not find an open preview port starting at ${start}`)
}

async function canListen(port) {
  return await new Promise((resolve) => {
    const server = createServer()
    server.once('error', () => resolve(false))
    server.once('listening', () => {
      server.close(() => resolve(true))
    })
    server.listen(port, '127.0.0.1')
  })
}

async function waitForPreview({
  port,
  basePath,
  fetchImpl,
  output,
  startupError,
  processStatus,
}) {
  const routeUrl = previewUrl(port, basePath, '/')
  for (let attempt = 0; attempt < 80; attempt += 1) {
    const error = startupError()
    if (error) {
      throw new Error(`Web/PWA preview process failed to start: ${error.message}\n${output()}`)
    }
    const status = processStatus()
    if (status.exitCode !== null || status.signalCode !== null) {
      const reason = status.exitCode !== null
        ? `exit code ${status.exitCode}`
        : `signal ${status.signalCode}`
      throw new Error(`Web/PWA preview process stopped before readiness with ${reason}\n${output()}`)
    }
    try {
      const response = await fetchImpl(routeUrl)
      if (response.ok) return
    } catch {}
    await delay(250)
  }
  throw new Error(`Web/PWA preview did not become ready at ${routeUrl}\n${output()}`)
}

async function stopPreview(child) {
  if (childStopped(child)) return

  child.kill('SIGTERM')
  for (let attempt = 0; attempt < 20; attempt += 1) {
    if (childStopped(child)) return
    await delay(100)
  }
  child.kill('SIGKILL')
}

function childStopped(child) {
  return child.exitCode !== null || child.signalCode !== null
}

function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms))
}
