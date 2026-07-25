import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'

import WhatsNew from '../WhatsNew.vue'

describe('WhatsNew', () => {
  beforeEach(() => {
    localStorage.clear()
    window.history.replaceState({}, '', '/')
  })

  afterEach(() => {
    window.history.replaceState({}, '', '/')
  })

  it('opens for an unseen release during normal navigation', async () => {
    const wrapper = mount(WhatsNew)
    await flushPromises()

    expect(wrapper.find('[role="dialog"]').exists()).toBe(true)
  })

  it('does not cover an authoring playtest', async () => {
    window.history.replaceState({}, '', '/game?previewCampaign=volume1_campaign&authoring=1')

    const wrapper = mount(WhatsNew)
    await flushPromises()

    expect(wrapper.find('[role="dialog"]').exists()).toBe(false)
    expect(localStorage.getItem('monogatari-version-seen')).toBeNull()
  })
})
