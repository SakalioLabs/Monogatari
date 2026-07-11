import type { StoryDialogueInfo, WebDialogueNode } from './storyContent'

export interface DialogueState {
  is_active: boolean
  speaker: string | null
  text: string
  emotion: string | null
  choices: { index: number; text: string }[]
  live2d_expression: string | null
}

export interface BrowserDialogueTransition {
  node_id: string | null
  state: DialogueState
  completed: boolean
  blocked_reason: 'choice_required' | null
  relationship_changes: Record<string, number>
}

export type StoryPlaytestErrorCode =
  | 'dialogue_start_missing'
  | 'dialogue_node_missing'
  | 'choice_index_invalid'
  | 'choice_target_missing'
  | 'relationship_target_missing'
  | 'next_target_missing'

export class StoryPlaytestError extends Error {
  constructor(
    public readonly code: StoryPlaytestErrorCode,
    message: string,
    public readonly node_id: string | null = null,
    public readonly target_node_id: string | null = null,
    public readonly choice_index: number | null = null,
  ) {
    super(message)
    this.name = 'StoryPlaytestError'
  }
}

export function applyBrowserRelationshipChanges<T extends {
  id: string
  relationships?: Record<string, number>
}>(characters: readonly T[], changes: Record<string, number>): T[] {
  const characterIds = new Set(characters.map((character) => character.id))
  const missingCharacterId = Object.keys(changes).find((characterId) => !characterIds.has(characterId))
  if (missingCharacterId) {
    throw new StoryPlaytestError(
      'relationship_target_missing',
      `Dialogue choice changes unknown character "${missingCharacterId}".`,
      null,
      missingCharacterId,
    )
  }
  return characters.map((character) => {
    const delta = Number(changes[character.id])
    if (!Number.isFinite(delta)) return character
    const relationships = { ...(character.relationships || {}) }
    relationships.player = Math.min(1, Math.max(-1, (relationships.player || 0) + delta))
    return { ...character, relationships }
  })
}

export function startBrowserDialogue(dialogue: StoryDialogueInfo): BrowserDialogueTransition {
  const startNodeId = dialogue.start_node_id.trim()
  if (!startNodeId) {
    throw new StoryPlaytestError(
      'dialogue_start_missing',
      `Dialogue "${dialogue.id}" does not define a start node.`,
    )
  }
  return transitionToNode(dialogue, startNodeId)
}

export function selectBrowserDialogueChoice(
  dialogue: StoryDialogueInfo,
  nodeId: string | null,
  choiceIndex: number,
): BrowserDialogueTransition {
  const node = requireDialogueNode(dialogue, nodeId)
  if (!Number.isInteger(choiceIndex) || choiceIndex < 0 || choiceIndex >= (node.choices?.length ?? 0)) {
    throw new StoryPlaytestError(
      'choice_index_invalid',
      `Dialogue node "${nodeId}" does not have choice ${choiceIndex}.`,
      nodeId,
      null,
      choiceIndex,
    )
  }
  const choice = node.choices![choiceIndex]
  const targetNodeId = choice.next_node_id.trim()
  if (!targetNodeId || !dialogue.nodes?.[targetNodeId]) {
    throw new StoryPlaytestError(
      'choice_target_missing',
      `Dialogue node "${nodeId}" choice ${choiceIndex + 1} targets missing node "${targetNodeId || '<empty>'}".`,
      nodeId,
      targetNodeId || null,
      choiceIndex,
    )
  }
  return {
    ...transitionToNode(dialogue, targetNodeId),
    relationship_changes: normalizeRelationshipChanges(choice.relationship_changes),
  }
}

export function advanceBrowserDialogue(
  dialogue: StoryDialogueInfo,
  nodeId: string | null,
): BrowserDialogueTransition {
  const node = requireDialogueNode(dialogue, nodeId)
  if ((node.choices?.length ?? 0) > 0) {
    return {
      node_id: nodeId,
      state: dialogueState(node),
      completed: false,
      blocked_reason: 'choice_required',
      relationship_changes: {},
    }
  }

  const targetNodeId = node.next_node_id?.trim() || ''
  if (!targetNodeId) return completedTransition()
  if (!dialogue.nodes?.[targetNodeId]) {
    throw new StoryPlaytestError(
      'next_target_missing',
      `Dialogue node "${nodeId}" targets missing node "${targetNodeId}".`,
      nodeId,
      targetNodeId,
    )
  }
  return transitionToNode(dialogue, targetNodeId)
}

export function inactiveDialogueState(): DialogueState {
  return {
    is_active: false,
    speaker: null,
    text: '',
    emotion: null,
    choices: [],
    live2d_expression: null,
  }
}

function transitionToNode(dialogue: StoryDialogueInfo, nodeId: string): BrowserDialogueTransition {
  const node = requireDialogueNode(dialogue, nodeId)
  return {
    node_id: nodeId,
    state: dialogueState(node),
    completed: false,
    blocked_reason: null,
    relationship_changes: {},
  }
}

function requireDialogueNode(dialogue: StoryDialogueInfo, nodeId: string | null): WebDialogueNode {
  const normalizedNodeId = nodeId?.trim() || ''
  const node = normalizedNodeId ? dialogue.nodes?.[normalizedNodeId] : null
  if (!node) {
    throw new StoryPlaytestError(
      normalizedNodeId ? 'dialogue_node_missing' : 'dialogue_start_missing',
      normalizedNodeId
        ? `Dialogue "${dialogue.id}" is missing node "${normalizedNodeId}".`
        : `Dialogue "${dialogue.id}" does not define an active node.`,
      normalizedNodeId || null,
    )
  }
  return node
}

function dialogueState(node: WebDialogueNode): DialogueState {
  return {
    is_active: true,
    speaker: node.speaker_id || null,
    text: node.text,
    emotion: node.emotion || null,
    choices: (node.choices || []).map((choice, index) => ({ index, text: choice.text })),
    live2d_expression: node.emotion || null,
  }
}

function completedTransition(): BrowserDialogueTransition {
  return {
    node_id: null,
    state: inactiveDialogueState(),
    completed: true,
    blocked_reason: null,
    relationship_changes: {},
  }
}

function normalizeRelationshipChanges(value: Record<string, number> | undefined): Record<string, number> {
  return Object.fromEntries(Object.entries(value || {})
    .map(([characterId, delta]) => [characterId.trim(), Number(delta)] as const)
    .filter(([characterId, delta]) => characterId.length > 0 && Number.isFinite(delta)))
}
