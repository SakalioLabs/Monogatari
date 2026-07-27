import { describe, expect, it } from 'vitest'
import {
  classifyAuthoringApiRuntimeHealth,
  resolveAuthoringApiKey,
  resolveAuthoringApiRuntime,
} from '../authoringRuntimeConfig'

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

describe('classifyAuthoringApiRuntimeHealth', () => {
  it.each([
    ['', null, false, 'credential_missing'],
    ['runtime-key', null, false, 'upstream_unreachable'],
    ['runtime-key', 200, true, null],
    ['runtime-key', 204, true, null],
    ['runtime-key', 401, false, 'authentication_failed'],
    ['runtime-key', 403, false, 'authentication_failed'],
    ['runtime-key', 404, false, 'upstream_rejected'],
    ['runtime-key', 500, false, 'upstream_rejected'],
  ] as const)(
    'classifies credential=%s status=%s',
    (credential, status, ready, issue) => {
      expect(classifyAuthoringApiRuntimeHealth(credential, status)).toEqual({ ready, issue })
    },
  )
})

describe('resolveAuthoringApiRuntime', () => {
  it('uses explicit development runtime overrides without exposing the credential', () => {
    expect(resolveAuthoringApiRuntime({
      MONOGATARI_AI_BASE_URL: ' http://127.0.0.1:8317/v1 ',
      MONOGATARI_AI_MODEL: ' live-roleplay-model ',
      MONOGATARI_AI_API_KEY: 'server-only-secret',
    }, {
      provider: 'onnx',
      api: {
        model: 'packaged-default',
        maxTokens: 320,
        temperature: 0.6,
        topP: 0.85,
      },
    })).toEqual({
      baseUrl: 'http://127.0.0.1:8317/v1',
      model: 'live-roleplay-model',
      maxNewTokens: 320,
      temperature: 0.6,
      topP: 0.85,
    })
  })

  it('uses project API settings when no development override is present', () => {
    expect(resolveAuthoringApiRuntime({}, {
      provider: 'api',
      api: {
        baseUrl: 'https://example.test/v1',
        model: 'story-model',
      },
    })).toMatchObject({
      baseUrl: 'https://example.test/v1',
      model: 'story-model',
    })
  })

  it('does not silently enable an API runtime for an ONNX project', () => {
    expect(resolveAuthoringApiRuntime({}, {
      provider: 'onnx',
      api: {
        baseUrl: 'https://example.test/v1',
        model: 'unused',
      },
    })).toBeNull()
  })
})
