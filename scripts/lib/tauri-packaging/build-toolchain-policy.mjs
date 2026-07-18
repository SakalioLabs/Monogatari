import { readFile as readFileFromDisk } from 'node:fs/promises'
import path from 'node:path'

const buildMetadataRequirements = [
  ['cargo:rustc-env=MONOGATARI_GIT_COMMIT', 'inject the build git commit into the Tauri binary'],
  ['cargo:rustc-env=MONOGATARI_GIT_SHORT_COMMIT', 'inject a short build git commit into the Tauri binary'],
  ['rev-parse', 'derive build commit metadata from git'],
  ['symbolic-ref', 'rerun the build script when the current branch ref changes'],
]

const rustToolchainRequirements = [
  ['toolchain', 'channel = "nightly-2026-07-03"', 'pin the verified Rust nightly by exact date'],
  ['toolchain', 'profile = "minimal"', 'keep release toolchain installation minimal'],
  ['toolchain', 'components = ["clippy", "rustfmt"]', 'install the linter and formatter used by release verification'],
  ['releaseVerifier', "const rustVerificationEnv = Object.freeze({ CARGO_INCREMENTAL: '0' })", 'disable incremental compilation across every Rust release gate'],
  ['releaseVerifier', 'async function runRustVerification(', 'centralize Rust release command execution'],
  ['releaseVerifier', 'env: { ...(options.env ?? {}), ...rustVerificationEnv }', 'enforce the shared Rust release environment'],
]

export async function collectTauriBuildToolchainEvidence(options = {}) {
  const { repositoryRoot, rustDirectory, tauriAppDirectory } = resolveBoundaries(options)
  const readFile = options.readTextFile ?? readFileFromDisk
  const sources = {
    build: await readFile(path.join(tauriAppDirectory, 'build.rs'), 'utf8'),
    toolchain: await readFile(path.join(rustDirectory, 'rust-toolchain.toml'), 'utf8'),
    releaseVerifier: await readFile(path.join(repositoryRoot, 'scripts', 'verify-release.mjs'), 'utf8'),
  }
  const issues = []

  appendRequirements(
    sources.build,
    buildMetadataRequirements,
    'Tauri build metadata',
    issues,
  )
  for (const [sourceName, needle, description] of rustToolchainRequirements) {
    if (!sources[sourceName].includes(needle)) {
      issues.push(`Rust release toolchain must ${description}`)
    }
  }

  const forbiddenTestProfileOverride = ['CARGO', 'PROFILE', 'TEST', 'DEBUG'].join('_')
  if (sources.releaseVerifier.includes(forbiddenTestProfileOverride)) {
    issues.push('Rust release verification must not override the Tauri test debug-profile environment')
  }

  return {
    issues,
    requirementCounts: {
      buildMetadata: buildMetadataRequirements.length,
      rustToolchain: rustToolchainRequirements.length,
    },
    structuralCheckCount: 1,
  }
}

function appendRequirements(source, requirements, label, issues) {
  for (const [needle, description] of requirements) {
    if (!source.includes(needle)) issues.push(`${label} must ${description}`)
  }
}

function resolveBoundaries(options) {
  const boundaries = {
    repositoryRoot: options.repositoryRoot,
    rustDirectory: options.rustDirectory,
    tauriAppDirectory: options.tauriAppDirectory,
  }
  for (const [name, value] of Object.entries(boundaries)) {
    if (typeof value !== 'string' || value.length === 0) {
      throw new Error(`Tauri build/toolchain policy requires ${name}.`)
    }
  }
  return boundaries
}
