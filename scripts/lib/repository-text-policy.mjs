import { readFile as readFileFromDisk, stat as statFromDisk } from 'node:fs/promises'
import path from 'node:path'

import { createRepositoryFileWalker } from './repository-file-walker.mjs'

const MAX_SCANNED_TEXT_BYTES = 4 * 1024 * 1024

const repositoryTextExtensionList = Object.freeze([
  '.cs',
  '.css',
  '.html',
  '.js',
  '.json',
  '.md',
  '.mjs',
  '.ps1',
  '.rs',
  '.toml',
  '.ts',
  '.vue',
  '.xml',
  '.yml',
  '.yaml',
])

const frontendSourceExtensionList = Object.freeze(['.css', '.html', '.js', '.mjs', '.ts', '.vue'])

const sensitivePatterns = Object.freeze([
  { label: 'OpenAI-style API key', pattern: /sk-[A-Za-z0-9_-]{20,}/ },
  { label: 'GitHub classic token', pattern: /ghp_[A-Za-z0-9]{20,}/ },
  { label: 'GitHub fine-grained token', pattern: /github_pat_[A-Za-z0-9_]{20,}/ },
])

const uiTextArtifactPatterns = Object.freeze([
  { label: 'replacement character', pattern: /\uFFFD/ },
  { label: 'mojibake separator', pattern: /\u74BA\?/ },
  { label: 'mojibake CJK fragment', pattern: /[\u9354\u9288\u979D\u9802]/ },
  { label: 'stray Chinese road separator', pattern: /\s\u8DEF\s/ },
])

export function createRepositoryTextPolicy(options = {}) {
  const { repositoryRoot, frontendDirectory } = resolveBoundaries(options)
  const readTextFile = options.readTextFile ?? readFileFromDisk
  const statFile = options.statFile ?? statFromDisk
  const log = options.log ?? console.log
  const relative = options.relativePath
    ?? ((file) => path.relative(repositoryRoot, file).replaceAll('\\', '/'))
  const walkFiles = options.walkFiles ?? createRepositoryFileWalker({
    readDirectory: options.readDirectory,
    excludedDirectoryNames: options.excludedDirectoryNames,
  })
  for (const [name, value] of Object.entries({ readTextFile, statFile, log, relative, walkFiles })) {
    if (typeof value !== 'function') {
      throw new Error(`Repository text policy requires ${name} to be a function.`)
    }
  }

  const repositoryTextExtensions = new Set(repositoryTextExtensionList)
  const frontendSourceExtensions = new Set(frontendSourceExtensionList)

  async function collectSensitivePatternEvidence() {
    const issues = []
    const hits = []
    let scannedFileCount = 0
    let skippedLargeFileCount = 0
    let files = []

    try {
      files = (await walkFiles(repositoryRoot))
        .filter((file) => repositoryTextExtensions.has(path.extname(file)))
        .sort()
    } catch (error) {
      issues.push(`Sensitive pattern discovery failed: ${errorMessage(error)}`)
    }

    for (const file of files) {
      let info
      try {
        info = await statFile(file)
      } catch (error) {
        issues.push(`${relative(file)} could not be inspected: ${errorMessage(error)}`)
        continue
      }
      if (!Number.isSafeInteger(info?.size) || info.size < 0) {
        issues.push(`${relative(file)} returned invalid file size metadata`)
        continue
      }
      if (info.size > MAX_SCANNED_TEXT_BYTES) {
        skippedLargeFileCount += 1
        continue
      }

      let content
      try {
        content = await readTextFile(file, 'utf8')
      } catch (error) {
        issues.push(`${relative(file)} could not be read: ${errorMessage(error)}`)
        continue
      }
      if (typeof content !== 'string') {
        issues.push(`${relative(file)} did not return UTF-8 text`)
        continue
      }

      scannedFileCount += 1
      for (const rule of sensitivePatterns) {
        if (rule.pattern.test(content)) {
          hits.push({ path: relative(file), label: rule.label })
        }
      }
    }

    return {
      issues,
      hits,
      scannedFileCount,
      skippedLargeFileCount,
    }
  }

  async function verifySensitivePatterns() {
    const evidence = await collectSensitivePatternEvidence()
    const failures = [
      ...evidence.issues,
      ...evidence.hits.map((hit) => `${hit.path} (${hit.label})`),
    ]
    if (failures.length > 0) {
      throw new Error(`Sensitive token pattern scan failed:\n${failures.join('\n')}`)
    }
    log('[release] Sensitive token pattern scan OK')
    return evidence
  }

  async function collectUiTextArtifactEvidence() {
    const sourceDirectory = path.join(frontendDirectory, 'src')
    const issues = []
    const hits = []
    let scannedFileCount = 0
    let files = []

    try {
      files = (await walkFiles(sourceDirectory))
        .filter((file) => {
          const sourceRelative = path.relative(sourceDirectory, file).replaceAll('\\', '/')
          return sourceRelative !== 'locales'
            && !sourceRelative.startsWith('locales/')
            && frontendSourceExtensions.has(path.extname(file))
        })
        .sort()
    } catch (error) {
      issues.push(`UI text artifact discovery failed: ${errorMessage(error)}`)
    }

    for (const file of files) {
      let content
      try {
        content = await readTextFile(file, 'utf8')
      } catch (error) {
        issues.push(`${relative(file)} could not be read: ${errorMessage(error)}`)
        continue
      }
      if (typeof content !== 'string') {
        issues.push(`${relative(file)} did not return UTF-8 text`)
        continue
      }

      scannedFileCount += 1
      for (const rule of uiTextArtifactPatterns) {
        if (rule.pattern.test(content)) {
          hits.push({ path: relative(file), label: rule.label })
        }
      }
    }

    return {
      issues,
      hits,
      scannedFileCount,
    }
  }

  async function verifyUiTextArtifacts() {
    const evidence = await collectUiTextArtifactEvidence()
    const failures = [
      ...evidence.issues,
      ...evidence.hits.map((hit) => `${hit.path} (${hit.label})`),
    ]
    if (failures.length > 0) {
      throw new Error(`UI text artifact scan failed:\n${failures.join('\n')}`)
    }
    log('[release] UI text artifact scan OK')
    return evidence
  }

  return Object.freeze({
    collectSensitivePatternEvidence,
    collectUiTextArtifactEvidence,
    frontendSourceExtensions: new Set(frontendSourceExtensions),
    verifySensitivePatterns,
    verifyUiTextArtifacts,
  })
}

export async function collectRepositorySensitivePatternEvidence(options = {}) {
  return createRepositoryTextPolicy(options).collectSensitivePatternEvidence()
}

export async function collectFrontendTextArtifactEvidence(options = {}) {
  return createRepositoryTextPolicy(options).collectUiTextArtifactEvidence()
}

function resolveBoundaries(options) {
  const repositoryRoot = options.repositoryRoot
  const frontendDirectory = options.frontendDirectory
  for (const [name, value] of Object.entries({ repositoryRoot, frontendDirectory })) {
    if (typeof value !== 'string' || value.length === 0) {
      throw new Error(`Repository text policy requires ${name}.`)
    }
  }
  const relativeFrontend = path.relative(path.resolve(repositoryRoot), path.resolve(frontendDirectory))
  if (relativeFrontend === '..' || relativeFrontend.startsWith(`..${path.sep}`) || path.isAbsolute(relativeFrontend)) {
    throw new Error('Repository text policy frontendDirectory must stay inside repositoryRoot.')
  }
  return { repositoryRoot, frontendDirectory }
}

function errorMessage(error) {
  return error instanceof Error ? error.message : String(error)
}
