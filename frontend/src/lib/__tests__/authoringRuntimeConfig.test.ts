import { describe, expect, it } from 'vitest'
import { resolveAuthoringApiKey } from '../authoringRuntimeConfig'

describe('resolveAuthoringApiKey', () => {
  it('uses the documented AI runtime credential', () => {
    expect(resolveAuthoringApiKey({
      MONOGATARI_AI_API_KEY: '  current-key  ',
      MONOGATARI_API_KEY: 'legacy-key',
    })).toBe('current-key')
  })

  it('retains the legacy credential as a compatibility fallback', () => {
    expect(resolveAuthoringApiKey({
      MONOGATARI_API_KEY: ' legacy-key ',
    })).toBe('legacy-key')
  })

  it('returns an empty credential when the runtime is not configured', () => {
    expect(resolveAuthoringApiKey({})).toBe('')
  })
})
