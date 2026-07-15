import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import {
  collectTauriPackagingEvidence,
  createTauriPackagingVerifier,
} from '../lib/tauri-packaging-verifier.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')
const frontendDirectory = path.join(repositoryRoot, 'frontend')
const rustDirectory = path.join(repositoryRoot, 'rust-engine')
const tauriAppDirectory = path.join(rustDirectory, 'crates', 'tauri-app')
const boundaries = {
  repositoryRoot,
  frontendDirectory,
  rustDirectory,
  tauriAppDirectory,
}

test('checked-in desktop packaging and command contracts return structured passing evidence', async () => {
  const evidence = await collectTauriPackagingEvidence(boundaries)
  assert.deepEqual(evidence.issues, [])
  assert.deepEqual(evidence.targets, ['msi', 'nsis'])
  assert.equal(evidence.iconCount, 5)
})

test('requires every repository filesystem boundary before reading', () => {
  assert.throws(() => createTauriPackagingVerifier(), /requires repositoryRoot/)
  assert.throws(
    () => createTauriPackagingVerifier({
      repositoryRoot,
      frontendDirectory,
      rustDirectory,
    }),
    /requires tauriAppDirectory/,
  )
})

test('release runner delegates Tauri packaging evidence to the importable module', async () => {
  const source = await readFile(path.join(repositoryRoot, 'scripts', 'verify-release.mjs'), 'utf8')
  const moduleSource = await readFile(
    path.join(repositoryRoot, 'scripts', 'lib', 'tauri-packaging-verifier.mjs'),
    'utf8',
  )
  const packagePolicySource = await readFile(
    path.join(repositoryRoot, 'scripts', 'lib', 'tauri-packaging', 'package-policy.mjs'),
    'utf8',
  )

  assert(source.includes("from './lib/tauri-packaging-verifier.mjs'"))
  assert(source.includes('createTauriPackagingVerifier({'))
  assert(!source.includes('async function verifyTauriPackagingConfig'))
  assert(!source.includes('const installationVerificationRequirements'))
  assert(moduleSource.includes('collectTauriPackagingEvidence'))
  assert(moduleSource.includes('collectTauriPackagePolicyEvidence'))
  assert(!moduleSource.includes('config.productName'))
  assert(moduleSource.includes('const installationVerificationRequirements'))
  assert(packagePolicySource.includes('config.productName'))
})
