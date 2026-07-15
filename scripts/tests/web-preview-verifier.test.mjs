import assert from 'node:assert/strict'
import { EventEmitter } from 'node:events'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import { PassThrough } from 'node:stream'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import {
  collectWebPreviewEvidence,
  createWebPreviewVerifier,
  normalizeWebBasePath,
  previewUrl,
} from '../lib/web-preview-verifier.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')

test('normalizes Web base paths and builds scoped preview URLs', () => {
  assert.equal(normalizeWebBasePath(), '/')
  assert.equal(normalizeWebBasePath('./'), '/')
  assert.equal(normalizeWebBasePath('Monogatari'), '/Monogatari/')
  assert.equal(normalizeWebBasePath('/Monogatari/'), '/Monogatari/')
  assert.equal(normalizeWebBasePath('https://example.test/app/'), 'https://example.test/app/')
  assert.equal(previewUrl(4187, '/', '/workspace'), 'http://127.0.0.1:4187/workspace')
  assert.equal(
    previewUrl(4187, '/Monogatari/', 'workspace'),
    'http://127.0.0.1:4187/Monogatari/workspace',
  )
})

test('collects successful route and project-content preview evidence without starting Vite', async () => {
  const evidence = await collectWebPreviewEvidence({
    port: 4187,
    basePath: '/Monogatari/',
    routes: [{ path: '/' }, { path: '/workspace' }],
    fetchImpl: async (url) => successfulResponse(new URL(url).pathname),
  })

  assert.equal(evidence.normalizedBase, '/Monogatari/')
  assert.equal(evidence.routeCount, 2)
  assert.deepEqual(evidence.issues, [])
})

test('returns actionable preview response failures as evidence', async () => {
  const evidence = await collectWebPreviewEvidence({
    port: 4187,
    routes: [{ path: '/workspace' }],
    fetchImpl: async () => response({
      status: 503,
      contentType: 'text/plain',
      body: 'unavailable',
      json: {},
    }),
  })

  assert(evidence.issues.includes('/workspace: expected HTTP 200 from preview, got 503'))
  assert(evidence.issues.includes('/workspace: expected text/html from preview, got text/plain'))
  assert(evidence.issues.includes('/workspace: preview response did not include the Vue app mount point'))
  assert(evidence.issues.includes('story events: expected HTTP 200 application/json, got 503 text/plain'))
  assert(evidence.issues.includes('story events: preview catalog schema is invalid'))
  assert(evidence.issues.includes('/scenes/sakura_park.json: project content preview response is invalid'))
})

test('requires an explicit frontend process boundary', () => {
  assert.throws(() => createWebPreviewVerifier(), /requires a frontend directory/)
  assert.throws(
    () => createWebPreviewVerifier({ frontendDirectory: 'frontend', fetchImpl: null }),
    /requires a fetch implementation/,
  )
})

test('fails promptly with bounded process output when Vite exits before readiness', async () => {
  const child = new EventEmitter()
  child.stdout = new PassThrough()
  child.stderr = new PassThrough()
  child.exitCode = null
  child.signalCode = null
  child.kill = () => true
  const verify = createWebPreviewVerifier({
    frontendDirectory: 'frontend',
    logger: { log() {} },
    spawnProcess: () => child,
    fetchImpl: async () => {
      child.stderr.write('vite startup failed')
      child.exitCode = 1
      return { ok: false }
    },
  })

  await assert.rejects(
    verify(),
    (error) => error.message.includes('exit code 1') && error.message.includes('vite startup failed'),
  )
})

test('release runner delegates live preview ownership to the importable module', async () => {
  const source = await readFile(path.join(repositoryRoot, 'scripts', 'verify-release.mjs'), 'utf8')
  const moduleSource = await readFile(
    path.join(repositoryRoot, 'scripts', 'lib', 'web-preview-verifier.mjs'),
    'utf8',
  )

  assert(source.includes("from './lib/web-preview-verifier.mjs'"))
  assert(source.includes('createWebPreviewVerifier({ frontendDirectory: frontendDir })'))
  assert(!source.includes('async function waitForPreview'))
  assert(!source.includes("viteBin,\n    'preview'"))
  assert(moduleSource.includes('collectWebPreviewEvidence'))
  assert(moduleSource.includes('maxPreviewOutputCharacters'))
})

function successfulResponse(pathname) {
  if (pathname.endsWith('/events/story_events.json')) {
    return response({
      contentType: 'application/json',
      json: { schema: 'monogatari-story-event-catalog/v1' },
    })
  }
  if (pathname.endsWith('/scenes/sakura_park.json')) {
    return response({ contentType: 'application/json', json: { id: 'sakura_park' } })
  }
  if (pathname.endsWith('/dialogue/sakura_park_walk.json')) {
    return response({ contentType: 'application/json', json: { id: 'sakura_park_walk' } })
  }
  if (pathname.endsWith('/endings/best_friend_ending.json')) {
    return response({
      contentType: 'application/json',
      json: { schema: 'monogatari-story-ending/v1' },
    })
  }
  return response({ contentType: 'text/html; charset=utf-8', body: '<div id="app"></div>' })
}

function response({ status = 200, contentType, body = '', json = null }) {
  return {
    status,
    ok: status >= 200 && status < 300,
    headers: {
      get(name) {
        return name.toLowerCase() === 'content-type' ? contentType : null
      },
    },
    async text() {
      return body
    },
    async json() {
      return json
    },
  }
}
