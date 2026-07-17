import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import {
  collectLocaleCoverageEvidence,
  createLocaleCoveragePolicy,
  localeMessages,
} from '../lib/locale-coverage-policy.mjs'
import { requiredLocaleFiles } from '../lib/web-distribution-verifier.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')
const frontendDirectory = path.join(repositoryRoot, 'frontend')
const boundaries = { repositoryRoot, frontendDirectory, requiredLocaleFiles }

test('checked-in locale catalogs return passing mirror and shape evidence', async () => {
  const messages = []
  const policy = createLocaleCoveragePolicy({
    ...boundaries,
    log(message) {
      messages.push(message)
    },
  })
  const evidence = await policy.verifyLocaleCoverage()

  assert.deepEqual(evidence, {
    issues: [],
    baseKeyCount: 1825,
    publicLocaleCount: 7,
    embeddedLocaleCount: 3,
  })
  assert.deepEqual(messages, ['[release] Locale coverage OK (1825 keys, 7 public locale(s))'])
})

test('locale evidence keeps shape, key, value, public, and embedded drift actionable', async () => {
  const localeFiles = ['en.json', 'fr-FR.json']
  const documents = new Map([
    [localePath('data', 'en.json'), locale('en', { alpha: 'A', beta: 'B' })],
    [localePath('public', 'en.json'), locale('en', { alpha: 'A', beta: 7 })],
    [localePath('data', 'fr-FR.json'), locale('fr-FR', { alpha: 'Un', gamma: 'Trois' })],
    [localePath('public', 'fr-FR.json'), locale('fr-FR', { alpha: 'Un', beta: 'Deux' })],
    [localePath('source', 'fr-FR.json'), locale('fr', { alpha: 'Un', beta: 'Deux' })],
  ])
  const evidence = await collectLocaleCoverageEvidence({
    repositoryRoot,
    frontendDirectory,
    requiredLocaleFiles: localeFiles,
    embeddedLocaleFiles: ['fr-FR.json'],
    async readTextFile(file) {
      const document = documents.get(file)
      if (!document) throw new Error(`unexpected fixture path: ${file}`)
      return document
    },
  })

  for (const issue of [
    'frontend/public/locales/en.json: locale key beta must be a string',
    'frontend/public/locales/en.json must match data/locales/en.json',
    'data/locales/fr-FR.json: missing locale keys beta',
    'data/locales/fr-FR.json: unexpected locale keys gamma',
    'frontend/public/locales/fr-FR.json must match data/locales/fr-FR.json',
    'frontend/src/locales/fr-FR.json: locale must be fr-FR',
    'frontend/src/locales/fr-FR.json must match data/locales/fr-FR.json',
  ]) {
    assert(evidence.issues.includes(issue), issue)
  }
  assert.equal(evidence.baseKeyCount, 2)
})

test('locale parse and read failures remain structured without duplicate data errors', async () => {
  const evidence = await collectLocaleCoverageEvidence({
    repositoryRoot,
    frontendDirectory,
    requiredLocaleFiles: ['en.json', 'fr-FR.json'],
    embeddedLocaleFiles: ['fr-FR.json'],
    async readTextFile(file) {
      if (file === localePath('public', 'fr-FR.json')) throw new Error('fixture public read denied')
      if (file === localePath('source', 'fr-FR.json')) return '{"broken":}'
      if (file.endsWith('en.json')) return locale('en', { alpha: 'A' })
      return locale('fr-FR', { alpha: 'Un' })
    },
  })

  assert.equal(evidence.issues.length, 2)
  assert.equal(
    evidence.issues[0],
    'frontend/public/locales/fr-FR.json could not be read as locale JSON: fixture public read denied',
  )
  assert(evidence.issues[1].startsWith(
    'frontend/src/locales/fr-FR.json could not be read as locale JSON:',
  ))
})

test('locale helpers and policy reject invalid document and boundary shapes', async () => {
  assert.equal(localeMessages(null), null)
  assert.equal(localeMessages({ strings: [] }), null)
  assert.deepEqual(localeMessages({ strings: { key: 'value' } }), { key: 'value' })
  assert.throws(() => createLocaleCoveragePolicy(), /requires repositoryRoot/)
  assert.throws(
    () => createLocaleCoveragePolicy({ repositoryRoot, frontendDirectory }),
    /requiredLocaleFiles must be a non-empty bounded array/,
  )
  assert.throws(
    () => createLocaleCoveragePolicy({ ...boundaries, requiredLocaleFiles: ['en.json', '../fr-FR.json'] }),
    /unique portable JSON filenames/,
  )
  assert.throws(
    () => createLocaleCoveragePolicy({ ...boundaries, embeddedLocaleFiles: ['fr-FR.json'] }),
    /embedded locale must also be public/,
  )
  assert.throws(
    () => createLocaleCoveragePolicy({ ...boundaries, readTextFile: 42 }),
    /requires readTextFile to be a function/,
  )

  const evidence = await collectLocaleCoverageEvidence({
    repositoryRoot,
    frontendDirectory,
    requiredLocaleFiles: ['en.json'],
    embeddedLocaleFiles: ['en.json'],
    async readTextFile() {
      return locale('en', {})
    },
  })
  assert(evidence.issues.includes('data/locales/en.json must include a non-empty strings object'))
})

test('release runner delegates locale coverage without retaining catalog rules', async () => {
  const runnerSource = await readFile(path.join(repositoryRoot, 'scripts', 'verify-release.mjs'), 'utf8')
  const policySource = await readFile(
    path.join(repositoryRoot, 'scripts', 'lib', 'locale-coverage-policy.mjs'),
    'utf8',
  )

  assert(runnerSource.includes('createLocaleCoveragePolicy'))
  assert(runnerSource.includes('verifyLocaleCoverage'))
  assert(runnerSource.includes('localeMessages'))
  assert(!runnerSource.includes('async function verifyLocaleCoverage'))
  assert(!runnerSource.includes('function verifyLocaleShape'))
  assert(!runnerSource.includes('function stableStringify'))
  assert(policySource.includes('async function collectLocaleCoverageEvidence'))
  assert(policySource.includes('function verifyLocaleShape'))
  assert(policySource.includes('isDeepStrictEqual'))
})

function localePath(root, localeFile) {
  if (root === 'data') return path.join(repositoryRoot, 'data', 'locales', localeFile)
  if (root === 'public') return path.join(frontendDirectory, 'public', 'locales', localeFile)
  return path.join(frontendDirectory, 'src', 'locales', localeFile)
}

function locale(id, strings) {
  return JSON.stringify({ locale: id, strings })
}
