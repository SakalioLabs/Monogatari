import { ref } from 'vue'
import { hasTauriRuntime } from './tauri'

export const PROJECT_ACTIVITY_EVENT = 'project-activity'

export type ProjectActivityOperation = 'open' | 'create' | 'import' | 'initialize'

export type ProjectActivityPhase =
  | 'checking_project'
  | 'loading_content'
  | 'initializing_ai'
  | 'authorizing_assets'
  | 'inspecting_package'
  | 'extracting_package'
  | 'validating_import'
  | 'finalizing_import'
  | 'ready'

export interface ProjectActivity {
  operation: ProjectActivityOperation
  phase: ProjectActivityPhase
  project_path?: string
}

export const activeProjectActivity = ref<ProjectActivity | null>(null)

let activitySequence = 0

/**
 * Keeps a blocking desktop operation visible while the backend validates or loads a project.
 * The listener is scoped to the invoking command so stale Tauri events cannot update a later run.
 */
export async function runProjectActivity<T>(
  initialActivity: ProjectActivity,
  action: () => Promise<T>,
): Promise<T> {
  const sequence = ++activitySequence
  activeProjectActivity.value = initialActivity
  const unlisten = await listenForActivity(initialActivity.operation, sequence)
  await waitForNextPaint()

  try {
    return await action()
  } finally {
    unlisten?.()
    if (sequence === activitySequence) activeProjectActivity.value = null
  }
}

async function listenForActivity(
  operation: ProjectActivityOperation,
  sequence: number,
): Promise<(() => void) | undefined> {
  if (!hasTauriRuntime()) return undefined

  try {
    const { listen } = await import('@tauri-apps/api/event')
    return await listen<ProjectActivity>(PROJECT_ACTIVITY_EVENT, ({ payload }) => {
      if (sequence !== activitySequence || !isProjectActivity(payload) || payload.operation !== operation) return
      activeProjectActivity.value = payload
    })
  } catch {
    // Progress events are optional. The initial activity still makes the wait state visible.
    return undefined
  }
}

function isProjectActivity(value: unknown): value is ProjectActivity {
  if (!value || typeof value !== 'object') return false
  const activity = value as Partial<ProjectActivity>
  return isOperation(activity.operation) && isPhase(activity.phase)
}

function isOperation(value: unknown): value is ProjectActivityOperation {
  return value === 'open' || value === 'create' || value === 'import' || value === 'initialize'
}

function isPhase(value: unknown): value is ProjectActivityPhase {
  return value === 'checking_project'
    || value === 'loading_content'
    || value === 'initializing_ai'
    || value === 'authorizing_assets'
    || value === 'inspecting_package'
    || value === 'extracting_package'
    || value === 'validating_import'
    || value === 'finalizing_import'
    || value === 'ready'
}

function waitForNextPaint(): Promise<void> {
  if (typeof window === 'undefined' || typeof window.requestAnimationFrame !== 'function') {
    return Promise.resolve()
  }
  return new Promise((resolve) => window.requestAnimationFrame(() => resolve()))
}
