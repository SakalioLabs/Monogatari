import { beforeEach, describe, expect, it, vi } from 'vitest'

vi.mock('../tauri', () => ({
  hasTauriRuntime: vi.fn(() => false),
  invokeCommand: vi.fn(),
}))

import { hasTauriRuntime, invokeCommand } from '../tauri'
import {
  loadProjectLaunchTarget,
  parseProjectLaunchTarget,
  resolveProjectLaunch,
} from '../projectLaunch'

const campaign = (id: string) => ({ id, entries: [] }) as never
const roleplay = (id: string) => ({ id, nodes: [] }) as never

describe('project launch contract', () => {
  beforeEach(() => {
    vi.mocked(hasTauriRuntime).mockReturnValue(false)
    vi.mocked(invokeCommand).mockReset()
    vi.restoreAllMocks()
  })

  it('accepts only bounded portable campaign and roleplay targets', () => {
    expect(parseProjectLaunchTarget({ kind: 'campaign', id: 'volume6_campaign' }))
      .toEqual({ kind: 'campaign', id: 'volume6_campaign' })
    expect(parseProjectLaunchTarget({ kind: 'roleplay', id: 'chapter2_roleplay' }))
      .toEqual({ kind: 'roleplay', id: 'chapter2_roleplay' })
    expect(parseProjectLaunchTarget({ kind: 'dialogue', id: 'intro' })).toBeNull()
    expect(parseProjectLaunchTarget({ kind: 'campaign', id: '../outside' })).toBeNull()
  })

  it('resolves the configured live target before deterministic catalog fallbacks', () => {
    const campaigns = [campaign('volume1_campaign'), campaign('volume6_campaign')]
    const roleplays = [roleplay('chapter1_roleplay')]

    expect(resolveProjectLaunch(
      { kind: 'campaign', id: 'volume6_campaign' },
      campaigns,
      roleplays,
    )).toEqual({ kind: 'campaign', definition: campaigns[1] })
    expect(resolveProjectLaunch(
      { kind: 'roleplay', id: 'chapter1_roleplay' },
      campaigns,
      roleplays,
    )).toEqual({ kind: 'roleplay', definition: roleplays[0] })
    expect(resolveProjectLaunch(
      { kind: 'campaign', id: 'missing' },
      campaigns,
      roleplays,
    )).toEqual({ kind: 'campaign', definition: campaigns[0] })
  })

  it('loads the web launch target from the project manifest', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => new Response(JSON.stringify({
      schema: 'monogatari-web-project-assets/v1',
      launch: { kind: 'campaign', id: 'volume6_campaign' },
    }), { status: 200 })))

    await expect(loadProjectLaunchTarget())
      .resolves.toEqual({ kind: 'campaign', id: 'volume6_campaign' })
  })

  it('loads the desktop launch target from active project settings', async () => {
    vi.mocked(hasTauriRuntime).mockReturnValue(true)
    vi.mocked(invokeCommand).mockResolvedValue({
      config: {
        play: {
          launch: { kind: 'roleplay', id: 'volume6_chapter2_roleplay' },
        },
      },
    })

    await expect(loadProjectLaunchTarget())
      .resolves.toEqual({ kind: 'roleplay', id: 'volume6_chapter2_roleplay' })
    expect(invokeCommand).toHaveBeenCalledWith('get_project_config', {
      projectPath: null,
    })
  })
})
