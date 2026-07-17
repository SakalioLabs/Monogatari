import { readdir, readFile as readFileFromDisk } from 'node:fs/promises'
import path from 'node:path'

import { isPortableProjectContentId } from './portable-id.mjs'

const dialogueControlPattern = /[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f]/

export function createProjectDialoguePolicy(options = {}) {
  const {
    repositoryRoot,
    dataRoots,
  } = resolveBoundaries(options)
  const readFile = options.readTextFile ?? readFileFromDisk
  const readDirectory = options.readDirectory ?? readdir
  const log = options.log ?? console.log
  const relative = options.relativePath
    ?? ((file) => path.relative(repositoryRoot, file).replaceAll('\\', '/'))
  const rendererDataRoots = dataRoots
  const portableStoryEventId = isPortableProjectContentId

  async function jsonFilesInDir(dir, issues) {
    try {
      return (await readDirectory(dir, { withFileTypes: true }))
        .filter((entry) => entry.isFile() && entry.name.endsWith('.json'))
        .map((entry) => path.join(dir, entry.name))
    } catch (error) {
      issues.push(`${relative(dir)}: ${error.message}`)
      return []
    }
  }

  function stableStringify(value) {
    if (Array.isArray(value)) return `[${value.map(stableStringify).join(',')}]`
    if (value && typeof value === 'object') {
      return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${stableStringify(value[key])}`).join(',')}}`
    }
    return JSON.stringify(value)
  }

  function nonEmptyString(value) {
    return typeof value === 'string' && value.trim().length > 0
  }

  async function collectDialogueEvidence() {
    const issues = []
    const catalogs = []
    let fileCount = 0
    let nodeCount = 0
    let choiceCount = 0
    const topFields = new Set(['id', 'title', 'description', 'start_node_id', 'nodes', 'variables'])
    const nodeFields = new Set([
      'id', 'speaker_id', 'text', 'next_node_id', 'choices', 'condition', 'script', 'emotion',
      'use_llm', 'llm_prompt', 'llm_system_prompt', 'is_ending', 'ending_type', 'scene_id',
    ])
    const choiceFields = new Set(['text', 'next_node_id', 'relationship_changes', 'condition'])

    for (const dataRoot of rendererDataRoots) {
      const characterIds = new Set()
      for (const file of await jsonFilesInDir(path.join(dataRoot.dir, 'characters'), issues)) {
        const value = JSON.parse(await readFile(file, 'utf8'))
        const characters = Array.isArray(value) ? value : [value]
        for (const character of characters) {
          if (portableStoryEventId(character?.id, 128)) characterIds.add(character.id)
        }
      }

      const sceneIds = new Set()
      for (const file of await jsonFilesInDir(path.join(dataRoot.dir, 'scenes'), issues)) {
        const scene = JSON.parse(await readFile(file, 'utf8'))
        if (portableStoryEventId(scene?.id, 128)) sceneIds.add(scene.id)
      }

      const files = await jsonFilesInDir(path.join(dataRoot.dir, 'dialogue'), issues)
      const ids = new Set()
      const documents = []
      fileCount += files.length
      for (const file of files) {
        const label = relative(file)
        const dialogue = JSON.parse(await readFile(file, 'utf8'))
        const unknownTopFields = Object.keys(dialogue ?? {}).filter((field) => !topFields.has(field))
        if (unknownTopFields.length > 0) issues.push(`${label}: unknown dialogue fields ${unknownTopFields.join(', ')}`)
        if (!portableStoryEventId(dialogue?.id, 128)) issues.push(`${label}: id must be a portable identifier`)
        else if (ids.has(dialogue.id)) issues.push(`${dataRoot.label}: duplicate dialogue id ${dialogue.id}`)
        else ids.add(dialogue.id)
        if (!nonEmptyString(dialogue?.title) || dialogue.title.length > 256 || dialogueControlPattern.test(dialogue.title)) {
          issues.push(`${label}: title must contain 1 to 256 supported characters`)
        }
        if (dialogue?.description !== undefined
          && (!nonEmptyString(dialogue.description) || dialogue.description.length > 2048 || dialogueControlPattern.test(dialogue.description))) {
          issues.push(`${label}: description must contain 1 to 2048 supported characters`)
        }
        if (!dialogue?.nodes || typeof dialogue.nodes !== 'object' || Array.isArray(dialogue.nodes)) {
          issues.push(`${label}: nodes must be an object`)
          continue
        }
        const nodeIds = Object.keys(dialogue.nodes)
        const nodeSet = new Set(nodeIds)
        nodeCount += nodeIds.length
        if (nodeIds.length < 1 || nodeIds.length > 2048) issues.push(`${label}: must contain 1 to 2048 nodes`)
        if (!portableStoryEventId(dialogue.start_node_id, 128) || !nodeSet.has(dialogue.start_node_id)) {
          issues.push(`${label}: start_node_id must identify an existing node`)
        }
        const variables = dialogue.variables ?? {}
        if (!variables || typeof variables !== 'object' || Array.isArray(variables)) {
          issues.push(`${label}: variables must be an object`)
        } else {
          const variableIds = Object.keys(variables)
          if (variableIds.length > 512) issues.push(`${label}: variables cannot exceed 512 entries`)
          for (const variableId of variableIds) {
            if (!portableStoryEventId(variableId, 128)) issues.push(`${label}: invalid variable id ${variableId}`)
          }
        }

        for (const [nodeId, node] of Object.entries(dialogue.nodes)) {
          const nodeLabel = `${label}:${nodeId}`
          if (!portableStoryEventId(nodeId, 128)) issues.push(`${nodeLabel}: node id must be portable`)
          if (!node || typeof node !== 'object' || Array.isArray(node)) {
            issues.push(`${nodeLabel}: node must be an object`)
            continue
          }
          const unknownNodeFields = Object.keys(node).filter((field) => !nodeFields.has(field))
          if (unknownNodeFields.length > 0) issues.push(`${nodeLabel}: unknown fields ${unknownNodeFields.join(', ')}`)
          if (node.id !== undefined && node.id !== nodeId) issues.push(`${nodeLabel}: embedded id must match the node map key`)
          if (!nonEmptyString(node.text) || node.text.length > 16384 || dialogueControlPattern.test(node.text)) {
            issues.push(`${nodeLabel}: text must contain 1 to 16384 supported characters`)
          }
          if (node.speaker_id !== undefined && node.speaker_id !== null) {
            if (!portableStoryEventId(node.speaker_id, 128)) issues.push(`${nodeLabel}: speaker_id must be portable`)
            else if (!characterIds.has(node.speaker_id)) issues.push(`${nodeLabel}: unknown speaker ${node.speaker_id}`)
          }
          if (node.scene_id !== undefined && node.scene_id !== null) {
            if (!portableStoryEventId(node.scene_id, 128)) issues.push(`${nodeLabel}: scene_id must be portable`)
            else if (!sceneIds.has(node.scene_id)) issues.push(`${nodeLabel}: unknown scene ${node.scene_id}`)
          }
          const choices = node.choices ?? []
          if (!Array.isArray(choices)) {
            issues.push(`${nodeLabel}: choices must be an array`)
            continue
          }
          choiceCount += choices.length
          if (choices.length > 32) issues.push(`${nodeLabel}: choices cannot exceed 32 entries`)
          if (node.next_node_id !== undefined && node.next_node_id !== null && choices.length > 0) {
            issues.push(`${nodeLabel}: cannot combine next_node_id with choices`)
          }
          if (node.next_node_id !== undefined && node.next_node_id !== null && !nodeSet.has(node.next_node_id)) {
            issues.push(`${nodeLabel}: missing next node ${String(node.next_node_id)}`)
          }
          if (node.is_ending !== undefined && typeof node.is_ending !== 'boolean') {
            issues.push(`${nodeLabel}: is_ending must be boolean`)
          }
          if (node.is_ending === true && (node.next_node_id != null || choices.length > 0)) {
            issues.push(`${nodeLabel}: ending nodes cannot have outgoing transitions`)
          }
          if (node.ending_type !== undefined && node.ending_type !== null) {
            if (node.is_ending !== true) issues.push(`${nodeLabel}: ending_type requires is_ending=true`)
            if (!nonEmptyString(node.ending_type) || node.ending_type.length > 64 || dialogueControlPattern.test(node.ending_type)) {
              issues.push(`${nodeLabel}: ending_type must contain 1 to 64 supported characters`)
            }
          }
          if (node.use_llm !== undefined && typeof node.use_llm !== 'boolean') issues.push(`${nodeLabel}: use_llm must be boolean`)
          if (node.use_llm === true && !nonEmptyString(node.llm_prompt)) issues.push(`${nodeLabel}: use_llm requires llm_prompt`)
          for (const [field, maxLength] of [['condition', 2000], ['script', 20000], ['llm_prompt', 20000], ['llm_system_prompt', 20000]]) {
            if (node[field] !== undefined && node[field] !== null
              && (!nonEmptyString(node[field]) || node[field].length > maxLength || dialogueControlPattern.test(node[field]))) {
              issues.push(`${nodeLabel}: ${field} must contain 1 to ${maxLength} supported characters`)
            }
          }

          choices.forEach((choice, index) => {
            const choiceLabel = `${nodeLabel}:choice-${index + 1}`
            if (!choice || typeof choice !== 'object' || Array.isArray(choice)) {
              issues.push(`${choiceLabel}: choice must be an object`)
              return
            }
            const unknownChoiceFields = Object.keys(choice).filter((field) => !choiceFields.has(field))
            if (unknownChoiceFields.length > 0) issues.push(`${choiceLabel}: unknown fields ${unknownChoiceFields.join(', ')}`)
            if (!nonEmptyString(choice.text) || choice.text.length > 2048 || dialogueControlPattern.test(choice.text)) {
              issues.push(`${choiceLabel}: text must contain 1 to 2048 supported characters`)
            }
            if (!portableStoryEventId(choice.next_node_id, 128) || !nodeSet.has(choice.next_node_id)) {
              issues.push(`${choiceLabel}: target must identify an existing node`)
            }
            if (choice.condition !== undefined && choice.condition !== null
              && (!nonEmptyString(choice.condition) || choice.condition.length > 2000 || dialogueControlPattern.test(choice.condition))) {
              issues.push(`${choiceLabel}: condition must contain 1 to 2000 supported characters`)
            }
            const changes = choice.relationship_changes ?? {}
            if (!changes || typeof changes !== 'object' || Array.isArray(changes)) {
              issues.push(`${choiceLabel}: relationship_changes must be an object`)
            } else {
              if (Object.keys(changes).length > 128) issues.push(`${choiceLabel}: relationship_changes cannot exceed 128 entries`)
              for (const [characterId, delta] of Object.entries(changes)) {
                if (!characterIds.has(characterId)) issues.push(`${choiceLabel}: unknown relationship character ${characterId}`)
                if (!Number.isFinite(delta) || delta < -1 || delta > 1) {
                  issues.push(`${choiceLabel}: relationship delta for ${characterId} must be between -1 and 1`)
                }
              }
            }
          })
        }

        if (nodeSet.has(dialogue.start_node_id)) {
          const reachable = new Set()
          const queue = [dialogue.start_node_id]
          while (queue.length > 0) {
            const nodeId = queue.shift()
            if (reachable.has(nodeId) || !dialogue.nodes[nodeId]) continue
            reachable.add(nodeId)
            const node = dialogue.nodes[nodeId]
            if (typeof node.next_node_id === 'string') queue.push(node.next_node_id)
            if (Array.isArray(node.choices)) {
              for (const choice of node.choices) {
                if (typeof choice?.next_node_id === 'string') queue.push(choice.next_node_id)
              }
            }
          }
          const unreachable = nodeIds.filter((nodeId) => !reachable.has(nodeId))
          if (unreachable.length > 0) issues.push(`${label}: unreachable nodes ${unreachable.join(', ')}`)
        }
        documents.push(dialogue)
      }
      documents.sort((left, right) => String(left.id).localeCompare(String(right.id)))
      catalogs.push({ label: dataRoot.label, documents })
    }

    if (catalogs.length === 2 && stableStringify(catalogs[0].documents) !== stableStringify(catalogs[1].documents)) {
      issues.push(`${catalogs[0].label} and ${catalogs[1].label} dialogue catalogs must match`)
    }
    return {
      issues,
      fileCount,
      nodeCount,
      choiceCount,
    }
  }

  async function verifyDialogueCatalogs() {
    const evidence = await collectDialogueEvidence()
    if (evidence.issues.length > 0) {
      throw new Error(`Dialogue catalog verification failed:\n${evidence.issues.join('\n')}`)
    }
    log(
      `[release] Dialogue catalogs OK (${evidence.fileCount} files, ${evidence.nodeCount} nodes, ${evidence.choiceCount} choices)`,
    )
    return evidence
  }

  return Object.freeze({
    collectDialogueEvidence,
    verifyDialogueCatalogs,
  })
}

export async function collectProjectDialogueEvidence(options = {}) {
  return createProjectDialoguePolicy(options).collectDialogueEvidence()
}

function resolveBoundaries(options) {
  const repositoryRoot = options.repositoryRoot
  const rustDirectory = options.rustDirectory
  for (const [name, value] of Object.entries({ repositoryRoot, rustDirectory })) {
    if (typeof value !== 'string' || value.length === 0) {
      throw new Error(`Project Dialogue policy requires ${name}.`)
    }
  }

  const dataRoots = options.dataRoots ?? [
    { label: 'data', dir: path.join(repositoryRoot, 'data') },
    { label: 'rust-engine/data', dir: path.join(rustDirectory, 'data') },
  ]
  if (!Array.isArray(dataRoots) || dataRoots.length === 0) {
    throw new Error('Project Dialogue policy requires at least one data root.')
  }
  const labels = new Set()
  for (const [index, dataRoot] of dataRoots.entries()) {
    if (!dataRoot || typeof dataRoot.label !== 'string' || dataRoot.label.length === 0) {
      throw new Error(`Project Dialogue policy dataRoots[${index}] requires label.`)
    }
    if (typeof dataRoot.dir !== 'string' || dataRoot.dir.length === 0) {
      throw new Error(`Project Dialogue policy dataRoots[${index}] requires dir.`)
    }
    if (labels.has(dataRoot.label)) {
      throw new Error(`Project Dialogue policy data root label is duplicated: ${dataRoot.label}`)
    }
    labels.add(dataRoot.label)
  }

  return {
    repositoryRoot,
    dataRoots: dataRoots.map((dataRoot) => ({ ...dataRoot })),
  }
}
