import { contentAccess, loadStoryContentAccess, type StoryContentAccessEntry } from './storyAccess'
import { hasTauriRuntime, invokeCommand } from './tauri'

export interface StorySceneInfo {
  id: string
  name: string
  background_path: string | null
  bgm_path: string | null
  weather: string | null
  time_of_day: string | null
  tags: string[]
  source: string
  background_exists: boolean
  absolute_background_path: string | null
  access: StoryContentAccessEntry
}

export interface StoryDialogueInfo {
  id: string
  title: string
  start_node_id: string
  node_count: number
  nodes?: Record<string, WebDialogueNode>
  access: StoryContentAccessEntry
}

export interface WebDialogueNode {
  speaker_id?: string | null
  text: string
  emotion?: string | null
  next_node_id?: string | null
  choices?: Array<{ text: string; next_node_id: string }>
}

export interface StoryEndingInfo {
  schema: string
  id: string
  title: string
  description: string
  scene_id: string
  dialogue_id: string
  access: StoryContentAccessEntry
}

export interface StoryEndingDefinition {
  schema: string
  id: string
  title: string
  description: string
  scene_id: string
  dialogue_id: string
}

interface WebProjectManifest {
  schema: string
  scene_files?: string[]
  dialogue_files?: string[]
  ending_files?: string[]
}

interface SceneDocument {
  id: string
  name: string
  background_path?: string | null
  backgroundPath?: string | null
  bgm_path?: string | null
  bgmPath?: string | null
  weather?: string | null
  time_of_day?: string | null
  tags?: string[]
}

interface DialogueDocument {
  id: string
  title: string
  start_node_id: string
  nodes: Record<string, WebDialogueNode>
}

const BROWSER_ENDING_DRAFT_KEY = 'monogatari:story-ending-catalog:v1'

function baseUrl(): URL {
  const base = import.meta.env.BASE_URL || '/'
  return base === './' ? new URL('./', window.location.href) : new URL(base, window.location.origin)
}

function projectUrl(relativePath: string): string {
  return new URL(relativePath.replace(/^\/+/, ''), baseUrl()).toString()
}

async function webProjectManifest(): Promise<WebProjectManifest> {
  const response = await fetch(projectUrl('project-assets.json'), { cache: 'no-cache' })
  if (!response.ok) throw new Error(`Project content manifest returned HTTP ${response.status}`)
  const manifest = await response.json() as WebProjectManifest
  if (manifest.schema !== 'monogatari-web-project-assets/v1') {
    throw new Error(`Unsupported project content manifest: ${String(manifest.schema)}`)
  }
  return manifest
}

async function fetchDocuments<T>(paths: string[] | undefined): Promise<T[]> {
  if (!paths?.length) return []
  return Promise.all(paths.map(async (file) => {
    const response = await fetch(projectUrl(file), { cache: 'no-cache' })
    if (!response.ok) throw new Error(`${file} returned HTTP ${response.status}`)
    return response.json() as Promise<T>
  }))
}

export async function loadStoryScenes(): Promise<StorySceneInfo[]> {
  if (hasTauriRuntime()) return invokeCommand<StorySceneInfo[]>('list_story_scenes')
  const access = await loadStoryContentAccess()
  try {
    const manifest = await webProjectManifest()
    const documents = await fetchDocuments<SceneDocument>(manifest.scene_files)
    return documents.map((scene) => {
      const backgroundPath = scene.background_path ?? scene.backgroundPath ?? null
      return {
        id: scene.id,
        name: scene.name,
        background_path: backgroundPath,
        bgm_path: scene.bgm_path ?? scene.bgmPath ?? null,
        weather: scene.weather ?? null,
        time_of_day: scene.time_of_day ?? null,
        tags: Array.isArray(scene.tags) ? scene.tags : [],
        source: 'web_project',
        background_exists: Boolean(backgroundPath),
        absolute_background_path: null,
        access: contentAccess(access, 'scene', scene.id),
      }
    }).sort((left, right) => left.id.localeCompare(right.id))
  } catch {
    return browserSceneFallback.map((scene) => ({
      ...scene,
      access: contentAccess(access, 'scene', scene.id),
    }))
  }
}

export async function loadStoryDialogues(): Promise<StoryDialogueInfo[]> {
  if (hasTauriRuntime()) return invokeCommand<StoryDialogueInfo[]>('list_dialogues')
  const access = await loadStoryContentAccess()
  try {
    const manifest = await webProjectManifest()
    const documents = await fetchDocuments<DialogueDocument>(manifest.dialogue_files)
    return documents.map((dialogue) => ({
      id: dialogue.id,
      title: dialogue.title,
      start_node_id: dialogue.start_node_id,
      node_count: Object.keys(dialogue.nodes || {}).length,
      nodes: dialogue.nodes || {},
      access: contentAccess(access, 'dialogue', dialogue.id),
    })).sort((left, right) => left.id.localeCompare(right.id))
  } catch {
    return browserDialogueFallback.map((dialogue) => ({
      ...dialogue,
      access: contentAccess(access, 'dialogue', dialogue.id),
    }))
  }
}

export async function loadStoryEndings(): Promise<StoryEndingInfo[]> {
  if (hasTauriRuntime()) return invokeCommand<StoryEndingInfo[]>('list_story_endings')
  const access = await loadStoryContentAccess()
  const browserDrafts = loadBrowserStoryEndingDrafts()
  if (browserDrafts !== null) {
    return browserDrafts
      .map((ending) => ({ ...ending, access: contentAccess(access, 'ending', ending.id) }))
      .sort((left, right) => left.id.localeCompare(right.id))
  }
  try {
    const manifest = await webProjectManifest()
    const documents = await fetchDocuments<StoryEndingDefinition>(manifest.ending_files)
    return documents
      .filter((ending) => ending.schema === 'monogatari-story-ending/v1')
      .map((ending) => ({ ...ending, access: contentAccess(access, 'ending', ending.id) }))
      .sort((left, right) => left.id.localeCompare(right.id))
  } catch {
    return browserEndingFallback.map((ending) => ({
      ...ending,
      access: contentAccess(access, 'ending', ending.id),
    }))
  }
}

export function loadBrowserStoryEndingDrafts(): StoryEndingDefinition[] | null {
  if (typeof window === 'undefined') return null
  const raw = window.localStorage.getItem(BROWSER_ENDING_DRAFT_KEY)
  if (raw === null) return null
  try {
    const value = JSON.parse(raw) as unknown
    if (!Array.isArray(value)) return null
    const endings = value.filter(isStoryEndingDefinition)
    return endings.length === value.length ? endings : null
  } catch {
    return null
  }
}

export function saveBrowserStoryEndingDrafts(endings: StoryEndingDefinition[]): void {
  if (typeof window === 'undefined') return
  window.localStorage.setItem(BROWSER_ENDING_DRAFT_KEY, JSON.stringify(endings))
}

function isStoryEndingDefinition(value: unknown): value is StoryEndingDefinition {
  if (!value || typeof value !== 'object') return false
  const ending = value as Record<string, unknown>
  return ending.schema === 'monogatari-story-ending/v1'
    && ['id', 'title', 'description', 'scene_id', 'dialogue_id']
      .every((field) => typeof ending[field] === 'string')
}

function titleFromId(id: string): string {
  return id.split('_').map((word) => word.charAt(0).toUpperCase() + word.slice(1)).join(' ')
}

const browserSceneFallback: Omit<StorySceneInfo, 'access'>[] = [
  ['crossroads', 'assets/backgrounds/crossroads.svg'],
  ['festival_night', 'assets/backgrounds/festival_night.svg'],
  ['great_library', 'assets/backgrounds/great_library.svg'],
  ['hiro_workshop', 'assets/backgrounds/hiro_workshop.svg'],
  ['sakura_park', 'assets/backgrounds/sakura_park.svg'],
  ['studio_night', 'assets/backgrounds/studio_night.svg'],
].map(([id, backgroundPath]) => ({
  id,
  name: titleFromId(id),
  background_path: backgroundPath,
  bgm_path: null,
  weather: null,
  time_of_day: null,
  tags: [],
  source: 'web_bundled_fallback',
  background_exists: true,
  absolute_background_path: null,
}))

const browserDialogueFallback: Omit<StoryDialogueInfo, 'access'>[] = [
  'aoi_clinic_visit', 'cafe_encounter', 'example_dialogue', 'festival_preparations',
  'hiro_workshop', 'kenji_dojo_visit', 'luna_stargazing', 'mei_crossroads',
  'noodle_and_soul', 'observatory_night', 'post_office_tales', 'ren_investigation',
  'sakura_park_walk', 'through_the_lens', 'whispering_leaf', 'woodcarver_workshop',
  'writers_retreat', 'yuki_library',
].map((id) => ({
  id,
  title: titleFromId(id),
  start_node_id: 'start',
  node_count: 1,
  nodes: { start: { text: `Browser fallback for ${titleFromId(id)}.`, choices: [] } },
}))

const browserEndingFallback: Omit<StoryEndingInfo, 'access'>[] = [{
  schema: 'monogatari-story-ending/v1',
  id: 'best_friend_ending',
  title: 'Under the Festival Stars',
  description: 'A quiet promise closes the night after your bond reaches its strongest point.',
  scene_id: 'festival_night',
  dialogue_id: 'observatory_night',
}]
