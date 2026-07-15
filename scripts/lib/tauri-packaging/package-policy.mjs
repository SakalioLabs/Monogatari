import { readFile as readFileFromDisk, stat } from 'node:fs/promises'
import path from 'node:path'

import {
  requiredTauriCspFragments,
  verifyCspPolicy,
} from '../web-hosting-verifier.mjs'

export async function collectTauriPackagePolicyEvidence(options = {}) {
  const {
    repositoryRoot: root,
    frontendDirectory: frontendDir,
    rustDirectory: rustDir,
    tauriAppDirectory: tauriAppDir,
  } = resolveBoundaries(options)
  const readFile = options.readTextFile ?? readFileFromDisk
  const issues = []
  const config = JSON.parse(
    await readFile(path.join(tauriAppDir, 'tauri.conf.json'), 'utf8'),
  )
  const frontendPackage = JSON.parse(
    await readFile(path.join(frontendDir, 'package.json'), 'utf8'),
  )
  const viteConfigSource = await readFile(path.join(frontendDir, 'vite.config.ts'), 'utf8')
  const cargoWorkspace = await readFile(path.join(rustDir, 'Cargo.toml'), 'utf8')
  const tauriCargoSource = await readFile(path.join(tauriAppDir, 'Cargo.toml'), 'utf8')
  const mobilePreflightSource = await readFile(
    path.join(root, 'scripts', 'verify-tauri-mobile-preflight.mjs'),
    'utf8',
  )
  const mobileDeploymentDocs = await readFile(
    path.join(root, 'docs', 'MOBILE_DEPLOYMENT.md'),
    'utf8',
  )
  const workspaceVersion = cargoWorkspace.match(
    /\[workspace\.package\][\s\S]*?\nversion\s*=\s*"([^"]+)"/,
  )?.[1]

  if (config.productName !== 'Monogatari') {
    issues.push('tauri.conf.json productName must stay Monogatari for installer identity')
  }
  if (config.identifier !== 'com.sakaliolabs.monogatari') {
    issues.push('tauri.conf.json identifier must stay com.sakaliolabs.monogatari')
  }
  if (config.version !== frontendPackage.version) {
    issues.push(
      `tauri.conf.json version ${config.version} must match frontend/package.json ${frontendPackage.version}`,
    )
  }
  if (workspaceVersion && config.version !== workspaceVersion) {
    issues.push(
      `tauri.conf.json version ${config.version} must match rust-engine/Cargo.toml workspace version ${workspaceVersion}`,
    )
  }
  if (
    path.resolve(tauriAppDir, config.build?.frontendDist ?? '') !==
    path.join(frontendDir, 'dist')
  ) {
    issues.push(
      'tauri.conf.json build.frontendDist must resolve to the repository frontend/dist directory',
    )
  }
  if (!String(config.build?.beforeBuildCommand ?? '').includes('npm run build')) {
    issues.push(
      'tauri.conf.json build.beforeBuildCommand must run the production frontend build before desktop packaging',
    )
  }
  if (
    !String(config.build?.beforeBuildCommand ?? '').includes(
      'scripts/sync-project-mirror.mjs --check',
    )
  ) {
    issues.push(
      'tauri.conf.json build.beforeBuildCommand must verify the packaged desktop project mirror',
    )
  }

  const csp = config.app?.security?.csp
  if (!nonEmptyString(csp)) {
    issues.push(
      'tauri.conf.json app.security.csp must define a production Content Security Policy',
    )
  } else {
    verifyCspPolicy(
      csp,
      requiredTauriCspFragments,
      'tauri.conf.json app.security.csp',
      issues,
    )
  }

  const mobileDeploymentRequirements = [
    [
      viteConfigSource,
      'const mobileDevHost = process.env.TAURI_DEV_HOST',
      'let Tauri mobile commands select the Vite dev host',
    ],
    [
      viteConfigSource,
      'host: mobileDevHost || false',
      'bind Vite to the Tauri-selected mobile host',
    ],
    [
      viteConfigSource,
      'hmr: mobileDevHost',
      'configure mobile HMR when a Tauri host is provided',
    ],
    [tauriCargoSource, 'tauri = { version = "2"', 'stay on the Tauri v2 mobile-capable line'],
    [tauriCargoSource, 'tauri-plugin-shell = "2"', 'stay on the v2 shell plugin line'],
    [
      tauriCargoSource,
      'tauri-plugin-dialog = "2.7.1"',
      'pin the native project package dialog plugin',
    ],
    [
      cargoWorkspace,
      'zip = { version = "8.6.0"',
      'pin the project package ZIP implementation',
    ],
    [
      JSON.stringify(frontendPackage),
      '@tauri-apps/plugin-dialog',
      'ship the native project package dialog frontend',
    ],
    [
      mobilePreflightSource,
      'cargo tauri android init',
      'verify Android init documentation',
    ],
    [mobilePreflightSource, 'cargo tauri ios init', 'verify iOS init documentation'],
    [mobilePreflightSource, 'ANDROID_HOME', 'verify Android SDK environment documentation'],
    [
      mobilePreflightSource,
      'iOS commands require a macOS host',
      'verify the iOS host constraint',
    ],
    [mobileDeploymentDocs, 'cargo tauri android build', 'document Android release builds'],
    [mobileDeploymentDocs, 'cargo tauri ios build', 'document iOS release builds'],
    [mobileDeploymentDocs, 'TAURI_DEV_HOST', 'document the mobile dev host contract'],
    [
      mobileDeploymentDocs,
      'node scripts/verify-tauri-mobile-preflight.mjs',
      'document the mobile preflight evidence command',
    ],
  ]
  for (const [source, needle, description] of mobileDeploymentRequirements) {
    if (!source.includes(needle)) {
      issues.push(`Tauri mobile preflight must ${description}`)
    }
  }

  const bundle = config.bundle ?? {}
  if (bundle.active !== true) {
    issues.push('tauri.conf.json bundle.active must be true for release packaging')
  }

  const targets =
    bundle.targets === 'all' ? ['all'] : Array.isArray(bundle.targets) ? bundle.targets : []
  for (const target of ['msi', 'nsis']) {
    if (!targets.includes(target) && !targets.includes('all')) {
      issues.push(
        `tauri.conf.json bundle.targets must include ${target} for Windows installer coverage`,
      )
    }
  }

  for (const [field, description] of [
    ['publisher', 'publisher/manufacturer metadata'],
    ['category', 'application category metadata'],
    ['shortDescription', 'short store/installer description'],
    ['longDescription', 'long store/installer description'],
    ['copyright', 'copyright metadata'],
  ]) {
    if (!nonEmptyString(bundle[field])) {
      issues.push(`tauri.conf.json bundle.${field} must define ${description}`)
    }
  }

  const icons = Array.isArray(bundle.icon) ? bundle.icon : []
  for (const icon of [
    'icons/icon_32x32.png',
    'icons/icon_128x128.png',
    'icons/icon_256x256.png',
    'icons/icon_512x512.png',
    'icons/icon.ico',
  ]) {
    if (!icons.includes(icon)) {
      issues.push(`tauri.conf.json bundle.icon must include ${icon}`)
    } else if (!(await fileExists(path.join(tauriAppDir, icon)))) {
      issues.push(`tauri.conf.json bundle.icon references a missing file: ${icon}`)
    }
  }

  const resourceEntries = Array.isArray(bundle.resources)
    ? bundle.resources.map((entry) => [entry, null])
    : bundle.resources && typeof bundle.resources === 'object'
      ? Object.entries(bundle.resources)
      : []
  const bundledDesktopData = resourceEntries.find(
    ([source]) => path.resolve(tauriAppDir, source) === path.join(rustDir, 'data'),
  )
  if (!bundledDesktopData) {
    issues.push(
      'tauri.conf.json bundle.resources must include ../../data so installed builds use the verified desktop project mirror',
    )
  } else {
    const [source, target] = bundledDesktopData
    if (target !== 'data') {
      issues.push(
        'tauri.conf.json bundle.resources must map ../../data to clean data/ resource output',
      )
    }
    const dataRoot = path.resolve(tauriAppDir, source)
    for (const dir of [
      'assets',
      'characters',
      'dialogue',
      'endings',
      'events',
      'knowledge',
      'locales',
      'quality_suites',
      'scenes',
      'workflows',
    ]) {
      if (!(await directoryExists(path.join(dataRoot, dir)))) {
        issues.push(`bundled data resource is missing ${dir}/`)
      }
    }
    if (!(await fileExists(path.join(dataRoot, 'settings.json')))) {
      issues.push('bundled data resource is missing settings.json')
    }
    if (await fileExists(path.join(dataRoot, '.monogatari-mcp-project.lock'))) {
      issues.push('bundled data resource must not contain the transient MCP project lease file')
    }
  }

  const windows = bundle.windows ?? {}
  if (windows.allowDowngrades !== false) {
    issues.push(
      'tauri.conf.json bundle.windows.allowDowngrades must be false for commercial release safety',
    )
  }
  if (windows.wix?.upgradeCode !== 'c4c2d20f-f307-5c7b-91e6-5edeea14fdd0') {
    issues.push(
      'tauri.conf.json bundle.windows.wix.upgradeCode must pin the established Monogatari MSI upgrade identity',
    )
  }
  if (windows.webviewInstallMode?.type !== 'downloadBootstrapper') {
    issues.push(
      'tauri.conf.json bundle.windows.webviewInstallMode.type must be downloadBootstrapper for normal public Windows installers',
    )
  }
  if (windows.webviewInstallMode?.silent !== true) {
    issues.push('tauri.conf.json bundle.windows.webviewInstallMode.silent must be true')
  }

  return {
    issues,
    targets,
    iconCount: icons.length,
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
      throw new Error(`Tauri package policy requires ${name}.`)
    }
  }
  return boundaries
}

async function fileExists(filePath) {
  try {
    return (await stat(filePath)).isFile()
  } catch {
    return false
  }
}

async function directoryExists(filePath) {
  try {
    return (await stat(filePath)).isDirectory()
  } catch {
    return false
  }
}

function nonEmptyString(value) {
  return typeof value === 'string' && value.trim().length > 0
}
