import type { StoryContentAccessEntry } from './storyAccess'
import type { StoryEndingDefinition } from './storyContent'
import {
  hasStoryEndingIdCollision,
  STORY_ENDING_SCHEMA,
  type StoryEndingAuthoringEntry,
} from './storyEndings'

export interface StoryEndingReferenceContext {
  original_ending_id: string | null
  existing_ending_ids: readonly string[]
  scene_ids: readonly string[]
  dialogue_ids: readonly string[]
}

export type StoryEndingReferenceIssueCode =
  | 'scene_missing'
  | 'dialogue_missing'
  | 'id_collision'

export interface StoryEndingReferenceIssue {
  code: StoryEndingReferenceIssueCode
  target_id: string
}

export type StoryEndingCoverageWarningCode =
  | 'no_unlock'
  | 'scene_not_unlocked'
  | 'dialogue_not_unlocked'

export interface StoryEndingCoverageWarning {
  code: StoryEndingCoverageWarningCode
  event_id?: string
}

export interface StoryEndingCoverageContext {
  original_ending_id: string | null
  ending_unlock_event_ids: readonly string[]
  scene_access: StoryContentAccessEntry | null
  dialogue_access: StoryContentAccessEntry | null
}

export function cloneStoryEndingDefinition(
  ending: StoryEndingDefinition,
): StoryEndingDefinition {
  return {
    schema: ending.schema,
    id: ending.id,
    title: ending.title,
    description: ending.description,
    scene_id: ending.scene_id,
    dialogue_id: ending.dialogue_id,
  }
}

export function storyEndingDefinitionFromEntry(
  entry: StoryEndingAuthoringEntry,
): StoryEndingDefinition {
  return cloneStoryEndingDefinition(entry)
}

export function storyEndingDraftSnapshot(
  ending: StoryEndingDefinition | null | undefined,
): string {
  return ending ? JSON.stringify(ending) : ''
}

export function filterStoryEndingEntries(
  endings: readonly StoryEndingAuthoringEntry[],
  query: string,
): StoryEndingAuthoringEntry[] {
  const needle = query.trim().toLocaleLowerCase()
  return endings.filter((ending) => !needle
    || ending.id.toLocaleLowerCase().includes(needle)
    || ending.title.toLocaleLowerCase().includes(needle)
    || ending.description.toLocaleLowerCase().includes(needle))
}

export function nextStoryEndingId(
  existingIds: readonly string[],
  base = 'new_ending',
): string {
  const normalizedBase = base.trim() || 'new_ending'
  if (!hasStoryEndingIdCollision(existingIds, normalizedBase)) return normalizedBase
  let index = 2
  while (hasStoryEndingIdCollision(existingIds, `${normalizedBase}_${index}`)) index += 1
  return `${normalizedBase}_${index}`
}

export function createStoryEndingDraft(
  existingIds: readonly string[],
  title: string,
  description: string,
  sceneId: string,
  dialogueId: string,
): StoryEndingDefinition {
  return {
    schema: STORY_ENDING_SCHEMA,
    id: nextStoryEndingId(existingIds),
    title,
    description,
    scene_id: sceneId,
    dialogue_id: dialogueId,
  }
}

export function duplicateStoryEndingDraft(
  source: StoryEndingDefinition,
  existingIds: readonly string[],
  title: string,
): StoryEndingDefinition {
  return {
    ...cloneStoryEndingDefinition(source),
    id: nextStoryEndingId(existingIds, `${source.id}_copy`),
    title,
  }
}

export function validateStoryEndingReferences(
  ending: StoryEndingDefinition,
  context: StoryEndingReferenceContext,
): StoryEndingReferenceIssue[] {
  const issues: StoryEndingReferenceIssue[] = []
  if (ending.scene_id && !context.scene_ids.includes(ending.scene_id)) {
    issues.push({ code: 'scene_missing', target_id: ending.scene_id })
  }
  if (ending.dialogue_id && !context.dialogue_ids.includes(ending.dialogue_id)) {
    issues.push({ code: 'dialogue_missing', target_id: ending.dialogue_id })
  }
  if (!context.original_ending_id
    && hasStoryEndingIdCollision(context.existing_ending_ids, ending.id)) {
    issues.push({ code: 'id_collision', target_id: ending.id.trim() })
  }
  return issues
}

export function storyEndingCoverageWarnings(
  context: StoryEndingCoverageContext,
): StoryEndingCoverageWarning[] {
  if (!context.original_ending_id) return []
  if (context.ending_unlock_event_ids.length === 0) return [{ code: 'no_unlock' }]

  const warnings: StoryEndingCoverageWarning[] = []
  for (const eventId of context.ending_unlock_event_ids) {
    if (!context.scene_access?.unlock_event_ids.includes(eventId)) {
      warnings.push({ code: 'scene_not_unlocked', event_id: eventId })
    }
    if (!context.dialogue_access?.unlock_event_ids.includes(eventId)) {
      warnings.push({ code: 'dialogue_not_unlocked', event_id: eventId })
    }
  }
  return warnings
}
