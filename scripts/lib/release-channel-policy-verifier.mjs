export const RELEASE_CHANNEL_POLICY_SCHEMA = 'monogatari-release-channel-policy/v1'

const requiredChannels = ['stable', 'beta', 'alpha', 'nightly', 'internal']

const manifestPolicyRequirements = [
  ['release-channel-policy.json', 'load the checked-in release channel policy'],
  ['distributionSummary', 'emit channel distribution policy evidence into release manifests'],
  ['signatureEvidenceFor', 'read installer signing evidence sidecars'],
  ['monogatari-signature-evidence/v1', 'validate stable signature evidence schema'],
  ['artifact_sha256', 'bind signature evidence to installer checksums'],
  ['installerPreflightAllowed', 'gate missing-installer preflight exceptions through policy'],
  ['allow_unsigned_installers', 'gate unsigned installer exceptions through policy'],
  ['missing_evidence', 'surface missing installer signature evidence'],
  ['invalid_evidence', 'surface invalid installer signature evidence'],
  ['source_state', 'emit git source state evidence into release manifests'],
  ['gitSourceState', 'centralize release manifest git source state evidence'],
  ['tracked_worktree_dirty', 'record whether tracked source files were dirty at manifest generation'],
  ['clean_worktree_required', 'record whether clean source state was required for manifest generation'],
  ['--allow-dirty-worktree', 'require an explicit internal diagnostic override for dirty final manifests'],
  ['--untracked-files=no', 'inspect tracked git worktree status without collecting untracked secret files'],
  ['collectQualitySuiteSources', 'emit checked-in quality suite source evidence into release manifests'],
  ['quality_suite_set', 'include aggregate quality suite set evidence in release manifests'],
  ['qualitySuiteSetSummary', 'centralize aggregate quality suite set evidence'],
  ['qualitySuiteSetSha256', 'fingerprint quality suite source sets deterministically'],
  ['monogatari-quality-suite-set/v1', 'version the quality suite set fingerprint contract'],
  ['suite_count', 'record quality suite set source counts in release manifests'],
  ['fingerprint_algorithm', 'record quality suite set fingerprint algorithms in release manifests'],
  ['content_sha256', 'record aggregate quality suite set fingerprints in release manifests'],
  ['quality_suites', 'include quality suite source evidence in release manifests'],
  ['data/quality_suites/character_stability.json', 'require the default character stability suite in release manifests'],
  ['scenario_count', 'record quality suite scenario counts in release manifests'],
  ['categories', 'record quality suite category coverage in release manifests'],
  ['collectWorkflowSources', 'emit checked-in workflow source evidence into release manifests'],
  ['workflow_source_set', 'include aggregate workflow source set evidence in release manifests'],
  ['workflowSourceSetSummary', 'centralize aggregate workflow source set evidence'],
  ['workflowSourceSetSha256', 'fingerprint workflow source sets deterministically'],
  ['monogatari-workflow-source-set/v1', 'version the workflow source set fingerprint contract'],
  ['data/workflows/score_gate_demo.json', 'require the default score-gate workflow source in release manifests'],
  ['workflow_count', 'record workflow source counts in release manifests'],
  ['node_count', 'record workflow node counts in release manifests'],
  ['connection_count', 'record workflow connection counts in release manifests'],
  ['node_types', 'record workflow node type coverage in release manifests'],
  ['collectProjectContentSources', 'emit checked-in project content source evidence into release manifests'],
  ['project_content_source_set', 'include aggregate project content source set evidence in release manifests'],
  ['projectContentSourceSetSummary', 'centralize aggregate project content source set evidence'],
  ['projectContentSourceSetSha256', 'fingerprint project content source sets deterministically'],
  ['monogatari-project-content-source-set/v1', 'version the project content source set fingerprint contract'],
  ['data/characters/sakura.json', 'require default Sakura character content in release manifests'],
  ['data/dialogue/sakura_park_walk.json', 'require default Sakura dialogue content in release manifests'],
  ['data/knowledge/sakura_nature.json', 'require default Sakura knowledge content in release manifests'],
  ['data/scenes/sakura_park.json', 'require default Sakura scene content in release manifests'],
  ['data/assets/characters/sakura_sprite.svg', 'require default Sakura renderer asset content in release manifests'],
  ['data/assets/models/fox.glb', 'require the real animated GLB renderer fixture in release manifests'],
  ['data/assets/models/fox.LICENSE.txt', 'require the GLB fixture attribution in release manifests'],
  ['data/characters/renderer_fox.json', 'require the character record that exercises the real GLB fixture'],
  ['data/events/story_events.json', 'require project story event content in release manifests'],
  ['data/endings/best_friend_ending.json', 'require project story ending content in release manifests'],
  ["'endings'", 'include story endings in project content source categories'],
  ['dialogueNodeCount', 'count object-map and legacy array dialogue nodes in project content summaries'],
  ['schema_versions', 'record story ending schema versions in release manifests'],
  ['event_types', 'record story event type coverage in release manifests'],
  ['character_scoped_count', 'record character-scoped story event counts in release manifests'],
  ['repeatable_count', 'record repeatable story event counts in release manifests'],
  ['action_count', 'record typed story event action counts in release manifests'],
  ['action_types', 'record typed story event action coverage in release manifests'],
  ['category_counts', 'record project content category counts in release manifests'],
  ['category_bytes', 'record project content category byte counts in release manifests'],
  ['category_fingerprint_algorithm', 'record project content category fingerprint algorithms in release manifests'],
  ['category_fingerprints', 'record project content category fingerprints in release manifests'],
  ['knowledge_ref_count', 'record character knowledge reference counts in release manifests'],
  ['size_bytes', 'record project content source sizes in release manifests'],
]

export function releaseChannelPolicyIssues(policy, manifestSource) {
  const issues = []
  const document = isRecord(policy) ? policy : {}
  const channels = isRecord(document.channels) ? document.channels : {}

  if (document.schema !== RELEASE_CHANNEL_POLICY_SCHEMA) {
    issues.push(`release-channel-policy.json must use schema ${RELEASE_CHANNEL_POLICY_SCHEMA}`)
  }
  for (const channel of requiredChannels) {
    if (!isRecord(channels[channel])) {
      issues.push(`release-channel-policy.json must define ${channel}`)
    }
  }

  const stable = isRecord(channels.stable) ? channels.stable : {}
  if (stable.audience !== 'public') {
    issues.push('stable release channel must target public audience')
  }
  if (stable.github?.prerelease !== false || stable.github?.make_latest !== true) {
    issues.push('stable release channel must publish as latest non-prerelease GitHub Release')
  }
  const installers = Array.isArray(stable.required_desktop_installers)
    ? stable.required_desktop_installers
    : []
  for (const kind of ['msi-installer', 'nsis-installer']) {
    if (!installers.includes(kind)) {
      issues.push(`stable release channel must require ${kind}`)
    }
  }
  if (stable.code_signing?.required !== true || stable.code_signing?.minimum_status !== 'verified') {
    issues.push('stable release channel must require verified code signing')
  }
  if (stable.preflight?.allow_missing_installers !== true) {
    issues.push('stable release channel must explicitly allow missing installers only for release-gate preflight')
  }
  if (stable.preflight?.allow_unsigned_installers !== false) {
    issues.push('stable release channel must not allow unsigned installers')
  }

  const source = typeof manifestSource === 'string' ? manifestSource : ''
  for (const [needle, description] of manifestPolicyRequirements) {
    if (!source.includes(needle)) {
      issues.push(`create-release-manifest.mjs must ${description}`)
    }
  }
  return issues
}

function isRecord(value) {
  return Boolean(value) && typeof value === 'object' && !Array.isArray(value)
}
