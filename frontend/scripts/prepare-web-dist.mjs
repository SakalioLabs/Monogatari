import { copyFile, stat, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const scriptDir = path.dirname(fileURLToPath(import.meta.url))
const distDir = path.resolve(scriptDir, '..', 'dist')
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

console.log('[web-dist] Static hosting assets ready: 404.html, .nojekyll, manifest.webmanifest, sw.js, offline.html, PWA icons')
