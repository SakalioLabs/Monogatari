import { readdir as readDirectoryFromDisk } from 'node:fs/promises'
import path from 'node:path'

export const defaultRepositoryScanExcludedDirectories = Object.freeze([
  '.git',
  'node_modules',
  'target',
  'dist',
  'release',
  'bin',
  'obj',
])

export function createRepositoryFileWalker(options = {}) {
  const readDirectory = options.readDirectory ?? readDirectoryFromDisk
  if (typeof readDirectory !== 'function') {
    throw new Error('Repository file walker requires readDirectory to be a function.')
  }

  const excludedDirectoryNames = options.excludedDirectoryNames
    ?? defaultRepositoryScanExcludedDirectories
  if (
    !Array.isArray(excludedDirectoryNames)
    || excludedDirectoryNames.some((name) => typeof name !== 'string' || name.length === 0)
  ) {
    throw new Error('Repository file walker excludedDirectoryNames must be an array of names.')
  }
  const excluded = new Set(excludedDirectoryNames)

  return async function walkFiles(directory) {
    if (typeof directory !== 'string' || directory.length === 0) {
      throw new Error('Repository file walker requires a directory.')
    }

    const files = []
    await visit(directory, files)
    return files
  }

  async function visit(directory, files) {
    const entries = await readDirectory(directory, { withFileTypes: true })
    entries.sort(compareEntries)
    for (const entry of entries) {
      const entryPath = path.join(directory, entry.name)
      if (entry.isDirectory()) {
        if (!excluded.has(entry.name)) await visit(entryPath, files)
      } else if (entry.isFile()) {
        files.push(entryPath)
      }
    }
  }
}

function compareEntries(left, right) {
  if (left.name < right.name) return -1
  if (left.name > right.name) return 1
  return 0
}
