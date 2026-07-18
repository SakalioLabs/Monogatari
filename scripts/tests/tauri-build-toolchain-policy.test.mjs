import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import { collectTauriBuildToolchainEvidence } from '../lib/tauri-packaging/build-toolchain-policy.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')
const rustDirectory = path.join(repositoryRoot, 'rust-engine')
const tauriAppDirectory = path.join(rustDirectory, 'crates', 'tauri-app')
const boundaries = { repositoryRoot, rustDirectory, tauriAppDirectory }

test('checked-in Tauri build metadata and Rust toolchain return passing evidence', async () => {
  const evidence = await collectTauriBuildToolchainEvidence(boundaries)
  assert.deepEqual(evidence.issues, [])
  assert.deepEqual(evidence.requirementCounts, {
    buildMetadata: 4,
    rustToolchain: 6,
  })
  assert.equal(evidence.structuralCheckCount, 1)
})

test('build commit, pinned toolchain, and release environment drift stays actionable', async () => {
  const buildPath = path.join(tauriAppDirectory, 'build.rs')
  const toolchainPath = path.join(rustDirectory, 'rust-toolchain.toml')
  const releaseVerifierPath = path.join(repositoryRoot, 'scripts', 'verify-release.mjs')
  const forbiddenTestProfileOverride = ['CARGO', 'PROFILE', 'TEST', 'DEBUG'].join('_')
  const evidence = await collectTauriBuildToolchainEvidence({
    ...boundaries,
    async readTextFile(filePath, encoding) {
      const source = await readFile(filePath, encoding)
      const resolved = path.resolve(filePath)
      if (resolved === buildPath) {
        return source.replaceAll(
          'cargo:rustc-env=MONOGATARI_GIT_COMMIT',
          'cargo:rustc-env=DRIFTED_GIT_COMMIT',
        )
      }
      if (resolved === toolchainPath) {
        return source.replace('nightly-2026-07-03', 'nightly')
      }
      if (resolved === releaseVerifierPath) {
        return [
          source
            .replace(
              "const rustVerificationEnv = Object.freeze({ CARGO_INCREMENTAL: '0' })",
              'const rustVerificationEnv = Object.freeze({})',
            )
            .replace('async function runRustVerification(', 'async function runUnstableRustVerification(')
            .replace('env: { ...(options.env ?? {}), ...rustVerificationEnv }', 'env: {}'),
          `const ${forbiddenTestProfileOverride} = '1'`,
          '',
        ].join('\\n')
      }
      return source
    },
  })

  for (const issue of [
    'Tauri build metadata must inject the build git commit into the Tauri binary',
    'Rust release toolchain must pin the verified Rust nightly by exact date',
    'Rust release toolchain must disable incremental compilation across every Rust release gate',
    'Rust release toolchain must centralize Rust release command execution',
    'Rust release toolchain must enforce the shared Rust release environment',
    'Rust release verification must not override the Tauri test debug-profile environment',
  ]) {
    assert(evidence.issues.includes(issue), issue)
  }
})

test('build and toolchain policy requires every filesystem boundary', async () => {
  await assert.rejects(
    () => collectTauriBuildToolchainEvidence(),
    /requires repositoryRoot/,
  )
  await assert.rejects(
    () => collectTauriBuildToolchainEvidence({ repositoryRoot }),
    /requires rustDirectory/,
  )
  await assert.rejects(
    () => collectTauriBuildToolchainEvidence({ repositoryRoot, rustDirectory }),
    /requires tauriAppDirectory/,
  )
})
