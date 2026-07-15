import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import { releaseChannelPolicyIssues } from '../lib/release-channel-policy-verifier.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')
const policyPath = path.join(repositoryRoot, 'scripts', 'release-channel-policy.json')
const manifestPath = path.join(repositoryRoot, 'scripts', 'create-release-manifest.mjs')
const releaseVerifierPath = path.join(repositoryRoot, 'scripts', 'verify-release.mjs')
const matrixPath = path.join(repositoryRoot, 'scripts', 'module-test-matrix.json')

async function checkedInInputs() {
  return {
    policy: JSON.parse(await readFile(policyPath, 'utf8')),
    manifest: await readFile(manifestPath, 'utf8'),
  }
}

test('checked-in release policy and manifest satisfy the shared contract', async () => {
  const { policy, manifest } = await checkedInInputs()
  assert.deepEqual(releaseChannelPolicyIssues(policy, manifest), [])
})

test('stable release policy failures remain independently actionable', async () => {
  const { policy, manifest } = await checkedInInputs()
  const invalid = JSON.parse(JSON.stringify(policy))
  delete invalid.channels.beta
  invalid.channels.stable.audience = 'internal'
  invalid.channels.stable.required_desktop_installers = ['msi-installer']
  invalid.channels.stable.code_signing.required = false
  invalid.channels.stable.preflight.allow_unsigned_installers = true

  const issues = releaseChannelPolicyIssues(invalid, manifest)
  assert(issues.includes('release-channel-policy.json must define beta'))
  assert(issues.includes('stable release channel must target public audience'))
  assert(issues.includes('stable release channel must require nsis-installer'))
  assert(issues.includes('stable release channel must require verified code signing'))
  assert(issues.includes('stable release channel must not allow unsigned installers'))
})

test('release manifest source evidence is validated without running the release gate', async () => {
  const { policy, manifest } = await checkedInInputs()
  const invalid = manifest.replaceAll('installerPreflightAllowed', 'removedInstallerPreflightGate')
  const issues = releaseChannelPolicyIssues(policy, invalid)

  assert(issues.includes(
    'create-release-manifest.mjs must gate missing-installer preflight exceptions through policy',
  ))
})

test('malformed policy inputs fail closed instead of throwing type errors', () => {
  const issues = releaseChannelPolicyIssues(null, null)
  assert(issues.includes(
    'release-channel-policy.json must use schema monogatari-release-channel-policy/v1',
  ))
  assert(issues.includes('release-channel-policy.json must define stable'))
  assert(issues.some((issue) => issue.startsWith('create-release-manifest.mjs must ')))
})

test('release orchestration delegates policy rules and shares one automation entry', async () => {
  const releaseVerifier = await readFile(releaseVerifierPath, 'utf8')
  const matrix = JSON.parse(await readFile(matrixPath, 'utf8'))
  const automation = matrix.modules.find((module) => module.id === 'automation-contracts')

  assert(releaseVerifier.includes("from './lib/release-channel-policy-verifier.mjs'"))
  assert(releaseVerifier.includes('releaseChannelPolicyIssues(policy, manifestScript)'))
  assert(!releaseVerifier.includes('const manifestPolicyRequirements'))
  assert(releaseVerifier.includes("['--test', 'scripts/tests/automation-contracts.test.mjs']"))
  assert.deepEqual(automation?.args, ['--test', 'scripts/tests/automation-contracts.test.mjs'])
})
