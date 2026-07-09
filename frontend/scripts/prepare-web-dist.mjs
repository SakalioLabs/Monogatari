import { copyFile, cp, readdir, stat, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const scriptDir = path.dirname(fileURLToPath(import.meta.url))
const rootDir = path.resolve(scriptDir, '..', '..')
const distDir = path.resolve(scriptDir, '..', 'dist')
const projectAssetsDir = path.join(rootDir, 'data', 'assets')
const distProjectAssetsDir = path.join(distDir, 'assets')
const projectAssetManifestPath = path.join(distDir, 'project-assets.json')
const staticHostingHeadersPath = path.join(distDir, '_headers')
const webSecurityCsp =
  "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' asset: http://asset.localhost data: blob:; media-src 'self' asset: http://asset.localhost data: blob:; font-src 'self' data:; connect-src 'self' asset: http://asset.localhost https: http://localhost:* http://127.0.0.1:* ws://localhost:* ws://127.0.0.1:*; worker-src 'self' blob:; object-src 'none'; base-uri 'self'; form-action 'none'; frame-src 'none'; frame-ancestors 'none'"
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
    '  Permissions-Policy: camera=(), microphone=(), geolocation=(), payment=(), usb=(), serial=(), bluetooth=()',
    '',
  ].join('\n')
}

async function projectAssetManifest() {
  const assetFiles = (await walkFiles(projectAssetsDir, []))
    .map((file) => `/assets/${path.relative(projectAssetsDir, file).replaceAll(path.sep, '/')}`)
    .sort()

  return {
    schema: 'monogatari-web-project-assets/v1',
    generated_by: 'frontend/scripts/prepare-web-dist.mjs',
    assets: assetFiles,
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
await writeFile(projectAssetManifestPath, `${JSON.stringify(await projectAssetManifest(), null, 2)}\n`)
await writeFile(staticHostingHeadersPath, staticHostingHeaders())

console.log('[web-dist] Static hosting assets ready: 404.html, .nojekyll, _headers, manifest.webmanifest, sw.js, offline.html, PWA icons, project assets, project asset manifest')
