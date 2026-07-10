import { readdir, readFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import path from 'node:path'

const frontendDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const repoRoot = path.resolve(frontendDir, '..')
const sourceDir = path.join(frontendDir, 'src')
const localeDir = path.join(repoRoot, 'data', 'locales')
const issues = []
const localeNames = ['en', 'zh-CN', 'zh', 'ja-JP', 'ja', 'ko-KR', 'ko']

const catalogs = Object.fromEntries(await Promise.all(localeNames.map(async (locale) => {
  const document = await readJson(path.join(localeDir, `${locale}.json`))
  return [locale, document.strings ?? {}]
})))

const englishKeys = Object.keys(catalogs.en).sort()
for (const locale of localeNames) {
  const catalog = catalogs[locale]
  const keys = Object.keys(catalog).sort()
  const missing = englishKeys.filter((key) => !(key in catalog))
  const extra = keys.filter((key) => !(key in catalogs.en))

  if (missing.length > 0) issues.push(`${locale} is missing keys: ${missing.join(', ')}`)
  if (extra.length > 0) issues.push(`${locale} has keys absent from en: ${extra.join(', ')}`)

  for (const key of englishKeys) {
    const value = catalog[key]
    if (typeof value !== 'string' || value.trim().length === 0) {
      issues.push(`${locale}.${key} must be a non-empty string`)
      continue
    }
    if (/\?{2,}|\uFFFD/.test(value)) {
      issues.push(`${locale}.${key} contains replacement characters or encoding damage`)
    }

    const expectedTokens = interpolationTokens(catalogs.en[key])
    const actualTokens = interpolationTokens(value)
    if (expectedTokens.join('|') !== actualTokens.join('|')) {
      issues.push(`${locale}.${key} interpolation tokens differ from en (${actualTokens.join(', ') || 'none'})`)
    }
  }
}

const sourceFiles = await collectSourceFiles(sourceDir)
const referencedKeys = new Map()
for (const filePath of sourceFiles) {
  const source = await readFile(filePath, 'utf8')
  const keyPattern = /\bt\(\s*(['"])([^'"\n]+)\1/g
  for (const match of source.matchAll(keyPattern)) {
    const key = match[2]
    if (!referencedKeys.has(key)) referencedKeys.set(key, [])
    referencedKeys.get(key).push(path.relative(frontendDir, filePath))
  }
}

for (const [key, files] of referencedKeys) {
  if (!(key in catalogs.en)) {
    issues.push(`translation key ${key} is referenced but missing from catalogs (${[...new Set(files)].join(', ')})`)
  }
}

const localizedSurfaces = [
  'src/App.vue',
  'src/components/GlobalSearch.vue',
  'src/views/AnalyticsView.vue',
  'src/views/AudioView.vue',
  'src/views/BacklogView.vue',
  'src/views/CGGalleryView.vue',
  'src/views/CharacterEditorView.vue',
  'src/views/CharacterGalleryView.vue',
  'src/views/ChatView.vue',
  'src/views/GameView.vue',
  'src/views/GroupChatView.vue',
  'src/views/HomeView.vue',
  'src/views/KnowledgeBaseView.vue',
  'src/views/MarketplaceView.vue',
  'src/views/PluginView.vue',
  'src/views/QualitySuiteView.vue',
  'src/views/SceneAssetsView.vue',
  'src/views/SettingsView.vue',
  'src/views/TitleScreenView.vue',
  'src/views/WorkflowEditor.vue',
]

for (const relativePath of localizedSurfaces) {
  const source = await readFile(path.join(frontendDir, relativePath), 'utf8')
  if (!source.includes('useI18n')) {
    issues.push(`${relativePath} is a localized surface but no longer uses useI18n`)
  }
}

const [runtimeSource, mainSource] = await Promise.all([
  readFile(path.join(sourceDir, 'lib', 'i18n.ts'), 'utf8'),
  readFile(path.join(sourceDir, 'main.ts'), 'utf8'),
])
for (const locale of ['en', 'zh-CN', 'ja-JP', 'ko-KR']) {
  if (!runtimeSource.includes(`code: '${locale}'`)) {
    issues.push(`i18n runtime must expose ${locale}`)
  }
}
for (const [needle, message] of [
  ['Promise.all([', 'i18n runtime must load fallback and target catalogs together'],
  ['requestId !== loadSequence', 'i18n runtime must reject stale locale responses'],
  ['document.documentElement.lang = locale', 'i18n runtime must update the document language'],
  ['await loadI18n()', 'application bootstrap must await locale loading before mount'],
]) {
  const source = needle === 'await loadI18n()' ? mainSource : runtimeSource
  if (!source.includes(needle)) issues.push(message)
}

if (issues.length > 0) {
  throw new Error(`i18n coverage verification failed:\n${issues.join('\n')}`)
}

console.log(`[i18n] OK: ${localeNames.length} catalogs, ${englishKeys.length} keys, ${referencedKeys.size} referenced keys, and ${localizedSurfaces.length} localized surfaces verified`)

async function readJson(filePath) {
  return JSON.parse(await readFile(filePath, 'utf8'))
}

async function collectSourceFiles(directory) {
  const entries = await readdir(directory, { withFileTypes: true })
  const files = await Promise.all(entries.map(async (entry) => {
    const filePath = path.join(directory, entry.name)
    if (entry.isDirectory()) return collectSourceFiles(filePath)
    return /\.(?:ts|vue)$/.test(entry.name) ? [filePath] : []
  }))
  return files.flat()
}

function interpolationTokens(value) {
  if (typeof value !== 'string') return []
  return [...value.matchAll(/\{([a-zA-Z0-9_]+)\}/g)].map((match) => match[1]).sort()
}
