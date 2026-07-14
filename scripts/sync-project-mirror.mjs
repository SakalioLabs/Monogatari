#!/usr/bin/env node

import path from 'node:path'
import { fileURLToPath } from 'node:url'

import { projectMirrorDiff, synchronizeProjectMirror } from './lib/project-mirror.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const sourceRoot = path.join(repositoryRoot, 'data')
const mirrorRoot = path.join(repositoryRoot, 'rust-engine', 'data')
const write = process.argv.slice(2).includes('--write')
const unknown = process.argv.slice(2).filter((argument) => argument !== '--check' && argument !== '--write')
if (unknown.length > 0 || (process.argv.includes('--check') && write)) {
  console.error('Usage: node scripts/sync-project-mirror.mjs [--check|--write]')
  process.exit(2)
}

const result = write
  ? await synchronizeProjectMirror(sourceRoot, mirrorRoot)
  : await projectMirrorDiff(sourceRoot, mirrorRoot)

if (!result.valid) {
  console.error('[project-mirror] Built-in project roots differ.')
  for (const [label, files] of [['missing', result.missing], ['changed', result.changed], ['extra', result.extra]]) {
    if (files.length > 0) console.error(`[project-mirror] ${label}: ${files.join(', ')}`)
  }
  process.exit(1)
}

console.log(`[project-mirror] OK: ${result.sourceCount} files are byte-equivalent${write ? ' after synchronization' : ''}`)
