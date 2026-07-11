import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import BackToTop from '../BackToTop.vue'
import ConfirmDialog from '../ConfirmDialog.vue'
import LoadingSpinner from '../LoadingSpinner.vue'
import ToastNotification from '../ToastNotification.vue'
import { showToast, toasts } from '../../lib/toast'

describe('shared components', () => {
  beforeEach(() => {
    toasts.value = []
  })

  it('exposes confirm dialog semantics and closes through confirm, cancel, or Escape', async () => {
    const wrapper = mount(ConfirmDialog, {
      props: {
        visible: true,
        title: 'Delete chapter',
        message: 'This cannot be undone.',
      },
    })
    expect(wrapper.get('[role="dialog"]').attributes()).toMatchObject({
      'aria-modal': 'true',
      'aria-label': 'Delete chapter',
    })

    await wrapper.get('.btn-danger').trigger('click')
    expect(wrapper.emitted('confirm')).toHaveLength(1)
    expect(wrapper.emitted('update:visible')?.[0]).toEqual([false])

    await wrapper.setProps({ visible: true })
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    await nextTick()
    expect(wrapper.emitted('cancel')).toHaveLength(1)

    await wrapper.setProps({ visible: true })
    await wrapper.get('.confirm-overlay').trigger('click')
    expect(wrapper.emitted('cancel')).toHaveLength(2)
    wrapper.unmount()
  })

  it('renders an accessible loading state with stable dimensions', () => {
    const wrapper = mount(LoadingSpinner, {
      props: { text: 'Loading scenes', size: 36, thickness: 4, inline: true },
    })
    expect(wrapper.get('[role="status"]').attributes('aria-label')).toBe('Loading scenes')
    expect(wrapper.get('.loading-wrap').classes()).toContain('inline')
    expect(wrapper.get('.loading-spinner').attributes('style')).toContain('width: 36px')
    expect(wrapper.text()).toContain('Loading scenes')
  })

  it('announces toast severity and removes expired notifications', async () => {
    vi.useFakeTimers()
    const wrapper = mount(ToastNotification)
    showToast('Saved', 'success', 500)
    showToast('Failed', 'error', 500)
    await nextTick()

    expect(wrapper.get('.toast-container').attributes('aria-live')).toBe('polite')
    expect(wrapper.findAll('[role="status"]')).toHaveLength(1)
    expect(wrapper.findAll('[role="alert"]')).toHaveLength(1)

    vi.advanceTimersByTime(500)
    await nextTick()
    expect(wrapper.findAll('.toast')).toHaveLength(0)
  })

  it('shows back-to-top after scrolling and requests smooth scrolling', async () => {
    Object.defineProperty(window, 'scrollY', { configurable: true, value: 400 })
    const scrollTo = vi.spyOn(window, 'scrollTo').mockImplementation(() => undefined)
    const wrapper = mount(BackToTop)

    window.dispatchEvent(new Event('scroll'))
    await nextTick()
    await wrapper.get('button').trigger('click')

    expect(scrollTo).toHaveBeenCalledWith({ top: 0, behavior: 'smooth' })
    wrapper.unmount()
  })
})
