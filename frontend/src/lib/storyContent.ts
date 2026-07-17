import { contentAccess, loadStoryContentAccess, type StoryContentAccessEntry } from './storyAccess'
import { hasTauriRuntime, invokeCommand } from './tauri'

export interface StorySceneInfo {
  id: string
  name: string
  background_path: string | null
  model_3d_path?: string | null
  bgm_path: string | null
  weather: string | null
  time_of_day: string | null
  tags: string[]
  source: string
  background_exists: boolean
  absolute_background_path: string | null
  model_3d_exists?: boolean
  absolute_model_3d_path?: string | null
  access: StoryContentAccessEntry
}

export interface StoryCharacterInfo {
  id: string
  name: string
  description: string
  background?: string
  emotion: string
  currentEmotion?: string
  personality?: Record<string, unknown>
  portrait_path: string | null
  sprite_path: string | null
  sprite_paths?: Record<string, string>
  live2d_model_path?: string | null
  model_3d_path?: string | null
  relationships?: Record<string, number>
  knowledge_entries?: Array<{ topic: string; content: string }>
  knowledge_refs?: string[]
  knowledge?: string[]
  emotion_modifiers?: Record<string, string>
}

export interface StoryDialogueInfo {
  id: string
  title: string
  description?: string | null
  start_node_id: string
  node_count: number
  nodes?: Record<string, WebDialogueNode>
  variables?: Record<string, unknown>
  access: StoryContentAccessEntry
}

export interface WebDialogueChoice {
  text: string
  next_node_id: string
  relationship_changes?: Record<string, number>
  condition?: string | null
}

export interface WebDialogueNode {
  speaker_id?: string | null
  scene_id?: string | null
  text: string
  emotion?: string | null
  next_node_id?: string | null
  choices?: WebDialogueChoice[]
  condition?: string | null
  script?: string | null
  use_llm?: boolean
  llm_prompt?: string | null
  llm_system_prompt?: string | null
  is_ending?: boolean
  ending_type?: string | null
}

export interface DialogueDefinition {
  id: string
  title: string
  description: string | null
  start_node_id: string
  nodes: Record<string, WebDialogueNode>
  variables: Record<string, unknown>
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
  character_files?: string[]
}

export interface SceneDefinition {
  id: string
  name: string
  background_path: string | null
  model_3d_path?: string | null
  bgm_path: string | null
  weather: string | null
  time_of_day: string | null
  tags: string[]
}

interface SceneDocument extends Partial<SceneDefinition> {
  id: string
  name: string
  backgroundPath?: string | null
  model3dPath?: string | null
  model3DPath?: string | null
  bgmPath?: string | null
}

interface DialogueDocument extends Partial<DialogueDefinition> {
  id: string
  title: string
  start_node_id: string
  nodes: Record<string, WebDialogueNode>
}

const BROWSER_ENDING_DRAFT_KEY = 'monogatari:story-ending-catalog:v1'
const BROWSER_SCENE_DRAFT_KEY = 'monogatari:scene-authoring-catalog:v1'
const BROWSER_DIALOGUE_DRAFT_KEY = 'monogatari:dialogue-authoring-catalog:v1'
const BROWSER_CHARACTER_DRAFT_KEY = 'monogatari:character-authoring-catalog:v1'

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
  const browserDrafts = loadBrowserSceneDrafts()
  if (browserDrafts !== null) {
    return browserDrafts
      .map((scene) => storySceneInfo(scene, 'browser_draft', access))
      .sort((left, right) => left.id.localeCompare(right.id))
  }
  try {
    const manifest = await webProjectManifest()
    const documents = await fetchDocuments<SceneDocument>(manifest.scene_files)
    return documents.map((scene) => {
      const backgroundPath = scene.background_path ?? scene.backgroundPath ?? null
      const model3dPath = scene.model_3d_path ?? scene.model3dPath ?? scene.model3DPath ?? null
      return storySceneInfo({
        id: scene.id,
        name: scene.name,
        background_path: backgroundPath,
        model_3d_path: model3dPath,
        bgm_path: scene.bgm_path ?? scene.bgmPath ?? null,
        weather: scene.weather ?? null,
        time_of_day: scene.time_of_day ?? null,
        tags: Array.isArray(scene.tags) ? scene.tags : [],
      }, 'web_project', access)
    }).sort((left, right) => left.id.localeCompare(right.id))
  } catch {
    return browserSceneFallback.map((scene) => ({
      ...scene,
      access: contentAccess(access, 'scene', scene.id),
    }))
  }
}

export async function loadStoryCharacters(): Promise<StoryCharacterInfo[]> {
  if (hasTauriRuntime()) {
    return normalizeStoryCharacters(await invokeCommand<StoryCharacterInfo[]>('get_characters'))
  }
  const browserDrafts = loadBrowserCharacterDrafts()
  if (browserDrafts !== null) return normalizeStoryCharacters(browserDrafts)
  try {
    const manifest = await webProjectManifest()
    const documents = await fetchDocuments<StoryCharacterInfo | StoryCharacterInfo[]>(manifest.character_files)
    return normalizeStoryCharacters(documents.flatMap(document => Array.isArray(document) ? document : [document]))
  } catch {
    return browserCharacterFallback.map((character) => ({ ...character }))
  }
}

function normalizeStoryCharacters(documents: StoryCharacterInfo[]): StoryCharacterInfo[] {
  const byId = new Map<string, StoryCharacterInfo>()
  for (const character of documents) {
    if (!character.id || !character.name) continue
    byId.set(character.id, {
      ...character,
      description: character.description || '',
      emotion: character.emotion || character.currentEmotion || 'neutral',
      portrait_path: character.portrait_path || null,
      sprite_path: character.sprite_path || null,
      knowledge_refs: character.knowledge_refs || character.knowledge || [],
    })
  }
  return [...byId.values()].sort((left, right) => left.id.localeCompare(right.id))
}

export function loadBrowserCharacterDrafts(): StoryCharacterInfo[] | null {
  if (typeof window === 'undefined') return null
  const raw = window.localStorage.getItem(BROWSER_CHARACTER_DRAFT_KEY)
  if (raw === null) return null
  try {
    const value = JSON.parse(raw) as unknown
    if (!Array.isArray(value)) return null
    const characters = value.filter(isStoryCharacterInfo)
    return characters.length === value.length ? normalizeStoryCharacters(characters) : null
  } catch {
    return null
  }
}

export function saveBrowserCharacterDrafts(characters: StoryCharacterInfo[]): void {
  if (typeof window === 'undefined') return
  window.localStorage.setItem(
    BROWSER_CHARACTER_DRAFT_KEY,
    JSON.stringify(normalizeStoryCharacters(characters)),
  )
}

export function resetBrowserCharacterDrafts(): void {
  if (typeof window === 'undefined') return
  window.localStorage.removeItem(BROWSER_CHARACTER_DRAFT_KEY)
}

function isStoryCharacterInfo(value: unknown): value is StoryCharacterInfo {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return false
  const input = value as Record<string, unknown>
  return typeof input.id === 'string'
    && input.id.trim().length > 0
    && typeof input.name === 'string'
    && input.name.trim().length > 0
}

export function loadBrowserSceneDrafts(): SceneDefinition[] | null {
  if (typeof window === 'undefined') return null
  const raw = window.localStorage.getItem(BROWSER_SCENE_DRAFT_KEY)
  if (raw === null) return null
  try {
    const value = JSON.parse(raw) as unknown
    if (!Array.isArray(value)) return null
    const scenes = value.filter(isSceneDefinition)
    return scenes.length === value.length ? scenes : null
  } catch {
    return null
  }
}

export function saveBrowserSceneDrafts(scenes: SceneDefinition[]): void {
  if (typeof window === 'undefined') return
  window.localStorage.setItem(BROWSER_SCENE_DRAFT_KEY, JSON.stringify(scenes))
}

export async function loadStoryDialogues(): Promise<StoryDialogueInfo[]> {
  if (hasTauriRuntime()) return invokeCommand<StoryDialogueInfo[]>('list_dialogues')
  const access = await loadStoryContentAccess()
  const browserDrafts = loadBrowserDialogueDrafts()
  if (browserDrafts !== null) {
    return browserDrafts.map((dialogue) => storyDialogueInfo(dialogue, access))
      .sort((left, right) => left.id.localeCompare(right.id))
  }
  try {
    const manifest = await webProjectManifest()
    const documents = await fetchDocuments<DialogueDocument>(manifest.dialogue_files)
    return documents.map((dialogue) => storyDialogueInfo({
      id: dialogue.id,
      title: dialogue.title,
      description: dialogue.description ?? null,
      start_node_id: dialogue.start_node_id,
      nodes: dialogue.nodes || {},
      variables: dialogue.variables || {},
    }, access)).sort((left, right) => left.id.localeCompare(right.id))
  } catch {
    return browserDialogueFallback.map((dialogue) => ({
      ...dialogue,
      access: contentAccess(access, 'dialogue', dialogue.id),
    }))
  }
}

export function loadBrowserDialogueDrafts(): DialogueDefinition[] | null {
  if (typeof window === 'undefined') return null
  const raw = window.localStorage.getItem(BROWSER_DIALOGUE_DRAFT_KEY)
  if (raw === null) return null
  try {
    const value = JSON.parse(raw) as unknown
    if (!Array.isArray(value)) return null
    const dialogues = value.filter(isDialogueDefinition)
    return dialogues.length === value.length ? dialogues : null
  } catch {
    return null
  }
}

export function saveBrowserDialogueDrafts(dialogues: DialogueDefinition[]): void {
  if (typeof window === 'undefined') return
  window.localStorage.setItem(BROWSER_DIALOGUE_DRAFT_KEY, JSON.stringify(dialogues))
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

function isSceneDefinition(value: unknown): value is SceneDefinition {
  if (!value || typeof value !== 'object') return false
  const scene = value as Record<string, unknown>
  return typeof scene.id === 'string'
    && typeof scene.name === 'string'
    && ['background_path', 'model_3d_path', 'bgm_path', 'weather', 'time_of_day']
      .every((field) => scene[field] === undefined || scene[field] === null || typeof scene[field] === 'string')
    && Array.isArray(scene.tags)
    && scene.tags.every((tag) => typeof tag === 'string')
}

function isDialogueDefinition(value: unknown): value is DialogueDefinition {
  if (!value || typeof value !== 'object') return false
  const dialogue = value as Record<string, unknown>
  return typeof dialogue.id === 'string'
    && typeof dialogue.title === 'string'
    && (dialogue.description === null || typeof dialogue.description === 'string')
    && typeof dialogue.start_node_id === 'string'
    && Boolean(dialogue.nodes) && typeof dialogue.nodes === 'object' && !Array.isArray(dialogue.nodes)
    && Boolean(dialogue.variables) && typeof dialogue.variables === 'object' && !Array.isArray(dialogue.variables)
}

function storySceneInfo(
  scene: SceneDefinition,
  source: string,
  access: Awaited<ReturnType<typeof loadStoryContentAccess>>,
): StorySceneInfo {
  return {
    ...scene,
    source,
    background_exists: Boolean(scene.background_path),
    absolute_background_path: null,
    model_3d_exists: Boolean(scene.model_3d_path),
    absolute_model_3d_path: null,
    access: contentAccess(access, 'scene', scene.id),
  }
}

function storyDialogueInfo(
  dialogue: DialogueDefinition,
  access: Awaited<ReturnType<typeof loadStoryContentAccess>>,
): StoryDialogueInfo {
  return {
    ...dialogue,
    node_count: Object.keys(dialogue.nodes).length,
    access: contentAccess(access, 'dialogue', dialogue.id),
  }
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
  model_3d_path: null,
  bgm_path: null,
  weather: null,
  time_of_day: null,
  tags: [],
  source: 'web_bundled_fallback',
  background_exists: true,
  absolute_background_path: null,
  model_3d_exists: false,
  absolute_model_3d_path: null,
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
  description: null,
  start_node_id: 'start',
  node_count: 1,
  nodes: { start: { text: `Browser fallback for ${titleFromId(id)}.`, choices: [] } },
  variables: {},
}))

const browserEndingFallback: Omit<StoryEndingInfo, 'access'>[] = [{
  schema: 'monogatari-story-ending/v1',
  id: 'best_friend_ending',
  title: 'Under the Festival Stars',
  description: 'A quiet promise closes the night after your bond reaches its strongest point.',
  scene_id: 'festival_night',
  dialogue_id: 'observatory_night',
}]

const browserCharacterFallback: StoryCharacterInfo[] = [
  {
    id: 'sakura',
    name: 'Sakura',
    description: 'A cheerful, nature-loving artist.',
    emotion: 'happy',
    portrait_path: 'assets/characters/sakura_portrait.svg',
    sprite_path: 'assets/characters/sakura_sprite.svg',
  },
  {
    id: 'luna',
    name: 'Luna',
    description: 'A quiet observer drawn to the night sky.',
    emotion: 'neutral',
    portrait_path: 'assets/characters/luna_portrait.svg',
    sprite_path: 'assets/characters/luna_sprite.svg',
  },
]
