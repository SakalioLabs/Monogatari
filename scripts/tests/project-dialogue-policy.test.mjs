import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import {
  collectProjectDialogueEvidence,
  createProjectDialoguePolicy,
} from '../lib/project-content/dialogue-policy.mjs'
import { isPortableProjectContentId } from '../lib/project-content/portable-id.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')
const rustDirectory = path.join(repositoryRoot, 'rust-engine')
const boundaries = { repositoryRoot, rustDirectory }

test('checked-in Dialogue catalogs return cross-root passing evidence', async () => {
  const evidence = await collectProjectDialogueEvidence(boundaries)
  assert.deepEqual(evidence.issues, [])
  assert.deepEqual(
    {
      fileCount: evidence.fileCount,
      nodeCount: evidence.nodeCount,
      choiceCount: evidence.choiceCount,
    },
    { fileCount: 44, nodeCount: 654, choiceCount: 582 },
  )
})

test('document, graph, character, relationship, and mirror drift stays independently actionable', async () => {
  const dialoguePath = path.join(repositoryRoot, 'data', 'dialogue', 'example_dialogue.json')
  const evidence = await collectProjectDialogueEvidence({
    ...boundaries,
    async readTextFile(filePath, encoding) {
      const source = await readFile(filePath, encoding)
      if (path.resolve(filePath) !== dialoguePath) return source

      const dialogue = JSON.parse(source)
      dialogue.unexpected_field = true
      dialogue.start_node_id = 'missing_node'
      dialogue.nodes.start.speaker_id = 'missing_character'
      dialogue.nodes.start.choices = [{
        text: 'Broken branch',
        next_node_id: 'missing_node',
        relationship_changes: { missing_character: 2 },
      }]
      return JSON.stringify(dialogue)
    },
  })

  for (const issue of [
    'data/dialogue/example_dialogue.json: unknown dialogue fields unexpected_field',
    'data/dialogue/example_dialogue.json: start_node_id must identify an existing node',
    'data/dialogue/example_dialogue.json:start: unknown speaker missing_character',
    'data/dialogue/example_dialogue.json:start:choice-1: target must identify an existing node',
    'data/dialogue/example_dialogue.json:start:choice-1: unknown relationship character missing_character',
    'data/dialogue/example_dialogue.json:start:choice-1: relationship delta for missing_character must be between -1 and 1',
    'data and rust-engine/data dialogue catalogs must match',
  ]) {
    assert(evidence.issues.includes(issue), issue)
  }
})

test('portable project content ids reject traversal, whitespace, and overlong values', () => {
  assert.equal(isPortableProjectContentId('dialogue.chapter_01', 128), true)
  assert.equal(isPortableProjectContentId('../dialogue', 128), false)
  assert.equal(isPortableProjectContentId(' dialogue', 128), false)
  assert.equal(isPortableProjectContentId('x'.repeat(129), 128), false)
})

test('Project Dialogue policy requires explicit roots and unique data-root labels', () => {
  assert.throws(() => createProjectDialoguePolicy(), /requires repositoryRoot/)
  assert.throws(
    () => createProjectDialoguePolicy({ repositoryRoot }),
    /requires rustDirectory/,
  )
  assert.throws(
    () => createProjectDialoguePolicy({
      ...boundaries,
      dataRoots: [
        { label: 'project', dir: path.join(repositoryRoot, 'data') },
        { label: 'project', dir: path.join(rustDirectory, 'data') },
      ],
    }),
    /label is duplicated/,
  )
})

test('release runner delegates Dialogue policy without retaining graph rules', async () => {
  const runnerSource = await readFile(path.join(repositoryRoot, 'scripts', 'verify-release.mjs'), 'utf8')
  const policySource = await readFile(
    path.join(repositoryRoot, 'scripts', 'lib', 'project-content', 'dialogue-policy.mjs'),
    'utf8',
  )
  const storyPolicySource = await readFile(
    path.join(repositoryRoot, 'scripts', 'lib', 'project-content', 'story-event-policy.mjs'),
    'utf8',
  )

  assert(runnerSource.includes('createProjectDialoguePolicy'))
  assert(runnerSource.includes('verifyDialogueCatalogs'))
  assert(!runnerSource.includes('async function verifyDialogueCatalogs'))
  assert(!runnerSource.includes('unknown dialogue fields'))
  assert(!runnerSource.includes('dialogueControlPattern'))
  assert(policySource.includes('async function collectDialogueEvidence'))
  assert(policySource.includes('unknown dialogue fields'))
  assert(policySource.includes('dialogueControlPattern'))
  assert(storyPolicySource.includes("from './portable-id.mjs'"))
  assert(!storyPolicySource.includes('export function isPortableProjectContentId'))
})
