import {
  loadBrowserStoryEndingDrafts,
  loadStoryEndings,
  saveBrowserStoryEndingDrafts,
  type StoryEndingDefinition,
  type StoryEndingInfo,
} from './storyContent'
import { hasTauriRuntime, invokeCommand } from './tauri'
import type { StoryContentAccessEntry } from './storyAccess'

export const STORY_ENDING_SCHEMA = 'monogatari-story-ending/v1'
export const STORY_ENDING_CATALOG_SCHEMA = 'monogatari-story-ending-catalog/v1'

export interface StoryEndingAuthoringEntry extends StoryEndingDefinition {
  source_path: string
  content_fingerprint: string
  access: StoryContentAccessEntry
}

export interface StoryEndingCatalogSnapshot {
  schema: string
  catalog_fingerprint: string
  ending_count: number
  endings: StoryEndingAuthoringEntry[]
}

const PORTABLE_ID = /^[A-Za-z0-9_.-]{1,128}$/

export async function loadStoryEndingCatalog(): Promise<StoryEndingCatalogSnapshot> {
  if (hasTauriRuntime()) {
    return invokeCommand<StoryEndingCatalogSnapshot>('get_story_ending_catalog')
  }
  return browserCatalogSnapshot()
}

export async function saveStoryEnding(
  ending: StoryEndingDefinition,
  originalEndingId: string | null,
  expectedCatalogFingerprint: string,
): Promise<StoryEndingCatalogSnapshot> {
  const issues = validateStoryEndingDefinition(ending)
  if (issues.length > 0) throw new Error(issues[0])
  if (hasTauriRuntime()) {
    return invokeCommand<StoryEndingCatalogSnapshot>('save_story_ending', {
      ending,
      originalEndingId,
      expectedCatalogFingerprint,
    })
  }

  const current = await browserCatalogSnapshot()
  ensureExpectedFingerprint(current, expectedCatalogFingerprint)
  const definitions = current.endings.map(endingDefinition)
  const existingIndex = definitions.findIndex((item) => item.id === ending.id)
  if (originalEndingId) {
    if (originalEndingId !== ending.id) {
      throw new Error('Ending ids are immutable after creation. Duplicate the ending to use a new id.')
    }
    if (existingIndex < 0) throw new Error(`Ending "${originalEndingId}" no longer exists. Reload first.`)
    definitions.splice(existingIndex, 1, normalizedEnding(ending))
  } else {
    if (existingIndex >= 0) throw new Error(`Ending "${ending.id}" already exists.`)
    definitions.push(normalizedEnding(ending))
  }
  definitions.sort((left, right) => left.id.localeCompare(right.id))
  saveBrowserStoryEndingDrafts(definitions)
  return browserCatalogSnapshot()
}

export async function deleteStoryEnding(
  endingId: string,
  expectedCatalogFingerprint: string,
): Promise<StoryEndingCatalogSnapshot> {
  if (hasTauriRuntime()) {
    return invokeCommand<StoryEndingCatalogSnapshot>('delete_story_ending', {
      endingId,
      expectedCatalogFingerprint,
    })
  }

  const current = await browserCatalogSnapshot()
  ensureExpectedFingerprint(current, expectedCatalogFingerprint)
  const target = current.endings.find((ending) => ending.id === endingId)
  if (!target) throw new Error(`Ending "${endingId}" does not exist.`)
  if (target.access.unlock_event_ids.length > 0) {
    throw new Error(`Ending "${endingId}" is still unlocked by: ${target.access.unlock_event_ids.join(', ')}.`)
  }
  saveBrowserStoryEndingDrafts(
    current.endings.filter((ending) => ending.id !== endingId).map(endingDefinition),
  )
  return browserCatalogSnapshot()
}

export function validateStoryEndingDefinition(ending: StoryEndingDefinition): string[] {
  const issues: string[] = []
  if (ending.schema !== STORY_ENDING_SCHEMA) issues.push(`Schema must be ${STORY_ENDING_SCHEMA}.`)
  for (const [label, value] of [
    ['Ending ID', ending.id],
    ['Scene ID', ending.scene_id],
    ['Dialogue ID', ending.dialogue_id],
  ] as const) {
    if (!PORTABLE_ID.test(value) || value.trim() !== value) {
      issues.push(`${label} must be a portable 1-128 character id.`)
    }
  }
  const titleLength = ending.title.trim().length
  if (titleLength < 1 || titleLength > 256) issues.push('Title must contain 1-256 characters.')
  const descriptionLength = ending.description.trim().length
  if (descriptionLength < 1 || descriptionLength > 2048) {
    issues.push('Description must contain 1-2048 characters.')
  }
  return issues
}

async function browserCatalogSnapshot(): Promise<StoryEndingCatalogSnapshot> {
  const draftActive = loadBrowserStoryEndingDrafts() !== null
  const endings = await loadStoryEndings()
  const entries = endings
    .map((ending) => authoringEntry(ending, draftActive))
    .sort((left, right) => left.id.localeCompare(right.id))
  return {
    schema: STORY_ENDING_CATALOG_SCHEMA,
    catalog_fingerprint: browserFingerprint(entries.map((ending) => ({
      source_path: ending.source_path,
      ending: endingDefinition(ending),
    }))),
    ending_count: entries.length,
    endings: entries,
  }
}

function authoringEntry(ending: StoryEndingInfo, draftActive: boolean): StoryEndingAuthoringEntry {
  const definition = endingDefinition(ending)
  return {
    ...definition,
    source_path: `${draftActive ? 'browser-draft/' : ''}endings/${ending.id}.json`,
    content_fingerprint: browserFingerprint(definition),
    access: ending.access,
  }
}

function endingDefinition(ending: StoryEndingDefinition): StoryEndingDefinition {
  return normalizedEnding(ending)
}

function normalizedEnding(ending: StoryEndingDefinition): StoryEndingDefinition {
  return {
    schema: STORY_ENDING_SCHEMA,
    id: ending.id.trim(),
    title: ending.title.trim(),
    description: ending.description.trim(),
    scene_id: ending.scene_id.trim(),
    dialogue_id: ending.dialogue_id.trim(),
  }
}

function ensureExpectedFingerprint(
  current: StoryEndingCatalogSnapshot,
  expected: string,
): void {
  if (current.catalog_fingerprint !== expected) {
    throw new Error('Ending catalog changed since it was opened. Reload before saving.')
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
