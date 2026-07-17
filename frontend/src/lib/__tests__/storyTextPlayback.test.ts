import { describe, expect, it } from 'vitest'

import {
  createStoryTextPlaybackController,
  storyTextSegments,
  type StoryTextPlaybackScheduler,
} from '../storyTextPlayback'

class PlaybackScheduler implements StoryTextPlaybackScheduler {
  private nextId = 1
  private readonly intervals = new Map<number, () => void>()
  private readonly timeouts = new Map<number, () => void>()

  setInterval(callback: () => void): number {
    const timerId = this.nextId++
    this.intervals.set(timerId, callback)
    return timerId
  }

  clearInterval(timerId: number) {
    this.intervals.delete(timerId)
  }

  setTimeout(callback: () => void): number {
    const timerId = this.nextId++
    this.timeouts.set(timerId, callback)
    return timerId
  }

  clearTimeout(timerId: number) {
    this.timeouts.delete(timerId)
  }

  tickIntervals() {
    for (const [timerId, callback] of [...this.intervals]) {
      if (this.intervals.has(timerId)) callback()
    }
  }

  flushTimeouts() {
    for (const [timerId, callback] of [...this.timeouts]) {
      if (!this.timeouts.delete(timerId)) continue
      callback()
    }
  }

  get intervalCount(): number {
    return this.intervals.size
  }

  get timeoutCount(): number {
    return this.timeouts.size
  }
}

function playbackFixture() {
  const scheduler = new PlaybackScheduler()
  const state = {
    text: '',
    typing: false,
    typingEvents: [] as boolean[],
    autoAdvance: true,
    autoAdvanceCount: 0,
  }
  const controller = createStoryTextPlaybackController({
    scheduler,
    readTextIntervalMs: () => 30,
    readAutoAdvanceDelayMs: () => 3_000,
    shouldAutoAdvance: () => state.autoAdvance,
    onTextChange: (text) => { state.text = text },
    onTypingChange: (typing) => {
      state.typing = typing
      state.typingEvents.push(typing)
    },
    onAutoAdvance: () => { state.autoAdvanceCount += 1 },
  })
  return { controller, scheduler, state }
}

describe('story text playback', () => {
  it('segments complete graphemes for multilingual typewriter playback', () => {
    expect(storyTextSegments('A\u{1F469}\u200D\u{1F4BB}e\u0301\u6F9C')).toEqual([
      'A',
      '\u{1F469}\u200D\u{1F4BB}',
      'e\u0301',
      '\u6F9C',
    ])
  })

  it('types one grapheme per tick and schedules auto advance after completion', () => {
    const { controller, scheduler, state } = playbackFixture()
    controller.start('ABC')

    expect(state).toMatchObject({ text: '', typing: true })
    scheduler.tickIntervals()
    expect(state.text).toBe('A')
    scheduler.tickIntervals()
    expect(state.text).toBe('AB')
    scheduler.tickIntervals()

    expect(state).toMatchObject({ text: 'ABC', typing: false })
    expect(state.typingEvents).toEqual([true, false])
    expect(scheduler.intervalCount).toBe(0)
    expect(scheduler.timeoutCount).toBe(1)
    scheduler.flushTimeouts()
    expect(state.autoAdvanceCount).toBe(1)
  })

  it('completes the active line without scheduling automatic advancement', () => {
    const { controller, scheduler, state } = playbackFixture()
    controller.start('Complete this line')
    scheduler.tickIntervals()

    expect(controller.complete()).toBe(true)
    expect(state.text).toBe('Complete this line')
    expect(state.typing).toBe(false)
    expect(scheduler.intervalCount).toBe(0)
    expect(scheduler.timeoutCount).toBe(0)
    expect(controller.complete()).toBe(false)
  })

  it('cancels stale timers when a new line starts', () => {
    const { controller, scheduler, state } = playbackFixture()
    controller.start('Old')
    scheduler.tickIntervals()
    expect(state.text).toBe('O')

    controller.start('New')
    expect(state.text).toBe('')
    expect(scheduler.intervalCount).toBe(1)
    scheduler.tickIntervals()
    expect(state.text).toBe('N')
  })

  it('rechecks autoplay policy and clears every timer on cancellation or disposal', () => {
    const { controller, scheduler, state } = playbackFixture()
    controller.start('A')
    scheduler.tickIntervals()
    expect(scheduler.timeoutCount).toBe(1)
    state.autoAdvance = false
    scheduler.flushTimeouts()
    expect(state.autoAdvanceCount).toBe(0)

    state.autoAdvance = true
    controller.start('B')
    scheduler.tickIntervals()
    expect(scheduler.timeoutCount).toBe(1)
    expect(controller.complete()).toBe(false)
    expect(scheduler.timeoutCount).toBe(0)

    controller.start('B')
    scheduler.tickIntervals()
    controller.cancel()
    expect(scheduler.timeoutCount).toBe(0)

    controller.start('C')
    controller.dispose()
    expect(scheduler.intervalCount).toBe(0)
    expect(scheduler.timeoutCount).toBe(0)
    expect(state.typing).toBe(false)
  })
})
