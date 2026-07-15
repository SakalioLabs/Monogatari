import { cleanRendererPathMap } from './rendererAssets'
import type { StoryCharacterInfo } from './storyContent'

export const CHARACTER_EMOTIONS = [
  'neutral',
  'happy',
  'sad',
  'angry',
  'surprised',
  'love',
  'embarrassed',
  'thoughtful',
  'excited',
  'anxious',
] as const

export interface CharacterKnowledgeEntry {
  topic: string
  content: string
}

export interface CharacterForm {
  id: string
  name: string
  description: string
  background: string
  speech_style: string
  default_emotion: string
  live2d_model_path: string
  model_3d_path: string
  portrait_path: string
  sprite_path: string
  sprite_paths: Record<string, string>
  openness: number
  conscientiousness: number
  extraversion: number
  agreeableness: number
  neuroticism: number
  relationships: Record<string, number>
  knowledge_entries: CharacterKnowledgeEntry[]
  knowledge_refs: string
  emotion_modifiers: Record<string, string>
}

export type CharacterTraitKey =
  | 'openness'
  | 'conscientiousness'
  | 'extraversion'
  | 'agreeableness'
  | 'neuroticism'

export interface CharacterSummary {
  id: string
  name: string
  description: string
  emotion: string
  live2d_model_path: string | null
  portrait_path: string | null
  sprite_path: string | null
}

export type CharacterValidationCode =
  | 'required'
  | 'invalid_id'
  | 'duplicate_id'
  | 'unknown_knowledge'
  | 'invalid_private_knowledge'

export interface CharacterValidationIssue {
  code: CharacterValidationCode
  knowledge_refs: string[]
}

export interface CharacterValidationContext {
  isNew: boolean
  existingCharacterIds: readonly string[]
  knownKnowledgeIds: readonly string[] | null
}

export function createCharacterForm(): CharacterForm {
  return {
    id: '',
    name: '',
    description: '',
    background: '',
    speech_style: '',
    default_emotion: 'neutral',
    live2d_model_path: '',
    model_3d_path: '',
    portrait_path: '',
    sprite_path: '',
    sprite_paths: {},
    openness: 0.5,
    conscientiousness: 0.5,
    extraversion: 0.5,
    agreeableness: 0.5,
    neuroticism: 0.5,
    relationships: {},
    knowledge_entries: [],
    knowledge_refs: '',
    emotion_modifiers: {},
  }
}

export function characterFormFromStory(character: StoryCharacterInfo): CharacterForm {
  const personality = character.personality ?? {}
  const knowledgeRefs = character.knowledge_refs?.length
    ? character.knowledge_refs
    : character.knowledge ?? []
  return {
    id: character.id,
    name: character.name || '',
    description: character.description || '',
    background: character.background || '',
    speech_style: stringValue(personality.speech_style),
    default_emotion: character.emotion || character.currentEmotion || 'neutral',
    live2d_model_path: character.live2d_model_path || '',
    model_3d_path: character.model_3d_path || '',
    portrait_path: character.portrait_path || '',
    sprite_path: character.sprite_path || '',
    sprite_paths: cleanRendererPathMap(character.sprite_paths),
    openness: clampUnit(personality.openness, 0.5),
    conscientiousness: clampUnit(personality.conscientiousness, 0.5),
    extraversion: clampUnit(personality.extraversion, 0.5),
    agreeableness: clampUnit(personality.agreeableness, 0.5),
    neuroticism: clampUnit(personality.neuroticism, 0.5),
    relationships: normalizeRelationships(character.relationships),
    knowledge_entries: normalizeKnowledgeEntries(character.knowledge_entries),
    knowledge_refs: parseCharacterKnowledgeRefs(knowledgeRefs.join(',')).join(', '),
    emotion_modifiers: normalizeStringRecord(character.emotion_modifiers),
  }
}

export function characterSummaryFromStory(character: StoryCharacterInfo): CharacterSummary {
  return {
    id: character.id,
    name: character.name,
    description: character.description,
    emotion: character.emotion || character.currentEmotion || 'neutral',
    live2d_model_path: character.live2d_model_path ?? null,
    portrait_path: character.portrait_path ?? null,
    sprite_path: character.sprite_path ?? null,
  }
}

export function filterCharacterSummaries(
  characters: readonly CharacterSummary[],
  search: string,
): CharacterSummary[] {
  const query = search.trim().toLowerCase()
  if (!query) return [...characters]
  return characters.filter((character) => (
    character.id.toLowerCase().includes(query)
    || character.name.toLowerCase().includes(query)
    || character.description.toLowerCase().includes(query)
  ))
}

export function parseCharacterKnowledgeRefs(value: string): string[] {
  return [...new Set(value.split(',').map((reference) => reference.trim()).filter(Boolean))]
}

export function toggleCharacterKnowledgeRef(value: string, reference: string): string {
  const normalizedReference = reference.trim()
  const references = new Set(parseCharacterKnowledgeRefs(value))
  if (!normalizedReference) return [...references].join(', ')
  if (references.has(normalizedReference)) references.delete(normalizedReference)
  else references.add(normalizedReference)
  return [...references].join(', ')
}

export function validateCharacterForm(
  form: CharacterForm,
  context: CharacterValidationContext,
): CharacterValidationIssue | null {
  const id = form.id.trim()
  if (!id || !form.name.trim()) return { code: 'required', knowledge_refs: [] }
  if (!/^[A-Za-z0-9_-]{1,128}$/.test(id)) return { code: 'invalid_id', knowledge_refs: [] }
  if (context.isNew) {
    const normalizedId = id.toLowerCase()
    if (context.existingCharacterIds.some((candidate) => candidate.trim().toLowerCase() === normalizedId)) {
      return { code: 'duplicate_id', knowledge_refs: [] }
    }
  }
  if (context.knownKnowledgeIds !== null) {
    const known = new Set(context.knownKnowledgeIds)
    const missing = parseCharacterKnowledgeRefs(form.knowledge_refs).filter((reference) => !known.has(reference))
    if (missing.length > 0) return { code: 'unknown_knowledge', knowledge_refs: missing }
  }
  if (form.knowledge_entries.some((entry) => !entry.topic.trim() || !entry.content.trim())) {
    return { code: 'invalid_private_knowledge', knowledge_refs: [] }
  }
  return null
}

export function buildStoryCharacter(form: CharacterForm): StoryCharacterInfo {
  return {
    id: form.id.trim(),
    name: form.name.trim(),
    description: form.description.trim(),
    background: form.background.trim(),
    personality: {
      openness: clampUnit(form.openness, 0),
      conscientiousness: clampUnit(form.conscientiousness, 0),
      extraversion: clampUnit(form.extraversion, 0),
      agreeableness: clampUnit(form.agreeableness, 0),
      neuroticism: clampUnit(form.neuroticism, 0),
      speech_style: form.speech_style.trim(),
    },
    emotion: form.default_emotion.trim() || 'neutral',
    live2d_model_path: optionalPath(form.live2d_model_path),
    model_3d_path: optionalPath(form.model_3d_path),
    portrait_path: optionalPath(form.portrait_path),
    sprite_path: optionalPath(form.sprite_path),
    sprite_paths: cleanRendererPathMap(form.sprite_paths),
    relationships: normalizeRelationships(form.relationships),
    knowledge_entries: normalizeKnowledgeEntries(form.knowledge_entries, true),
    knowledge_refs: parseCharacterKnowledgeRefs(form.knowledge_refs),
    emotion_modifiers: normalizeStringRecord(form.emotion_modifiers),
  }
}

export function characterFormSnapshot(form: CharacterForm): string {
  return JSON.stringify(buildStoryCharacter(form))
}

export function fillMissingCharacterSpritePaths(
  paths: Readonly<Record<string, string>>,
  fallback: string,
  emotions: readonly string[] = CHARACTER_EMOTIONS,
): Record<string, string> {
  const normalized = cleanRendererPathMap(paths)
  const normalizedFallback = fallback.trim()
  if (!normalizedFallback) return normalized
  for (const emotion of emotions) {
    if (!normalized[emotion]) normalized[emotion] = normalizedFallback
  }
  return normalized
}

export function characterSpritePlaceholder(
  characterId: string,
  emotion: string,
  fallback: string,
): string {
  const normalizedFallback = fallback.trim()
  if (normalizedFallback) return normalizedFallback
  return `assets/sprites/${characterId.trim() || 'character'}_${emotion}.png`
}

function clampUnit(value: unknown, fallback: number): number {
  if (value === null || value === undefined) return fallback
  const numeric = Number(value)
  if (Number.isNaN(numeric)) return fallback
  return Math.max(0, Math.min(1, numeric))
}

function clampRelationship(value: unknown): number {
  const numeric = Number(value)
  if (Number.isNaN(numeric)) return 0
  return Math.max(-1, Math.min(1, numeric))
}

function normalizeRelationships(value: Readonly<Record<string, number>> | undefined): Record<string, number> {
  if (!value) return {}
  return Object.fromEntries(Object.entries(value)
    .map(([key, score]) => [key.trim(), clampRelationship(score)] as const)
    .filter(([key]) => Boolean(key)))
}

function normalizeKnowledgeEntries(
  value: readonly CharacterKnowledgeEntry[] | undefined,
  trim = false,
): CharacterKnowledgeEntry[] {
  return (value ?? []).map((entry) => ({
    topic: trim ? stringValue(entry.topic).trim() : stringValue(entry.topic),
    content: trim ? stringValue(entry.content).trim() : stringValue(entry.content),
  }))
}

function normalizeStringRecord(value: Readonly<Record<string, string>> | undefined): Record<string, string> {
  if (!value) return {}
  return Object.fromEntries(Object.entries(value)
    .map(([key, item]) => [key.trim(), stringValue(item)] as const)
    .filter(([key]) => Boolean(key)))
}

function stringValue(value: unknown): string {
  return value === null || value === undefined ? '' : String(value)
}

function optionalPath(value: string): string | null {
  return value.trim() || null
}
