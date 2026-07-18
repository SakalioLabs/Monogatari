import { describe, expect, it } from 'vitest'

import type { SettingsDocument } from '../settingsContract'
import {
  buildSettingsConfig,
  createBrowserProjectManifest,
  createBrowserProjectState,
  createBrowserSyncStatus,
  createEmptyEngineStatus,
  createPreviewProjectState,
  formatSettingsBytes,
  getSettingsConfigValue,
  normalizeSettingsSection,
  safeSettingsFileName,
  sanitizeManifestSettings,
  scrubRuntimeSecretSettings,
  scrubRuntimeSecretString,
  setSettingsConfigValue,
} from '../settingsDomain'

const runtime = {
  modelId: 'local/test-model',
  dtype: 'q8' as const,
  maxNewTokens: 384,
  temperature: 0.45,
  topP: 0.82,
}

function secretSamples(): string[] {
  return [
    ['github', 'pat', 'A'.repeat(24)].join('_'),
    ['ghp', 'B'.repeat(24)].join('_'),
    `sk-${'C'.repeat(24)}`,
    `glpat-${'D'.repeat(24)}`,
    `xoxb-${'E'.repeat(24)}`,
    `AIza${'F'.repeat(30)}`,
    `AKIA${'G'.repeat(16)}`,
  ]
}

describe('Settings contracts and preview state', () => {
  it('normalizes persisted sections at the domain boundary', () => {
    expect(normalizeSettingsSection('ai')).toBe('ai')
    expect(normalizeSettingsSection('diagnostics')).toBe('diagnostics')
    expect(normalizeSettingsSection('unknown')).toBe('project')
    expect(normalizeSettingsSection(null)).toBe('project')
  })

  it('provides every required Rust engine-status field', () => {
    expect(createEmptyEngineStatus()).toEqual({
      initialized: false,
      character_count: 0,
      dialogue_count: 0,
      knowledge_count: 0,
      story_event_count: 0,
      story_event_catalog_fingerprint: '',
      applied_story_event_count: 0,
      unlocked_story_content_count: 0,
      story_progress_fingerprint: '',
      ai_engines: [],
      active_ai_engine: null,
    })
  })

  it('derives browser sync readiness without retaining credentials', () => {
    expect(createBrowserSyncStatus('local', '', '')).toMatchObject({
      connected: false,
      status: 'local_clean',
      endpoint_configured: false,
      token_configured: false,
    })
    expect(createBrowserSyncStatus('custom', '', 'runtime-only')).toMatchObject({
      connected: false,
      status: 'endpoint_missing',
      token_configured: true,
    })
    expect(createBrowserSyncStatus('custom', 'https://sync.invalid', '')).toMatchObject({
      connected: false,
      status: 'token_missing',
      endpoint_configured: true,
    })
    const ready = createBrowserSyncStatus('custom', 'https://sync.invalid', 'runtime-only')
    expect(ready).toMatchObject({ connected: true, status: 'remote_ready' })
    expect(JSON.stringify(ready)).not.toContain('runtime-only')
  })

  it('creates isolated preview state from the packaged WebGPU runtime', () => {
    const first = createPreviewProjectState(runtime)
    const second = createPreviewProjectState(runtime)

    expect(first.paths.map(({ key }) => key)).toEqual([
      'characters', 'dialogue', 'roleplays', 'knowledge', 'scenes', 'assets', 'events', 'endings', 'saves', 'quality_suites',
    ])
    expect(getSettingsConfigValue(first.config, ['ai', 'webgpu'])).toEqual({
      model_id: runtime.modelId,
      dtype: runtime.dtype,
      max_new_tokens: runtime.maxNewTokens,
      temperature: runtime.temperature,
      top_p: runtime.topP,
    })

    setSettingsConfigValue(first.config, ['paths', 'characters'], 'changed')
    first.paths[0].relative_path = 'changed'
    expect(getSettingsConfigValue(second.config, ['paths', 'characters'])).toBe('characters')
    expect(second.paths[0].relative_path).toBe('characters')
  })
})

describe('Settings document operations', () => {
  it('reads and writes nested values while rejecting unsafe paths', () => {
    const config: SettingsDocument = {}
    setSettingsConfigValue(config, ['ai', 'webgpu', 'model_id'], 'local/model')
    expect(getSettingsConfigValue(config, ['ai', 'webgpu', 'model_id'])).toBe('local/model')
    expect(getSettingsConfigValue(config, ['ai', 'missing'])).toBeUndefined()

    expect(() => setSettingsConfigValue(config, [], true)).toThrow('cannot be empty')
    for (const segment of ['', 'bad.path', '__proto__', 'prototype', 'constructor']) {
      expect(() => setSettingsConfigValue(config, ['ai', segment, 'polluted'], true)).toThrow('Unsafe')
    }
    expect(({} as Record<string, unknown>).polluted).toBeUndefined()
  })

  it('builds canonical persisted settings without mutating source data', () => {
    const samples = secretSamples()
    const source: SettingsDocument = {
      custom: { preserved: true, note: `credential=${samples[0]}` },
      auth: { accessToken: samples[1] },
      engine: { target_fps: 30 },
      paths: { custom: 'custom-content' },
    }
    const before = JSON.parse(JSON.stringify(source))
    const config = buildSettingsConfig(source, {
      projectTitle: 'Updated project',
      targetFps: Number.NaN,
      provider: 'webgpu',
      apiBaseUrl: 'https://api.invalid/v1',
      apiModel: 'test-model',
      modelPath: 'models/test.onnx',
      tokenizerPath: 'models/tokenizer.json',
      webGpuModelId: 'local/webgpu-model',
      webGpuDtype: 'fp16',
      webGpuMaxNewTokens: 0,
      paths: { characters: 'cast', dialogue: 'script' },
    })

    expect(source).toEqual(before)
    expect(config.custom).toEqual({ preserved: true, note: 'credential=<redacted>' })
    expect(config.auth).toEqual({ accessToken: '' })
    expect(getSettingsConfigValue(config, ['render', 'title'])).toBe('Updated project')
    expect(getSettingsConfigValue(config, ['engine', 'target_fps'])).toBe(60)
    expect(getSettingsConfigValue(config, ['ai', 'api', 'api_key'])).toBe('')
    expect(getSettingsConfigValue(config, ['ai', 'onnx', 'use_directml'])).toBe(true)
    expect(getSettingsConfigValue(config, ['ai', 'webgpu', 'max_new_tokens'])).toBe(256)
    expect(getSettingsConfigValue(config, ['paths', 'custom'])).toBe('custom-content')
    expect(getSettingsConfigValue(config, ['paths', 'characters'])).toBe('cast')
    for (const sample of samples) expect(JSON.stringify(config)).not.toContain(sample)
  })

  it('derives provider warnings and path edits from an isolated browser copy', () => {
    const preview = createPreviewProjectState(runtime)
    const apiConfig: SettingsDocument = {
      ai: { provider: 'api', api: { base_url: '' } },
      paths: { characters: 'cast' },
    }
    const apiState = createBrowserProjectState(apiConfig, preview)
    expect(apiState.issues.map(({ code }) => code)).toEqual(['api_base_url_missing', 'api_key_missing'])
    expect(apiState.warning_count).toBe(2)
    expect(apiState.paths.find(({ key }) => key === 'characters')?.relative_path).toBe('cast')
    setSettingsConfigValue(apiState.config, ['ai', 'provider'], 'webgpu')
    expect(getSettingsConfigValue(apiConfig, ['ai', 'provider'])).toBe('api')

    expect(createBrowserProjectState({ ai: { provider: 'onnx', onnx: { model_path: '' } } }, preview)
      .issues.map(({ code }) => code)).toEqual(['onnx_model_missing'])
    expect(createBrowserProjectState({ ai: { provider: 'webgpu', webgpu: { model_id: '' } } }, preview)
      .issues.map(({ code }) => code)).toEqual(['webgpu_model_missing'])
  })
})

describe('Settings secret boundaries and exports', () => {
  it('scrubs nested keys, token formats, authorization values, JWTs, and assignments', () => {
    const samples = secretSamples()
    const bearer = `Bearer ${'H'.repeat(24)}`
    const jwt = `eyJ${'I'.repeat(12)}.${'J'.repeat(12)}.${'K'.repeat(12)}`
    const input = {
      api_key: samples[0],
      nested: [{ password: 'plain-password' }, { note: [...samples, bearer, jwt].join(' ') }],
    }
    const scrubbed = scrubRuntimeSecretSettings(input)

    expect(input.api_key).toBe(samples[0])
    expect(scrubbed).toEqual({
      api_key: '',
      nested: [{ password: '' }, { note: '<redacted> <redacted> <redacted> <redacted> <redacted> <redacted> <redacted> Bearer <redacted> <redacted>' }],
    })
    expect(scrubRuntimeSecretString('api_key="alpha" password: beta; token=gamma'))
      .toBe('api_key="<redacted>" password: <redacted>; token=<redacted>')
  })

  it('redacts manifest credentials without mutating settings', () => {
    const sample = secretSamples()[2]
    const settings = {
      ai: { api: { api_key: sample }, diagnostic: `failed with ${sample}` },
      cloud: { authorization: 'runtime-only' },
    }
    const sanitized = sanitizeManifestSettings(settings)

    expect(settings.ai.api.api_key).toBe(sample)
    expect(sanitized).toEqual({
      ai: { api: { api_key: '<redacted>' }, diagnostic: 'failed with <redacted>' },
      cloud: { authorization: '<redacted>' },
    })
  })

  it('creates a stable schema-backed browser manifest with redacted settings', () => {
    const manifest = createBrowserProjectManifest({
      exportedAt: '2026-07-15T00:00:00.000Z',
      projectPath: 'Browser preview',
      settings: { ai: { api: { api_key: 'runtime-only' } } },
    })

    expect(manifest).toMatchObject({
      format: 'monogatari-project',
      schema: 'monogatari-project-export@1',
      exported_at: '2026-07-15T00:00:00.000Z',
      project_path: 'Browser preview',
      settings: { ai: { api: { api_key: '<redacted>' } } },
      content_summary: {
        schema: 'monogatari-project-content-summary/v1',
        category_fingerprint_algorithm: 'sha256:path-size-file-sha256-v1',
      },
      package: {
        fingerprint_algorithm: 'sha256:path-size-file-sha256-v1',
        content_sha256: 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
      },
      archive: { format: 'zip', manifest_path: 'monogatari-project.json', extension: '.monogatari' },
    })
    expect(manifest.content_summary.exported_categories).toEqual([
      'characters', 'dialogue', 'roleplays', 'knowledge', 'scenes', 'assets', 'events', 'endings', 'locales', 'quality_suites', 'workflows',
    ])
    expect(manifest.package.directories).toEqual([
      'assets', 'characters', 'dialogue', 'endings', 'events', 'knowledge', 'locales', 'quality_suites', 'roleplays', 'scenes', 'workflows',
    ])
  })

  it('formats portable filenames and bounded byte values', () => {
    expect(safeSettingsFileName('  Project: Alpha / Beta  ')).toBe('Project-Alpha-Beta')
    expect(safeSettingsFileName('***')).toBe('monogatari-project')
    expect(formatSettingsBytes(Number.NaN)).toBe('0 B')
    expect(formatSettingsBytes(-1)).toBe('0 B')
    expect(formatSettingsBytes(999)).toBe('999 B')
    expect(formatSettingsBytes(1024)).toBe('1.0 KiB')
    expect(formatSettingsBytes(10 * 1024)).toBe('10 KiB')
    expect(formatSettingsBytes(1.5 * 1024 * 1024)).toBe('1.5 MiB')
  })
})
