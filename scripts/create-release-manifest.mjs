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

const args = process.argv.slice(2)
const argSet = new Set(args)
const checkOnly = argSet.has('--check')
const allowMissingInstallers = argSet.has('--allow-missing-installers')
const channel = readArg('channel') ?? process.env.MONOGATARI_RELEASE_CHANNEL ?? 'stable'
const outPath = path.resolve(root, readArg('out') ?? defaultOutPath)

const validChannels = new Set(['stable', 'beta', 'alpha', 'nightly', 'internal'])
const expectedWebArtifacts = [
  'index.html',
  '404.html',
  '.nojekyll',
  'manifest.webmanifest',
  'sw.js',
  'offline.html',
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
  if (!validChannels.has(channel)) {
    issues.push(`Release channel must be one of: ${Array.from(validChannels).join(', ')}`)
  }

  const versions = await readVersions()
  const versionValues = new Set(Object.values(versions).filter(Boolean))
  if (versionValues.size !== 1) {
    issues.push(`Version mismatch across release sources: ${JSON.stringify(versions)}`)
  }

  const artifacts = []
  const missingExpectedArtifacts = []
  await collectWebArtifacts(artifacts, missingExpectedArtifacts)
  await collectDesktopInstallers(artifacts, missingExpectedArtifacts)

  if (artifacts.length === 0) {
    issues.push('No release artifacts found. Build Web/PWA or desktop bundles before creating a manifest.')
  }

  const missingWeb = missingExpectedArtifacts.filter((artifact) => artifact.category === 'web')
  if (missingWeb.length > 0) {
    issues.push(`Missing required Web/PWA artifacts: ${missingWeb.map((artifact) => artifact.path).join(', ')}`)
  }
  const missingInstallers = missingExpectedArtifacts.filter((artifact) => artifact.category === 'desktop-installer')
  if (missingInstallers.length > 0 && !allowMissingInstallers) {
    issues.push(`Missing desktop installer artifacts: ${missingInstallers.map((artifact) => artifact.id).join(', ')}`)
  }

  if (issues.length > 0) {
    throw new Error(`Release manifest validation failed:\n${issues.join('\n')}`)
  }

  const manifest = {
    schema: 'monogatari-release-manifest/v1',
    product: 'Monogatari',
    version: Array.from(versionValues)[0] ?? '0.0.0',
    channel,
    generated_at: new Date().toISOString(),
    git_commit: gitCommit(),
    sources: versions,
    expected_artifacts: expectedArtifactContracts(),
    missing_expected_artifacts: missingExpectedArtifacts,
    signing: signingSummary(artifacts),
    artifacts: artifacts.sort((a, b) => a.path.localeCompare(b.path)),
  }

  if (checkOnly) {
    console.log(
      `[release-manifest] OK (${manifest.artifacts.length} artifact(s), ${manifest.missing_expected_artifacts.length} missing expected artifact(s), channel=${channel})`,
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

async function collectDesktopInstallers(artifacts, missingExpectedArtifacts) {
  const files = await walkFiles(tauriBundleDir)
  const installers = files.filter((file) => desktopInstallerExtensions.has(path.extname(file).toLowerCase()))
  const installerKinds = new Set()
  for (const file of installers) {
    const contract = desktopInstallerExtensions.get(path.extname(file).toLowerCase())
    installerKinds.add(contract.kind)
    artifacts.push(await artifactEntry(file, {
      category: 'desktop-installer',
      platform: contract.platform,
      kind: contract.kind,
      signed: false,
      signature_status: 'unchecked',
    }))
  }

  for (const expected of [
    { id: 'desktop:windows-msi', platform: 'windows', kind: 'msi-installer' },
    { id: 'desktop:windows-nsis', platform: 'windows', kind: 'nsis-installer' },
  ]) {
    if (!installerKinds.has(expected.kind)) {
      missingExpectedArtifacts.push({
        ...expected,
        category: 'desktop-installer',
        path: `rust-engine/target/release/bundle/**/*${expected.kind === 'msi-installer' ? '.msi' : '.exe'}`,
      })
    }
  }
}

async function artifactEntry(file, metadata) {
  const info = await stat(file)
  return {
    id: artifactId(file),
    ...metadata,
    path: relative(file),
    size_bytes: info.size,
    sha256: await sha256(file),
  }
}

function artifactId(file) {
  return relative(file).replace(/[^a-zA-Z0-9._-]+/g, ':')
}

function expectedArtifactContracts() {
  return [
    ...expectedWebArtifacts.map((artifactPath) => ({
      id: `web:${artifactPath}`,
      category: 'web',
      platform: 'web',
      path: `frontend/dist/${artifactPath}`,
      required_for_release: true,
    })),
    {
      id: 'desktop:windows-msi',
      category: 'desktop-installer',
      platform: 'windows',
      kind: 'msi-installer',
      required_for_release: true,
      signing_required: true,
    },
    {
      id: 'desktop:windows-nsis',
      category: 'desktop-installer',
      platform: 'windows',
      kind: 'nsis-installer',
      required_for_release: true,
      signing_required: true,
    },
  ]
}

function signingSummary(artifacts) {
  const desktopArtifacts = artifacts.filter((artifact) => artifact.category === 'desktop-installer')
  return {
    code_signing_required: true,
    policy: 'Desktop installer signatures must be applied and verified before public GitHub Release publication.',
    signed_artifact_count: desktopArtifacts.filter((artifact) => artifact.signed === true).length,
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

function relative(file) {
  return relativeTo(file, root)
}

function relativeTo(file, base) {
  return path.relative(base, file).replaceAll(path.sep, '/')
}
