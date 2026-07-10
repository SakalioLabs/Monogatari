import { readFile, writeFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import path from 'node:path'

const frontendDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const repoRoot = path.resolve(frontendDir, '..')
const dataDir = path.join(repoRoot, 'data', 'locales')
const publicDir = path.join(frontendDir, 'public', 'locales')
const sourceDir = path.join(frontendDir, 'src', 'locales')
const writeMode = process.argv.includes('--write')
const issues = []

const localeAliases = {
  en: 'en',
  'zh-CN': 'zh-CN',
  zh: 'zh-CN',
  'ja-JP': 'ja-JP',
  ja: 'ja-JP',
  'ko-KR': 'ko-KR',
  ko: 'ko-KR',
}

const primaryCatalogs = Object.fromEntries(await Promise.all(
  [...new Set(Object.values(localeAliases))].map(async (locale) => [
    locale,
    await readJson(path.join(dataDir, `${locale}.json`)),
  ]),
))

for (const [locale, primaryLocale] of Object.entries(localeAliases)) {
  const expected = {
    ...primaryCatalogs[primaryLocale],
    locale,
  }
  const canonicalPath = path.join(dataDir, `${locale}.json`)
  const publicPath = path.join(publicDir, `${locale}.json`)

  if (writeMode) {
    if (locale !== primaryLocale) await writeJson(canonicalPath, expected)
    await writeJson(publicPath, expected)
  } else {
    await requireCatalog(canonicalPath, expected, `${locale} canonical catalog`)
    await requireCatalog(publicPath, expected, `${locale} public catalog`)
  }

  if (locale === primaryLocale && locale !== 'en') {
    const sourcePath = path.join(sourceDir, `${locale}.json`)
    if (writeMode) {
      await writeJson(sourcePath, expected)
    } else {
      await requireCatalog(sourcePath, expected, `${locale} embedded catalog`)
    }
  }
}

if (issues.length > 0) {
  throw new Error(`Locale synchronization failed:\n${issues.join('\n')}`)
}

console.log(writeMode
  ? '[locales] Canonical catalogs synchronized to aliases and runtime copies'
  : '[locales] OK: canonical, alias, public, and embedded catalogs are synchronized')

async function readJson(filePath) {
  return JSON.parse(await readFile(filePath, 'utf8'))
}

async function writeJson(filePath, value) {
  await writeFile(filePath, `${JSON.stringify(value, null, 2)}\n`, 'utf8')
}

async function requireCatalog(filePath, expected, label) {
  try {
    const actual = await readJson(filePath)
    if (JSON.stringify(actual) !== JSON.stringify(expected)) {
      issues.push(`${label} is out of sync; run npm run sync:locales`)
    }
  } catch (error) {
    issues.push(`${label} cannot be read: ${error.message}`)
  }
}
