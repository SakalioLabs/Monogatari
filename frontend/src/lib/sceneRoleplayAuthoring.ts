import {
  SCENE_ROLEPLAY_SCHEMA,
  loadSceneRoleplays,
  validateBrowserSceneRoleplayDefinition,
  type RoleplayCondition,
  type RoleplayScoreDimension,
  type RoleplayTarget,
  type SceneRoleplayDefinition,
  type SceneRoleplayNode,
} from './sceneRoleplay'
import {
  loadBrowserRoleplayDrafts,
  saveBrowserRoleplayDrafts,
} from './sceneRoleplayContent'
import { loadRoleplayCampaigns } from './roleplayCampaign'
import { hasTauriRuntime, invokeCommand } from './tauri'

export const SCENE_ROLEPLAY_AUTHORING_SCHEMA = 'monogatari-scene-roleplay-authoring-catalog/v1'

export interface SceneRoleplayAuthoringEntry {
  definition: SceneRoleplayDefinition
  source_path: string
  content_fingerprint: string
}

export interface SceneRoleplayAuthoringCatalog {
  schema: string
  catalog_fingerprint: string
  roleplay_count: number
  node_count: number
  score_dimension_count: number
  roleplays: SceneRoleplayAuthoringEntry[]
}

const PORTABLE_ID = /^[A-Za-z0-9_.-]{1,128}$/

export async function loadSceneRoleplayAuthoringCatalog(): Promise<SceneRoleplayAuthoringCatalog> {
  if (hasTauriRuntime()) {
    return invokeCommand<SceneRoleplayAuthoringCatalog>('get_scene_roleplay_authoring_catalog')
  }
  return browserCatalog()
}

export async function saveSceneRoleplayDefinition(
  definition: SceneRoleplayDefinition,
  originalRoleplayId: string | null,
  expectedCatalogFingerprint: string,
): Promise<SceneRoleplayAuthoringCatalog> {
  const normalized = normalizeSceneRoleplayDefinition(definition)
  const issues = validateSceneRoleplayDefinition(normalized)
  if (issues.length) throw new Error(issues[0])
  if (hasTauriRuntime()) {
    return invokeCommand<SceneRoleplayAuthoringCatalog>('save_scene_roleplay_definition', {
      definition: normalized,
      originalRoleplayId,
      expectedCatalogFingerprint,
    })
  }

  const current = await browserCatalog()
  ensureFingerprint(current, expectedCatalogFingerprint)
  const definitions = current.roleplays.map(entry => cloneSceneRoleplayDefinition(entry.definition))
  const existingIndex = definitions.findIndex(item => item.id === normalized.id)
  if (originalRoleplayId) {
    if (originalRoleplayId !== normalized.id) {
      throw new Error('Roleplay ids are immutable after creation. Duplicate it to use a new id.')
    }
    if (existingIndex < 0) throw new Error(`Roleplay "${originalRoleplayId}" no longer exists. Reload first.`)
    definitions.splice(existingIndex, 1, normalized)
  } else {
    if (definitions.some(item => item.id.toLowerCase() === normalized.id.toLowerCase())) {
      throw new Error(`Roleplay "${normalized.id}" already exists.`)
    }
    definitions.push(normalized)
  }
  definitions.sort((left, right) => left.id.localeCompare(right.id))
  saveBrowserRoleplayDrafts(definitions)
  return browserCatalog()
}

export async function deleteSceneRoleplayDefinition(
  roleplayId: string,
  expectedCatalogFingerprint: string,
): Promise<SceneRoleplayAuthoringCatalog> {
  if (hasTauriRuntime()) {
    return invokeCommand<SceneRoleplayAuthoringCatalog>('delete_scene_roleplay_definition', {
      roleplayId,
      expectedCatalogFingerprint,
    })
  }
  const current = await browserCatalog()
  ensureFingerprint(current, expectedCatalogFingerprint)
  if (!current.roleplays.some(entry => entry.definition.id === roleplayId)) {
    throw new Error(`Roleplay "${roleplayId}" does not exist.`)
  }
  const definitions = current.roleplays.map(entry =>
    cloneSceneRoleplayDefinition(entry.definition))
  const campaigns = await loadRoleplayCampaigns(definitions)
  const referencingCampaigns = campaigns
    .filter(campaign => campaign.entries.some(entry => entry.roleplay_id === roleplayId))
    .map(campaign => campaign.id)
  if (referencingCampaigns.length) {
    throw new Error(
      `Roleplay "${roleplayId}" is used by Campaign: ${referencingCampaigns.join(', ')}.`,
    )
  }
  saveBrowserRoleplayDrafts(definitions.filter(definition => definition.id !== roleplayId))
  return browserCatalog()
}

export function createSceneRoleplayDraft(existingIds: readonly string[] = []): SceneRoleplayDefinition {
  const id = nextPortableId('new_roleplay', existingIds)
  const endingId = 'ending_id'
  const node = createSceneRoleplayNodeDraft('opening', 'scene_id', 'character_id', endingId)
  return {
    schema: SCENE_ROLEPLAY_SCHEMA,
    id,
    title: 'New live roleplay',
    start_node_id: node.id,
    exhaustion_ending_id: endingId,
    max_total_turns: 12,
    score_dimensions: [{
      id: 'story_progress',
      label: 'Story progress',
      description: 'How strongly the player has advanced the current scene objective.',
      min: -5,
      max: 5,
      initial: 0,
    }],
    nodes: [node],
    inference: {
      max_context_characters: 6_000,
      max_recent_turns: 6,
      npc_max_tokens: 96,
      evaluator_max_tokens: 128,
    },
  }
}

export function createSceneRoleplayNodeDraft(
  id: string,
  sceneId: string,
  characterId: string,
  endingId: string,
): SceneRoleplayNode {
  return {
    id,
    scene_id: sceneId,
    character_id: characterId,
    supporting_character_ids: [],
    emotion: null,
    opening_narration: 'The scene begins.',
    situation: 'Describe only the observable current situation and active constraints.',
    player_goal: 'Engage the character and move the scene forward through free-form conversation.',
    character_goal: 'Respond from the character profile while pursuing a scene-local motive.',
    participant_goals: {},
    knowledge_refs: [],
    intrusion_response: null,
    response_guard: null,
    fallback_evaluation: null,
    min_turns: 1,
    max_turns: 4,
    score_rules: [{
      dimension_id: 'story_progress',
      guidance: 'Reward concrete progress toward the player goal; penalize unsupported claims.',
      max_delta_per_turn: 1,
    }],
    relationship_rule: null,
    evidence_rules: [],
    transitions: [],
    timeout_target: { kind: 'ending', ending_id: endingId },
  }
}

export function createRoleplayScoreDimension(existingIds: readonly string[]): RoleplayScoreDimension {
  return {
    id: nextPortableId('score', existingIds),
    label: 'Score',
    description: 'Describe what this story score measures.',
    min: -5,
    max: 5,
    initial: 0,
  }
}

export function createRoleplayTarget(kind: RoleplayTarget['kind'], id: string): RoleplayTarget {
  return kind === 'node' ? { kind, node_id: id } : { kind, ending_id: id }
}

export function createRoleplayCondition(
  kind: RoleplayCondition['kind'],
  definition: SceneRoleplayDefinition,
  node: SceneRoleplayNode,
): RoleplayCondition {
  if (kind === 'score_at_least' || kind === 'score_at_most') {
    return { kind, dimension_id: definition.score_dimensions[0]?.id || '', value: 1 }
  }
  if (kind === 'evidence_observed') {
    return { kind, evidence_id: node.evidence_rules[0]?.id || '' }
  }
  if (kind === 'relationship_at_least' || kind === 'relationship_at_most') {
    return { kind, character_id: node.character_id, value: 0.25 }
  }
  return { kind, value: 1 }
}

export function cloneSceneRoleplayDefinition(
  definition: SceneRoleplayDefinition,
): SceneRoleplayDefinition {
  return JSON.parse(JSON.stringify(definition)) as SceneRoleplayDefinition
}

export function normalizeSceneRoleplayDefinition(
  definition: SceneRoleplayDefinition,
): SceneRoleplayDefinition {
  const normalized = cloneSceneRoleplayDefinition(definition)
  normalized.schema = SCENE_ROLEPLAY_SCHEMA
  normalized.id = normalized.id.trim()
  normalized.title = normalized.title.trim()
  normalized.start_node_id = normalized.start_node_id.trim()
  normalized.exhaustion_ending_id = normalized.exhaustion_ending_id.trim()
  normalized.score_dimensions = normalized.score_dimensions.map(dimension => ({
    ...dimension,
    id: dimension.id.trim(),
    label: dimension.label.trim(),
    description: dimension.description.trim(),
  }))
  normalized.nodes = normalized.nodes.map(node => ({
    ...node,
    id: node.id.trim(),
    scene_id: node.scene_id.trim(),
    character_id: node.character_id.trim(),
    supporting_character_ids: uniqueStrings(node.supporting_character_ids),
    emotion: node.emotion?.trim() || null,
    opening_narration: node.opening_narration.trim(),
    situation: node.situation.trim(),
    player_goal: node.player_goal.trim(),
    character_goal: node.character_goal.trim(),
    participant_goals: Object.fromEntries(Object.entries(node.participant_goals || {})
      .map(([id, goal]) => [id.trim(), goal.trim()])
      .filter(([id, goal]) => id && goal)),
    knowledge_refs: uniqueStrings(node.knowledge_refs),
    score_rules: node.score_rules.map(rule => ({
      ...rule,
      dimension_id: rule.dimension_id.trim(),
      guidance: rule.guidance.trim(),
    })),
    evidence_rules: node.evidence_rules.map(rule => ({
      id: rule.id.trim(),
      description: rule.description.trim(),
    })),
    transitions: node.transitions.map(transition => ({
      ...transition,
      id: transition.id.trim(),
    })),
  }))
  return normalized
}

export function validateSceneRoleplayDefinition(definition: SceneRoleplayDefinition): string[] {
  const issues: string[] = []
  if (!PORTABLE_ID.test(definition.id) || definition.id.trim() !== definition.id) {
    issues.push('Roleplay ID must be a portable 1-128 character id.')
  }
  if (!definition.title.trim() || [...definition.title].length > 256) {
    issues.push('Roleplay title must contain 1-256 characters.')
  }
  try {
    validateBrowserSceneRoleplayDefinition(cloneSceneRoleplayDefinition(definition))
  } catch (error) {
    issues.push(error instanceof Error ? error.message : String(error))
  }
  return [...new Set(issues)]
}

export function sceneRoleplayDraftSnapshot(definition: SceneRoleplayDefinition | null): string {
  return definition ? JSON.stringify(normalizeSceneRoleplayDefinition(definition)) : ''
}

async function browserCatalog(): Promise<SceneRoleplayAuthoringCatalog> {
  const draftActive = loadBrowserRoleplayDrafts() !== null
  const definitions = await loadSceneRoleplays()
  const roleplays = definitions.map(definition => ({
    definition: cloneSceneRoleplayDefinition(definition),
    source_path: `${draftActive ? 'browser-draft/' : ''}roleplays/${definition.id}.json`,
    content_fingerprint: browserFingerprint(definition),
  }))
  return {
    schema: SCENE_ROLEPLAY_AUTHORING_SCHEMA,
    catalog_fingerprint: browserFingerprint(roleplays.map(entry => ({
      source_path: entry.source_path,
      definition: entry.definition,
    }))),
    roleplay_count: roleplays.length,
    node_count: roleplays.reduce((total, entry) => total + entry.definition.nodes.length, 0),
    score_dimension_count: roleplays.reduce(
      (total, entry) => total + entry.definition.score_dimensions.length,
      0,
    ),
    roleplays,
  }
}

function ensureFingerprint(current: SceneRoleplayAuthoringCatalog, expected: string): void {
  if (current.catalog_fingerprint !== expected) {
    throw new Error('Roleplay catalog changed since it was opened. Reload before saving.')
  }
}

function nextPortableId(base: string, existingIds: readonly string[]): string {
  const keys = new Set(existingIds.map(id => id.toLowerCase()))
  if (!keys.has(base)) return base
  for (let suffix = 2; suffix < 10_000; suffix += 1) {
    const candidate = `${base}_${suffix}`
    if (!keys.has(candidate)) return candidate
  }
  throw new Error(`Unable to allocate an id based on "${base}".`)
}

function uniqueStrings(values: readonly string[]): string[] {
  return [...new Set(values.map(value => value.trim()).filter(Boolean))].sort()
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
