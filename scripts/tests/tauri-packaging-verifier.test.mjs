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
  const buildToolchainSource = await readFile(
    path.join(
      repositoryRoot,
      'scripts',
      'lib',
      'tauri-packaging',
      'build-toolchain-policy.mjs',
    ),
    'utf8',
  )
  const installationPolicySource = await readFile(
    path.join(repositoryRoot, 'scripts', 'lib', 'tauri-packaging', 'installation-policy.mjs'),
    'utf8',
  )
  const commandRegistrationSource = await readFile(
    path.join(
      repositoryRoot,
      'scripts',
      'lib',
      'tauri-packaging',
      'command-registration-policy.mjs',
    ),
    'utf8',
  )
  const conversationSafetySource = await readFile(
    path.join(
      repositoryRoot,
      'scripts',
      'lib',
      'tauri-packaging',
      'conversation-safety-policy.mjs',
    ),
    'utf8',
  )
  const qualityWorkflowSource = await readFile(
    path.join(
      repositoryRoot,
      'scripts',
      'lib',
      'tauri-packaging',
      'quality-workflow-policy.mjs',
    ),
    'utf8',
  )
  const projectPackageSource = await readFile(
    path.join(
      repositoryRoot,
      'scripts',
      'lib',
      'tauri-packaging',
      'project-package-policy.mjs',
    ),
    'utf8',
  )
  const projectRuntimeSource = await readFile(
    path.join(
      repositoryRoot,
      'scripts',
      'lib',
      'tauri-packaging',
      'project-runtime-policy.mjs',
    ),
    'utf8',
  )
  const storyContentSource = await readFile(
    path.join(
      repositoryRoot,
      'scripts',
      'lib',
      'tauri-packaging',
      'story-content-policy.mjs',
    ),
    'utf8',
  )

  assert(source.includes("from './lib/tauri-packaging-verifier.mjs'"))
  assert(source.includes('createTauriPackagingVerifier({'))
  assert(!source.includes('async function verifyTauriPackagingConfig'))
  assert(!source.includes('const installationVerificationRequirements'))
  assert(moduleSource.includes('collectTauriPackagingEvidence'))
  assert(moduleSource.includes('collectTauriBuildToolchainEvidence'))
  assert(moduleSource.includes('collectTauriPackagePolicyEvidence'))
  assert(moduleSource.includes('collectTauriInstallationPolicyEvidence'))
  assert(moduleSource.includes('collectTauriCommandRegistrationEvidence'))
  assert(moduleSource.includes('collectTauriConversationSafetyEvidence'))
  assert(moduleSource.includes('collectTauriQualityWorkflowEvidence'))
  assert(moduleSource.includes('collectTauriProjectPackageEvidence'))
  assert(moduleSource.includes('collectTauriProjectRuntimeEvidence'))
  assert(moduleSource.includes('collectTauriStoryContentEvidence'))
  assert(!moduleSource.includes('readFileFromDisk'))
  assert(!moduleSource.includes('config.productName'))
  assert(!moduleSource.includes('const installationVerificationRequirements'))
  assert(!moduleSource.includes('commands::project_archive::commands::export_project_archive'))
  assert(!moduleSource.includes('commands::story_events::get_story_progress'))
  assert(!moduleSource.includes('const multilingualPromptGuardRequirements'))
  assert(!moduleSource.includes('const chatSafetyTraceRequirements'))
  assert(!moduleSource.includes('const headlessWorkflowPreviewRequirements'))
  assert(!moduleSource.includes('const headlessQualityExecutionRequirements'))
  assert(!moduleSource.includes('const qualityRuntimeTraceRequirements'))
  assert(!moduleSource.includes('const buildMetadataRequirements'))
  assert(!moduleSource.includes('const rustToolchainRequirements'))
  assert(!moduleSource.includes('const runtimeDataRootRequirements'))
  assert(!moduleSource.includes('Tauri project-scoped command'))
  assert(!moduleSource.includes('const storyEventCatalogRequirements'))
  assert(!moduleSource.includes('monogatari-story-event-catalog/v1'))
  assert(packagePolicySource.includes('config.productName'))
  assert(buildToolchainSource.includes('const buildMetadataRequirements'))
  assert(buildToolchainSource.includes('const rustToolchainRequirements'))
  assert(installationPolicySource.includes('const requirements'))
  assert(commandRegistrationSource.includes('extractTauriCommandRegistrations'))
  assert(conversationSafetySource.includes('const multilingualPromptGuardRequirements'))
  assert(conversationSafetySource.includes('const chatSafetyTraceRequirements'))
  assert(qualityWorkflowSource.includes('const qualityInputRequirements'))
  assert(qualityWorkflowSource.includes('const workflowPreviewRequirements'))
  assert(qualityWorkflowSource.includes('const qualityExecutionRequirements'))
  assert(qualityWorkflowSource.includes('const authoringRuntimeTraceRequirements'))
  assert(projectPackageSource.includes('const projectExportRequirements'))
  assert(projectPackageSource.includes('const packageProtocolRequirements'))
  assert(projectRuntimeSource.includes('const runtimeRootRequirements'))
  assert(projectRuntimeSource.includes('const runtimeCompatibilityRequirements'))
  assert(storyContentSource.includes('const storyCatalogRequirements'))
  assert(storyContentSource.includes('const eventRuntimeRequirements'))
  assert(storyContentSource.includes('const dialogueAuthoringRequirements'))
  assert(storyContentSource.includes('const crossRuntimeRequirements'))
})
