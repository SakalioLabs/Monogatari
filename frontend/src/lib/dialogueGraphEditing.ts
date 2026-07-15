import type {
  DialogueAuthoringCatalogSnapshot,
  DialogueAuthoringEntry,
  DialogueChoiceDefinition,
  DialogueDefinition,
  DialogueNodeDefinition,
} from './dialogueAuthoring'
import { cloneJsonRecord, parseJsonRecord } from './jsonValue'

export type DialogueFlowMode = 'linear' | 'choices' | 'end'

export interface DialogueCharacterIdentity {
  id: string
  name: string
}

export interface DialogueNodeAppendResult {
  dialogue: DialogueDefinition
  node_id: string
}

export type DialogueNodeRenameError = 'node_missing' | 'invalid_id' | 'node_exists'

export interface DialogueNodeRenameResult {
  dialogue: DialogueDefinition
  node_id: string
  changed: boolean
  error: DialogueNodeRenameError | null
}

export type DialogueNodeDeleteError = 'node_missing' | 'last_node' | 'node_referenced' | 'start_node'

export interface DialogueNodeDeleteResult {
  dialogue: DialogueDefinition
  selected_node_id: string | null
  references: string[]
  changed: boolean
  error: DialogueNodeDeleteError | null
}

const PORTABLE_ID = /^[A-Za-z0-9_.-]{1,128}$/

export function cloneDialogueDefinition(dialogue: DialogueDefinition): DialogueDefinition {
  return {
    id: dialogue.id,
    title: dialogue.title,
    description: dialogue.description,
    start_node_id: dialogue.start_node_id,
    nodes: Object.fromEntries(Object.entries(dialogue.nodes)
      .map(([nodeId, node]) => [nodeId, cloneDialogueNode(node)])),
    variables: cloneJsonRecord(dialogue.variables),
  }
}

export function dialogueDefinitionFromEntry(entry: DialogueAuthoringEntry): DialogueDefinition {
  return cloneDialogueDefinition({
    id: entry.id,
    title: entry.title,
    description: entry.description,
    start_node_id: entry.start_node_id,
    nodes: entry.nodes,
    variables: entry.variables,
  })
}

export function filterDialogueEntries(
  entries: readonly DialogueAuthoringEntry[],
  query: string,
): DialogueAuthoringEntry[] {
  const normalized = query.trim().toLocaleLowerCase()
  return entries.filter((dialogue) => !normalized
    || dialogue.id.toLocaleLowerCase().includes(normalized)
    || dialogue.title.toLocaleLowerCase().includes(normalized)
    || dialogue.description?.toLocaleLowerCase().includes(normalized))
}

export function parseDialogueVariables(value: string): Record<string, unknown> | null {
  return parseJsonRecord(value)
}

export function dialogueDraftSnapshot(
  dialogue: DialogueDefinition | null | undefined,
  variablesText: string,
): string {
  return dialogue ? JSON.stringify({ dialogue, variablesText }) : ''
}

export function hasDialogueIdCollision(
  existingIds: readonly string[],
  candidateId: string,
): boolean {
  const candidateKey = portableIdKey(candidateId)
  return candidateKey.length > 0 && existingIds.some((id) => portableIdKey(id) === candidateKey)
}

export function nextDialogueId(
  existingIds: readonly string[],
  base = 'new_dialogue',
): string {
  const normalizedBase = base.trim() || 'new_dialogue'
  const keys = new Set(existingIds.map(portableIdKey))
  if (!keys.has(portableIdKey(normalizedBase))) return normalizedBase
  let index = 2
  while (keys.has(portableIdKey(`${normalizedBase}_${index}`))) index += 1
  return `${normalizedBase}_${index}`
}

export function createDialogueDraft(
  existingIds: readonly string[],
  title: string,
  line: string,
  speakerId: string | null = null,
): DialogueDefinition {
  return {
    id: nextDialogueId(existingIds),
    title,
    description: null,
    start_node_id: 'start',
    nodes: { start: createDialogueNode(speakerId, line) },
    variables: {},
  }
}

export function duplicateDialogueDraft(
  source: DialogueDefinition,
  existingIds: readonly string[],
  title: string,
): DialogueDefinition {
  const duplicate = cloneDialogueDefinition(source)
  duplicate.id = nextDialogueId(existingIds, `${source.id}_copy`)
  duplicate.title = title
  return duplicate
}

export function createDialogueNode(
  speakerId: string | null = null,
  text = '',
): DialogueNodeDefinition {
  return {
    speaker_id: speakerId,
    text,
    next_node_id: null,
    choices: [],
    condition: null,
    script: null,
    emotion: null,
    use_llm: false,
    llm_prompt: null,
    llm_system_prompt: null,
    is_ending: false,
    ending_type: null,
  }
}

export function nextDialogueNodeId(
  dialogue: DialogueDefinition,
  base = 'node',
): string {
  const ids = new Set(Object.keys(dialogue.nodes))
  if (!ids.has(base)) return base
  let index = 2
  while (ids.has(`${base}_${index}`)) index += 1
  return `${base}_${index}`
}

export function appendDialogueNode(
  source: DialogueDefinition,
  speakerId: string | null = null,
  base = 'node',
): DialogueNodeAppendResult {
  const dialogue = cloneDialogueDefinition(source)
  const nodeId = nextDialogueNodeId(dialogue, base)
  dialogue.nodes = {
    ...dialogue.nodes,
    [nodeId]: createDialogueNode(speakerId),
  }
  return { dialogue, node_id: nodeId }
}

export function dialogueNodeOrder(dialogue: DialogueDefinition): string[] {
  const result: string[] = []
  const visited = new Set<string>()
  const queue = [dialogue.start_node_id]
  while (queue.length > 0) {
    const nodeId = queue.shift()!
    if (visited.has(nodeId) || !dialogue.nodes[nodeId]) continue
    visited.add(nodeId)
    result.push(nodeId)
    const node = dialogue.nodes[nodeId]
    if (node.next_node_id) queue.push(node.next_node_id)
    node.choices.forEach((choice) => queue.push(choice.next_node_id))
  }
  result.push(...Object.keys(dialogue.nodes).filter((nodeId) => !visited.has(nodeId)).sort())
  return result
}

export function dialogueTargetNodeIds(dialogue: DialogueDefinition): string[] {
  return Object.keys(dialogue.nodes).sort()
}

export function dialogueImplicitTerminalIds(dialogue: DialogueDefinition): string[] {
  return Object.entries(dialogue.nodes)
    .filter(([, node]) => !node.next_node_id && node.choices.length === 0 && !node.is_ending)
    .map(([nodeId]) => nodeId)
}

export function dialogueTerminalCount(dialogue: DialogueDefinition): number {
  return Object.values(dialogue.nodes)
    .filter((node) => !node.next_node_id && node.choices.length === 0)
    .length
}

export function dialogueFlowMode(node: DialogueNodeDefinition): DialogueFlowMode {
  if (node.choices.length > 0) return 'choices'
  if (node.next_node_id) return 'linear'
  return 'end'
}

export function renameDialogueNode(
  source: DialogueDefinition,
  before: string,
  requestedId: string,
): DialogueNodeRenameResult {
  const after = requestedId.trim()
  if (!Object.hasOwn(source.nodes, before)) {
    return { dialogue: source, node_id: before, changed: false, error: 'node_missing' }
  }
  if (!PORTABLE_ID.test(after)) {
    return { dialogue: source, node_id: before, changed: false, error: 'invalid_id' }
  }
  if (after !== before && Object.hasOwn(source.nodes, after)) {
    return { dialogue: source, node_id: before, changed: false, error: 'node_exists' }
  }
  if (after === before) {
    return { dialogue: source, node_id: before, changed: false, error: null }
  }

  const dialogue = cloneDialogueDefinition(source)
  dialogue.nodes = Object.fromEntries(Object.entries(dialogue.nodes)
    .map(([nodeId, node]) => [nodeId === before ? after : nodeId, node]))
  if (dialogue.start_node_id === before) dialogue.start_node_id = after
  for (const node of Object.values(dialogue.nodes)) {
    if (node.next_node_id === before) node.next_node_id = after
    node.choices.forEach((choice) => {
      if (choice.next_node_id === before) choice.next_node_id = after
    })
  }
  return { dialogue, node_id: after, changed: true, error: null }
}

export function deleteDialogueNode(
  source: DialogueDefinition,
  nodeId: string,
): DialogueNodeDeleteResult {
  if (!Object.hasOwn(source.nodes, nodeId)) return deleteError(source, nodeId, 'node_missing')
  if (Object.keys(source.nodes).length <= 1) return deleteError(source, nodeId, 'last_node')

  const references = new Set<string>()
  for (const [sourceId, node] of Object.entries(source.nodes)) {
    if (node.next_node_id === nodeId || node.choices.some((choice) => choice.next_node_id === nodeId)) {
      references.add(sourceId)
    }
  }
  if (references.size > 0) {
    return {
      dialogue: source,
      selected_node_id: nodeId,
      references: [...references],
      changed: false,
      error: 'node_referenced',
    }
  }
  if (source.start_node_id === nodeId) return deleteError(source, nodeId, 'start_node')

  const dialogue = cloneDialogueDefinition(source)
  dialogue.nodes = Object.fromEntries(Object.entries(dialogue.nodes)
    .filter(([candidate]) => candidate !== nodeId))
  const selectedNodeId = dialogueNodeOrder(source).find((candidate) => candidate !== nodeId)
    || Object.keys(dialogue.nodes)[0]
    || null
  return {
    dialogue,
    selected_node_id: selectedNodeId,
    references: [],
    changed: true,
    error: null,
  }
}

export function setDialogueNodeFlowMode(
  source: DialogueNodeDefinition,
  nodeId: string,
  mode: DialogueFlowMode,
  targetNodeIds: readonly string[],
  newChoiceText: string,
): DialogueNodeDefinition {
  const node = cloneDialogueNode(source)
  const fallbackTarget = targetNodeIds.find((targetId) => targetId !== nodeId) || ''
  if (mode === 'linear') {
    node.choices = []
    node.is_ending = false
    node.ending_type = null
    node.next_node_id ||= fallbackTarget || null
  } else if (mode === 'choices') {
    node.next_node_id = null
    node.is_ending = false
    node.ending_type = null
    if (node.choices.length === 0) node.choices.push(createDialogueChoice(newChoiceText, fallbackTarget))
  } else {
    node.next_node_id = null
    node.choices = []
    node.is_ending = true
  }
  return node
}

export function appendDialogueChoice(
  source: DialogueNodeDefinition,
  nodeId: string,
  targetNodeIds: readonly string[],
  text: string,
): DialogueNodeDefinition {
  const node = cloneDialogueNode(source)
  if (node.choices.length >= 32) return node
  const target = targetNodeIds.find((targetId) => targetId !== nodeId) || ''
  node.next_node_id = null
  node.is_ending = false
  node.ending_type = null
  node.choices.push(createDialogueChoice(text, target))
  return node
}

export function removeDialogueChoice(
  source: DialogueNodeDefinition,
  index: number,
): DialogueNodeDefinition {
  const node = cloneDialogueNode(source)
  if (Number.isInteger(index) && index >= 0 && index < node.choices.length) node.choices.splice(index, 1)
  return node
}

export function dialogueRelationshipEntries(
  choice: DialogueChoiceDefinition,
): Array<[string, number]> {
  return Object.entries(choice.relationship_changes)
    .sort(([left], [right]) => left.localeCompare(right))
}

export function availableDialogueRelationshipCharacters(
  choice: DialogueChoiceDefinition,
  characters: readonly DialogueCharacterIdentity[],
): DialogueCharacterIdentity[] {
  return characters.filter((character) => !Object.hasOwn(choice.relationship_changes, character.id))
}

export function addDialogueRelationship(
  source: DialogueChoiceDefinition,
  characterId: string,
  delta = 0.1,
): DialogueChoiceDefinition {
  const choice = cloneDialogueChoice(source)
  if (characterId && !Object.hasOwn(choice.relationship_changes, characterId)) {
    choice.relationship_changes[characterId] = delta
  }
  return choice
}

export function removeDialogueRelationship(
  source: DialogueChoiceDefinition,
  characterId: string,
): DialogueChoiceDefinition {
  const choice = cloneDialogueChoice(source)
  delete choice.relationship_changes[characterId]
  return choice
}

export function renameDialogueRelationship(
  source: DialogueChoiceDefinition,
  before: string,
  after: string,
): DialogueChoiceDefinition {
  const choice = cloneDialogueChoice(source)
  if (!after || after === before || !Object.hasOwn(choice.relationship_changes, before)
    || Object.hasOwn(choice.relationship_changes, after)) return choice
  const delta = choice.relationship_changes[before]
  delete choice.relationship_changes[before]
  choice.relationship_changes[after] = delta
  return choice
}

export function setDialogueRelationshipDelta(
  source: DialogueChoiceDefinition,
  characterId: string,
  value: unknown,
): DialogueChoiceDefinition {
  const choice = cloneDialogueChoice(source)
  if (Object.hasOwn(choice.relationship_changes, characterId)) {
    choice.relationship_changes[characterId] = Number(value)
  }
  return choice
}

export function mergeDialogueCharacters(
  catalog: DialogueAuthoringCatalogSnapshot,
  projectCharacters: readonly DialogueCharacterIdentity[],
): DialogueCharacterIdentity[] {
  const byId = new Map<string, DialogueCharacterIdentity>()
  for (const character of [...deriveDialogueCharacters(catalog), ...projectCharacters]) {
    byId.set(character.id, { ...character })
  }
  return [...byId.values()].sort((left, right) => left.name.localeCompare(right.name) || left.id.localeCompare(right.id))
}

export function deriveDialogueCharacters(
  catalog: DialogueAuthoringCatalogSnapshot,
): DialogueCharacterIdentity[] {
  const ids = new Set<string>()
  for (const dialogue of catalog.dialogues) {
    for (const node of Object.values(dialogue.nodes)) {
      if (node.speaker_id) ids.add(node.speaker_id)
      node.choices.forEach((choice) => {
        Object.keys(choice.relationship_changes).forEach((id) => ids.add(id))
      })
    }
  }
  return [...ids].sort().map((id) => ({ id, name: dialogueTitleFromId(id) }))
}

export function dialogueTitleFromId(id: string): string {
  return id.split(/[_-]/)
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(' ')
}

function createDialogueChoice(text: string, targetNodeId: string): DialogueChoiceDefinition {
  return {
    text,
    next_node_id: targetNodeId,
    relationship_changes: {},
    condition: null,
  }
}

function cloneDialogueNode(node: DialogueNodeDefinition): DialogueNodeDefinition {
  return {
    ...node,
    choices: node.choices.map(cloneDialogueChoice),
  }
}

function cloneDialogueChoice(choice: DialogueChoiceDefinition): DialogueChoiceDefinition {
  return {
    ...choice,
    relationship_changes: { ...choice.relationship_changes },
  }
}

function deleteError(
  dialogue: DialogueDefinition,
  nodeId: string,
  error: DialogueNodeDeleteError,
): DialogueNodeDeleteResult {
  return {
    dialogue,
    selected_node_id: nodeId,
    references: [],
    changed: false,
    error,
  }
}

function portableIdKey(value: string): string {
  return value.trim().toLowerCase()
}
