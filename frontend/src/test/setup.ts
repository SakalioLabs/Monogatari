import { afterEach, vi } from 'vitest'

afterEach(() => {
  document.body.replaceChildren()
  localStorage.clear()
  sessionStorage.clear()
  vi.useRealTimers()
})
