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
  assert(evidence.jsonFileCount >= 270)
  assert.deepEqual(messages, [`[release] JSON parse OK (${evidence.jsonFileCount} files)`])

  const files = await createRepositoryFileWalker()(repositoryRoot)
  const projectPrefix = path.join(repositoryRoot, 'projects', 'konosuba') + path.sep
  const projectJsonFiles = files.filter(file => file.startsWith(projectPrefix) && file.endsWith('.json'))
  assert.equal(projectJsonFiles.length, 224)
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'campaigns', 'volume1_campaign.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'campaigns', 'volume2_campaign.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'campaigns', 'volume3_campaign.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'roleplays', 'chapter4_roleplay.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'roleplays', 'volume2_chapter1_roleplay.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'roleplays', 'volume2_chapter2_roleplay.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'roleplays', 'volume2_chapter3_roleplay.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'roleplays', 'volume2_chapter4_roleplay.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'roleplays', 'volume2_chapter5_roleplay.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'roleplays', 'volume2_epilogue_roleplay.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'roleplays', 'volume3_chapter1_roleplay.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'roleplays', 'volume3_chapter2_roleplay.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'quality_suites', 'chapter4_roleplay.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'quality_suites', 'volume2_chapter1_roleplay.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'quality_suites', 'volume2_chapter2_roleplay.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'quality_suites', 'volume2_chapter3_roleplay.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'quality_suites', 'volume2_chapter4_roleplay.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'quality_suites', 'volume2_chapter5_roleplay.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'quality_suites', 'volume2_epilogue_roleplay.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'quality_suites', 'volume3_chapter1_roleplay.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'quality_suites', 'volume3_chapter2_roleplay.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'characters', 'eris.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'characters', 'anna_filante.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'characters', 'succubus_receptionist.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'characters', 'succubus_runner.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'characters', 'sena.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'characters', 'alderp.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'characters', 'yunyun.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'characters', 'balter.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'characters', 'lord_dustiness.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'knowledge', 'volume2_winter_general.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'knowledge', 'volume2_keele_request.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'knowledge', 'volume2_ghost_displacement.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'knowledge', 'volume2_succubus_cafe.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'knowledge', 'volume2_dream_reality.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'knowledge', 'volume2_mansion_privacy.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'knowledge', 'volume2_coronatite_core.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'knowledge', 'volume2_magic_transfer.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'knowledge', 'volume2_royal_charge_procedure.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'knowledge', 'volume3_truth_bell_limits.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'knowledge', 'volume3_challenge_consent_and_stakes.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'knowledge', 'volume3_privacy_and_darkness_welfare.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'knowledge', 'volume3_darkness_identity_disclosure.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'knowledge', 'volume3_meeting_consent_terms.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'knowledge', 'volume3_training_stop_rules.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'knowledge', 'volume3_reputation_and_privacy_record.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'scenes', 'axel_winter_spirit_field.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'scenes', 'keele_hidden_chamber.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'scenes', 'wiz_magic_item_shop.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'scenes', 'haunted_mansion_night.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'scenes', 'axel_succubus_cafe_alley.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'scenes', 'succubus_cafe_consultation.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'scenes', 'mansion_bath_corridor.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'scenes', 'destroyer_core_chamber.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'scenes', 'axel_guild_arrest.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'scenes', 'axel_courtroom.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'scenes', 'axel_snowy_toad_field.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'scenes', 'axel_winter_market.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'scenes', 'dustiness_manor_reception.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'scenes', 'dustiness_manor_winter_garden.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'scenes', 'dustiness_manor_training_yard.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'endings', 'volume2_destroyer_defeated.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'endings', 'volume2_charge_faced_together.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'endings', 'volume3_evidence_bound_stay.json')))
  assert(projectJsonFiles.includes(path.join(projectPrefix, 'endings', 'volume3_open_invitation_and_welfare_plan.json')))
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
