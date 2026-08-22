import { hasTauriRuntime, invokeCommand } from './tauri'
import type { RoleplayCampaignDefinition } from './roleplayCampaign'
import type { SceneRoleplayDefinition } from './sceneRoleplay'
import type { StoryDialogueInfo } from './storyContent'

export type ProjectLaunchTarget =
  | { kind: 'campaign'; id: string }
  | { kind: 'roleplay'; id: string }
  | { kind: 'dialogue'; id: string }

export type ResolvedProjectLaunch =
  | { kind: 'campaign'; definition: RoleplayCampaignDefinition }
  | { kind: 'roleplay'; definition: SceneRoleplayDefinition }
  | { kind: 'dialogue'; definition: StoryDialogueInfo }
  | null

interface ProjectConfigState {
  config?: unknown
}

interface WebProjectManifest {
  schema?: unknown
  launch?: unknown
}

export function parseProjectLaunchTarget(value: unknown): ProjectLaunchTarget | null {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return null
  const candidate = value as Record<string, unknown>
  if (candidate.kind !== 'campaign' && candidate.kind !== 'roleplay' && candidate.kind !== 'dialogue') return null
  if (typeof candidate.id !== 'string' || !/^[A-Za-z0-9_-]{1,128}$/.test(candidate.id)) {
    return null
  }
  return { kind: candidate.kind, id: candidate.id }
}

export function resolveProjectLaunch(
  target: ProjectLaunchTarget | null,
  campaigns: RoleplayCampaignDefinition[],
  roleplays: SceneRoleplayDefinition[],
  dialogues: StoryDialogueInfo[],
): ResolvedProjectLaunch {
  if (target?.kind === 'campaign') {
    const definition = campaigns.find(candidate => candidate.id === target.id)
    if (definition) return { kind: 'campaign', definition }
  }
  if (target?.kind === 'roleplay') {
    const definition = roleplays.find(candidate => candidate.id === target.id)
    if (definition) return { kind: 'roleplay', definition }
  }
  if (target?.kind === 'dialogue') {
    const definition = dialogues.find(candidate => candidate.id === target.id)
    if (definition) return { kind: 'dialogue', definition }
  }
  if (campaigns[0]) return { kind: 'campaign', definition: campaigns[0] }
  if (roleplays[0]) return { kind: 'roleplay', definition: roleplays[0] }
  if (dialogues[0]) return { kind: 'dialogue', definition: dialogues[0] }
  return null
}

export async function loadProjectLaunchTarget(): Promise<ProjectLaunchTarget | null> {
  if (hasTauriRuntime()) {
    const state = await invokeCommand<ProjectConfigState>('get_project_config', {
      projectPath: null,
    })
    return parseProjectLaunchTarget(readSettingsLaunch(state.config))
  }

  const response = await fetch(projectUrl('project-assets.json'), { cache: 'no-cache' })
  if (!response.ok) throw new Error(`Project manifest returned HTTP ${response.status}`)
  const manifest = await response.json() as WebProjectManifest
  if (manifest.schema !== 'monogatari-web-project-assets/v1') {
    throw new Error(`Unsupported project manifest: ${String(manifest.schema)}`)
  }
  return parseProjectLaunchTarget(manifest.launch)
}

function readSettingsLaunch(config: unknown): unknown {
  if (!config || typeof config !== 'object' || Array.isArray(config)) return null
  const play = (config as Record<string, unknown>).play
  if (!play || typeof play !== 'object' || Array.isArray(play)) return null
  return (play as Record<string, unknown>).launch
}

function projectUrl(relativePath: string): string {
  const base = import.meta.env.BASE_URL || '/'
  const baseUrl = base === './' ? new URL('./', window.location.href) : new URL(base, window.location.origin)
  return new URL(relativePath.replace(/^\/+/, ''), baseUrl).toString()
}
