export interface StoryTextPlaybackScheduler {
  setInterval(callback: () => void, delay: number): number
  clearInterval(timerId: number): void
  setTimeout(callback: () => void, delay: number): number
  clearTimeout(timerId: number): void
}

export interface StoryTextPlaybackOptions {
  scheduler: StoryTextPlaybackScheduler
  readTextIntervalMs: () => number
  readAutoAdvanceDelayMs: () => number
  shouldAutoAdvance: () => boolean
  onTextChange: (text: string) => void
  onTypingChange: (isTyping: boolean) => void
  onAutoAdvance: () => void
}

export interface StoryTextPlaybackController {
  start: (text: string) => void
  complete: () => boolean
  cancel: () => void
  dispose: () => void
}

export function storyTextSegments(text: string): string[] {
  if (typeof Intl.Segmenter === 'function') {
    const segmenter = new Intl.Segmenter(undefined, { granularity: 'grapheme' })
    return Array.from(segmenter.segment(text), ({ segment }) => segment)
  }
  return Array.from(text)
}

export function createStoryTextPlaybackController(
  options: StoryTextPlaybackOptions,
): StoryTextPlaybackController {
  let typingTimer: number | null = null
  let autoAdvanceTimer: number | null = null
  let fullText = ''
  let isTyping = false

  function setTyping(nextValue: boolean) {
    if (isTyping === nextValue) return
    isTyping = nextValue
    options.onTypingChange(nextValue)
  }

  function clearTypingTimer() {
    if (typingTimer === null) return
    options.scheduler.clearInterval(typingTimer)
    typingTimer = null
  }

  function clearAutoAdvanceTimer() {
    if (autoAdvanceTimer === null) return
    options.scheduler.clearTimeout(autoAdvanceTimer)
    autoAdvanceTimer = null
  }

  function scheduleAutoAdvance() {
    if (!options.shouldAutoAdvance()) return
    autoAdvanceTimer = options.scheduler.setTimeout(() => {
      autoAdvanceTimer = null
      if (options.shouldAutoAdvance()) options.onAutoAdvance()
    }, normalizedDelay(options.readAutoAdvanceDelayMs(), 0))
  }

  function finishTyping(allowAutoAdvance: boolean) {
    clearTypingTimer()
    setTyping(false)
    if (allowAutoAdvance) scheduleAutoAdvance()
  }

  function cancel() {
    clearTypingTimer()
    clearAutoAdvanceTimer()
    fullText = ''
    setTyping(false)
  }

  function start(text: string) {
    cancel()
    fullText = text
    options.onTextChange('')
    const segments = storyTextSegments(text)
    if (segments.length === 0) {
      scheduleAutoAdvance()
      return
    }

    let visibleSegmentCount = 0
    setTyping(true)
    typingTimer = options.scheduler.setInterval(() => {
      visibleSegmentCount += 1
      options.onTextChange(segments.slice(0, visibleSegmentCount).join(''))
      if (visibleSegmentCount >= segments.length) finishTyping(true)
    }, normalizedDelay(options.readTextIntervalMs(), 1))
  }

  function complete(): boolean {
    if (!isTyping) {
      clearAutoAdvanceTimer()
      return false
    }
    clearAutoAdvanceTimer()
    options.onTextChange(fullText)
    finishTyping(false)
    return true
  }

  return {
    start,
    complete,
    cancel,
    dispose: cancel,
  }
}

function normalizedDelay(value: number, minimum: number): number {
  return Number.isFinite(value) ? Math.max(minimum, Math.round(value)) : minimum
}
