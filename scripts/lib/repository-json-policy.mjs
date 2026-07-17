import { readFile as readFileFromDisk } from 'node:fs/promises'
import path from 'node:path'

import { createRepositoryFileWalker } from './repository-file-walker.mjs'

export function createRepositoryJsonPolicy(options = {}) {
  const repositoryRoot = options.repositoryRoot
  if (typeof repositoryRoot !== 'string' || repositoryRoot.length === 0) {
    throw new Error('Repository JSON policy requires repositoryRoot.')
  }

  const readTextFile = options.readTextFile ?? readFileFromDisk
  const log = options.log ?? console.log
  const relative = options.relativePath
    ?? ((file) => path.relative(repositoryRoot, file).replaceAll('\\', '/'))
  const walkFiles = options.walkFiles ?? createRepositoryFileWalker({
    readDirectory: options.readDirectory,
    excludedDirectoryNames: options.excludedDirectoryNames,
  })
  for (const [name, value] of Object.entries({ readTextFile, log, relative, walkFiles })) {
    if (typeof value !== 'function') {
      throw new Error('Repository JSON policy requires ' + name + ' to be a function.')
    }
  }

  async function collectRepositoryJsonEvidence() {
    const issues = []
    let files = []

    try {
      files = (await walkFiles(repositoryRoot))
        .filter((file) => path.extname(file) === '.json')
        .sort()
    } catch (error) {
      issues.push('Repository JSON discovery failed: ' + error.message)
    }

    for (const file of files) {
      try {
        JSON.parse(await readTextFile(file, 'utf8'))
      } catch (error) {
        issues.push(relative(file) + ': ' + error.message)
      }
    }

    return {
      issues,
      jsonFileCount: files.length,
    }
  }

  async function verifyRepositoryJsonFiles() {
    const evidence = await collectRepositoryJsonEvidence()
    if (evidence.issues.length > 0) {
      throw new Error('Invalid JSON files:\n' + evidence.issues.join('\n'))
    }
    log('[release] JSON parse OK (' + evidence.jsonFileCount + ' files)')
    return evidence
  }

  return Object.freeze({
    collectRepositoryJsonEvidence,
    verifyRepositoryJsonFiles,
  })
}

export async function collectRepositoryJsonEvidence(options = {}) {
  return createRepositoryJsonPolicy(options).collectRepositoryJsonEvidence()
}
