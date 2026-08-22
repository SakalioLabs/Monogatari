import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import ProjectActivityOverlay from '../ProjectActivityOverlay.vue'
import { activeProjectActivity } from '../../lib/projectActivity'

describe('ProjectActivityOverlay', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    vi.setSystemTime(new Date('2026-08-22T00:00:00Z'))
    activeProjectActivity.value = null
  })

  afterEach(() => {
    activeProjectActivity.value = null
    vi.useRealTimers()
  })

  it('announces loading progress and retains elapsed time across phases', async () => {
    activeProjectActivity.value = {
      operation: 'open',
      phase: 'checking_project',
      project_path: 'C:/Stories/large-project',
    }
    const wrapper = mount(ProjectActivityOverlay)
    await nextTick()

    expect(wrapper.get('[role="status"]').text()).toContain('Opening project')
    expect(wrapper.get('[role="status"]').text()).toContain('Checking project files')

    vi.advanceTimersByTime(2_000)
    await nextTick()
    activeProjectActivity.value = {
      operation: 'open',
      phase: 'loading_content',
      project_path: 'C:/Stories/large-project',
    }
    await nextTick()

    expect(wrapper.get('[role="status"]').text()).toContain('Loading story content and references')
    expect(wrapper.get('[role="status"]').text()).toContain('2s elapsed')
    wrapper.unmount()
  })
})
