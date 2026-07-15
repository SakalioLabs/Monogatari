import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import { collectTauriPackagePolicyEvidence } from '../lib/tauri-packaging/package-policy.mjs'

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

test('checked-in Tauri package and mobile policy returns structured passing evidence', async () => {
  const evidence = await collectTauriPackagePolicyEvidence(boundaries)
  assert.deepEqual(evidence.issues, [])
  assert.deepEqual(evidence.targets, ['msi', 'nsis'])
  assert.equal(evidence.iconCount, 5)
})

test('injected package and mobile drift returns independent actionable issues', async () => {
  const configPath = path.join(tauriAppDirectory, 'tauri.conf.json')
  const viteConfigPath = path.join(frontendDirectory, 'vite.config.ts')
  const evidence = await collectTauriPackagePolicyEvidence({
    ...boundaries,
    async readTextFile(filePath, encoding) {
      const source = await readFile(filePath, encoding)
      if (path.resolve(filePath) === viteConfigPath) {
        return source.replace('host: mobileDevHost || false', 'host: false')
      }
      if (path.resolve(filePath) !== configPath) return source
      const config = JSON.parse(source)
      config.productName = 'Drifted Product'
      config.bundle.targets = ['nsis']
      config.bundle.icon = []
      return JSON.stringify(config)
    },
  })

  assert(
    evidence.issues.includes(
      'tauri.conf.json productName must stay Monogatari for installer identity',
    ),
  )
  assert(
    evidence.issues.includes(
      'tauri.conf.json bundle.targets must include msi for Windows installer coverage',
    ),
  )
  assert(evidence.issues.includes('tauri.conf.json bundle.icon must include icons/icon.ico'))
  assert(
    evidence.issues.includes(
      'Tauri mobile preflight must bind Vite to the Tauri-selected mobile host',
    ),
  )
})

test('package policy requires every repository filesystem boundary', async () => {
  await assert.rejects(() => collectTauriPackagePolicyEvidence(), /requires repositoryRoot/)
  await assert.rejects(
    () =>
      collectTauriPackagePolicyEvidence({
        repositoryRoot,
        frontendDirectory,
        rustDirectory,
      }),
    /requires tauriAppDirectory/,
  )
})
