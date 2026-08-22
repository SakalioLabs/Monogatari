import { beforeEach, describe, expect, it, vi } from 'vitest'

const tauri = vi.hoisted(() => ({
  hasTauriRuntime: vi.fn(),
}))

vi.mock('../tauri', () => tauri)

import { activeProjectActivity, runProjectActivity } from '../projectActivity'

describe('projectActivity', () => {
  beforeEach(() => {
    activeProjectActivity.value = null
    tauri.hasTauriRuntime.mockReset()
    tauri.hasTauriRuntime.mockReturnValue(false)
    vi.stubGlobal('requestAnimationFrame', (callback: FrameRequestCallback) => {
      callback(0)
      return 1
    })
  })

  it('publishes the initial loading state before work starts and clears it afterward', async () => {
    let observedPhase = ''

    await expect(runProjectActivity({
      operation: 'open',
      phase: 'checking_project',
      project_path: 'C:/Stories/large-project',
    }, async () => {
      observedPhase = activeProjectActivity.value?.phase ?? ''
      return 'ready'
    })).resolves.toBe('ready')

    expect(observedPhase).toBe('checking_project')
    expect(activeProjectActivity.value).toBeNull()
  })
})
