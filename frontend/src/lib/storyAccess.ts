import { hasTauriRuntime, invokeCommand } from './tauri'
import { loadStoryEventCatalog, type StoryEventAction } from './storyEvents'
import { loadStoryProgress, type StoryProgressSnapshot } from './storyProgress'

export type StoryContentKind = 'scene' | 'dialogue' | 'ending'

export interface StoryContentAccessEntry {
  content_type: StoryContentKind
  content_id: string
  gated: boolean
  unlocked: boolean
  unlock_event_ids: string[]
}

export interface StoryContentAccessSnapshot {
  schema: string
  catalog_fingerprint: string
  progress_fingerprint: string
  gated_content_count: number
  unlocked_gated_content_count: number
  locked_content_count: number
  entries: StoryContentAccessEntry[]
}

export async function loadStoryContentAccess(): Promise<StoryContentAccessSnapshot> {
  if (hasTauriRuntime()) {
    return invokeCommand<StoryContentAccessSnapshot>('get_story_content_access')
  }
  const [catalog, progress] = await Promise.all([loadStoryEventCatalog(), loadStoryProgress()])
  return deriveStoryContentAccess(catalog.catalog_fingerprint, catalog.events, progress)
}

export function deriveStoryContentAccess(
  catalogFingerprint: string,
  events: Array<{ event_id: string; actions: StoryEventAction[] }>,
  progress: StoryProgressSnapshot,
): StoryContentAccessSnapshot {
  const sources = new Map<string, Set<string>>()
  for (const event of events) {
    for (const action of event.actions) {
      const target = actionTarget(action)
      if (!target) continue
      const key = `${target.content_type}:${target.content_id}`
      if (!sources.has(key)) sources.set(key, new Set())
      sources.get(key)?.add(event.event_id)
    }
  }
  for (const [contentType, ids] of [
    ['scene', progress.unlocked_scene_ids],
    ['dialogue', progress.unlocked_dialogue_ids],
    ['ending', progress.unlocked_ending_ids],
  ] as const) {
    for (const id of ids) {
      const key = `${contentType}:${id}`
      if (!sources.has(key)) sources.set(key, new Set())
    }
  }

  const entries = [...sources.entries()].map(([key, eventIds]) => {
    const separator = key.indexOf(':')
    const contentType = key.slice(0, separator) as StoryContentKind
    const contentId = key.slice(separator + 1)
    const gated = eventIds.size > 0
    return {
      content_type: contentType,
      content_id: contentId,
      gated,
      unlocked: !gated || progressIds(progress, contentType).includes(contentId),
      unlock_event_ids: [...eventIds].sort(),
    }
  }).sort((left, right) => left.content_type.localeCompare(right.content_type)
    || left.content_id.localeCompare(right.content_id))
  const gatedCount = entries.filter((entry) => entry.gated).length
  const unlockedCount = entries.filter((entry) => entry.gated && entry.unlocked).length
  return {
    schema: 'monogatari-story-content-access/v1',
    catalog_fingerprint: catalogFingerprint,
    progress_fingerprint: progress.progress_fingerprint,
    gated_content_count: gatedCount,
    unlocked_gated_content_count: unlockedCount,
    locked_content_count: gatedCount - unlockedCount,
    entries,
  }
}

function actionTarget(action: StoryEventAction): { content_type: StoryContentKind; content_id: string } | null {
  if (action.type === 'unlock_scene') return { content_type: 'scene', content_id: action.scene_id }
  if (action.type === 'unlock_dialogue') return { content_type: 'dialogue', content_id: action.dialogue_id }
  if (action.type === 'unlock_ending') return { content_type: 'ending', content_id: action.ending_id }
  return null
}

function progressIds(progress: StoryProgressSnapshot, contentType: StoryContentKind): string[] {
  if (contentType === 'scene') return progress.unlocked_scene_ids
  if (contentType === 'dialogue') return progress.unlocked_dialogue_ids
  return progress.unlocked_ending_ids
}

export function contentAccess(
  snapshot: StoryContentAccessSnapshot,
  contentType: StoryContentKind,
  contentId: string,
): StoryContentAccessEntry {
  return snapshot.entries.find((entry) => entry.content_type === contentType && entry.content_id === contentId)
    || { content_type: contentType, content_id: contentId, gated: false, unlocked: true, unlock_event_ids: [] }
}
