import { hasTauriRuntime, invokeCommand } from './tauri'
import type { StoryEventAction } from './storyEvents'

export interface StoryEventActionResult {
  action: StoryEventAction
  changed: boolean
}

export interface StoryEventApplication {
  schema: string
  event_id: string
  event_type: string
  character_id?: string | null
  applied: boolean
  application_count: number
  rule_fingerprint?: string | null
  catalog_fingerprint: string
  progress_fingerprint: string
  action_results: StoryEventActionResult[]
}

export interface AppliedStoryEvent {
  event_id: string
  event_type: string
  character_id?: string | null
  rule_fingerprint?: string | null
  application_count: number
}

export interface StoryProgressSnapshot {
  schema: string
  catalog_fingerprint?: string | null
  progress_fingerprint: string
  applied_event_count: number
  total_application_count: number
  applied_events: AppliedStoryEvent[]
  unlocked_scene_ids: string[]
  unlocked_dialogue_ids: string[]
  unlocked_ending_ids: string[]
}

export async function loadStoryProgress(): Promise<StoryProgressSnapshot> {
  if (hasTauriRuntime()) {
    return invokeCommand<StoryProgressSnapshot>('get_story_progress')
  }
  return {
    schema: 'monogatari-story-progress/v1',
    catalog_fingerprint: null,
    progress_fingerprint: '',
    applied_event_count: 0,
    total_application_count: 0,
    applied_events: [],
    unlocked_scene_ids: [],
    unlocked_dialogue_ids: [],
    unlocked_ending_ids: [],
  }
}
