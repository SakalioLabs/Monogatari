import assert from 'node:assert/strict'
import { readdir, readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')
const projectRoot = path.join(repositoryRoot, 'projects', 'konosuba')

async function readJsonDirectory(directory) {
  const root = path.join(projectRoot, directory)
  const fileNames = (await readdir(root)).filter(name => name.endsWith('.json')).sort()
  return Promise.all(fileNames.map(async (fileName) => ({
    fileName,
    value: JSON.parse(await readFile(path.join(root, fileName), 'utf8')),
  })))
}

test('KonoSuba live roleplays keep character, scene, knowledge, and participant motives closed', async () => {
  const [characterDocuments, knowledgeDocuments, sceneDocuments, roleplayDocuments] = await Promise.all([
    readJsonDirectory('characters'),
    readJsonDirectory('knowledge'),
    readJsonDirectory('scenes'),
    readJsonDirectory('roleplays'),
  ])
  const characterIds = new Set(characterDocuments.map(document => document.value.id))
  const knowledgeIds = new Set(knowledgeDocuments.flatMap(document => {
    const entries = Array.isArray(document.value) ? document.value : [document.value]
    return entries.map(entry => entry.id)
  }))
  const sceneIds = new Set(sceneDocuments.map(document => document.value.id))
  const usedCharacterIds = new Set()
  let nodeCount = 0
  let supportingParticipantCount = 0

  for (const { fileName, value: roleplay } of roleplayDocuments) {
    for (const node of roleplay.nodes) {
      nodeCount += 1
      const participantIds = [node.character_id, ...(node.supporting_character_ids || [])]
      assert.equal(
        new Set(participantIds).size,
        participantIds.length,
        `${fileName}:${node.id} repeats a participant`,
      )
      for (const characterId of participantIds) {
        assert(characterIds.has(characterId), `${fileName}:${node.id} references unknown character ${characterId}`)
        usedCharacterIds.add(characterId)
      }
      for (const characterId of node.supporting_character_ids || []) {
        supportingParticipantCount += 1
        assert(
          node.participant_goals?.[characterId]?.trim(),
          `${fileName}:${node.id} lacks a scene-local motive for ${characterId}`,
        )
      }
      for (const characterId of Object.keys(node.participant_goals || {})) {
        assert(
          participantIds.includes(characterId),
          `${fileName}:${node.id} assigns a motive to absent character ${characterId}`,
        )
      }
      assert(sceneIds.has(node.scene_id), `${fileName}:${node.id} references unknown scene ${node.scene_id}`)
      for (const knowledgeId of node.knowledge_refs || []) {
        assert(
          knowledgeIds.has(knowledgeId),
          `${fileName}:${node.id} references unknown knowledge ${knowledgeId}`,
        )
      }
    }
  }

  assert.equal(characterDocuments.length, 40)
  assert.equal(knowledgeIds.size, 217)
  assert.equal(sceneDocuments.length, 108)
  assert.equal(roleplayDocuments.length, 33)
  assert.equal(nodeCount, 257)
  assert.equal(supportingParticipantCount, 548)
  assert.deepEqual([...characterIds].filter(id => !usedCharacterIds.has(id)), [])
})
