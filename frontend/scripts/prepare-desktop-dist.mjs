import { copyFile, readdir, stat, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const frontendDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const distDir = path.join(frontendDir, 'dist')
const prohibitedProjectEntries = [
  'campaigns',
  'characters',
  'dialogue',
  'endings',
  'events',
  'knowledge',
  'models',
  'project-assets.json',
  'roleplays',
  'scenes',
  'settings.json',
  'workflows',
]

async function exists(target) {
  try {
    await stat(target)
    return true
  } catch {
    return false
  }
}

const present = []
for (const entry of prohibitedProjectEntries) {
  if (await exists(path.join(distDir, entry))) present.push(entry)
}
if (present.length > 0) {
  throw new Error(`Desktop shell contains project content: ${present.join(', ')}`)
}

await copyFile(path.join(distDir, 'index.html'), path.join(distDir, '404.html'))
const rootEntries = (await readdir(distDir)).sort()
await writeFile(
  path.join(distDir, 'desktop-shell.json'),
  `${JSON.stringify({
    schema: 'monogatari-desktop-shell/v1',
    project_content_embedded: false,
    root_entries: rootEntries,
  }, null, 2)}\n`,
  'utf8',
)

console.log('[desktop-dist] Project-free desktop shell ready')
