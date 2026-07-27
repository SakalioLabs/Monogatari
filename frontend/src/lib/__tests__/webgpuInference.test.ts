import { describe, expect, it } from 'vitest'

import {
  compactWebGpuChatMessages,
  DEFAULT_WEBGPU_RUNTIME_CONFIG,
  isWebGpuMemoryError,
  webGpuMemoryRecoveryProfiles,
} from '../webgpuInference'

describe('WebGPU inference memory policy', () => {
  it('defaults to the bounded instruction model used by packaged and development runtimes', () => {
    expect(DEFAULT_WEBGPU_RUNTIME_CONFIG).toMatchObject({
      modelId: 'onnx-community/Qwen2.5-0.5B-Instruct',
      dtype: 'q4',
      maxNewTokens: 96,
    })
  })

  it('keeps system identity and the latest player turn inside a total character budget', () => {
    const messages = [
      { role: 'system' as const, content: `identity ${'s'.repeat(8_000)}` },
      ...Array.from({ length: 10 }, (_, index) => ({
        role: index % 2 === 0 ? 'user' as const : 'assistant' as const,
        content: `history-${index} ${'h'.repeat(1_500)}`,
      })),
      { role: 'user' as const, content: 'latest player request' },
    ]
    const compacted = compactWebGpuChatMessages(messages, 4_000)
    const total = compacted.reduce((sum, message) => sum + [...message.content].length, 0)

    expect(total).toBeLessThanOrEqual(4_000)
    expect(compacted[0].role).toBe('system')
    expect(compacted[0].content).toContain('identity')
    expect(compacted.at(-1)).toEqual({ role: 'user', content: 'latest player request' })
    expect(compacted.length).toBeLessThan(messages.length)
  })

  it('recognizes the reported ORT allocation failure without classifying unrelated errors', () => {
    expect(isWebGpuMemoryError(new Error(
      'failed to call OrtRun(). ERROR_CODE: 6, ERROR_MESSAGE: std::bad_alloc',
    ))).toBe(true)
    expect(isWebGpuMemoryError('GPU out of memory')).toBe(true)
    expect(isWebGpuMemoryError(new Error('network request failed'))).toBe(false)
  })

  it('reduces both context and output before a final minimum-memory attempt', () => {
    expect(webGpuMemoryRecoveryProfiles(3_000, 96)).toEqual([
      { maxContextCharacters: 3_000, maxNewTokens: 48 },
      { maxContextCharacters: 1_024, maxNewTokens: 24 },
    ])
    expect(webGpuMemoryRecoveryProfiles(1_024, 16)).toEqual([
      { maxContextCharacters: 1_024, maxNewTokens: 16 },
    ])
  })
})
