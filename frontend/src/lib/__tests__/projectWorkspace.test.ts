import { beforeEach, describe, expect, it, vi } from 'vitest'

const tauri = vi.hoisted(() => ({
  hasTauriRuntime: vi.fn(),
  invokeCommand: vi.fn(),
}))

vi.mock('../tauri', () => tauri)

import {
  createWorkspaceProject,
  ensureActiveProject,
  projectWorkspace,
  refreshProjectWorkspace,
} from '../projectWorkspace'

describe('projectWorkspace', () => {
  beforeEach(() => {
    projectWorkspace.value = null
    tauri.hasTauriRuntime.mockReset()
    tauri.invokeCommand.mockReset()
  })

  it('exposes the packaged project when running in a browser build', async () => {
    tauri.hasTauriRuntime.mockReturnValue(false)
    tauri.invokeCommand.mockImplementation(async (_command, _args, fallback) => fallback)

    const workspace = await refreshProjectWorkspace()

    expect(workspace.active_project?.project_title).toBe('Packaged Web Project')
    expect(await ensureActiveProject()).toBe(true)
  })

  it('creates a desktop project and refreshes the device-local registry', async () => {
    tauri.hasTauriRuntime.mockReturnValue(true)
    const created = {
      project_path: 'C:\\Stories\\new-story',
      project_title: 'New Story',
      last_opened_at: '2026-07-29T00:00:00Z',
      available: true,
    }
    const workspace = {
      active_project: created,
      recent_projects: [created],
      sample_projects: [],
    }
    tauri.invokeCommand
      .mockResolvedValueOnce(created)
      .mockResolvedValueOnce(workspace)

    await expect(createWorkspaceProject({
      parentDirectory: 'C:\\Stories',
      directoryName: 'new-story',
      projectTitle: 'New Story',
    })).resolves.toEqual(created)

    expect(tauri.invokeCommand).toHaveBeenNthCalledWith(1, 'create_project', {
      parentDirectory: 'C:\\Stories',
      directoryName: 'new-story',
      projectTitle: 'New Story',
    })
    expect(projectWorkspace.value).toEqual(workspace)
  })
})
