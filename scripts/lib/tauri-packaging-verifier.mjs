import { collectTauriBuildToolchainEvidence } from './tauri-packaging/build-toolchain-policy.mjs'
import { collectTauriCommandRegistrationEvidence } from './tauri-packaging/command-registration-policy.mjs'
import { collectTauriConversationSafetyEvidence } from './tauri-packaging/conversation-safety-policy.mjs'
import { collectTauriInstallationPolicyEvidence } from './tauri-packaging/installation-policy.mjs'
import { collectTauriPackagePolicyEvidence } from './tauri-packaging/package-policy.mjs'
import { collectTauriProjectPackageEvidence } from './tauri-packaging/project-package-policy.mjs'
import { collectTauriProjectRuntimeEvidence } from './tauri-packaging/project-runtime-policy.mjs'
import { collectTauriQualityWorkflowEvidence } from './tauri-packaging/quality-workflow-policy.mjs'
import { collectTauriStoryContentEvidence } from './tauri-packaging/story-content-policy.mjs'

export function createTauriPackagingVerifier(options = {}) {
  const boundaries = resolveBoundaries(options)
  const logger = options.logger ?? console
  return async function verifyTauriPackagingConfig() {
    const evidence = await collectTauriPackagingEvidence({
      ...options,
      ...boundaries,
    })
    if (evidence.issues.length > 0) {
      throw new Error(`Tauri packaging config verification failed:\n${evidence.issues.join('\n')}`)
    }
    logger.log(
      `[release] Tauri packaging config OK (${evidence.targets.join(', ')} target(s), ${evidence.iconCount} icon(s))`,
    )
    return evidence
  }
}

export async function collectTauriPackagingEvidence(options = {}) {
  const {
    repositoryRoot: root,
    frontendDirectory: frontendDir,
    rustDirectory: rustDir,
    tauriAppDirectory: tauriAppDir,
  } = resolveBoundaries(options)
  const packagePolicyEvidence = await collectTauriPackagePolicyEvidence({
    ...options,
    repositoryRoot: root,
    frontendDirectory: frontendDir,
    rustDirectory: rustDir,
    tauriAppDirectory: tauriAppDir,
  })
  const installationPolicyEvidence = await collectTauriInstallationPolicyEvidence({
    ...options,
    repositoryRoot: root,
    tauriAppDirectory: tauriAppDir,
  })
  const commandRegistrationEvidence = await collectTauriCommandRegistrationEvidence({
    ...options,
    tauriAppDirectory: tauriAppDir,
  })
  const conversationSafetyEvidence = await collectTauriConversationSafetyEvidence({
    ...options,
    rustDirectory: rustDir,
    tauriAppDirectory: tauriAppDir,
  })
  const qualityWorkflowEvidence = await collectTauriQualityWorkflowEvidence({
    ...options,
    rustDirectory: rustDir,
    tauriAppDirectory: tauriAppDir,
  })
  const buildToolchainEvidence = await collectTauriBuildToolchainEvidence({
    ...options,
    repositoryRoot: root,
    rustDirectory: rustDir,
    tauriAppDirectory: tauriAppDir,
  })
  const projectPackageEvidence = await collectTauriProjectPackageEvidence({
    ...options,
    rustDirectory: rustDir,
    tauriAppDirectory: tauriAppDir,
  })
  const projectRuntimeEvidence = await collectTauriProjectRuntimeEvidence({
    ...options,
    rustDirectory: rustDir,
    tauriAppDirectory: tauriAppDir,
  })
  const storyContentEvidence = await collectTauriStoryContentEvidence({
    ...options,
    rustDirectory: rustDir,
    tauriAppDirectory: tauriAppDir,
  })
  const issues = [
    ...packagePolicyEvidence.issues,
    ...installationPolicyEvidence.issues,
    ...commandRegistrationEvidence.issues,
    ...conversationSafetyEvidence.issues,
    ...qualityWorkflowEvidence.issues,
    ...buildToolchainEvidence.issues,
    ...projectPackageEvidence.issues,
    ...projectRuntimeEvidence.issues,
    ...storyContentEvidence.issues,
  ]
  return {
    issues,
    targets: packagePolicyEvidence.targets,
    iconCount: packagePolicyEvidence.iconCount,
  }
}

function resolveBoundaries(options) {
  const boundaries = {
    repositoryRoot: options.repositoryRoot,
    frontendDirectory: options.frontendDirectory,
    rustDirectory: options.rustDirectory,
    tauriAppDirectory: options.tauriAppDirectory,
  }
  for (const [name, value] of Object.entries(boundaries)) {
    if (typeof value !== 'string' || value.length === 0) {
      throw new Error(`Tauri packaging verifier requires ${name}.`)
    }
  }
  return boundaries
}
