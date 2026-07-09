import { execFileSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import { mkdir, readdir, readFile, stat, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const frontendDir = path.join(root, 'frontend')
const rustDir = path.join(root, 'rust-engine')
const tauriAppDir = path.join(rustDir, 'crates', 'tauri-app')
const webDistDir = path.join(frontendDir, 'dist')
const tauriBundleDir = path.join(rustDir, 'target', 'release', 'bundle')
const defaultOutPath = path.join(root, 'release', 'monogatari-release-manifest.json')
const releasePolicyPath = path.join(root, 'scripts', 'release-channel-policy.json')
const qualitySuiteSourceDirs = [
  { id: 'project-data', dir: path.join(root, 'data', 'quality_suites') },
  { id: 'tauri-data', dir: path.join(rustDir, 'data', 'quality_suites') },
]
const requiredQualitySuiteSources = [
  'data/quality_suites/character_stability.json',
]

const args = process.argv.slice(2)
const argSet = new Set(args)
const checkOnly = argSet.has('--check')
const allowMissingInstallers = argSet.has('--allow-missing-installers')
const allowDirtyWorktree = argSet.has('--allow-dirty-worktree')
const channel = readArg('channel') ?? process.env.MONOGATARI_RELEASE_CHANNEL ?? 'stable'
const outPath = path.resolve(root, readArg('out') ?? defaultOutPath)

const expectedWebArtifacts = [
  'index.html',
  '404.html',
  '_headers',
  '_redirects',
  'staticwebapp.config.json',
  'vercel.json',
  '.nojekyll',
  'manifest.webmanifest',
  'sw.js',
  'offline.html',
  'project-assets.json',
  'favicon.svg',
  'icons/app-icon.svg',
  'icons/maskable-icon.svg',
]
const desktopInstallerExtensions = new Map([
  ['.msi', { platform: 'windows', kind: 'msi-installer' }],
  ['.exe', { platform: 'windows', kind: 'nsis-installer' }],
  ['.dmg', { platform: 'macos', kind: 'dmg-installer' }],
  ['.appimage', { platform: 'linux', kind: 'appimage-installer' }],
  ['.deb', { platform: 'linux', kind: 'deb-package' }],
  ['.rpm', { platform: 'linux', kind: 'rpm-package' }],
])

main().catch((error) => {
  console.error(`[release-manifest] ${error.message}`)
  process.exit(1)
})

async function main() {
  const issues = []
  const releasePolicy = await readReleasePolicy(issues)
  const channelPolicy = releasePolicy?.channels?.[channel] ?? null
  if (!channelPolicy) {
    issues.push(`Release channel must be one of: ${Object.keys(releasePolicy?.channels ?? {}).join(', ')}`)
  }

  const versions = await readVersions()
  const sourceState = gitSourceState()
  const versionValues = new Set(Object.values(versions).filter(Boolean))
  if (versionValues.size !== 1) {
    issues.push(`Version mismatch across release sources: ${JSON.stringify(versions)}`)
  }
  if (!sourceState.git_commit) {
    issues.push('Release manifests require a git commit id for source traceability.')
  }
  if (!checkOnly && sourceState.tracked_worktree_dirty && !allowDirtyWorktree) {
    issues.push(
      'Final release manifest generation requires a clean tracked git worktree. Commit or stash tracked changes, or pass --allow-dirty-worktree for internal diagnostics.',
    )
  }

  const artifacts = []
  const missingExpectedArtifacts = []
  await collectWebArtifacts(artifacts, missingExpectedArtifacts)
  await collectDesktopInstallers(artifacts, missingExpectedArtifacts, issues, channelPolicy)
  const qualitySuites = await collectQualitySuiteSources(issues)
  const qualitySuiteSet = qualitySuiteSetSummary(qualitySuites)

  if (artifacts.length === 0) {
    issues.push('No release artifacts found. Build Web/PWA or desktop bundles before creating a manifest.')
  }

  const missingWeb = missingExpectedArtifacts.filter((artifact) => artifact.category === 'web')
  if (missingWeb.length > 0) {
    issues.push(`Missing required Web/PWA artifacts: ${missingWeb.map((artifact) => artifact.path).join(', ')}`)
  }
  const missingInstallers = missingExpectedArtifacts.filter((artifact) => artifact.category === 'desktop-installer')
  if (missingInstallers.length > 0 && !channelPolicy && !allowMissingInstallers) {
    issues.push(`Missing desktop installer artifacts: ${missingInstallers.map((artifact) => artifact.id).join(', ')}`)
  }
  enforceChannelPolicy(channelPolicy, artifacts, missingExpectedArtifacts, issues)

  if (issues.length > 0) {
    throw new Error(`Release manifest validation failed:\n${issues.join('\n')}`)
  }

  const manifest = {
    schema: 'monogatari-release-manifest/v1',
    product: 'Monogatari',
    version: Array.from(versionValues)[0] ?? '0.0.0',
    channel,
    generated_at: new Date().toISOString(),
    git_commit: sourceState.git_commit,
    sources: versions,
    source_state: sourceState,
    distribution: distributionSummary(releasePolicy, channelPolicy, missingInstallers),
    quality_suite_set: qualitySuiteSet,
    quality_suites: qualitySuites,
    expected_artifacts: expectedArtifactContracts(channelPolicy),
    missing_expected_artifacts: missingExpectedArtifacts,
    signing: signingSummary(artifacts, channelPolicy),
    artifacts: artifacts.sort((a, b) => a.path.localeCompare(b.path)),
  }

  if (checkOnly) {
    const qualitySuiteSetFingerprint = manifest.quality_suite_set.content_sha256.slice(0, 12)
    console.log(
      `[release-manifest] OK (${manifest.artifacts.length} artifact(s), ${manifest.quality_suite_set.suite_count} quality suite(s), quality suite set ${qualitySuiteSetFingerprint}, ${manifest.missing_expected_artifacts.length} missing expected artifact(s), channel=${channel})`,
    )
    return
  }

  if (!outPath.startsWith(root + path.sep)) {
    throw new Error('Release manifest output path must stay inside the repository.')
  }
  await mkdir(path.dirname(outPath), { recursive: true })
  await writeFile(outPath, `${JSON.stringify(manifest, null, 2)}\n`, 'utf8')
  console.log(`[release-manifest] Wrote ${relative(outPath)} (${manifest.artifacts.length} artifact(s))`)
}

function readArg(name) {
  const prefix = `--${name}=`
  const value = args.find((arg) => arg.startsWith(prefix))
  return value ? value.slice(prefix.length) : null
}

async function readReleasePolicy(issues) {
  try {
    const policy = JSON.parse(await readFile(releasePolicyPath, 'utf8'))
    issues.push(...validateReleasePolicy(policy))
    return policy
  } catch (error) {
    issues.push(`Release channel policy could not be read: ${error.message}`)
    return null
  }
}

function validateReleasePolicy(policy) {
  const issues = []
  if (policy?.schema !== 'monogatari-release-channel-policy/v1') {
    issues.push('Release channel policy schema must be monogatari-release-channel-policy/v1')
  }
  if (!policy?.channels || typeof policy.channels !== 'object') {
    issues.push('Release channel policy must define channels')
    return issues
  }

  for (const [name, channelPolicy] of Object.entries(policy.channels)) {
    const label = `channel ${name}`
    if (!nonEmptyString(channelPolicy.audience)) {
      issues.push(`${label}: audience is required`)
    }
    if (!Array.isArray(channelPolicy.required_artifact_categories)) {
      issues.push(`${label}: required_artifact_categories must be an array`)
    }
    if (!Array.isArray(channelPolicy.required_desktop_installers)) {
      issues.push(`${label}: required_desktop_installers must be an array`)
    }
    if (typeof channelPolicy.github?.prerelease !== 'boolean') {
      issues.push(`${label}: github.prerelease must be boolean`)
    }
    if (typeof channelPolicy.github?.make_latest !== 'boolean') {
      issues.push(`${label}: github.make_latest must be boolean`)
    }
    if (typeof channelPolicy.code_signing?.required !== 'boolean') {
      issues.push(`${label}: code_signing.required must be boolean`)
    }
    if (!nonEmptyString(channelPolicy.code_signing?.minimum_status)) {
      issues.push(`${label}: code_signing.minimum_status is required`)
    }
    if (typeof channelPolicy.preflight?.allow_missing_installers !== 'boolean') {
      issues.push(`${label}: preflight.allow_missing_installers must be boolean`)
    }
    if (typeof channelPolicy.preflight?.allow_unsigned_installers !== 'boolean') {
      issues.push(`${label}: preflight.allow_unsigned_installers must be boolean`)
    }
    if (!nonEmptyString(channelPolicy.preflight?.reason)) {
      issues.push(`${label}: preflight.reason is required`)
    }
  }

  const stable = policy.channels.stable
  if (!stable) {
    issues.push('Release channel policy must define stable')
  } else {
    if (stable.github?.prerelease !== false || stable.github?.make_latest !== true) {
      issues.push('stable channel must publish as latest non-prerelease GitHub Release')
    }
    if (stable.code_signing?.required !== true || stable.code_signing?.minimum_status !== 'verified') {
      issues.push('stable channel must require verified code signing')
    }
    for (const kind of ['msi-installer', 'nsis-installer']) {
      if (!stable.required_desktop_installers?.includes(kind)) {
        issues.push(`stable channel must require ${kind}`)
      }
    }
  }

  return issues
}

async function readVersions() {
  const frontendPackage = JSON.parse(await readFile(path.join(frontendDir, 'package.json'), 'utf8'))
  const tauriConfig = JSON.parse(await readFile(path.join(tauriAppDir, 'tauri.conf.json'), 'utf8'))
  const cargoWorkspace = await readFile(path.join(rustDir, 'Cargo.toml'), 'utf8')
  return {
    frontend_package: frontendPackage.version,
    tauri_config: tauriConfig.version,
    cargo_workspace: cargoWorkspace.match(/\[workspace\.package\][\s\S]*?\nversion\s*=\s*"([^"]+)"/)?.[1] ?? null,
  }
}

async function collectQualitySuiteSources(issues) {
  const sources = []
  for (const source of qualitySuiteSourceDirs) {
    const files = (await walkFiles(source.dir))
      .filter((file) => path.extname(file).toLowerCase() === '.json')

    for (const file of files) {
      const rel = relative(file)
      const bytes = await readFile(file)
      let suite
      try {
        suite = JSON.parse(bytes.toString('utf8'))
      } catch (error) {
        issues.push(`${rel}: quality suite source is not valid JSON: ${error.message}`)
        continue
      }

      const scenarios = Array.isArray(suite.scenarios) ? suite.scenarios : []
      sources.push({
        path: rel,
        source_root: source.id,
        name: suite.name ?? null,
        version: suite.version ?? null,
        scenario_count: scenarios.length,
        categories: Array.from(new Set(scenarios.map((scenario) => scenario.category).filter(nonEmptyString))).sort(),
        sha256: createHash('sha256').update(bytes).digest('hex'),
      })
    }
  }

  if (sources.length === 0) {
    issues.push('Release manifests require at least one checked-in quality suite source.')
  }
  for (const requiredPath of requiredQualitySuiteSources) {
    if (!sources.some((source) => source.path === requiredPath)) {
      issues.push(`Missing required quality suite source: ${requiredPath}`)
    }
  }

  return sources.sort((a, b) => a.path.localeCompare(b.path))
}

function qualitySuiteSetSummary(qualitySuites) {
  return {
    schema: 'monogatari-quality-suite-set/v1',
    suite_count: qualitySuites.length,
    scenario_count: qualitySuites.reduce((total, suite) => total + (suite.scenario_count ?? 0), 0),
    categories: Array.from(new Set(qualitySuites.flatMap((suite) => suite.categories ?? []))).sort(),
    fingerprint_algorithm: 'sha256:path-source-root-scenario-count-categories-suite-sha256-v1',
    content_sha256: qualitySuiteSetSha256(qualitySuites),
  }
}

function qualitySuiteSetSha256(qualitySuites) {
  const hash = createHash('sha256')
  for (const suite of [...qualitySuites].sort((a, b) => a.path.localeCompare(b.path))) {
    const categories = Array.isArray(suite.categories) ? [...suite.categories].sort() : []
    hash.update(suite.path ?? '')
    hash.update('\0')
    hash.update(suite.source_root ?? '')
    hash.update('\0')
    hash.update(String(suite.scenario_count ?? 0))
    hash.update('\0')
    hash.update(categories.join(','))
    hash.update('\0')
    hash.update(suite.sha256 ?? '')
    hash.update('\n')
  }
  return hash.digest('hex')
}

async function collectWebArtifacts(artifacts, missingExpectedArtifacts) {
  const files = await walkFiles(webDistDir)
  const fileSet = new Set(files.map((file) => relativeTo(file, webDistDir)))
  for (const expectedPath of expectedWebArtifacts) {
    if (!fileSet.has(expectedPath)) {
      missingExpectedArtifacts.push({
        id: `web:${expectedPath}`,
        category: 'web',
        platform: 'web',
        path: `frontend/dist/${expectedPath}`,
      })
    }
  }

  for (const file of files) {
    artifacts.push(await artifactEntry(file, {
      category: 'web',
      platform: 'web',
      kind: webArtifactKind(file),
      signed: null,
      signature_status: 'not-applicable',
    }))
  }
}

async function collectDesktopInstallers(artifacts, missingExpectedArtifacts, issues, channelPolicy) {
  const files = await walkFiles(tauriBundleDir)
  const installers = files.filter((file) => desktopInstallerExtensions.has(path.extname(file).toLowerCase()))
  const installerKinds = new Set()
  for (const file of installers) {
    const contract = desktopInstallerExtensions.get(path.extname(file).toLowerCase())
    installerKinds.add(contract.kind)
    const digest = await sha256(file)
    const signature = await signatureEvidenceFor(file, digest)
    issues.push(...signature.issues.map((issue) => `${relative(file)}: ${issue}`))
    artifacts.push(await artifactEntry(file, {
      category: 'desktop-installer',
      platform: contract.platform,
      kind: contract.kind,
      signed: signature.signed,
      signature_status: signature.status,
      signature_evidence: signature.evidence_path,
      signature_subject: signature.subject,
      signature_verifier: signature.verifier,
      signed_at: signature.signed_at,
      verified_at: signature.verified_at,
    }, digest))
  }

  const expectedInstallers = [
    { id: 'desktop:windows-msi', platform: 'windows', kind: 'msi-installer' },
    { id: 'desktop:windows-nsis', platform: 'windows', kind: 'nsis-installer' },
  ].filter((expected) => (channelPolicy?.required_desktop_installers ?? ['msi-installer', 'nsis-installer']).includes(expected.kind))
  for (const expected of expectedInstallers) {
    if (!installerKinds.has(expected.kind)) {
      missingExpectedArtifacts.push({
        ...expected,
        category: 'desktop-installer',
        path: `rust-engine/target/release/bundle/**/*${expected.kind === 'msi-installer' ? '.msi' : '.exe'}`,
      })
    }
  }
}

async function artifactEntry(file, metadata, knownSha256 = null) {
  const info = await stat(file)
  return {
    id: artifactId(file),
    ...metadata,
    path: relative(file),
    size_bytes: info.size,
    sha256: knownSha256 ?? await sha256(file),
  }
}

async function signatureEvidenceFor(file, artifactSha256) {
  const evidencePath = `${file}.sig.json`
  try {
    await stat(evidencePath)
  } catch {
    return {
      signed: false,
      status: 'missing-evidence',
      evidence_path: null,
      subject: null,
      verifier: null,
      signed_at: null,
      verified_at: null,
      issues: [],
    }
  }

  const issues = []
  let evidence
  try {
    evidence = JSON.parse(await readFile(evidencePath, 'utf8'))
  } catch (error) {
    return {
      signed: false,
      status: 'invalid-evidence',
      evidence_path: relative(evidencePath),
      subject: null,
      verifier: null,
      signed_at: null,
      verified_at: null,
      issues: [`signature evidence is not valid JSON: ${error.message}`],
    }
  }

  if (evidence.schema !== 'monogatari-signature-evidence/v1') {
    issues.push('signature evidence schema must be monogatari-signature-evidence/v1')
  }
  if (evidence.artifact_sha256 !== artifactSha256) {
    issues.push('signature evidence artifact_sha256 must match installer SHA-256')
  }
  if (evidence.status !== 'verified') {
    issues.push('signature evidence status must be verified')
  }
  for (const field of ['subject', 'verifier', 'verified_at']) {
    if (!nonEmptyString(evidence[field])) {
      issues.push(`signature evidence ${field} is required`)
    }
  }

  const signed = issues.length === 0
  return {
    signed,
    status: signed ? 'verified' : 'invalid-evidence',
    evidence_path: relative(evidencePath),
    subject: evidence.subject ?? null,
    verifier: evidence.verifier ?? null,
    signed_at: evidence.signed_at ?? null,
    verified_at: evidence.verified_at ?? null,
    issues,
  }
}

function artifactId(file) {
  return relative(file).replace(/[^a-zA-Z0-9._-]+/g, ':')
}

function expectedArtifactContracts(channelPolicy) {
  const requiredCategories = new Set(channelPolicy?.required_artifact_categories ?? ['web', 'desktop-installer'])
  const requiredInstallers = new Set(channelPolicy?.required_desktop_installers ?? ['msi-installer', 'nsis-installer'])
  return [
    ...expectedWebArtifacts.map((artifactPath) => ({
      id: `web:${artifactPath}`,
      category: 'web',
      platform: 'web',
      path: `frontend/dist/${artifactPath}`,
      required_for_release: requiredCategories.has('web'),
    })),
    {
      id: 'desktop:windows-msi',
      category: 'desktop-installer',
      platform: 'windows',
      kind: 'msi-installer',
      required_for_release: requiredCategories.has('desktop-installer') && requiredInstallers.has('msi-installer'),
      signing_required: channelPolicy?.code_signing?.required === true,
    },
    {
      id: 'desktop:windows-nsis',
      category: 'desktop-installer',
      platform: 'windows',
      kind: 'nsis-installer',
      required_for_release: requiredCategories.has('desktop-installer') && requiredInstallers.has('nsis-installer'),
      signing_required: channelPolicy?.code_signing?.required === true,
    },
  ]
}

function installerPreflightAllowed(channelPolicy) {
  return allowMissingInstallers && channelPolicy?.preflight?.allow_missing_installers === true
}

function enforceChannelPolicy(channelPolicy, artifacts, missingExpectedArtifacts, issues) {
  if (!channelPolicy) return

  const categories = new Set(artifacts.map((artifact) => artifact.category))
  for (const category of channelPolicy.required_artifact_categories ?? []) {
    if (!categories.has(category)) {
      const categoryMissing = missingExpectedArtifacts.some((artifact) => artifact.category === category)
      if (category !== 'desktop-installer' || !categoryMissing || !installerPreflightAllowed(channelPolicy)) {
        issues.push(`Release channel ${channel} requires artifact category ${category}`)
      }
    }
  }

  const missingInstallers = missingExpectedArtifacts
    .filter((artifact) => artifact.category === 'desktop-installer')
    .filter((artifact) => (channelPolicy.required_desktop_installers ?? []).includes(artifact.kind))
  if (missingInstallers.length > 0 && !installerPreflightAllowed(channelPolicy)) {
    issues.push(`Release channel ${channel} requires desktop installers: ${missingInstallers.map((artifact) => artifact.kind).join(', ')}`)
  }

  const desktopArtifacts = artifacts.filter((artifact) => artifact.category === 'desktop-installer')
  const signingRequired = channelPolicy.code_signing?.required === true
  if (signingRequired) {
    const unsigned = desktopArtifacts.filter((artifact) => artifact.signed !== true)
    if (unsigned.length > 0 && !(allowMissingInstallers && channelPolicy.preflight?.allow_unsigned_installers === true)) {
      issues.push(`Release channel ${channel} requires verified signatures for installers: ${unsigned.map((artifact) => artifact.path).join(', ')}`)
    }
  }
}

function distributionSummary(releasePolicy, channelPolicy, missingInstallers) {
  return {
    policy_schema: releasePolicy?.schema ?? null,
    policy_path: relative(releasePolicyPath),
    channel,
    audience: channelPolicy?.audience ?? null,
    github: channelPolicy?.github ?? null,
    required_artifact_categories: channelPolicy?.required_artifact_categories ?? [],
    required_desktop_installers: channelPolicy?.required_desktop_installers ?? [],
    preflight: {
      allow_missing_installers_requested: allowMissingInstallers,
      allow_missing_installers_used: missingInstallers.length > 0 && installerPreflightAllowed(channelPolicy),
      allow_missing_installers_allowed_by_policy: channelPolicy?.preflight?.allow_missing_installers === true,
      reason: channelPolicy?.preflight?.reason ?? null,
    },
  }
}

function signingSummary(artifacts, channelPolicy) {
  const desktopArtifacts = artifacts.filter((artifact) => artifact.category === 'desktop-installer')
  return {
    code_signing_required: channelPolicy?.code_signing?.required === true,
    minimum_status: channelPolicy?.code_signing?.minimum_status ?? 'verified',
    policy: 'Desktop installer signatures must be applied and verified before public GitHub Release publication.',
    signed_artifact_count: desktopArtifacts.filter((artifact) => artifact.signed === true).length,
    missing_evidence: desktopArtifacts
      .filter((artifact) => artifact.signature_status === 'missing-evidence')
      .map((artifact) => artifact.path),
    invalid_evidence: desktopArtifacts
      .filter((artifact) => artifact.signature_status === 'invalid-evidence')
      .map((artifact) => artifact.path),
    unsigned_artifacts: desktopArtifacts
      .filter((artifact) => artifact.signed !== true)
      .map((artifact) => artifact.path),
  }
}

function webArtifactKind(file) {
  const rel = relativeTo(file, webDistDir)
  if (rel === 'index.html') return 'web-entry'
  if (rel === 'manifest.webmanifest') return 'pwa-manifest'
  if (rel === 'sw.js') return 'service-worker'
  if (rel === 'offline.html') return 'offline-fallback'
  if (rel === '404.html') return 'spa-fallback'
  if (rel === '_headers') return 'static-hosting-headers'
  if (rel === '_redirects') return 'static-hosting-redirects'
  if (rel === 'staticwebapp.config.json') return 'azure-static-web-app-config'
  if (rel === 'vercel.json') return 'vercel-static-app-config'
  if (rel === 'project-assets.json') return 'project-asset-manifest'
  if (rel.startsWith('assets/')) return 'web-asset'
  if (rel.startsWith('icons/')) return 'pwa-icon'
  if (rel.startsWith('locales/')) return 'locale'
  return 'web-support'
}

async function sha256(file) {
  const bytes = await readFile(file)
  return createHash('sha256').update(bytes).digest('hex')
}

async function walkFiles(dir) {
  try {
    const info = await stat(dir)
    if (!info.isDirectory()) return []
  } catch {
    return []
  }

  const files = []
  const entries = await readdir(dir, { withFileTypes: true })
  for (const entry of entries) {
    const file = path.join(dir, entry.name)
    if (entry.isDirectory()) {
      files.push(...await walkFiles(file))
    } else if (entry.isFile()) {
      files.push(file)
    }
  }
  return files
}

function gitCommit() {
  try {
    return execFileSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).trim()
  } catch {
    return null
  }
}

function gitSourceState() {
  const trackedStatus = gitTrackedWorktreeStatus()
  return {
    git_commit: gitCommit(),
    tracked_worktree_dirty: trackedStatus.length > 0,
    tracked_worktree_status: trackedStatus,
    clean_worktree_required: !checkOnly && !allowDirtyWorktree,
    allow_dirty_worktree_requested: allowDirtyWorktree,
  }
}

function gitTrackedWorktreeStatus() {
  try {
    return execFileSync('git', ['status', '--short', '--untracked-files=no'], {
      cwd: root,
      encoding: 'utf8',
    })
      .split(/\r?\n/)
      .map((line) => line.trimEnd())
      .filter(Boolean)
  } catch {
    return []
  }
}

function relative(file) {
  return relativeTo(file, root)
}

function relativeTo(file, base) {
  return path.relative(base, file).replaceAll(path.sep, '/')
}

function nonEmptyString(value) {
  return typeof value === 'string' && value.trim().length > 0
}
