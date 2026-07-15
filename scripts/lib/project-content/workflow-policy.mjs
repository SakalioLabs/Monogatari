import { readdir as readDirectoryFromDisk, readFile as readFileFromDisk } from 'node:fs/promises'
import path from 'node:path'

import { createProjectStoryEventPolicy } from './story-event-policy.mjs'

export function createProjectWorkflowPolicy(options = {}) {
  const {
    repositoryRoot,
    rustDirectory,
    dataRoots: rendererDataRoots,
  } = resolveBoundaries(options)
  const readdir = options.readDirectory ?? readDirectoryFromDisk
  const readFile = options.readTextFile ?? readFileFromDisk
  const log = options.log ?? console.log
  const relative = options.relativePath
    ?? ((file) => path.relative(repositoryRoot, file).replaceAll('\\', '/'))
  const storyEventPolicy = options.storyEventPolicy ?? createProjectStoryEventPolicy({
    repositoryRoot,
    rustDirectory,
    dataRoots: rendererDataRoots,
    readDirectory: readdir,
    readTextFile: readFile,
    relativePath: relative,
    log,
  })
  const loadStoryEventCatalog = storyEventPolicy.loadStoryEventCatalog

  async function collectWorkflowEvidence() {
    const workflowDirs = rendererDataRoots.map((dataRoot) => path.join(dataRoot.dir, 'workflows'))
    const workflowFiles = []
    const issues = []
    const eventCatalogs = new Map()

    for (const dataRoot of rendererDataRoots) {
      eventCatalogs.set(dataRoot.label, (await loadStoryEventCatalog(dataRoot, issues)).events)
    }

    for (const workflowDir of workflowDirs) {
      try {
        for (const entry of await readdir(workflowDir, { withFileTypes: true })) {
          if (entry.isFile() && entry.name.endsWith('.json')) {
            workflowFiles.push(path.join(workflowDir, entry.name))
          }
        }
      } catch (error) {
        issues.push(`${relative(workflowDir)}: ${error.message}`)
      }
    }

    for (const workflowPath of workflowFiles) {
      const workflow = JSON.parse(await readFile(workflowPath, 'utf8'))
      const dataRoot = rendererDataRoots.find((candidate) => workflowPath.startsWith(candidate.dir))
      issues.push(...verifyProjectWorkflowShape(
        workflow,
        relative(workflowPath),
        eventCatalogs.get(dataRoot?.label) ?? new Map(),
      ))
    }

    if (workflowFiles.length === 0) {
      issues.push('No workflow files found in data/workflows or rust-engine/data/workflows')
    }

    return {
      issues,
      workflowCount: workflowFiles.length,
    }
  }

  async function verifyWorkflowFiles() {
    const evidence = await collectWorkflowEvidence()
    if (evidence.issues.length > 0) {
      throw new Error(`Workflow verification failed:\n${evidence.issues.join('\n')}`)
    }
    log(`[release] Workflow files OK (${evidence.workflowCount} workflow file(s))`)
    return evidence
  }

  return Object.freeze({
    collectWorkflowEvidence,
    verifyWorkflowFiles,
  })
}

export async function collectProjectWorkflowEvidence(options = {}) {
  return createProjectWorkflowPolicy(options).collectWorkflowEvidence()
}

const workflowStateKeyMaxChars = 128
const workflowStateKeyPattern = /^[A-Za-z0-9_.-]+$/
const workflowConditionMaxChars = 2000
const workflowConditionControlPattern = /[\u0000-\u0008\u000B\u000C\u000E-\u001F\u007F-\u009F]/u

function validateScriptStateKey(value) {
  if (typeof value !== 'string') return 'must be a string'
  const key = value.trim()
  if (!key) return null
  if (key.length > workflowStateKeyMaxChars) return `must be ${workflowStateKeyMaxChars} characters or fewer`
  if (key === '.' || key === '..') return 'must not be a current or parent directory marker'
  if (!workflowStateKeyPattern.test(key)) {
    return 'can contain only ASCII letters, numbers, dots, underscores, or hyphens'
  }
  return null
}

function validateWorkflowCondition(value) {
  if (typeof value !== 'string') return 'must be a string'
  if (!value.trim()) return null
  if (Array.from(value).length > workflowConditionMaxChars) {
    return `must be ${workflowConditionMaxChars} characters or fewer`
  }
  if (workflowConditionControlPattern.test(value)) return 'cannot contain control characters'
  return null
}

function workflowStateKeyFields(nodeType) {
  switch (nodeType) {
    case 'set_variable':
    case 'evaluation':
      return ['variable_name']
    case 'set_flag':
      return ['flag_name']
    default:
      return []
  }
}

export function verifyProjectWorkflowShape(workflow, label, storyEvents = new Map()) {
  const issues = []
  if (!nonEmptyString(workflow.id)) issues.push(`${label}: id is required`)
  if (!nonEmptyString(workflow.name)) issues.push(`${label}: name is required`)
  if (!nonEmptyString(workflow.start_node_id)) issues.push(`${label}: start_node_id is required`)
  if (!Array.isArray(workflow.nodes) || workflow.nodes.length === 0) {
    issues.push(`${label}: nodes must be a non-empty array`)
    return issues
  }

  const knownTypes = new Set([
    'start',
    'dialogue',
    'choice',
    'condition',
    'set_variable',
    'set_flag',
    'llm_generate',
    'evaluation',
    'scene_change',
    'trigger_event',
    'emotion_change',
    'relationship',
    'narration',
    'bgm',
    'sfx',
    'wait',
    'random_branch',
    'sub_workflow',
    'camera',
    'shake',
    'end',
  ])
  const nodeIds = new Set()
  const nodeById = new Map()

  for (const node of workflow.nodes) {
    const nodeLabel = `${label}:${node?.id || '<missing-node-id>'}`
    if (!nonEmptyString(node?.id)) {
      issues.push(`${nodeLabel}: node id is required`)
      continue
    }
    if (nodeIds.has(node.id)) issues.push(`${nodeLabel}: duplicate node id`)
    nodeIds.add(node.id)
    nodeById.set(node.id, node)
  }

  const startNode = nodeById.get(workflow.start_node_id)
  if (!startNode) {
    issues.push(`${label}: start_node_id does not match any node`)
  } else if (startNode.node_type !== 'start') {
    issues.push(`${label}:${startNode.id}: start_node_id must reference a start node`)
  }

  for (const node of workflow.nodes) {
    const nodeLabel = `${label}:${node?.id || '<missing-node-id>'}`
    if (!nonEmptyString(node?.node_type)) {
      issues.push(`${nodeLabel}: node_type is required`)
      continue
    }
    if (!knownTypes.has(node.node_type)) {
      issues.push(`${nodeLabel}: unknown node_type ${node.node_type}`)
      continue
    }
    const config = node.config ?? {}
    for (const field of workflowRequiredFields(node.node_type)) {
      if (!workflowConfigFieldPresent(node.node_type, config, field)) {
        issues.push(`${nodeLabel}: required field ${field} is missing`)
      }
    }
    for (const field of workflowStateKeyFields(node.node_type)) {
      const value = config[field]
      if (value === null || value === undefined || (typeof value === 'string' && !value.trim())) continue
      const error = validateScriptStateKey(value)
      if (error) issues.push(`${nodeLabel}: state key field ${field} is invalid: ${error}`)
    }
    if (node.node_type === 'condition') {
      const value = config.condition
      if (value !== null && value !== undefined && !(typeof value === 'string' && !value.trim())) {
        const error = validateWorkflowCondition(value)
        if (error) issues.push(`${nodeLabel}: condition field is invalid: ${error}`)
      }
    }
    if (node.node_type === 'trigger_event' && nonEmptyString(config.event_id)) {
      const definition = storyEvents.get(config.event_id.trim())
      if (!definition) {
        issues.push(`${nodeLabel}: story event ${config.event_id} is not in the project catalog`)
      } else if (nonEmptyString(config.event_type) && config.event_type.trim() !== definition.event_type) {
        issues.push(`${nodeLabel}: story event ${config.event_id} does not use type ${config.event_type}`)
      } else if (
        nonEmptyString(config.character_id)
        && definition.character_ids.length > 0
        && !definition.character_ids.includes(config.character_id.trim())
      ) {
        issues.push(`${nodeLabel}: story event ${config.event_id} is not available for character ${config.character_id}`)
      }
    }
    if (!Array.isArray(node.connections)) {
      issues.push(`${nodeLabel}: connections must be an array`)
      continue
    }
    const localTargets = new Set()
    for (const targetId of node.connections) {
      if (!nonEmptyString(targetId)) {
        issues.push(`${nodeLabel}: connection target id is empty`)
        continue
      }
      if (targetId === node.id) issues.push(`${nodeLabel}: node cannot connect to itself`)
      if (!nodeIds.has(targetId)) issues.push(`${nodeLabel}: connection target ${targetId} does not exist`)
      localTargets.add(targetId)
    }
  }

  return issues
}

function workflowRequiredFields(nodeType) {
  switch (nodeType) {
    case 'dialogue':
    case 'narration':
      return ['text']
    case 'choice':
      return ['choices']
    case 'condition':
      return ['condition']
    case 'set_variable':
      return ['variable_name', 'value']
    case 'set_flag':
      return ['flag_name', 'value']
    case 'llm_generate':
      return ['prompt']
    case 'evaluation':
      return ['criteria']
    case 'scene_change':
      return ['scene_id']
    case 'trigger_event':
      return ['event_id']
    case 'bgm':
      return ['track_path']
    case 'sfx':
      return ['sound_path']
    case 'wait':
    case 'shake':
      return ['duration_ms']
    case 'sub_workflow':
      return ['workflow_id']
    case 'emotion_change':
      return ['character_id', 'emotion']
    case 'relationship':
      return ['character_id', 'delta']
    default:
      return []
  }
}

function workflowConfigFieldPresent(nodeType, config, field) {
  const aliases = {
    'bgm:track_path': ['track_path', 'track'],
    'sfx:sound_path': ['sound_path', 'sound'],
    'wait:duration_ms': ['duration_ms', 'duration'],
    'shake:duration_ms': ['duration_ms', 'duration'],
  }[`${nodeType}:${field}`] ?? [field]

  return aliases.some((alias) => configValuePresent(config?.[alias]))
}

function configValuePresent(value) {
  if (value === undefined || value === null) return false
  if (typeof value === 'string') return value.trim().length > 0
  if (Array.isArray(value)) return value.length > 0
  if (typeof value === 'object') return Object.keys(value).length > 0
  return true
}

function nonEmptyString(value) {
  return typeof value === 'string' && value.trim().length > 0
}

function resolveBoundaries(options) {
  const repositoryRoot = options.repositoryRoot
  const rustDirectory = options.rustDirectory
  for (const [name, value] of Object.entries({ repositoryRoot, rustDirectory })) {
    if (typeof value !== 'string' || value.length === 0) {
      throw new Error(`Project Workflow policy requires ${name}.`)
    }
  }

  const dataRoots = options.dataRoots ?? [
    { label: 'data', dir: path.join(repositoryRoot, 'data') },
    { label: 'rust-engine/data', dir: path.join(rustDirectory, 'data') },
  ]
  if (!Array.isArray(dataRoots) || dataRoots.length === 0) {
    throw new Error('Project Workflow policy requires at least one data root.')
  }
  const labels = new Set()
  for (const [index, dataRoot] of dataRoots.entries()) {
    if (!dataRoot || typeof dataRoot.label !== 'string' || dataRoot.label.length === 0) {
      throw new Error(`Project Workflow policy dataRoots[${index}] requires label.`)
    }
    if (typeof dataRoot.dir !== 'string' || dataRoot.dir.length === 0) {
      throw new Error(`Project Workflow policy dataRoots[${index}] requires dir.`)
    }
    if (labels.has(dataRoot.label)) {
      throw new Error(`Project Workflow policy data root label is duplicated: ${dataRoot.label}`)
    }
    labels.add(dataRoot.label)
  }

  return {
    repositoryRoot,
    rustDirectory,
    dataRoots: dataRoots.map((dataRoot) => ({ ...dataRoot })),
  }
}
