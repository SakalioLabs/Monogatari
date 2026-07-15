import assert from 'node:assert/strict'
import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises'
import os from 'node:os'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import {
  collectWebDistributionEvidence,
  createWebDistributionVerifier,
  releaseSubpathBase,
  requiredLocaleFiles,
  verifyIndexAssetBase,
} from '../lib/web-distribution-verifier.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')

test('enforces root and subpath asset URL contracts independently', () => {
  assert.deepEqual(verifyIndexAssetBase(
    '<link href="/manifest.webmanifest"><script src="/assets/app.js"></script>',
    '/',
  ), [])
  assert.deepEqual(verifyIndexAssetBase(
    '<link href="/Monogatari/manifest.webmanifest"><link href="/Monogatari/favicon.svg"><script src="/Monogatari/assets/app.js"></script>',
    releaseSubpathBase,
  ), [])

  const rootIssues = verifyIndexAssetBase('<script src="/Monogatari/assets/app.js"></script>', '/')
  assert(rootIssues.some((issue) => issue.includes('root build must not retain subpath asset URL')))
  const subpathIssues = verifyIndexAssetBase('<script src="/assets/app.js"></script>', releaseSubpathBase)
  assert(subpathIssues.some((issue) => issue.includes('/Monogatari/manifest.webmanifest')))
  assert(subpathIssues.some((issue) => issue.includes('root-relative URL outside /Monogatari/')))
})

test('empty distribution fixtures return complete failure evidence without throwing', async () => {
  const fixture = await mkdtemp(path.join(os.tmpdir(), 'monogatari-web-dist-'))
  const frontendDirectory = path.join(fixture, 'frontend')
  await mkdir(path.join(frontendDirectory, 'dist', 'assets'), { recursive: true })
  await writeFile(path.join(frontendDirectory, 'dist', 'manifest.webmanifest'), JSON.stringify({
    icons: {},
    shortcuts: {},
  }))
  await writeFile(path.join(frontendDirectory, 'dist', 'vercel.json'), '{ invalid')
  try {
    const evidence = await collectWebDistributionEvidence({
      repositoryRoot: fixture,
      frontendDirectory,
      basePath: releaseSubpathBase,
    })
    assert.equal(evidence.normalizedBase, releaseSubpathBase)
    assert.equal(evidence.checkedFileCount, 17)
    assert(evidence.issues.includes('Missing Web/PWA dist asset: index.html'))
    assert(evidence.issues.includes('Missing Web/PWA inference runtime contract: inference-runtime.json'))
    assert(evidence.issues.includes('manifest.webmanifest icons must be an array'))
    assert(evidence.issues.includes('manifest.webmanifest shortcuts must be an array'))
    assert(evidence.issues.some((issue) => issue.startsWith(
      'vercel.json must contain valid JSON:',
    )))
    assert(evidence.issues.includes('data/assets must exist for Web/PWA project asset packaging'))
    assert(requiredLocaleFiles.every((locale) => evidence.issues.includes(
      `Missing Web/PWA locale fallback: ${locale}`,
    )))
  } finally {
    await rm(fixture, { recursive: true, force: true })
  }
})

test('requires explicit repository and frontend filesystem boundaries', () => {
  assert.throws(() => createWebDistributionVerifier(), /requires a repository root/)
  assert.throws(
    () => createWebDistributionVerifier({ repositoryRoot }),
    /requires a frontend directory/,
  )
})

test('release runner delegates static distribution ownership to the importable module', async () => {
  const source = await readFile(path.join(repositoryRoot, 'scripts', 'verify-release.mjs'), 'utf8')
  const moduleSource = await readFile(
    path.join(repositoryRoot, 'scripts', 'lib', 'web-distribution-verifier.mjs'),
    'utf8',
  )

  assert(source.includes("from './lib/web-distribution-verifier.mjs'"))
  assert(source.includes('createWebDistributionVerifier({'))
  assert(!source.includes('async function verifyWebDist'))
  assert(!source.includes('async function verifyWebProjectAssets'))
  assert(!source.includes('function verifyIndexAssetBase'))
  assert(moduleSource.includes("from './web-hosting-verifier.mjs'"))
  assert(moduleSource.includes('collectWebDistributionEvidence'))
})
