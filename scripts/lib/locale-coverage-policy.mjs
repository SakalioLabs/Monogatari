import { readFile as readFileFromDisk } from 'node:fs/promises'
import path from 'node:path'
import { isDeepStrictEqual } from 'node:util'

const defaultEmbeddedLocaleFiles = Object.freeze(['zh-CN.json', 'ja-JP.json', 'ko-KR.json'])
const localeFilePattern = /^[A-Za-z0-9]+(?:-[A-Za-z0-9]+)*\.json$/

export function createLocaleCoveragePolicy(options = {}) {
  const { repositoryRoot, frontendDirectory, requiredLocaleFiles, embeddedLocaleFiles } = resolveBoundaries(options)
  const readTextFile = options.readTextFile ?? readFileFromDisk
  const log = options.log ?? console.log
  for (const [name, value] of Object.entries({ readTextFile, log })) {
    if (typeof value !== 'function') {
      throw new Error(`Locale coverage policy requires ${name} to be a function.`)
    }
  }

  const dataLocaleDirectory = path.join(repositoryRoot, 'data', 'locales')
  const publicLocaleDirectory = path.join(frontendDirectory, 'public', 'locales')
  const sourceLocaleDirectory = path.join(frontendDirectory, 'src', 'locales')

  async function collectLocaleCoverageEvidence() {
    const issues = []
    const documents = new Map()

    async function readLocaleDocument(filePath, label) {
      if (documents.has(filePath)) return documents.get(filePath)
      let locale = null
      try {
        const content = await readTextFile(filePath, 'utf8')
        if (typeof content !== 'string') throw new Error('reader did not return UTF-8 text')
        locale = JSON.parse(content)
      } catch (error) {
        issues.push(`${label} could not be read as locale JSON: ${errorMessage(error)}`)
      }
      documents.set(filePath, locale)
      return locale
    }

    const baseLocale = await readLocaleDocument(
      path.join(dataLocaleDirectory, 'en.json'),
      'data/locales/en.json',
    )
    const baseMessages = localeMessages(baseLocale)
    const baseKeys = baseMessages ? Object.keys(baseMessages).sort() : []
    if (baseKeys.length === 0) {
      issues.push('data/locales/en.json must include a non-empty strings object')
    }

    for (const localeFile of requiredLocaleFiles) {
      const dataLabel = `data/locales/${localeFile}`
      const publicLabel = `frontend/public/locales/${localeFile}`
      const dataLocale = await readLocaleDocument(path.join(dataLocaleDirectory, localeFile), dataLabel)
      const publicLocale = await readLocaleDocument(path.join(publicLocaleDirectory, localeFile), publicLabel)
      verifyLocaleShape(dataLocale, dataLabel, baseKeys, issues)
      verifyLocaleShape(publicLocale, publicLabel, baseKeys, issues)
      if (dataLocale && publicLocale && !isDeepStrictEqual(dataLocale, publicLocale)) {
        issues.push(`${publicLabel} must match ${dataLabel}`)
      }
    }

    for (const localeFile of embeddedLocaleFiles) {
      const sourceLabel = `frontend/src/locales/${localeFile}`
      const dataLabel = `data/locales/${localeFile}`
      const sourceLocale = await readLocaleDocument(path.join(sourceLocaleDirectory, localeFile), sourceLabel)
      const dataLocale = await readLocaleDocument(path.join(dataLocaleDirectory, localeFile), dataLabel)
      verifyLocaleShape(sourceLocale, sourceLabel, baseKeys, issues)
      if (sourceLocale && dataLocale && !isDeepStrictEqual(sourceLocale, dataLocale)) {
        issues.push(`${sourceLabel} must match ${dataLabel}`)
      }
    }

    return {
      issues,
      baseKeyCount: baseKeys.length,
      publicLocaleCount: requiredLocaleFiles.length,
      embeddedLocaleCount: embeddedLocaleFiles.length,
    }
  }

  async function verifyLocaleCoverage() {
    const evidence = await collectLocaleCoverageEvidence()
    if (evidence.issues.length > 0) {
      throw new Error(`Locale coverage verification failed:\n${evidence.issues.join('\n')}`)
    }
    log(`[release] Locale coverage OK (${evidence.baseKeyCount} keys, ${evidence.publicLocaleCount} public locale(s))`)
    return evidence
  }

  return Object.freeze({
    collectLocaleCoverageEvidence,
    verifyLocaleCoverage,
  })
}

export async function collectLocaleCoverageEvidence(options = {}) {
  return createLocaleCoveragePolicy(options).collectLocaleCoverageEvidence()
}

export function localeMessages(locale) {
  if (!locale || typeof locale !== 'object' || Array.isArray(locale)) return null
  if (!locale.strings || typeof locale.strings !== 'object' || Array.isArray(locale.strings)) return null
  return locale.strings
}

function verifyLocaleShape(locale, label, baseKeys, issues) {
  if (!locale) return
  const expectedLocale = label.split('/').at(-1).replace(/\.json$/, '')
  if (locale.locale !== expectedLocale) {
    issues.push(`${label}: locale must be ${expectedLocale}`)
  }

  const messages = localeMessages(locale)
  if (!messages) {
    issues.push(`${label}: strings object is required`)
    return
  }

  const keys = Object.keys(messages).sort()
  const keySet = new Set(keys)
  const baseKeySet = new Set(baseKeys)
  const missing = baseKeys.filter((key) => !keySet.has(key))
  const extra = keys.filter((key) => !baseKeySet.has(key))
  if (missing.length > 0) {
    issues.push(`${label}: missing locale keys ${missing.slice(0, 10).join(', ')}`)
  }
  if (extra.length > 0) {
    issues.push(`${label}: unexpected locale keys ${extra.slice(0, 10).join(', ')}`)
  }
  for (const key of keys) {
    if (typeof messages[key] !== 'string') {
      issues.push(`${label}: locale key ${key} must be a string`)
    }
  }
}

function resolveBoundaries(options) {
  const repositoryRoot = options.repositoryRoot
  const frontendDirectory = options.frontendDirectory
  for (const [name, value] of Object.entries({ repositoryRoot, frontendDirectory })) {
    if (typeof value !== 'string' || value.length === 0) {
      throw new Error(`Locale coverage policy requires ${name}.`)
    }
  }

  const requiredLocaleFiles = validateLocaleFiles(options.requiredLocaleFiles, 'requiredLocaleFiles')
  if (!requiredLocaleFiles.includes('en.json')) {
    throw new Error('Locale coverage policy requiredLocaleFiles must include en.json.')
  }
  const embeddedLocaleFiles = validateLocaleFiles(
    options.embeddedLocaleFiles ?? defaultEmbeddedLocaleFiles,
    'embeddedLocaleFiles',
  )
  for (const localeFile of embeddedLocaleFiles) {
    if (!requiredLocaleFiles.includes(localeFile)) {
      throw new Error(`Locale coverage policy embedded locale must also be public: ${localeFile}`)
    }
  }
  return { repositoryRoot, frontendDirectory, requiredLocaleFiles, embeddedLocaleFiles }
}

function validateLocaleFiles(value, label) {
  if (!Array.isArray(value) || value.length === 0 || value.length > 64) {
    throw new Error(`Locale coverage policy ${label} must be a non-empty bounded array.`)
  }
  const files = [...value]
  if (new Set(files).size !== files.length || files.some((file) => typeof file !== 'string' || !localeFilePattern.test(file))) {
    throw new Error(`Locale coverage policy ${label} must contain unique portable JSON filenames.`)
  }
  return files
}

function errorMessage(error) {
  return error instanceof Error ? error.message : String(error)
}
