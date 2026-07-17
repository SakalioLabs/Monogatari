import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import {
  collectProjectStoryEventEvidence,
  createProjectStoryEventPolicy,
} from '../lib/project-content/story-event-policy.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')
const rustDirectory = path.join(repositoryRoot, 'rust-engine')
const boundaries = { repositoryRoot, rustDirectory }

test('checked-in Story Event and Ending catalogs return cross-root passing evidence', async () => {
  const evidence = await collectProjectStoryEventEvidence(boundaries)
  assert.deepEqual(evidence.issues, [])
  assert.equal(evidence.fileCount, 4)
  assert.equal(evidence.eventCount, 10)
  assert.equal(evidence.endingCount, 7)
  assert.equal(
    evidence.catalogFingerprint,
    'dd87c91c64c6affaaa139c5aac073f9b0a47aab9f135bb95760c88f0099a0f0f',
  )
})

test('catalog, Ending, mirror, and Rust fingerprint drift stays independently actionable', async () => {
  const rootCatalogPath = path.join(repositoryRoot, 'data', 'events', 'story_events.json')
  const rootEndingPath = path.join(repositoryRoot, 'data', 'endings', 'best_friend_ending.json')
  const rustStoryEventPath = path.join(
    rustDirectory,
    'crates',
    'authoring',
    'src',
    'story_events.rs',
  )
  const evidence = await collectProjectStoryEventEvidence({
    ...boundaries,
    async readTextFile(filePath, encoding) {
      const source = await readFile(filePath, encoding)
      const resolved = path.resolve(filePath)
      if (resolved === rootCatalogPath) {
        const catalog = JSON.parse(source)
        catalog.events = catalog.events.filter((event) => event.event_id !== 'first_friend')
        return JSON.stringify(catalog)
      }
      if (resolved === rootEndingPath) {
        return source.replace('monogatari-story-ending/v1', 'story-ending-drifted')
      }
      if (resolved === rustStoryEventPath) {
        return source.replace(
          'checked_in_catalog_preserves_cross_runtime_catalog_fingerprint',
          'catalog_fingerprint_drifted',
        )
      }
      return source
    },
  })

  for (const issue of [
    'data: missing required story event first_friend',
    'data/endings/best_friend_ending.json: schema must be monogatari-story-ending/v1',
    'data and rust-engine/data story event catalogs must match',
    'data and rust-engine/data normalized story event fingerprints must match',
    'Rust story event catalog must pin the cross-runtime catalog fingerprint',
  ]) {
    assert(evidence.issues.includes(issue), issue)
  }
})

test('Project Story Event policy requires explicit roots and unique data-root labels', () => {
  assert.throws(() => createProjectStoryEventPolicy(), /requires repositoryRoot/)
  assert.throws(
    () => createProjectStoryEventPolicy({ repositoryRoot }),
    /requires rustDirectory/,
  )
  assert.throws(
    () => createProjectStoryEventPolicy({
      ...boundaries,
      dataRoots: [
        { label: 'project', dir: path.join(repositoryRoot, 'data') },
        { label: 'project', dir: path.join(rustDirectory, 'data') },
      ],
    }),
    /label is duplicated/,
  )
})

test('release runner delegates Story Event and Ending policy without retaining its rules', async () => {
  const runnerSource = await readFile(path.join(repositoryRoot, 'scripts', 'verify-release.mjs'), 'utf8')
  const policySource = await readFile(
    path.join(repositoryRoot, 'scripts', 'lib', 'project-content', 'story-event-policy.mjs'),
    'utf8',
  )

  assert(runnerSource.includes('createProjectStoryEventPolicy'))
  assert(runnerSource.includes('verifyStoryEventCatalogs'))
  assert(!runnerSource.includes('async function verifyStoryEventCatalogs'))
  assert(!runnerSource.includes('function storyEventCatalogFingerprint'))
  assert(!runnerSource.includes('monogatari-story-event-catalog-fingerprint/v1'))
  assert(policySource.includes('async function collectStoryEventEvidence'))
  assert(policySource.includes('async function loadStoryEventCatalog'))
  assert(policySource.includes('function storyEventCatalogFingerprint'))
  assert(policySource.includes('monogatari-story-event-catalog-fingerprint/v1'))
})
