import { readdir, readFile, stat } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const scriptDir = path.dirname(fileURLToPath(import.meta.url))
const distDir = path.resolve(scriptDir, '..', 'dist')
const assetsDir = path.join(distDir, 'assets')

const KB = 1024
const budgets = {
  entryJs: 180 * KB,
  entryCss: 40 * KB,
  anyJs: 900 * KB,
  anyCss: 80 * KB,
}

const allowedLazyChunks = [
  { pattern: /^three\.module-.*\.js$/, maxBytes: 850 * KB, label: 'Three.js core renderer' },
  { pattern: /^lib-.*\.js$/, maxBytes: 520 * KB, label: 'Pixi/Live2D runtime' },
  { pattern: /^index\.es-.*\.js$/, maxBytes: 400 * KB, label: 'Live2D display module' },
  { pattern: /^GLTFLoader-.*\.js$/, maxBytes: 90 * KB, label: 'GLTF loader' },
  { pattern: /^OrbitControls-.*\.js$/, maxBytes: 50 * KB, label: 'Orbit controls' },
  { pattern: /^url-.*\.js$/, maxBytes: 90 * KB, label: 'asset URL helper chunk' },
]

function formatBytes(value) {
  return `${(value / KB).toFixed(1)} KiB`
}

function assetNameFromUrl(value) {
  const clean = value.split('?')[0].split('#')[0]
  const marker = '/assets/'
  const idx = clean.lastIndexOf(marker)
  return idx >= 0 ? clean.slice(idx + marker.length) : path.basename(clean)
}

function pushIssue(issues, label, file, actual, expected) {
  issues.push(`${label} budget exceeded: ${file} is ${formatBytes(actual)} (limit ${formatBytes(expected)})`)
}

const indexHtml = await readFile(path.join(distDir, 'index.html'), 'utf8')
const entryJs = [...indexHtml.matchAll(/<script\b[^>]*\bsrc="([^"]+\.js)"[^>]*>/g)].map((match) =>
  assetNameFromUrl(match[1]),
)
const entryCss = [...indexHtml.matchAll(/<link\b[^>]*\bhref="([^"]+\.css)"[^>]*>/g)].map((match) =>
  assetNameFromUrl(match[1]),
)

const assetFiles = await readdir(assetsDir)
const assetSizes = new Map()
for (const file of assetFiles) {
  const info = await stat(path.join(assetsDir, file))
  if (info.isFile()) assetSizes.set(file, info.size)
}

const issues = []

for (const file of entryJs) {
  const size = assetSizes.get(file)
  if (size === undefined) {
    issues.push(`Entry script referenced by index.html is missing: ${file}`)
    continue
  }
  if (size > budgets.entryJs) pushIssue(issues, 'Entry JS', file, size, budgets.entryJs)
  if (allowedLazyChunks.some((chunk) => chunk.pattern.test(file))) {
    issues.push(`Renderer-only lazy chunk is referenced from index.html entry scripts: ${file}`)
  }
}

for (const file of entryCss) {
  const size = assetSizes.get(file)
  if (size === undefined) {
    issues.push(`Entry stylesheet referenced by index.html is missing: ${file}`)
    continue
  }
  if (size > budgets.entryCss) pushIssue(issues, 'Entry CSS', file, size, budgets.entryCss)
}

for (const [file, size] of assetSizes) {
  if (file.endsWith('.js') && size > budgets.anyJs) pushIssue(issues, 'JS asset', file, size, budgets.anyJs)
  if (file.endsWith('.css') && size > budgets.anyCss) pushIssue(issues, 'CSS asset', file, size, budgets.anyCss)

  const lazyBudget = allowedLazyChunks.find((chunk) => chunk.pattern.test(file))
  if (lazyBudget && size > lazyBudget.maxBytes) {
    pushIssue(issues, lazyBudget.label, file, size, lazyBudget.maxBytes)
  }
}

if (issues.length > 0) {
  console.error(`[bundle-budget] ${issues.length} budget issue(s):`)
  for (const issue of issues) console.error(`- ${issue}`)
  process.exit(1)
}

const largest = [...assetSizes.entries()]
  .filter(([file]) => file.endsWith('.js') || file.endsWith('.css'))
  .sort((a, b) => b[1] - a[1])
  .slice(0, 5)
  .map(([file, size]) => `${file} ${formatBytes(size)}`)
  .join('; ')

console.log(
  `[bundle-budget] OK: entry JS <= ${formatBytes(budgets.entryJs)}, entry CSS <= ${formatBytes(
    budgets.entryCss,
  )}. Largest assets: ${largest}`,
)
