import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import {
  collectProjectKnowledgeReferenceEvidence,
  createProjectKnowledgeReferencePolicy,
} from '../lib/project-content/knowledge-reference-policy.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')
const rustDirectory = path.join(repositoryRoot, 'rust-engine')
const boundaries = { repositoryRoot, rustDirectory }

test('checked-in Knowledge references return cross-root passing evidence', async () => {
  const messages = []
  const policy = createProjectKnowledgeReferencePolicy({
    ...boundaries,
    log(message) {
      messages.push(message)
    },
  })
  const evidence = await policy.verifyKnowledgeReferences()

  assert.deepEqual(evidence.issues, [])
  assert.deepEqual(
    {
      pinnedRefCount: evidence.pinnedRefCount,
      knowledgeCount: evidence.knowledgeCount,
      characterCount: evidence.characterCount,
    },
    { pinnedRefCount: 48, knowledgeCount: 66, characterCount: 40 },
  )
  assert.deepEqual(messages, [
    '[release] Knowledge refs OK (48 pinned ref(s), 66 knowledge record(s), 40 character record(s))',
  ])
})

test('record, id, alias, item, and missing-reference drift stays independently actionable', async () => {
  const invalidKnowledgePath = path.join(repositoryRoot, 'data', 'knowledge', 'aoi_herbal_lore.json')
  const missingIdPath = path.join(repositoryRoot, 'data', 'knowledge', 'constellation_map.json')
  const duplicateIdPath = path.join(repositoryRoot, 'data', 'knowledge', 'sakura_art_knowledge.json')
  const invalidCharacterPath = path.join(repositoryRoot, 'data', 'characters', 'kenji.json')
  const sakuraPath = path.join(repositoryRoot, 'data', 'characters', 'sakura.json')
  const evidence = await collectProjectKnowledgeReferenceEvidence({
    ...boundaries,
    async readTextFile(filePath, encoding) {
      const source = await readFile(filePath, encoding)
      const resolved = path.resolve(filePath)
      if (resolved === invalidKnowledgePath) return 'null'
      if (resolved === missingIdPath) {
        const entry = JSON.parse(source)
        entry.id = ''
        return JSON.stringify(entry)
      }
      if (resolved === duplicateIdPath) {
        const entries = JSON.parse(source)
        entries[0].id = 'sakura_nature'
        return JSON.stringify(entries)
      }
      if (resolved === invalidCharacterPath) return 'null'
      if (resolved === sakuraPath) {
        const character = JSON.parse(source)
        character.knowledge_refs = ['sakura_nature', 'missing_ref', ' ', 7]
        character.knowledgeRefs = { invalid: true }
        character.knowledge = ['sakura_art_knowledge']
        return JSON.stringify(character)
      }
      return source
    },
  })

  for (const issue of [
    'data/knowledge/aoi_herbal_lore.json: knowledge records must be JSON objects',
    'data/knowledge/constellation_map.json:<missing-knowledge-id>: knowledge id is required',
    'data/knowledge/sakura_nature.json:sakura_nature: duplicate knowledge id in data',
    'data/characters/kenji.json: character records must be JSON objects',
    'data/characters/sakura.json:sakura knowledgeRefs: pinned knowledge refs must be an array',
    'data/characters/sakura.json:sakura knowledge_refs[2]: pinned knowledge ref must be a non-empty string',
    'data/characters/sakura.json:sakura knowledge_refs[3]: pinned knowledge ref must be a non-empty string',
    'data/characters/sakura.json:sakura knowledge_refs: missing pinned knowledge ref "missing_ref" in data/knowledge',
    'data/characters/sakura.json:sakura knowledge: missing pinned knowledge ref "sakura_art_knowledge" in data/knowledge',
  ]) {
    assert(evidence.issues.includes(issue), issue)
  }
})

test('Project Knowledge Reference policy requires explicit roots and unique data-root labels', () => {
  assert.throws(() => createProjectKnowledgeReferencePolicy(), /requires repositoryRoot/)
  assert.throws(
    () => createProjectKnowledgeReferencePolicy({ repositoryRoot }),
    /requires rustDirectory/,
  )
  assert.throws(
    () => createProjectKnowledgeReferencePolicy({
      ...boundaries,
      dataRoots: [
        { label: 'project', dir: path.join(repositoryRoot, 'data') },
        { label: 'project', dir: path.join(rustDirectory, 'data') },
      ],
    }),
    /label is duplicated/,
  )
})

test('release runner delegates Knowledge references without retaining reference rules', async () => {
  const runnerSource = await readFile(path.join(repositoryRoot, 'scripts', 'verify-release.mjs'), 'utf8')
  const policySource = await readFile(
    path.join(repositoryRoot, 'scripts', 'lib', 'project-content', 'knowledge-reference-policy.mjs'),
    'utf8',
  )

  assert(runnerSource.includes('createProjectKnowledgeReferencePolicy'))
  assert(runnerSource.includes('verifyKnowledgeReferences'))
  assert(!runnerSource.includes('async function verifyKnowledgeRefs'))
  assert(!runnerSource.includes('function knowledgeRefFields'))
  assert(!runnerSource.includes('missing pinned knowledge ref'))
  assert(policySource.includes('async function collectKnowledgeReferenceEvidence'))
  assert(policySource.includes('function knowledgeRefFields'))
  assert(policySource.includes('missing pinned knowledge ref'))
})
