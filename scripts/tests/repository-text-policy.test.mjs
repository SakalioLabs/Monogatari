import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import {
  collectFrontendTextArtifactEvidence,
  collectRepositorySensitivePatternEvidence,
  createRepositoryTextPolicy,
} from '../lib/repository-text-policy.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')
const frontendDirectory = path.join(repositoryRoot, 'frontend')
const boundaries = { repositoryRoot, frontendDirectory }

test('checked-in repository text returns passing security and UI evidence', async () => {
  const messages = []
  const policy = createRepositoryTextPolicy({
    ...boundaries,
    log(message) {
      messages.push(message)
    },
  })

  const sensitive = await policy.verifySensitivePatterns()
  const ui = await policy.verifyUiTextArtifacts()

  assert.deepEqual(sensitive.issues, [])
  assert.deepEqual(sensitive.hits, [])
  assert(sensitive.scannedFileCount > 100)
  assert.deepEqual(ui.issues, [])
  assert.deepEqual(ui.hits, [])
  assert(ui.scannedFileCount > 100)
  assert.deepEqual(messages, [
    '[release] Sensitive token pattern scan OK',
    '[release] UI text artifact scan OK',
  ])
})

test('sensitive evidence isolates findings, read failures, metadata failures, and large files', async () => {
  const files = [
    path.join(repositoryRoot, 'notes.md'),
    path.join(repositoryRoot, 'src', 'broken.yml'),
    path.join(repositoryRoot, 'src', 'huge.rs'),
    path.join(repositoryRoot, 'src', 'secret.ts'),
    path.join(repositoryRoot, 'src', 'ignored.png'),
    path.join(repositoryRoot, 'src', 'metadata.cs'),
  ]
  const secret = ['github', 'pat', 'A'.repeat(24)].join('_')
  const evidence = await collectRepositorySensitivePatternEvidence({
    ...boundaries,
    async walkFiles() {
      return [...files].reverse()
    },
    async statFile(file) {
      if (file.endsWith('metadata.cs')) throw new Error('fixture stat denied')
      return { size: file.endsWith('huge.rs') ? 5 * 1024 * 1024 : 100 }
    },
    async readTextFile(file) {
      if (file.endsWith('broken.yml')) throw new Error('fixture read denied')
      if (file.endsWith('secret.ts')) return `export const token = '${secret}'`
      return 'safe fixture text'
    },
  })

  assert.deepEqual(evidence.hits, [{
    path: 'src/secret.ts',
    label: 'GitHub fine-grained token',
  }])
  assert.deepEqual(evidence.issues, [
    'src/broken.yml could not be read: fixture read denied',
    'src/metadata.cs could not be inspected: fixture stat denied',
  ])
  assert.equal(evidence.scannedFileCount, 2)
  assert.equal(evidence.skippedLargeFileCount, 1)
})

test('UI evidence excludes locale catalogs and reports artifact and read drift', async () => {
  const sourceDirectory = path.join(frontendDirectory, 'src')
  const replacementCharacter = String.fromCodePoint(0xfffd)
  const files = [
    path.join(sourceDirectory, 'App.vue'),
    path.join(sourceDirectory, 'Broken.css'),
    path.join(sourceDirectory, 'icon.svg'),
    path.join(sourceDirectory, 'locales', 'en.json'),
  ]
  const evidence = await collectFrontendTextArtifactEvidence({
    ...boundaries,
    async walkFiles(root) {
      assert.equal(root, sourceDirectory)
      return files
    },
    async readTextFile(file) {
      if (file.endsWith('Broken.css')) throw new Error('fixture UI read denied')
      return replacementCharacter
    },
  })

  assert.deepEqual(evidence.hits, [{
    path: 'frontend/src/App.vue',
    label: 'replacement character',
  }])
  assert.deepEqual(evidence.issues, [
    'frontend/src/Broken.css could not be read: fixture UI read denied',
  ])
  assert.equal(evidence.scannedFileCount, 1)
})

test('repository text discovery failures remain structured evidence', async () => {
  const evidence = await collectRepositorySensitivePatternEvidence({
    ...boundaries,
    async walkFiles() {
      throw new Error('fixture traversal denied')
    },
  })

  assert.deepEqual(evidence, {
    issues: ['Sensitive pattern discovery failed: fixture traversal denied'],
    hits: [],
    scannedFileCount: 0,
    skippedLargeFileCount: 0,
  })
})

test('repository text policy requires explicit safe callable boundaries', () => {
  assert.throws(() => createRepositoryTextPolicy(), /requires repositoryRoot/)
  assert.throws(
    () => createRepositoryTextPolicy({ repositoryRoot, frontendDirectory: path.resolve(repositoryRoot, '..') }),
    /must stay inside repositoryRoot/,
  )
  assert.throws(
    () => createRepositoryTextPolicy({ ...boundaries, readTextFile: 42 }),
    /requires readTextFile to be a function/,
  )
  assert.throws(
    () => createRepositoryTextPolicy({ ...boundaries, statFile: 42 }),
    /requires statFile to be a function/,
  )
})

test('release runner delegates repository text rules to the importable policy', async () => {
  const runnerSource = await readFile(path.join(repositoryRoot, 'scripts', 'verify-release.mjs'), 'utf8')
  const policySource = await readFile(
    path.join(repositoryRoot, 'scripts', 'lib', 'repository-text-policy.mjs'),
    'utf8',
  )

  assert(runnerSource.includes('createRepositoryTextPolicy'))
  assert(runnerSource.includes('verifySensitivePatterns'))
  assert(runnerSource.includes('verifyUiTextArtifacts'))
  assert(!runnerSource.includes('async function verifySensitivePatterns'))
  assert(!runnerSource.includes('async function verifyUiTextArtifacts'))
  assert(!runnerSource.includes('sensitivePatterns'))
  assert(!runnerSource.includes('uiTextArtifactPatterns'))
  assert(policySource.includes('async function collectSensitivePatternEvidence'))
  assert(policySource.includes('async function collectUiTextArtifactEvidence'))
  assert(policySource.includes('MAX_SCANNED_TEXT_BYTES'))
})
