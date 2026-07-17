import { evaluateLocalCondition, type LocalConditionValue } from './localCondition'
import type { StoryDialogueInfo, WebDialogueNode } from './storyContent'

export interface DialogueState {
  is_active: boolean
  speaker: string | null
  scene_id: string | null
  text: string
  emotion: string | null
  choices: { index: number; text: string }[]
  live2d_expression: string | null
}

export interface BrowserDialogueRuntime {
  node_id: string | null
  variables: Record<string, LocalConditionValue>
  flags: Record<string, boolean>
}

export interface BrowserDialogueTransition {
  runtime: BrowserDialogueRuntime
  state: DialogueState
  completed: boolean
  blocked_reason: 'choice_required' | null
  relationship_changes: Record<string, number>
}

export type StoryPlaytestErrorCode =
  | 'dialogue_start_missing'
  | 'dialogue_node_missing'
  | 'choice_index_invalid'
  | 'choice_unavailable'
  | 'choice_target_missing'
  | 'relationship_target_missing'
  | 'next_target_missing'
  | 'node_condition_blocked'
  | 'condition_cycle'
  | 'condition_unsupported'
  | 'script_unsupported'

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

export function startBrowserDialogue(
  dialogue: StoryDialogueInfo,
  initialFlags: Record<string, boolean> = {},
  previewNodeId?: string | null,
): BrowserDialogueTransition {
  const startNodeId = previewNodeId?.trim() || dialogue.start_node_id.trim()
  if (!startNodeId) {
    throw new StoryPlaytestError(
      'dialogue_start_missing',
      `Dialogue "${dialogue.id}" does not define a start node.`,
    )
  }
  return transitionToNode(dialogue, {
    node_id: startNodeId,
    variables: normalizeInitialVariables(dialogue.variables),
    flags: { ...initialFlags },
  })
}

export function selectBrowserDialogueChoice(
  dialogue: StoryDialogueInfo,
  runtime: BrowserDialogueRuntime,
  choiceIndex: number,
): BrowserDialogueTransition {
  const node = requireDialogueNode(dialogue, runtime.node_id)
  if (!Number.isInteger(choiceIndex) || choiceIndex < 0 || choiceIndex >= (node.choices?.length ?? 0)) {
    throw new StoryPlaytestError(
      'choice_index_invalid',
      `Dialogue node "${runtime.node_id}" does not have choice ${choiceIndex}.`,
      runtime.node_id,
      null,
      choiceIndex,
    )
  }
  const choice = node.choices![choiceIndex]
  if (!conditionMatches(choice.condition, runtime, runtime.node_id, choiceIndex)) {
    throw new StoryPlaytestError(
      'choice_unavailable',
      `Dialogue node "${runtime.node_id}" choice ${choiceIndex + 1} is not available.`,
      runtime.node_id,
      null,
      choiceIndex,
    )
  }
  const targetNodeId = choice.next_node_id.trim()
  if (!targetNodeId || !dialogue.nodes?.[targetNodeId]) {
    throw new StoryPlaytestError(
      'choice_target_missing',
      `Dialogue node "${runtime.node_id}" choice ${choiceIndex + 1} targets missing node "${targetNodeId || '<empty>'}".`,
      runtime.node_id,
      targetNodeId || null,
      choiceIndex,
    )
  }
  return {
    ...transitionToNode(dialogue, { ...runtime, node_id: targetNodeId }),
    relationship_changes: normalizeRelationshipChanges(choice.relationship_changes),
  }
}

export function advanceBrowserDialogue(
  dialogue: StoryDialogueInfo,
  runtime: BrowserDialogueRuntime,
): BrowserDialogueTransition {
  const node = requireDialogueNode(dialogue, runtime.node_id)
  if (availableChoices(node, runtime).length > 0) {
    return {
      runtime,
      state: dialogueState(node, runtime),
      completed: false,
      blocked_reason: 'choice_required',
      relationship_changes: {},
    }
  }

  const targetNodeId = node.next_node_id?.trim() || ''
  if (!targetNodeId) return completedTransition(runtime)
  if (!dialogue.nodes?.[targetNodeId]) {
    throw new StoryPlaytestError(
      'next_target_missing',
      `Dialogue node "${runtime.node_id}" targets missing node "${targetNodeId}".`,
      runtime.node_id,
      targetNodeId,
    )
  }
  return transitionToNode(dialogue, { ...runtime, node_id: targetNodeId })
}

export function inactiveDialogueState(): DialogueState {
  return {
    is_active: false,
    speaker: null,
    scene_id: null,
    text: '',
    emotion: null,
    choices: [],
    live2d_expression: null,
  }
}

function transitionToNode(
  dialogue: StoryDialogueInfo,
  initialRuntime: BrowserDialogueRuntime,
): BrowserDialogueTransition {
  let runtime = cloneRuntime(initialRuntime)
  const visited = new Set<string>()
  while (runtime.node_id) {
    if (visited.has(runtime.node_id) || visited.size > Object.keys(dialogue.nodes || {}).length) {
      throw new StoryPlaytestError(
        'condition_cycle',
        `Dialogue "${dialogue.id}" conditional skip transitions form a cycle at node "${runtime.node_id}".`,
        runtime.node_id,
      )
    }
    visited.add(runtime.node_id)
    const node = requireDialogueNode(dialogue, runtime.node_id)
    if (!conditionMatches(node.condition, runtime, runtime.node_id)) {
      const targetNodeId = node.next_node_id?.trim() || ''
      if (!targetNodeId) {
        throw new StoryPlaytestError(
          'node_condition_blocked',
          `Dialogue node "${runtime.node_id}" is disabled and has no linear fallback.`,
          runtime.node_id,
        )
      }
      if (!dialogue.nodes?.[targetNodeId]) {
        throw new StoryPlaytestError(
          'next_target_missing',
          `Dialogue node "${runtime.node_id}" targets missing node "${targetNodeId}".`,
          runtime.node_id,
          targetNodeId,
        )
      }
      runtime = { ...runtime, node_id: targetNodeId }
      continue
    }
    runtime = applyNodeScript(runtime, node.script, runtime.node_id)
    return {
      runtime,
      state: dialogueState(node, runtime),
      completed: false,
      blocked_reason: null,
      relationship_changes: {},
    }
  }
  return completedTransition(runtime)
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

function dialogueState(node: WebDialogueNode, runtime: BrowserDialogueRuntime): DialogueState {
  return {
    is_active: true,
    speaker: node.speaker_id || null,
    scene_id: node.scene_id || null,
    text: node.text,
    emotion: node.emotion || null,
    choices: availableChoices(node, runtime).map(({ choice, index }) => ({ index, text: choice.text })),
    live2d_expression: node.emotion || null,
  }
}

function availableChoices(node: WebDialogueNode, runtime: BrowserDialogueRuntime) {
  return (node.choices || [])
    .map((choice, index) => ({ choice, index }))
    .filter(({ choice, index }) => conditionMatches(choice.condition, runtime, runtime.node_id, index))
}

function conditionMatches(
  condition: string | null | undefined,
  runtime: BrowserDialogueRuntime,
  nodeId: string | null,
  choiceIndex: number | null = null,
): boolean {
  if (!condition?.trim()) return true
  const result = evaluateLocalCondition(condition, {
    variables: runtime.variables,
    flags: runtime.flags,
  })
  if (!result.supported) {
    throw new StoryPlaytestError(
      'condition_unsupported',
      `Dialogue condition cannot run in browser Playtest: ${result.error || 'unsupported condition'}.`,
      nodeId,
      null,
      choiceIndex,
    )
  }
  return result.result
}

function applyNodeScript(
  runtime: BrowserDialogueRuntime,
  script: string | null | undefined,
  nodeId: string,
): BrowserDialogueRuntime {
  const source = script?.trim()
  if (!source) return runtime
  const flag = source.match(/^setFlag\((['"])([A-Za-z0-9_.-]+)\1\s*,\s*(true|false)\)$/)
  if (flag) {
    return { ...runtime, flags: { ...runtime.flags, [flag[2]]: flag[3] === 'true' } }
  }
  const variable = source.match(/^setVariable\((['"])([A-Za-z0-9_.-]+)\1\s*,\s*(.+)\)$/)
  if (variable) {
    const value = scriptValue(variable[3])
    if (value !== null) {
      return { ...runtime, variables: { ...runtime.variables, [variable[2]]: value } }
    }
  }
  throw new StoryPlaytestError(
    'script_unsupported',
    `Dialogue node "${nodeId}" uses a script unsupported by browser Playtest.`,
    nodeId,
  )
}

function scriptValue(source: string): LocalConditionValue | null {
  const value = source.trim()
  if (value === 'true') return true
  if (value === 'false') return false
  if (/^-?\d+(?:\.\d+)?$/.test(value)) return Number(value)
  const quoted = value.match(/^(['"])(.*)\1$/)
  return quoted ? quoted[2].replace(/\\(['"\\])/g, '$1') : null
}

function completedTransition(runtime: BrowserDialogueRuntime): BrowserDialogueTransition {
  return {
    runtime: { ...cloneRuntime(runtime), node_id: null },
    state: inactiveDialogueState(),
    completed: true,
    blocked_reason: null,
    relationship_changes: {},
  }
}

function cloneRuntime(runtime: BrowserDialogueRuntime): BrowserDialogueRuntime {
  return {
    node_id: runtime.node_id,
    variables: { ...runtime.variables },
    flags: { ...runtime.flags },
  }
}

function normalizeInitialVariables(value: Record<string, unknown> | undefined): Record<string, LocalConditionValue> {
  const variables: Record<string, LocalConditionValue> = {}
  for (const [key, entry] of Object.entries(value || {})) {
    if (typeof entry === 'string' || typeof entry === 'boolean') variables[key] = entry
    if (typeof entry === 'number' && Number.isFinite(entry)) variables[key] = entry
  }
  return variables
}

function normalizeRelationshipChanges(value: Record<string, number> | undefined): Record<string, number> {
  return Object.fromEntries(Object.entries(value || {})
    .map(([characterId, delta]) => [characterId.trim(), Number(delta)] as const)
    .filter(([characterId, delta]) => characterId.length > 0 && Number.isFinite(delta)))
}
