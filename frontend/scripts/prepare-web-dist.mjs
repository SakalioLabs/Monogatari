import { copyFile, cp, readdir, stat, writeFile } from 'node:fs/promises'
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
const projectAssetManifestPath = path.join(distDir, 'project-assets.json')
const staticHostingHeadersPath = path.join(distDir, '_headers')
const staticHostingRedirectsPath = path.join(distDir, '_redirects')
const azureStaticWebAppConfigPath = path.join(distDir, 'staticwebapp.config.json')
const vercelConfigPath = path.join(distDir, 'vercel.json')
const webSecurityCsp =
  "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' asset: http://asset.localhost data: blob:; media-src 'self' asset: http://asset.localhost data: blob:; font-src 'self' data:; connect-src 'self' asset: http://asset.localhost https: http://localhost:* http://127.0.0.1:* ws://localhost:* ws://127.0.0.1:*; worker-src 'self' blob:; object-src 'none'; base-uri 'self'; form-action 'none'; frame-src 'none'; frame-ancestors 'none'"
const webPermissionsPolicy = 'camera=(), microphone=(), geolocation=(), payment=(), usb=(), serial=(), bluetooth=()'
const requiredFiles = [
  'index.html',
  'manifest.webmanifest',
  'sw.js',
  'offline.html',
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
    '/icons/* /icons/:splat 200',
    '/locales/* /locales/:splat 200',
    '/manifest.webmanifest /manifest.webmanifest 200',
    '/sw.js /sw.js 200',
    '/offline.html /offline.html 200',
    '/project-assets.json /project-assets.json 200',
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
        '/icons/*',
        '/locales/*',
        '/manifest.webmanifest',
        '/sw.js',
        '/offline.html',
        '/project-assets.json',
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

  return {
    schema: 'monogatari-web-project-assets/v1',
    generated_by: 'frontend/scripts/prepare-web-dist.mjs',
    assets: assetFiles,
    event_catalogs: eventCatalogFiles,
    scene_files: sceneFiles,
    dialogue_files: dialogueFiles,
    ending_files: endingFiles,
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
await writeFile(projectAssetManifestPath, `${JSON.stringify(await projectAssetManifest(), null, 2)}\n`)
await writeFile(staticHostingHeadersPath, staticHostingHeaders())
await writeFile(staticHostingRedirectsPath, staticHostingRedirects())
await writeFile(azureStaticWebAppConfigPath, `${JSON.stringify(azureStaticWebAppConfig(), null, 2)}\n`)
await writeFile(vercelConfigPath, `${JSON.stringify(vercelConfig(), null, 2)}\n`)

console.log('[web-dist] Static hosting assets ready: shell, PWA metadata, project assets, scenes, dialogues, endings, story events, and project asset manifest')
