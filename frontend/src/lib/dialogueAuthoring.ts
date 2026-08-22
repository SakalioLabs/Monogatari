import type { StoryContentAccessEntry } from './storyAccess'
import { hasDialogueIdCollision } from './dialogueGraphEditing'
import {
  loadBrowserDialogueDrafts,
  loadStoryDialogues,
  loadStoryEndings,
  saveBrowserDialogueDrafts,
  type DialogueDefinition as BrowserDialogueDefinition,
  type StoryDialogueInfo,
  type WebDialogueChoice,
  type WebDialogueFreeTalk,
  type WebDialogueNode,
  type WebDialogueResponseGeneration,
} from './storyContent'
import { hasTauriRuntime, invokeCommand } from './tauri'

export const DIALOGUE_AUTHORING_CATALOG_SCHEMA = 'monogatari-dialogue-authoring-catalog/v1'

export interface DialogueChoiceDefinition {
  text: string
  next_node_id: string
  relationship_changes: Record<string, number>
  condition: string | null
}

export interface DialogueNodeDefinition {
  speaker_id: string | null
  scene_id?: string | null
  text: string
  next_node_id: string | null
  choices: DialogueChoiceDefinition[]
  condition: string | null
  script: string | null
  emotion: string | null
  use_llm: boolean
  llm_prompt: string | null
  llm_system_prompt: string | null
  response_generation: DialogueResponseGenerationDefinition | null
  free_talk: DialogueFreeTalkDefinition | null
  is_ending: boolean
  ending_type: string | null
}

export interface DialogueResponseGenerationDefinition {
  character_id: string | null
  context: string
  grounding_markers: string[]
  forbidden_markers: string[]
  max_characters: number
  max_sentences: number
}

export interface DialogueFreeTalkDefinition {
  character_id: string
  context: string
  title: string | null
  opening_text: string | null
  fallback_text: string
  max_turns: number
  max_characters: number
}

export interface DialogueDefinition {
  id: string
  title: string
  description: string | null
  start_node_id: string
  nodes: Record<string, DialogueNodeDefinition>
  variables: Record<string, unknown>
}

export interface DialogueAuthoringEntry extends DialogueDefinition {
  source_path: string
  content_fingerprint: string
  access: StoryContentAccessEntry
}

export interface DialogueAuthoringCatalogSnapshot {
  schema: string
  catalog_fingerprint: string
  dialogue_count: number
  node_count: number
  choice_count: number
  llm_node_count: number
  dialogues: DialogueAuthoringEntry[]
}

const PORTABLE_ID = /^[A-Za-z0-9_.-]{1,128}$/

export async function loadDialogueAuthoringCatalog(): Promise<DialogueAuthoringCatalogSnapshot> {
  if (hasTauriRuntime()) {
    const snapshot = await invokeCommand<DialogueAuthoringCatalogSnapshot>('get_dialogue_authoring_catalog')
    return normalizeSnapshot(snapshot)
  }
  return browserCatalogSnapshot()
}

export async function saveDialogueDefinition(
  dialogue: DialogueDefinition,
  originalDialogueId: string | null,
  expectedCatalogFingerprint: string,
  characterIds: string[] = [],
  sceneIds: string[] = [],
): Promise<DialogueAuthoringCatalogSnapshot> {
  const normalized = normalizeDialogueDefinition(dialogue)
  const issues = validateDialogueDefinition(normalized, characterIds, sceneIds)
  if (issues.length > 0) throw new Error(issues[0])
  if (hasTauriRuntime()) {
    const snapshot = await invokeCommand<DialogueAuthoringCatalogSnapshot>('save_dialogue_definition', {
      dialogue: normalized,
      originalDialogueId,
      expectedCatalogFingerprint,
    })
    return normalizeSnapshot(snapshot)
  }

  const current = await browserCatalogSnapshot()
  ensureExpectedFingerprint(current, expectedCatalogFingerprint)
  const definitions = current.dialogues.map(dialogueDefinition)
  if (originalDialogueId) {
    if (originalDialogueId !== normalized.id) {
      throw new Error('Dialogue ids are immutable after creation. Duplicate the dialogue to use a new id.')
    }
    const existingIndex = definitions.findIndex((item) => item.id === originalDialogueId)
    if (existingIndex < 0) throw new Error(`Dialogue "${originalDialogueId}" no longer exists. Reload first.`)
    definitions.splice(existingIndex, 1, normalized)
  } else {
    if (hasDialogueIdCollision(definitions.map((item) => item.id), normalized.id)) {
      throw new Error(`Dialogue "${normalized.id}" already exists.`)
    }
    definitions.push(normalized)
  }
  definitions.sort((left, right) => left.id.localeCompare(right.id))
  saveBrowserDialogueDrafts(definitions.map(browserDialogueDefinition))
  return browserCatalogSnapshot()
}

export async function deleteDialogueDefinition(
  dialogueId: string,
  expectedCatalogFingerprint: string,
): Promise<DialogueAuthoringCatalogSnapshot> {
  if (hasTauriRuntime()) {
    const snapshot = await invokeCommand<DialogueAuthoringCatalogSnapshot>('delete_dialogue_definition', {
      dialogueId,
      expectedCatalogFingerprint,
    })
    return normalizeSnapshot(snapshot)
  }

  const current = await browserCatalogSnapshot()
  ensureExpectedFingerprint(current, expectedCatalogFingerprint)
  const target = current.dialogues.find((dialogue) => dialogue.id === dialogueId)
  if (!target) throw new Error(`Dialogue "${dialogueId}" does not exist.`)
  const references = target.access.unlock_event_ids.map((eventId) => `event:${eventId}`)
  const endings = await loadStoryEndings()
  references.push(...endings.filter((ending) => ending.dialogue_id === dialogueId).map((ending) => `ending:${ending.id}`))
  if (references.length > 0) {
    throw new Error(`Dialogue "${dialogueId}" is still referenced by: ${references.join(', ')}.`)
  }
  saveBrowserDialogueDrafts(
    current.dialogues.filter((dialogue) => dialogue.id !== dialogueId)
      .map((dialogue) => browserDialogueDefinition(dialogueDefinition(dialogue))),
  )
  return browserCatalogSnapshot()
}

export function normalizeDialogueDefinition(dialogue: DialogueDefinition): DialogueDefinition {
  const normalizedNodes = Object.entries(dialogue.nodes)
    .map(([nodeId, node]) => [nodeId.trim(), normalizeNode(node)] as const)
    .sort(([left], [right]) => left.localeCompare(right))
  const normalizedNodeIds = normalizedNodes.map(([nodeId]) => nodeId)
  if (new Set(normalizedNodeIds).size !== normalizedNodeIds.length) {
    throw new Error('Dialogue node IDs collide after whitespace normalization.')
  }
  const nodes = Object.fromEntries(normalizedNodes)
  return {
    id: dialogue.id.trim(),
    title: dialogue.title.trim(),
    description: optionalText(dialogue.description),
    start_node_id: dialogue.start_node_id.trim(),
    nodes,
    variables: sortObject(dialogue.variables),
  }
}

export function validateDialogueDefinition(
  dialogue: DialogueDefinition,
  characterIds: string[] = [],
  sceneIds: string[] = [],
): string[] {
  const issues: string[] = []
  const knownCharacters = new Set(characterIds)
  const knownScenes = new Set(sceneIds)
  if (!portableId(dialogue.id)) issues.push('Dialogue ID must be a portable 1-128 character id.')
  validateText(issues, 'Title', dialogue.title, 1, 256)
  if (dialogue.description !== null) validateText(issues, 'Description', dialogue.description, 1, 2048)
  const nodeIds = Object.keys(dialogue.nodes)
  const nodeSet = new Set(nodeIds)
  if (nodeIds.length < 1 || nodeIds.length > 2048) issues.push('A dialogue must contain 1-2048 nodes.')
  if (!portableId(dialogue.start_node_id) || !nodeSet.has(dialogue.start_node_id)) {
    issues.push(`Start node "${dialogue.start_node_id}" does not exist.`)
  }
  if (Object.keys(dialogue.variables).length > 512) issues.push('A dialogue can contain at most 512 variables.')
  for (const variableId of Object.keys(dialogue.variables)) {
    if (!portableId(variableId)) issues.push(`Variable "${variableId}" is not a portable state key.`)
  }

  for (const [nodeId, node] of Object.entries(dialogue.nodes)) {
    if (!portableId(nodeId)) issues.push(`Node ID "${nodeId}" is not portable.`)
    validateText(issues, `Node "${nodeId}" text`, node.text, 1, 16384)
    if (node.speaker_id && !portableId(node.speaker_id)) {
      issues.push(`Node "${nodeId}" speaker "${node.speaker_id}" is not portable.`)
    } else if (node.speaker_id && knownCharacters.size > 0 && !knownCharacters.has(node.speaker_id)) {
      issues.push(`Node "${nodeId}" references unknown speaker "${node.speaker_id}".`)
    }
    if (node.scene_id && !portableId(node.scene_id)) {
      issues.push(`Node "${nodeId}" scene "${node.scene_id}" is not portable.`)
    } else if (node.scene_id && knownScenes.size > 0 && !knownScenes.has(node.scene_id)) {
      issues.push(`Node "${nodeId}" references unknown scene "${node.scene_id}".`)
    }
    if (node.emotion !== null) validateText(issues, `Node "${nodeId}" emotion`, node.emotion, 1, 64)
    if (node.next_node_id && node.choices.length > 0) {
      issues.push(`Node "${nodeId}" cannot combine a linear target with choices.`)
    }
    if (node.next_node_id && !nodeSet.has(node.next_node_id)) {
      issues.push(`Node "${nodeId}" targets missing node "${node.next_node_id}".`)
    }
    if (node.choices.length > 32) issues.push(`Node "${nodeId}" can contain at most 32 choices.`)
    if (node.is_ending && (node.next_node_id !== null || node.choices.length > 0)) {
      issues.push(`Ending node "${nodeId}" cannot have outgoing transitions.`)
    }
    if (node.ending_type !== null && !node.is_ending) {
      issues.push(`Node "${nodeId}" needs End mode before setting an ending type.`)
    }
    if (node.ending_type !== null) validateText(issues, `Node "${nodeId}" ending type`, node.ending_type, 1, 64)
    if (node.condition !== null && node.next_node_id === null) {
      issues.push(`Conditional node "${nodeId}" requires a linear fallback target.`)
    }
    validateSource(issues, `Node "${nodeId}" condition`, node.condition, 2000)
    validateSource(issues, `Node "${nodeId}" script`, node.script, 20000)
    if (node.use_llm && node.response_generation) {
      issues.push(`Node "${nodeId}" cannot combine a legacy LLM prompt with a controlled character reply.`)
    }
    if (node.use_llm && !node.llm_prompt) issues.push(`Node "${nodeId}" enables LLM generation without a prompt.`)
    validateSource(issues, `Node "${nodeId}" LLM prompt`, node.llm_prompt, 20000)
    validateSource(issues, `Node "${nodeId}" LLM system prompt`, node.llm_system_prompt, 20000)
    validateResponseGeneration(issues, nodeId, node, knownCharacters)
    validateFreeTalk(issues, nodeId, node.free_talk, knownCharacters)

    node.choices.forEach((choice, choiceIndex) => {
      const label = `Node "${nodeId}" choice ${choiceIndex + 1}`
      validateText(issues, `${label} text`, choice.text, 1, 2048)
      if (!nodeSet.has(choice.next_node_id)) issues.push(`${label} targets missing node "${choice.next_node_id}".`)
      validateSource(issues, `${label} condition`, choice.condition, 2000)
      if (Object.keys(choice.relationship_changes).length > 128) {
        issues.push(`${label} can contain at most 128 relationship changes.`)
      }
      for (const [characterId, delta] of Object.entries(choice.relationship_changes)) {
        if (!portableId(characterId)) {
          issues.push(`${label} relationship character "${characterId}" is not portable.`)
        } else if (knownCharacters.size > 0 && !knownCharacters.has(characterId)) {
          issues.push(`${label} changes unknown character "${characterId}".`)
        }
        if (!Number.isFinite(delta) || delta < -1 || delta > 1) {
          issues.push(`${label} relationship delta for "${characterId}" must be between -1 and 1.`)
        }
      }
    })
  }

  if (nodeSet.has(dialogue.start_node_id)) {
    const reachable = new Set<string>()
    const queue = [dialogue.start_node_id]
    while (queue.length > 0) {
      const nodeId = queue.shift()!
      if (reachable.has(nodeId)) continue
      reachable.add(nodeId)
      const node = dialogue.nodes[nodeId]
      if (!node) continue
      if (node.next_node_id) queue.push(node.next_node_id)
      node.choices.forEach((choice) => queue.push(choice.next_node_id))
    }
    const unreachable = nodeIds.filter((nodeId) => !reachable.has(nodeId))
    if (unreachable.length > 0) issues.push(`Unreachable nodes: ${unreachable.join(', ')}.`)
  }
  return [...new Set(issues)]
}

function validateResponseGeneration(
  issues: string[],
  nodeId: string,
  node: DialogueNodeDefinition,
  knownCharacters: Set<string>,
): void {
  const generation = node.response_generation
  if (!generation) return
  const characterId = generation.character_id || node.speaker_id
  if (!characterId) {
    issues.push(`Node "${nodeId}" controlled character reply needs a speaker.`)
  } else if (!portableId(characterId)) {
    issues.push(`Node "${nodeId}" controlled reply character "${characterId}" is not portable.`)
  } else if (knownCharacters.size > 0 && !knownCharacters.has(characterId)) {
    issues.push(`Node "${nodeId}" controlled reply references unknown character "${characterId}".`)
  }
  if (node.speaker_id && generation.character_id && node.speaker_id !== generation.character_id) {
    issues.push(`Node "${nodeId}" controlled reply character must match its speaker.`)
  }
  validateText(issues, `Node "${nodeId}" controlled reply context`, generation.context, 1, 4096)
  validateGenerationMarkers(issues, `Node "${nodeId}" grounding markers`, generation.grounding_markers)
  validateGenerationMarkers(issues, `Node "${nodeId}" forbidden markers`, generation.forbidden_markers)
  if (!Number.isInteger(generation.max_characters) || generation.max_characters < 40 || generation.max_characters > 600) {
    issues.push(`Node "${nodeId}" controlled reply maximum length must be 40-600 characters.`)
  }
  if (!Number.isInteger(generation.max_sentences) || generation.max_sentences < 1 || generation.max_sentences > 6) {
    issues.push(`Node "${nodeId}" controlled reply sentence limit must be 1-6.`)
  }
}

function validateFreeTalk(
  issues: string[],
  nodeId: string,
  freeTalk: DialogueFreeTalkDefinition | null,
  knownCharacters: Set<string>,
): void {
  if (!freeTalk) return
  if (!portableId(freeTalk.character_id)) {
    issues.push(`Node "${nodeId}" free talk character "${freeTalk.character_id}" is not portable.`)
  } else if (knownCharacters.size > 0 && !knownCharacters.has(freeTalk.character_id)) {
    issues.push(`Node "${nodeId}" free talk references unknown character "${freeTalk.character_id}".`)
  }
  validateText(issues, `Node "${nodeId}" free talk context`, freeTalk.context, 1, 4096)
  if (freeTalk.title !== null) validateText(issues, `Node "${nodeId}" free talk title`, freeTalk.title, 1, 128)
  if (freeTalk.opening_text !== null) validateText(issues, `Node "${nodeId}" free talk opening`, freeTalk.opening_text, 1, 2048)
  validateText(issues, `Node "${nodeId}" free talk fallback`, freeTalk.fallback_text, 1, 2048)
  if (!Number.isInteger(freeTalk.max_turns) || freeTalk.max_turns < 1 || freeTalk.max_turns > 12) {
    issues.push(`Node "${nodeId}" free talk turns must be 1-12.`)
  }
  if (!Number.isInteger(freeTalk.max_characters) || freeTalk.max_characters < 40 || freeTalk.max_characters > 600) {
    issues.push(`Node "${nodeId}" free talk maximum length must be 40-600 characters.`)
  }
}

function validateGenerationMarkers(issues: string[], label: string, markers: string[]): void {
  if (markers.length > 16) issues.push(`${label} can contain at most 16 entries.`)
  const seen = new Set<string>()
  markers.forEach((marker) => {
    validateText(issues, label, marker, 1, 128)
    const key = marker.toLocaleLowerCase()
    if (seen.has(key)) issues.push(`${label} cannot contain duplicate entries.`)
    seen.add(key)
  })
}

async function browserCatalogSnapshot(): Promise<DialogueAuthoringCatalogSnapshot> {
  const draftActive = loadBrowserDialogueDrafts() !== null
  const dialogues = await loadStoryDialogues()
  const entries = dialogues.map((dialogue) => authoringEntry(dialogue, draftActive))
    .sort((left, right) => left.id.localeCompare(right.id))
  return snapshotFromEntries(entries)
}

function normalizeSnapshot(snapshot: DialogueAuthoringCatalogSnapshot): DialogueAuthoringCatalogSnapshot {
  const dialogues = snapshot.dialogues.map((entry) => ({
    ...normalizeDialogueDefinition(entry),
    source_path: entry.source_path,
    content_fingerprint: entry.content_fingerprint,
    access: entry.access,
  }))
  return { ...snapshot, dialogues }
}

function snapshotFromEntries(entries: DialogueAuthoringEntry[]): DialogueAuthoringCatalogSnapshot {
  const nodeCount = entries.reduce((count, dialogue) => count + Object.keys(dialogue.nodes).length, 0)
  const choiceCount = entries.reduce((count, dialogue) => count
    + Object.values(dialogue.nodes).reduce((nodeCount, node) => nodeCount + node.choices.length, 0), 0)
  const llmNodeCount = entries.reduce((count, dialogue) => count
    + Object.values(dialogue.nodes).filter((node) => node.use_llm || node.response_generation !== null).length, 0)
  return {
    schema: DIALOGUE_AUTHORING_CATALOG_SCHEMA,
    catalog_fingerprint: browserFingerprint(entries.map((dialogue) => ({
      source_path: dialogue.source_path,
      dialogue: dialogueDefinition(dialogue),
    }))),
    dialogue_count: entries.length,
    node_count: nodeCount,
    choice_count: choiceCount,
    llm_node_count: llmNodeCount,
    dialogues: entries,
  }
}

function authoringEntry(dialogue: StoryDialogueInfo, draftActive: boolean): DialogueAuthoringEntry {
  const definition = normalizeDialogueDefinition({
    id: dialogue.id,
    title: dialogue.title,
    description: dialogue.description ?? null,
    start_node_id: dialogue.start_node_id,
    nodes: Object.fromEntries(Object.entries(dialogue.nodes || {}).map(([nodeId, node]) => [nodeId, nodeFromWeb(node)])),
    variables: dialogue.variables || {},
  })
  return {
    ...definition,
    source_path: `${draftActive ? 'browser-draft/' : ''}dialogue/${dialogue.id}.json`,
    content_fingerprint: browserFingerprint(definition),
    access: dialogue.access,
  }
}

function nodeFromWeb(node: WebDialogueNode): DialogueNodeDefinition {
  return {
    speaker_id: node.speaker_id || null,
    scene_id: node.scene_id || null,
    text: node.text || '',
    next_node_id: node.next_node_id || null,
    choices: (node.choices || []).map(choiceFromWeb),
    condition: node.condition || null,
    script: node.script || null,
    emotion: node.emotion || null,
    use_llm: Boolean(node.use_llm),
    llm_prompt: node.llm_prompt || null,
    llm_system_prompt: node.llm_system_prompt || null,
    response_generation: responseGenerationFromWeb(node.response_generation),
    free_talk: freeTalkFromWeb(node.free_talk),
    is_ending: Boolean(node.is_ending),
    ending_type: node.ending_type || null,
  }
}

function responseGenerationFromWeb(
  value: WebDialogueResponseGeneration | null | undefined,
): DialogueResponseGenerationDefinition | null {
  if (!value) return null
  return {
    character_id: optionalText(value.character_id),
    context: value.context || '',
    grounding_markers: normalizeMarkerList(value.grounding_markers),
    forbidden_markers: normalizeMarkerList(value.forbidden_markers),
    max_characters: integerValue(value.max_characters, 240),
    max_sentences: integerValue(value.max_sentences, 3),
  }
}

function freeTalkFromWeb(value: WebDialogueFreeTalk | null | undefined): DialogueFreeTalkDefinition | null {
  if (!value) return null
  return {
    character_id: value.character_id || '',
    context: value.context || '',
    title: optionalText(value.title),
    opening_text: optionalText(value.opening_text),
    fallback_text: value.fallback_text || '',
    max_turns: integerValue(value.max_turns, 4),
    max_characters: integerValue(value.max_characters, 240),
  }
}

function choiceFromWeb(choice: WebDialogueChoice): DialogueChoiceDefinition {
  return {
    text: choice.text || '',
    next_node_id: choice.next_node_id || '',
    relationship_changes: { ...(choice.relationship_changes || {}) },
    condition: choice.condition || null,
  }
}

function normalizeNode(node: DialogueNodeDefinition): DialogueNodeDefinition {
  return {
    speaker_id: optionalText(node.speaker_id),
    scene_id: optionalText(node.scene_id),
    text: node.text.trim(),
    next_node_id: optionalText(node.next_node_id),
    choices: node.choices.map((choice) => ({
      text: choice.text.trim(),
      next_node_id: choice.next_node_id.trim(),
      relationship_changes: sortObject(choice.relationship_changes),
      condition: optionalText(choice.condition),
    })),
    condition: optionalText(node.condition),
    script: optionalText(node.script),
    emotion: optionalText(node.emotion),
    use_llm: Boolean(node.use_llm),
    llm_prompt: optionalText(node.llm_prompt),
    llm_system_prompt: optionalText(node.llm_system_prompt),
    response_generation: normalizeResponseGeneration(node.response_generation),
    free_talk: normalizeFreeTalk(node.free_talk),
    is_ending: Boolean(node.is_ending),
    ending_type: optionalText(node.ending_type),
  }
}

function normalizeResponseGeneration(
  value: DialogueResponseGenerationDefinition | null,
): DialogueResponseGenerationDefinition | null {
  if (!value) return null
  return {
    character_id: optionalText(value.character_id),
    context: value.context.trim(),
    grounding_markers: normalizeMarkerList(value.grounding_markers),
    forbidden_markers: normalizeMarkerList(value.forbidden_markers),
    max_characters: integerValue(value.max_characters, 240),
    max_sentences: integerValue(value.max_sentences, 3),
  }
}

function normalizeFreeTalk(value: DialogueFreeTalkDefinition | null): DialogueFreeTalkDefinition | null {
  if (!value) return null
  return {
    character_id: value.character_id.trim(),
    context: value.context.trim(),
    title: optionalText(value.title),
    opening_text: optionalText(value.opening_text),
    fallback_text: value.fallback_text.trim(),
    max_turns: integerValue(value.max_turns, 4),
    max_characters: integerValue(value.max_characters, 240),
  }
}

function dialogueDefinition(dialogue: DialogueDefinition): DialogueDefinition {
  return normalizeDialogueDefinition(dialogue)
}

function browserDialogueDefinition(dialogue: DialogueDefinition): BrowserDialogueDefinition {
  return {
    ...dialogue,
    nodes: dialogue.nodes,
  }
}

function optionalText(value: string | null | undefined): string | null {
  const normalized = value?.trim() || ''
  return normalized || null
}

function normalizeMarkerList(value: unknown): string[] {
  if (!Array.isArray(value)) return []
  const seen = new Set<string>()
  return value.flatMap((entry) => {
    const marker = typeof entry === 'string' ? entry.trim() : ''
    if (!marker || seen.has(marker.toLocaleLowerCase())) return []
    seen.add(marker.toLocaleLowerCase())
    return [marker]
  })
}

function integerValue(value: unknown, fallback: number): number {
  const number = Number(value)
  return Number.isFinite(number) ? Math.round(number) : fallback
}

function portableId(value: string): boolean {
  return PORTABLE_ID.test(value) && value.trim() === value
}

function validateText(issues: string[], label: string, value: string, min: number, max: number): void {
  const length = [...value].length
  if (length < min || length > max || /[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f]/.test(value)) {
    issues.push(`${label} must contain ${min}-${max} supported characters.`)
  }
}

function validateSource(issues: string[], label: string, value: string | null, max: number): void {
  if (value === null) return
  validateText(issues, label, value, 1, max)
}

function sortObject<T>(value: Record<string, T>): Record<string, T> {
  return Object.fromEntries(Object.entries(value).sort(([left], [right]) => left.localeCompare(right)))
}

function ensureExpectedFingerprint(current: DialogueAuthoringCatalogSnapshot, expected: string): void {
  if (current.catalog_fingerprint !== expected) {
    throw new Error('Dialogue catalog changed since it was opened. Reload before saving.')
  }
}

function browserFingerprint(value: unknown): string {
  const text = JSON.stringify(value)
  let left = 0x811c9dc5
  let right = 0x9e3779b9
  for (let index = 0; index < text.length; index += 1) {
    const code = text.charCodeAt(index)
    left = Math.imul(left ^ code, 0x01000193) >>> 0
    right = Math.imul(right ^ (code + index), 0x85ebca6b) >>> 0
  }
  return `browser-${left.toString(16).padStart(8, '0')}${right.toString(16).padStart(8, '0')}`
}
