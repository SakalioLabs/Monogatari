import { readdir, readFile, stat } from 'node:fs/promises'
import path from 'node:path'

import { normalizeWebBasePath } from './web-base-path.mjs'
import {
  extractHtmlCsp,
  requiredWebCspFragments,
  verifyAzureStaticWebAppConfig,
  verifyCspPolicy,
  verifyStaticHostingHeaders,
  verifyStaticHostingRedirects,
  verifyVercelConfig,
} from './web-hosting-verifier.mjs'

export const releaseSubpathBase = '/Monogatari/'
export const requiredLocaleFiles = Object.freeze([
  'en.json',
  'zh-CN.json',
  'zh.json',
  'ja-JP.json',
  'ja.json',
  'ko-KR.json',
  'ko.json',
])

const requiredWebDistFiles = Object.freeze([
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
  'offline-i18n.js',
  'project-assets.json',
  'inference-runtime.json',
  'events/story_events.json',
  'favicon.svg',
  'icons/app-icon.svg',
  'icons/maskable-icon.svg',
])
const requiredPwaIcons = Object.freeze(['icons/app-icon.svg', 'icons/maskable-icon.svg'])

export function createWebDistributionVerifier({
  repositoryRoot,
  frontendDirectory,
  logger = console,
} = {}) {
  if (typeof repositoryRoot !== 'string' || repositoryRoot.length === 0) {
    throw new Error('Web distribution verifier requires a repository root.')
  }
  if (typeof frontendDirectory !== 'string' || frontendDirectory.length === 0) {
    throw new Error('Web distribution verifier requires a frontend directory.')
  }

  return async function verifyWebDistribution({ basePath = '/' } = {}) {
    const evidence = await collectWebDistributionEvidence({
      repositoryRoot,
      frontendDirectory,
      basePath,
    })
    if (evidence.issues.length > 0) {
      throw new Error(`Web/PWA dist verification failed:\n${evidence.issues.join('\n')}`)
    }
    logger.log(`[release] Web/PWA dist assets OK (${evidence.normalizedBase} base)`)
    return evidence
  }
}

export async function collectWebDistributionEvidence({
  repositoryRoot,
  frontendDirectory,
  basePath = '/',
} = {}) {
  const distDir = path.join(frontendDirectory, 'dist')
  const issues = []
  const normalizedBase = normalizeWebBasePath(basePath)

  for (const file of requiredWebDistFiles) {
    if (!(await fileExists(path.join(distDir, file)))) {
      issues.push(`Missing Web/PWA dist asset: ${file}`)
    }
  }

  const indexHtml = await readMaybe(path.join(distDir, 'index.html'))
  const fallbackHtml = await readMaybe(path.join(distDir, '404.html'))
  const staticHostingHeaders = await readMaybe(path.join(distDir, '_headers'))
  const staticHostingRedirects = await readMaybe(path.join(distDir, '_redirects'))
  const azureStaticWebAppConfig = await readJsonMaybe(
    path.join(distDir, 'staticwebapp.config.json'),
    'staticwebapp.config.json',
    issues,
  )
  const vercelConfig = await readJsonMaybe(path.join(distDir, 'vercel.json'), 'vercel.json', issues)
  const manifest = await readJsonMaybe(
    path.join(distDir, 'manifest.webmanifest'),
    'manifest.webmanifest',
    issues,
  )
  const projectAssetManifest = await readJsonMaybe(
    path.join(distDir, 'project-assets.json'),
    'project-assets.json',
    issues,
  )
  const inferenceRuntime = await readJsonMaybe(
    path.join(distDir, 'inference-runtime.json'),
    'inference-runtime.json',
    issues,
  )
  const serviceWorker = await readMaybe(path.join(distDir, 'sw.js'))
  const bundledAssets = await readdir(path.join(distDir, 'assets')).catch(() => [])

  if (indexHtml && fallbackHtml && indexHtml !== fallbackHtml) {
    issues.push('404.html must match index.html for static-hosting SPA fallback')
  }
  if (indexHtml && !indexHtml.includes('manifest.webmanifest')) {
    issues.push('index.html must reference manifest.webmanifest')
  }
  if (indexHtml && !indexHtml.includes('icons/app-icon.svg')) {
    issues.push('index.html must reference the dedicated PWA app icon')
  }
  if (indexHtml) {
    verifyIndexAssetBase(indexHtml, normalizedBase, issues)
    const indexCsp = extractHtmlCsp(indexHtml)
    if (!indexCsp) {
      issues.push('index.html must include the Web/PWA Content Security Policy meta tag')
    } else {
      verifyCspPolicy(indexCsp, requiredWebCspFragments, 'index.html Web/PWA CSP', issues, {
        forbiddenFragments: ["frame-ancestors 'none'"],
      })
    }
  }
  if (fallbackHtml) {
    const fallbackCsp = extractHtmlCsp(fallbackHtml)
    if (!fallbackCsp) {
      issues.push('404.html must include the Web/PWA Content Security Policy meta tag')
    } else {
      verifyCspPolicy(fallbackCsp, requiredWebCspFragments, '404.html Web/PWA CSP', issues, {
        forbiddenFragments: ["frame-ancestors 'none'"],
      })
    }
  }
  if (staticHostingHeaders) verifyStaticHostingHeaders(staticHostingHeaders, issues)
  if (staticHostingRedirects) verifyStaticHostingRedirects(staticHostingRedirects, issues)
  if (azureStaticWebAppConfig) verifyAzureStaticWebAppConfig(azureStaticWebAppConfig, issues)
  if (vercelConfig) verifyVercelConfig(vercelConfig, issues)

  appendWebManifestIssues(manifest, issues)

  const localesDir = path.join(distDir, 'locales')
  for (const locale of requiredLocaleFiles) {
    if (!(await fileExists(path.join(localesDir, locale)))) {
      issues.push(`Missing Web/PWA locale fallback: ${locale}`)
    }
  }
  await verifyWebProjectAssets({ repositoryRoot, distDir, projectAssetManifest, issues })

  if (!bundledAssets.some((file) => /^ort-wasm-simd-threaded\.asyncify-[A-Za-z0-9_-]+\.wasm$/.test(file))) {
    issues.push('Missing bundled WebGPU ONNX runtime WASM asset')
  }

  appendInferenceRuntimeIssues(inferenceRuntime, issues)
  if (serviceWorker) {
    const packageJson = JSON.parse(await readFile(path.join(frontendDirectory, 'package.json'), 'utf8'))
    appendServiceWorkerIssues(serviceWorker, packageJson.version, issues)
  }

  return {
    normalizedBase,
    checkedFileCount: requiredWebDistFiles.length,
    issues,
  }
}

export function verifyIndexAssetBase(indexHtml, basePath, issues = []) {
  const urls = []
  const attrPattern = /\s(?:href|src)="([^"]+)"/g
  let match
  while ((match = attrPattern.exec(indexHtml)) !== null) urls.push(match[1])

  if (basePath === '/') {
    for (const url of urls) {
      if (url.startsWith(releaseSubpathBase)) {
        issues.push(`index.html root build must not retain subpath asset URL ${url}`)
      }
    }
    return issues
  }

  for (const needle of [
    `${basePath}manifest.webmanifest`,
    `${basePath}favicon.svg`,
    `${basePath}assets/`,
  ]) {
    if (!indexHtml.includes(needle)) {
      issues.push(`index.html subpath build must reference ${needle}`)
    }
  }
  for (const url of urls) {
    if (url.startsWith('/') && !url.startsWith(basePath)) {
      issues.push(`index.html subpath build has root-relative URL outside ${basePath}: ${url}`)
    }
  }
  return issues
}

function appendWebManifestIssues(manifest, issues) {
  if (!manifest) return
  if (!nonEmptyString(manifest.name)) issues.push('manifest.webmanifest name is required')
  if (!nonEmptyString(manifest.short_name)) issues.push('manifest.webmanifest short_name is required')
  if (manifest.display !== 'standalone') issues.push('manifest.webmanifest display must be standalone')
  const icons = Array.isArray(manifest.icons) ? manifest.icons : []
  if (!Array.isArray(manifest.icons)) {
    issues.push('manifest.webmanifest icons must be an array')
  } else if (icons.length === 0) {
    issues.push('manifest.webmanifest must include at least one icon')
  }
  if (manifest.start_url !== '.') issues.push('manifest.webmanifest start_url should stay relative')
  if (manifest.scope !== '.') issues.push('manifest.webmanifest scope should stay relative')
  for (const icon of icons) {
    if (typeof icon?.src !== 'string' || icon.src.startsWith('/')) {
      issues.push('manifest.webmanifest icon src values must stay relative for subpath deployments')
    }
  }
  for (const iconPath of requiredPwaIcons) {
    if (!icons.some((icon) => icon?.src === iconPath)) {
      issues.push(`manifest.webmanifest must include ${iconPath}`)
    }
  }
  if (!icons.some((icon) => String(icon?.purpose ?? '').includes('maskable'))) {
    issues.push('manifest.webmanifest must include a maskable icon for install surfaces')
  }
  let shortcuts = []
  if (manifest.shortcuts !== undefined) {
    if (Array.isArray(manifest.shortcuts)) shortcuts = manifest.shortcuts
    else issues.push('manifest.webmanifest shortcuts must be an array')
  }
  for (const shortcut of shortcuts) {
    if (typeof shortcut?.url !== 'string' || shortcut.url.startsWith('/')) {
      issues.push('manifest.webmanifest shortcut URLs must stay relative for subpath deployments')
    }
  }
}

function appendInferenceRuntimeIssues(inferenceRuntime, issues) {
  if (!inferenceRuntime) {
    issues.push('Missing Web/PWA inference runtime contract: inference-runtime.json')
    return
  }
  if (inferenceRuntime.schema !== 'monogatari-inference-runtime/v1') {
    issues.push('inference-runtime.json must use schema monogatari-inference-runtime/v1')
  }
  if (inferenceRuntime.target !== 'web' || inferenceRuntime.backend !== 'webgpu') {
    issues.push('inference-runtime.json must bind Web/PWA packages to WebGPU')
  }
  if (!nonEmptyString(inferenceRuntime.model_id)) {
    issues.push('inference-runtime.json model_id is required')
  }
  if (!['q4', 'q4f16', 'q8', 'fp16', 'fp32'].includes(inferenceRuntime.dtype)) {
    issues.push('inference-runtime.json dtype is unsupported')
  }
  if (!Number.isInteger(inferenceRuntime.max_new_tokens)
    || inferenceRuntime.max_new_tokens < 1
    || inferenceRuntime.max_new_tokens > 2048) {
    issues.push('inference-runtime.json max_new_tokens must be an integer from 1 to 2048')
  }
}

function appendServiceWorkerIssues(serviceWorker, version, issues) {
  if (!serviceWorker.includes(`monogatari-web-v${version}`)) {
    issues.push('sw.js cache name must include the frontend package version')
  }
  if (serviceWorker.includes('__APP_VERSION__') || serviceWorker.includes('__BUILD_FINGERPRINT__')) {
    issues.push('sw.js production cache identity placeholders must be replaced')
  }
  if (!new RegExp(`monogatari-web-v${version.replaceAll('.', '\\.')}-[a-f0-9]{12}`).test(serviceWorker)) {
    issues.push('sw.js cache name must include a 12-character production content fingerprint')
  }
  for (const locale of requiredLocaleFiles) {
    if (!serviceWorker.includes(`/locales/${locale}`)) {
      issues.push(`sw.js app shell must include /locales/${locale}`)
    }
  }
  for (const iconPath of requiredPwaIcons) {
    if (!serviceWorker.includes(`/${iconPath}`)) {
      issues.push(`sw.js app shell must include /${iconPath}`)
    }
  }
  if (!serviceWorker.includes('/project-assets.json')) {
    issues.push('sw.js app shell must include /project-assets.json')
  }
  if (!serviceWorker.includes('/inference-runtime.json')) {
    issues.push('sw.js app shell must include /inference-runtime.json')
  }
  if (!serviceWorker.includes('cacheProjectAssets()')) {
    issues.push('sw.js install flow must cache project assets from the generated manifest')
  }
  for (const prefix of ['/scenes/', '/dialogue/', '/endings/']) {
    if (!serviceWorker.includes(prefix)) {
      issues.push(`sw.js must cache and route project content under ${prefix}`)
    }
  }
}

async function verifyWebProjectAssets({ repositoryRoot, distDir, projectAssetManifest, issues }) {
  const sourceAssetsDir = path.join(repositoryRoot, 'data', 'assets')
  const sourceEventsDir = path.join(repositoryRoot, 'data', 'events')
  if (!(await directoryExists(sourceAssetsDir))) {
    issues.push('data/assets must exist for Web/PWA project asset packaging')
    return
  }

  const sourceAssets = await walkFiles(sourceAssetsDir)
  if (sourceAssets.length === 0) {
    issues.push('data/assets must contain project assets for Web/PWA packaging')
    return
  }
  if (!projectAssetManifest) {
    issues.push('Missing Web/PWA project asset manifest: project-assets.json')
    return
  }
  if (projectAssetManifest.schema !== 'monogatari-web-project-assets/v1') {
    issues.push('project-assets.json must use schema monogatari-web-project-assets/v1')
  }
  if (!Array.isArray(projectAssetManifest.assets)) {
    issues.push('project-assets.json assets must be an array')
    return
  }
  if (!Array.isArray(projectAssetManifest.event_catalogs)) {
    issues.push('project-assets.json event_catalogs must be an array')
    return
  }

  const manifestAssets = new Set(projectAssetManifest.assets)
  if (manifestAssets.size !== projectAssetManifest.assets.length) {
    issues.push('project-assets.json assets must not contain duplicates')
  }
  for (const sourceAsset of sourceAssets) {
    const relativeAssetPath = path.relative(sourceAssetsDir, sourceAsset)
    const manifestAssetPath = `/assets/${relativeAssetPath.replaceAll(path.sep, '/')}`
    if (!(await fileExists(path.join(distDir, 'assets', relativeAssetPath)))) {
      issues.push(`Missing Web/PWA project asset: assets/${relativeAssetPath.replaceAll(path.sep, '/')}`)
    }
    if (!manifestAssets.has(manifestAssetPath)) {
      issues.push(`project-assets.json must include ${manifestAssetPath}`)
    }
  }
  for (const assetPath of projectAssetManifest.assets) {
    if (typeof assetPath !== 'string' || !assetPath.startsWith('/assets/')) {
      issues.push(`project-assets.json asset paths must be root-relative /assets entries: ${assetPath}`)
    } else if (!(await fileExists(path.join(distDir, assetPath.slice(1))))) {
      issues.push(`project-assets.json references missing dist asset: ${assetPath}`)
    }
  }

  const sourceEvents = await walkFilesIfDirectory(sourceEventsDir, issues, 'data/events')
  const manifestEvents = new Set(projectAssetManifest.event_catalogs)
  if (manifestEvents.size !== projectAssetManifest.event_catalogs.length) {
    issues.push('project-assets.json event_catalogs must not contain duplicates')
  }
  for (const sourceEvent of sourceEvents) {
    const relativeEventPath = path.relative(sourceEventsDir, sourceEvent)
    const manifestEventPath = `/events/${relativeEventPath.replaceAll(path.sep, '/')}`
    if (!(await fileExists(path.join(distDir, 'events', relativeEventPath)))) {
      issues.push(`Missing Web/PWA story event catalog: events/${relativeEventPath.replaceAll(path.sep, '/')}`)
    }
    if (!manifestEvents.has(manifestEventPath)) {
      issues.push(`project-assets.json must include ${manifestEventPath}`)
    }
  }
  for (const eventPath of projectAssetManifest.event_catalogs) {
    if (typeof eventPath !== 'string' || !eventPath.startsWith('/events/')) {
      issues.push(`project-assets.json event catalog paths must be root-relative /events entries: ${eventPath}`)
    } else if (!(await fileExists(path.join(distDir, eventPath.slice(1))))) {
      issues.push(`project-assets.json references missing story event catalog: ${eventPath}`)
    }
  }

  for (const content of [
    { directory: 'scenes', manifestField: 'scene_files', prefix: '/scenes/' },
    { directory: 'dialogue', manifestField: 'dialogue_files', prefix: '/dialogue/' },
    { directory: 'endings', manifestField: 'ending_files', prefix: '/endings/' },
    { directory: 'characters', manifestField: 'character_files', prefix: '/characters/' },
    { directory: 'knowledge', manifestField: 'knowledge_files', prefix: '/knowledge/' },
  ]) {
    const manifestPaths = projectAssetManifest[content.manifestField]
    if (!Array.isArray(manifestPaths)) {
      issues.push(`project-assets.json ${content.manifestField} must be an array`)
      continue
    }
    const manifestSet = new Set(manifestPaths)
    if (manifestSet.size !== manifestPaths.length) {
      issues.push(`project-assets.json ${content.manifestField} must not contain duplicates`)
    }
    const sourceDir = path.join(repositoryRoot, 'data', content.directory)
    const sourceFiles = await walkFilesIfDirectory(sourceDir, issues, `data/${content.directory}`)
    for (const sourceFile of sourceFiles) {
      const relativePath = path.relative(sourceDir, sourceFile).replaceAll(path.sep, '/')
      const manifestPath = `${content.prefix}${relativePath}`
      if (!manifestSet.has(manifestPath)) issues.push(`project-assets.json must include ${manifestPath}`)
      if (!(await fileExists(path.join(distDir, content.directory, relativePath)))) {
        issues.push(`Missing Web/PWA project content: ${content.directory}/${relativePath}`)
      }
    }
    for (const manifestPath of manifestPaths) {
      if (typeof manifestPath !== 'string' || !manifestPath.startsWith(content.prefix)) {
        issues.push(`project-assets.json ${content.manifestField} path is invalid: ${manifestPath}`)
      } else if (!(await fileExists(path.join(distDir, manifestPath.slice(1))))) {
        issues.push(`project-assets.json references missing project content: ${manifestPath}`)
      }
    }
  }
}

async function walkFilesIfDirectory(directory, issues, label) {
  if (!(await directoryExists(directory))) {
    issues.push(`${label} must exist for Web/PWA project content packaging`)
    return []
  }
  return await walkFiles(directory)
}

async function walkFiles(directory, files = []) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const entryPath = path.join(directory, entry.name)
    if (entry.isDirectory()) await walkFiles(entryPath, files)
    else if (entry.isFile()) files.push(entryPath)
  }
  return files
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

async function readMaybe(filePath) {
  try {
    return await readFile(filePath, 'utf8')
  } catch {
    return null
  }
}

async function readJsonMaybe(filePath, label, issues) {
  const content = await readMaybe(filePath)
  if (content === null) return null
  try {
    return JSON.parse(content)
  } catch (error) {
    issues.push(`${label} must contain valid JSON: ${error.message}`)
    return null
  }
}

function nonEmptyString(value) {
  return typeof value === 'string' && value.trim().length > 0
}
