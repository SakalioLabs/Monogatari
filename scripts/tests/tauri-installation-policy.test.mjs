import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import { collectTauriInstallationPolicyEvidence } from '../lib/tauri-packaging/installation-policy.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')
const tauriAppDirectory = path.join(repositoryRoot, 'rust-engine', 'crates', 'tauri-app')
const boundaries = {
  repositoryRoot,
  tauriAppDirectory,
}

test('checked-in installed-runtime and installer audit policy returns passing evidence', async () => {
  const evidence = await collectTauriInstallationPolicyEvidence(boundaries)
  assert.deepEqual(evidence.issues, [])
  assert.equal(evidence.requirementCount, 32)
})

test('injected startup, runtime, and signature drift stays independently actionable', async () => {
  const mainPath = path.join(tauriAppDirectory, 'src', 'main.rs')
  const installationVerifierPath = path.join(
    tauriAppDirectory,
    'src',
    'installation_verifier.rs',
  )
  const windowsVerifierPath = path.join(repositoryRoot, 'scripts', 'verify-windows-installers.mjs')
  const evidence = await collectTauriInstallationPolicyEvidence({
    ...boundaries,
    async readTextFile(filePath, encoding) {
      const source = await readFile(filePath, encoding)
      const resolved = path.resolve(filePath)
      if (resolved === mainPath) {
        return source.replaceAll(
          'installation_verifier::run_requested_verification()',
          'installation_verifier::run_drifted_verification()',
        )
      }
      if (resolved === installationVerifierPath) {
        return source.replaceAll('--verify-installation', '--audit-installation')
      }
      if (resolved === windowsVerifierPath) {
        return source.replaceAll('Get-AuthenticodeSignature', 'Get-DriftedSignature')
      }
      return source
    },
  })

  assert(
    evidence.issues.includes(
      'Installed desktop verification must run headless installation verification before opening Tauri',
    ),
  )
  assert(
    evidence.issues.includes(
      'Installed desktop verification must expose an explicit installed-runtime verification flag',
    ),
  )
  assert(
    evidence.issues.includes(
      'Installed desktop verification must inspect real Authenticode status',
    ),
  )
})

test('installation policy requires both filesystem boundaries', async () => {
  await assert.rejects(
    () => collectTauriInstallationPolicyEvidence(),
    /requires repositoryRoot/,
  )
  await assert.rejects(
    () => collectTauriInstallationPolicyEvidence({ repositoryRoot }),
    /requires tauriAppDirectory/,
  )
})
