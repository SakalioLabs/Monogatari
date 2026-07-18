import assert from 'node:assert/strict'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import {
  createRepositoryFileWalker,
  defaultRepositoryScanExcludedDirectories,
} from '../lib/repository-file-walker.mjs'
import {
  collectRepositoryJsonEvidence,
  createRepositoryJsonPolicy,
} from '../lib/repository-json-policy.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')

test('checked-in repository JSON returns passing evidence', async () => {
  const messages = []
  const policy = createRepositoryJsonPolicy({
    repositoryRoot,
    log(message) {
      messages.push(message)
    },
  })
  const evidence = await policy.verifyRepositoryJsonFiles()

  assert.deepEqual(evidence.issues, [])
  assert.equal(evidence.jsonFileCount, 270)
  assert.deepEqual(messages, ['[release] JSON parse OK (270 files)'])
})

test('Repository JSON evidence isolates parse and read failures', async () => {
  const files = [
    path.join(repositoryRoot, 'good.json'),
    path.join(repositoryRoot, 'bad.json'),
    path.join(repositoryRoot, 'read.json'),
    path.join(repositoryRoot, 'ignored.JSON'),
    path.join(repositoryRoot, 'notes.txt'),
  ]
  const evidence = await collectRepositoryJsonEvidence({
    repositoryRoot,
    async walkFiles() {
      return files
    },
    async readTextFile(file) {
      if (file.endsWith('bad.json')) return '{\"broken\":}'
      if (file.endsWith('read.json')) throw new Error('fixture read denied')
      return '{\"valid\":true}'
    },
  })

  assert.equal(evidence.jsonFileCount, 3)
  assert.equal(evidence.issues.length, 2)
  assert(evidence.issues.some((issue) => issue.startsWith('bad.json: ')))
  assert(evidence.issues.includes('read.json: fixture read denied'))
})

test('Repository JSON discovery failures remain structured evidence', async () => {
  const evidence = await collectRepositoryJsonEvidence({
    repositoryRoot,
    async walkFiles() {
      throw new Error('fixture traversal denied')
    },
  })

  assert.deepEqual(evidence, {
    issues: ['Repository JSON discovery failed: fixture traversal denied'],
    jsonFileCount: 0,
  })
})

test('repository file walker is deterministic and excludes generated directories', async () => {
  const fixtureRoot = path.resolve(repositoryRoot, '..', 'repository-walker-fixture')
  const dataDir = path.join(fixtureRoot, 'data')
  const nestedDir = path.join(dataDir, 'nested')
  const visited = []
  const tree = new Map([
    [fixtureRoot, [
      entry('z.json', 'file'),
      entry('node_modules', 'directory'),
      entry('data', 'directory'),
      entry('ignored-link', 'other'),
    ]],
    [dataDir, [
      entry('b.json', 'file'),
      entry('nested', 'directory'),
    ]],
    [nestedDir, [
      entry('a.json', 'file'),
    ]],
  ])
  const walkFiles = createRepositoryFileWalker({
    async readDirectory(directory) {
      visited.push(directory)
      const entries = tree.get(directory)
      if (!entries) throw new Error('unexpected directory: ' + directory)
      return [...entries]
    },
  })

  assert(defaultRepositoryScanExcludedDirectories.includes('node_modules'))
  assert.deepEqual(await walkFiles(fixtureRoot), [
    path.join(dataDir, 'b.json'),
    path.join(nestedDir, 'a.json'),
    path.join(fixtureRoot, 'z.json'),
  ])
  assert(!visited.includes(path.join(fixtureRoot, 'node_modules')))
})

test('Repository JSON policy and walker require explicit callable boundaries', () => {
  assert.throws(() => createRepositoryJsonPolicy(), /requires repositoryRoot/)
  assert.throws(
    () => createRepositoryJsonPolicy({ repositoryRoot, readTextFile: 42 }),
    /requires readTextFile to be a function/,
  )
  assert.throws(
    () => createRepositoryJsonPolicy({ repositoryRoot, walkFiles: 42 }),
    /requires walkFiles to be a function/,
  )
  assert.throws(
    () => createRepositoryFileWalker({ readDirectory: 42 }),
    /requires readDirectory to be a function/,
  )
  assert.throws(
    () => createRepositoryFileWalker({ excludedDirectoryNames: ['target', ''] }),
    /excludedDirectoryNames must be an array of names/,
  )
})

test('release runner delegates JSON parsing and shares the repository walker', async () => {
  const { readFile } = await import('node:fs/promises')
  const runnerSource = await readFile(
    path.join(repositoryRoot, 'scripts', 'verify-release.mjs'),
    'utf8',
  )
  const policySource = await readFile(
    path.join(repositoryRoot, 'scripts', 'lib', 'repository-json-policy.mjs'),
    'utf8',
  )

  assert(runnerSource.includes('createRepositoryFileWalker'))
  assert(runnerSource.includes('createRepositoryJsonPolicy'))
  assert(runnerSource.includes('verifyRepositoryJsonFiles'))
  assert(runnerSource.includes('walkFiles,'))
  assert(!runnerSource.includes('async function walkFiles'))
  assert(!runnerSource.includes('async function verifyJsonFiles'))
  assert(!runnerSource.includes("path.extname(file) === '.json'"))
  assert(!runnerSource.includes('Invalid JSON files'))
  assert(policySource.includes('async function collectRepositoryJsonEvidence'))
  assert(policySource.includes("path.extname(file) === '.json'"))
  assert(policySource.includes('Invalid JSON files'))
})

function entry(name, type) {
  return {
    name,
    isDirectory() {
      return type === 'directory'
    },
    isFile() {
      return type === 'file'
    },
  }
}
