import { createHash } from 'node:crypto'
import { copyFile, cp, mkdir, readFile, readdir, stat, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const scriptDir = path.dirname(fileURLToPath(import.meta.url))
const rootDir = path.resolve(scriptDir, '..', '..')
const distDir = path.resolve(scriptDir, '..', 'dist')
const projectAssetsDir = path.join(rootDir, 'data', 'assets')
const distProjectAssetsDir = path.join(distDir, 'assets')
const projectEventsDir = path.join(rootDir, 'data', 'events')
const distProjectEventsDir = path.join(distDir, 'events')
const projectScenesDir = path.join(rootDir, 'data', 'scenes')
const distProjectScenesDir = path.join(distDir, 'scenes')
const projectDialoguesDir = path.join(rootDir, 'data', 'dialogue')
const distProjectDialoguesDir = path.join(distDir, 'dialogue')
const projectEndingsDir = path.join(rootDir, 'data', 'endings')
const distProjectEndingsDir = path.join(distDir, 'endings')
const projectCharactersDir = path.join(rootDir, 'data', 'characters')
const distProjectCharactersDir = path.join(distDir, 'characters')
const projectKnowledgeDir = path.join(rootDir, 'data', 'knowledge')
const distProjectKnowledgeDir = path.join(distDir, 'knowledge')
const projectSettingsPath = path.join(rootDir, 'data', 'settings.json')
const projectWebModelDir = path.join(rootDir, 'data', 'models', 'webgpu')
const distWebModelDir = path.join(distDir, 'models', 'webgpu')
const ortRuntimeSourceDir = path.resolve(scriptDir, '..', 'node_modules', 'onnxruntime-web', 'dist')
const distOrtRuntimeDir = path.join(distDir, 'ort')
const ortRuntimeFiles = [
  'ort-wasm-simd-threaded.jsep.mjs',
]
const inferenceRuntimePath = path.join(distDir, 'inference-runtime.json')
const projectAssetManifestPath = path.join(distDir, 'project-assets.json')
const distServiceWorkerPath = path.join(distDir, 'sw.js')
const frontendPackagePath = path.resolve(scriptDir, '..', 'package.json')
const staticHostingHeadersPath = path.join(distDir, '_headers')
const staticHostingRedirectsPath = path.join(distDir, '_redirects')
const azureStaticWebAppConfigPath = path.join(distDir, 'staticwebapp.config.json')
const vercelConfigPath = path.join(distDir, 'vercel.json')
const webSecurityCsp =
  "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' asset: http://asset.localhost data: blob:; media-src 'self' asset: http://asset.localhost data: blob:; font-src 'self' data:; connect-src 'self' asset: http://asset.localhost https: http://localhost:* http://127.0.0.1:* ws://localhost:* ws://127.0.0.1:*; worker-src 'self' blob:; object-src 'none'; base-uri 'self'; form-action 'none'; frame-src 'none'; frame-ancestors 'none'"
const webPermissionsPolicy = 'camera=(), microphone=(), geolocation=(), payment=(), usb=(), serial=(), bluetooth=()'
const requiredFiles = [
  'index.html',
  'manifest.webmanifest',
  'sw.js',
  'offline.html',
  'offline-i18n.js',
  'icons/app-icon.svg',
  'icons/maskable-icon.svg',
]

async function fileExists(filePath) {
  try {
    const info = await stat(filePath)
    return info.isFile()
  } catch {
    return false
  }
}

async function directoryExists(dirPath) {
  try {
    const info = await stat(dirPath)
    return info.isDirectory()
  } catch {
    return false
  }
}

async function walkFiles(dir, files = []) {
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const entryPath = path.join(dir, entry.name)
    if (entry.isDirectory()) {
      await walkFiles(entryPath, files)
    } else if (entry.isFile()) {
      files.push(entryPath)
    }
  }
  return files
}

async function injectServiceWorkerBuildId() {
  const packageDocument = JSON.parse(await readFile(frontendPackagePath, 'utf8'))
  const files = (await walkFiles(distDir, []))
    .filter((file) => path.resolve(file) !== path.resolve(distServiceWorkerPath))
    .sort((left, right) => left.localeCompare(right))
  const hash = createHash('sha256')

  for (const file of files) {
    const relativePath = path.relative(distDir, file).replaceAll(path.sep, '/')
    hash.update(relativePath)
    hash.update('\0')
    hash.update(await readFile(file))
    hash.update('\0')
  }

  const buildFingerprint = hash.digest('hex').slice(0, 12)
  const source = await readFile(distServiceWorkerPath, 'utf8')
  const output = source
    .replace('__APP_VERSION__', String(packageDocument.version))
    .replace('__BUILD_FINGERPRINT__', buildFingerprint)

  if (output.includes('__APP_VERSION__') || output.includes('__BUILD_FINGERPRINT__')) {
    throw new Error('Service worker build identity placeholders were not replaced')
  }

  await writeFile(distServiceWorkerPath, output)
  return `v${packageDocument.version}-${buildFingerprint}`
}

function staticHostingHeaders() {
  return [
    '/*',
    `  Content-Security-Policy: ${webSecurityCsp}`,
    '  X-Content-Type-Options: nosniff',
    '  Referrer-Policy: no-referrer',
    `  Permissions-Policy: ${webPermissionsPolicy}`,
    '',
  ].join('\n')
}

function staticHostingRedirects() {
  return [
    '/assets/* /assets/:splat 200',
    '/events/* /events/:splat 200',
    '/scenes/* /scenes/:splat 200',
    '/dialogue/* /dialogue/:splat 200',
    '/endings/* /endings/:splat 200',
    '/characters/* /characters/:splat 200',
    '/knowledge/* /knowledge/:splat 200',
    '/models/* /models/:splat 200',
    '/ort/* /ort/:splat 200',
    '/icons/* /icons/:splat 200',
    '/locales/* /locales/:splat 200',
    '/manifest.webmanifest /manifest.webmanifest 200',
    '/sw.js /sw.js 200',
    '/offline.html /offline.html 200',
    '/offline-i18n.js /offline-i18n.js 200',
    '/project-assets.json /project-assets.json 200',
    '/inference-runtime.json /inference-runtime.json 200',
    '/favicon.svg /favicon.svg 200',
    '/* /index.html 200',
    '',
  ].join('\n')
}

function azureStaticWebAppConfig() {
  return {
    navigationFallback: {
      rewrite: '/index.html',
      exclude: [
        '/assets/*',
        '/events/*',
        '/scenes/*',
        '/dialogue/*',
        '/endings/*',
        '/characters/*',
        '/knowledge/*',
        '/models/*',
        '/ort/*',
        '/icons/*',
        '/locales/*',
        '/manifest.webmanifest',
        '/sw.js',
        '/offline.html',
        '/offline-i18n.js',
        '/project-assets.json',
        '/inference-runtime.json',
        '/favicon.svg',
      ],
    },
    responseOverrides: {
      404: {
        rewrite: '/404.html',
      },
    },
    globalHeaders: {
      'content-security-policy': webSecurityCsp,
      'x-content-type-options': 'nosniff',
      'referrer-policy': 'no-referrer',
      'permissions-policy': webPermissionsPolicy,
    },
  }
}

function securityHeaderEntries() {
  return [
    { key: 'Content-Security-Policy', value: webSecurityCsp },
    { key: 'X-Content-Type-Options', value: 'nosniff' },
    { key: 'Referrer-Policy', value: 'no-referrer' },
    { key: 'Permissions-Policy', value: webPermissionsPolicy },
  ]
}

function vercelConfig() {
  return {
    $schema: 'https://openapi.vercel.sh/vercel.json',
    headers: [
      {
        source: '/(.*)',
        headers: securityHeaderEntries(),
      },
    ],
    rewrites: [
      {
        source: '/(.*)',
        destination: '/index.html',
      },
    ],
  }
}

async function projectAssetManifest() {
  const assetFiles = (await walkFiles(projectAssetsDir, []))
    .map((file) => `/assets/${path.relative(projectAssetsDir, file).replaceAll(path.sep, '/')}`)
    .sort()
  const eventCatalogFiles = (await walkFiles(projectEventsDir, []))
    .map((file) => `/events/${path.relative(projectEventsDir, file).replaceAll(path.sep, '/')}`)
    .sort()
  const sceneFiles = (await walkFiles(projectScenesDir, []))
    .map((file) => `/scenes/${path.relative(projectScenesDir, file).replaceAll(path.sep, '/')}`)
    .sort()
  const dialogueFiles = (await walkFiles(projectDialoguesDir, []))
    .map((file) => `/dialogue/${path.relative(projectDialoguesDir, file).replaceAll(path.sep, '/')}`)
    .sort()
  const endingFiles = (await walkFiles(projectEndingsDir, []))
    .map((file) => `/endings/${path.relative(projectEndingsDir, file).replaceAll(path.sep, '/')}`)
    .sort()
  const characterFiles = (await walkFiles(projectCharactersDir, []))
    .map((file) => `/characters/${path.relative(projectCharactersDir, file).replaceAll(path.sep, '/')}`)
    .sort()
  const knowledgeFiles = (await walkFiles(projectKnowledgeDir, []))
    .map((file) => `/knowledge/${path.relative(projectKnowledgeDir, file).replaceAll(path.sep, '/')}`)
    .sort()

  return {
    schema: 'monogatari-web-project-assets/v1',
    generated_by: 'frontend/scripts/prepare-web-dist.mjs',
    assets: assetFiles,
    event_catalogs: eventCatalogFiles,
    scene_files: sceneFiles,
    dialogue_files: dialogueFiles,
    ending_files: endingFiles,
    character_files: characterFiles,
    knowledge_files: knowledgeFiles,
  }
}

async function webInferenceRuntime() {
  const settings = JSON.parse(await readFile(projectSettingsPath, 'utf8'))
  const webgpu = settings.ai?.webgpu || {}
  return {
    schema: 'monogatari-inference-runtime/v1',
    target: 'web',
    backend: 'webgpu',
    model_id: webgpu.model_id || webgpu.modelId || 'onnx-community/Qwen2.5-0.5B-Instruct',
    dtype: webgpu.dtype || 'q4',
    max_new_tokens: Number(webgpu.max_new_tokens || webgpu.maxNewTokens || 256),
    temperature: Number(webgpu.temperature || 0.7),
    top_p: Number(webgpu.top_p || webgpu.topP || 0.9),
  }
}

const missing = []

for (const file of requiredFiles) {
  const filePath = path.join(distDir, file)
  if (!(await fileExists(filePath))) missing.push(file)
}

if (missing.length > 0) {
  console.error(`[web-dist] Missing required production asset(s): ${missing.join(', ')}`)
  process.exit(1)
}

await copyFile(path.join(distDir, 'index.html'), path.join(distDir, '404.html'))
await writeFile(path.join(distDir, '.nojekyll'), '')
await cp(projectAssetsDir, distProjectAssetsDir, { recursive: true, force: true })
await cp(projectEventsDir, distProjectEventsDir, { recursive: true, force: true })
await cp(projectScenesDir, distProjectScenesDir, { recursive: true, force: true })
await cp(projectDialoguesDir, distProjectDialoguesDir, { recursive: true, force: true })
await cp(projectEndingsDir, distProjectEndingsDir, { recursive: true, force: true })
await cp(projectCharactersDir, distProjectCharactersDir, { recursive: true, force: true })
await cp(projectKnowledgeDir, distProjectKnowledgeDir, { recursive: true, force: true })
if (await directoryExists(projectWebModelDir)) {
  await cp(projectWebModelDir, distWebModelDir, { recursive: true, force: true })
}
await mkdir(distOrtRuntimeDir, { recursive: true })
for (const file of ortRuntimeFiles) {
  await copyFile(path.join(ortRuntimeSourceDir, file), path.join(distOrtRuntimeDir, file))
}
await writeFile(inferenceRuntimePath, `${JSON.stringify(await webInferenceRuntime(), null, 2)}\n`)
await writeFile(projectAssetManifestPath, `${JSON.stringify(await projectAssetManifest(), null, 2)}\n`)
await writeFile(staticHostingHeadersPath, staticHostingHeaders())
await writeFile(staticHostingRedirectsPath, staticHostingRedirects())
await writeFile(azureStaticWebAppConfigPath, `${JSON.stringify(azureStaticWebAppConfig(), null, 2)}\n`)
await writeFile(vercelConfigPath, `${JSON.stringify(vercelConfig(), null, 2)}\n`)
const serviceWorkerBuildId = await injectServiceWorkerBuildId()

console.log(`[web-dist] Static hosting assets ready (${serviceWorkerBuildId}): shell, WebGPU inference contract, PWA metadata, and project content`)
