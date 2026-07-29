import { computed, ref } from 'vue'
import { hasTauriRuntime, invokeCommand } from './tauri'

export interface ProjectLauncherEntry {
  project_path: string
  project_title: string
  last_opened_at: string
  available: boolean
}

export interface SampleProjectEntry {
  id: string
  title: string
  description: string
  package_file_name: string
}

export interface ProjectWorkspaceState {
  active_project: ProjectLauncherEntry | null
  recent_projects: ProjectLauncherEntry[]
  sample_projects: SampleProjectEntry[]
}

const browserWorkspace: ProjectWorkspaceState = {
  active_project: {
    project_path: 'Browser distribution',
    project_title: 'Packaged Web Project',
    last_opened_at: new Date(0).toISOString(),
    available: true,
  },
  recent_projects: [],
  sample_projects: [],
}

export const projectWorkspace = ref<ProjectWorkspaceState | null>(null)
export const activeProject = computed(() => projectWorkspace.value?.active_project ?? null)

export async function refreshProjectWorkspace() {
  projectWorkspace.value = await invokeCommand<ProjectWorkspaceState>(
    'get_project_workspace',
    undefined,
    browserWorkspace,
  )
  return projectWorkspace.value
}

export async function ensureActiveProject() {
  if (!hasTauriRuntime()) return true
  const workspace = await refreshProjectWorkspace()
  return Boolean(workspace.active_project)
}

export async function openWorkspaceProject(projectPath: string) {
  const entry = await invokeCommand<ProjectLauncherEntry>('open_project', {
    projectPath,
  })
  await refreshProjectWorkspace()
  return entry
}

export async function createWorkspaceProject(input: {
  parentDirectory: string
  directoryName: string
  projectTitle: string
}) {
  const entry = await invokeCommand<ProjectLauncherEntry>('create_project', input)
  await refreshProjectWorkspace()
  return entry
}

export async function forgetWorkspaceProject(projectPath: string) {
  await invokeCommand<ProjectWorkspaceState>('forget_project', { projectPath })
  return refreshProjectWorkspace()
}

export async function closeWorkspaceProject() {
  await invokeCommand<ProjectWorkspaceState>('close_project')
  return refreshProjectWorkspace()
}
