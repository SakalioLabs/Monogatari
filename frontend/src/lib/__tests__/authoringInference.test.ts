import { afterEach, describe, expect, it, vi } from 'vitest'

import { generateAuthoringApiChat } from '../authoringInference'

describe('authoring API inference', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('loads a credential-free runtime and sends OpenAI-compatible chat requests', async () => {
    const fetchMock = vi.fn()
      .mockResolvedValueOnce(new Response(JSON.stringify({
        schema: 'monogatari-authoring-inference-runtime/v1',
        provider: 'api',
        endpoint: '/authoring-api/chat/completions',
        model: 'remote-roleplay-model',
        max_new_tokens: 192,
        temperature: 0.75,
        top_p: 0.9,
      }), { status: 200 }))
      .mockResolvedValueOnce(new Response(JSON.stringify({
        choices: [{ message: { content: 'Aqua answers in character.' } }],
      }), { status: 200 }))
    vi.stubGlobal('fetch', fetchMock)
    const onChunk = vi.fn()

    const output = await generateAuthoringApiChat(
      [{ role: 'user', content: 'What do you do?' }],
      { maxNewTokens: 80, temperature: 0.6, onChunk },
    )

    expect(output).toBe('Aqua answers in character.')
    expect(onChunk).toHaveBeenCalledWith('Aqua answers in character.')
    expect(fetchMock).toHaveBeenNthCalledWith(2, '/authoring-api/chat/completions', expect.objectContaining({
      method: 'POST',
      body: JSON.stringify({
        model: 'remote-roleplay-model',
        messages: [{ role: 'user', content: 'What do you do?' }],
        max_tokens: 80,
        temperature: 0.6,
        top_p: 0.9,
        stream: false,
      }),
    }))
    const request = fetchMock.mock.calls[1][1]
    expect(JSON.stringify(request)).not.toMatch(/api[_-]?key|authorization|bearer/i)
  })

  it('retries runtime discovery after a transient unavailable response', async () => {
    vi.resetModules()
    const fetchMock = vi.fn()
      .mockResolvedValueOnce(new Response('', { status: 503 }))
      .mockResolvedValueOnce(new Response(JSON.stringify({
        schema: 'monogatari-authoring-inference-runtime/v1',
        provider: 'api',
        endpoint: '/authoring-api/chat/completions',
        model: 'recovered-roleplay-model',
      }), { status: 200 }))
    vi.stubGlobal('fetch', fetchMock)
    const { loadAuthoringApiRuntime } = await import('../authoringInference')

    await expect(loadAuthoringApiRuntime()).resolves.toBeNull()
    await expect(loadAuthoringApiRuntime()).resolves.toMatchObject({
      provider: 'api',
      model: 'recovered-roleplay-model',
    })
    expect(fetchMock).toHaveBeenCalledTimes(2)
  })

  it('does not call chat completion when the configured API runtime is unreachable', async () => {
    vi.resetModules()
    const fetchMock = vi.fn().mockResolvedValueOnce(new Response(JSON.stringify({
      schema: 'monogatari-authoring-inference-runtime/v1',
      provider: 'api',
      endpoint: '/authoring-api/chat/completions',
      model: 'offline-roleplay-model',
      ready: false,
      issue: 'upstream_unreachable',
    }), { status: 200 }))
    vi.stubGlobal('fetch', fetchMock)
    const { generateAuthoringApiChat: generateUnavailableChat } = await import('../authoringInference')

    await expect(generateUnavailableChat([
      { role: 'user', content: 'Can you hear me?' },
    ])).rejects.toThrow('configured authoring API runtime is unreachable')
    expect(fetchMock).toHaveBeenCalledTimes(1)
  })

  it('rechecks an unreachable configured runtime on the next discovery', async () => {
    vi.resetModules()
    const runtime = {
      schema: 'monogatari-authoring-inference-runtime/v1',
      provider: 'api',
      endpoint: '/authoring-api/chat/completions',
      model: 'recovering-roleplay-model',
    }
    const fetchMock = vi.fn()
      .mockResolvedValueOnce(new Response(JSON.stringify({
        ...runtime,
        ready: false,
        issue: 'upstream_unreachable',
      }), { status: 200 }))
      .mockResolvedValueOnce(new Response(JSON.stringify({
        ...runtime,
        ready: true,
        issue: null,
      }), { status: 200 }))
    vi.stubGlobal('fetch', fetchMock)
    const { loadAuthoringApiRuntime } = await import('../authoringInference')

    await expect(loadAuthoringApiRuntime()).resolves.toMatchObject({ ready: false })
    await expect(loadAuthoringApiRuntime()).resolves.toMatchObject({ ready: true })
    expect(fetchMock).toHaveBeenCalledTimes(2)
  })

  it('honors the shared context budget before calling the remote API', async () => {
    vi.resetModules()
    const fetchMock = vi.fn()
      .mockResolvedValueOnce(new Response(JSON.stringify({
        schema: 'monogatari-authoring-inference-runtime/v1',
        provider: 'api',
        endpoint: '/authoring-api/chat/completions',
        model: 'bounded-roleplay-model',
      }), { status: 200 }))
      .mockResolvedValueOnce(new Response(JSON.stringify({
        choices: [{ message: { content: 'A bounded reply.' } }],
      }), { status: 200 }))
    vi.stubGlobal('fetch', fetchMock)
    const { generateAuthoringApiChat: generateBoundedChat } = await import('../authoringInference')

    await generateBoundedChat([
      { role: 'system', content: `scene:${'a'.repeat(2_000)}` },
      { role: 'user', content: `old:${'b'.repeat(2_000)}` },
      { role: 'assistant', content: `reply:${'c'.repeat(2_000)}` },
      { role: 'user', content: `latest:${'d'.repeat(2_000)}` },
    ], { maxContextCharacters: 1_024 })

    const body = JSON.parse(String(fetchMock.mock.calls[1][1]?.body))
    const characters = body.messages
      .reduce((total: number, message: { content: string }) => total + [...message.content].length, 0)
    expect(characters).toBeLessThanOrEqual(1_024)
    expect(body.messages.at(-1).content).toContain('latest:')
  })

  it('retries memory exhaustion with reduced context and output budgets', async () => {
    vi.resetModules()
    const fetchMock = vi.fn()
      .mockResolvedValueOnce(new Response(JSON.stringify({
        schema: 'monogatari-authoring-inference-runtime/v1',
        provider: 'api',
        endpoint: '/authoring-api/chat/completions',
        model: 'recoverable-roleplay-model',
      }), { status: 200 }))
      .mockResolvedValueOnce(new Response(JSON.stringify({
        error: { message: 'failed to call OrtRun(): std::bad_alloc' },
      }), { status: 500 }))
      .mockResolvedValueOnce(new Response(JSON.stringify({
        choices: [{ message: { content: 'Recovered in-character reply.' } }],
      }), { status: 200 }))
    vi.stubGlobal('fetch', fetchMock)
    const { generateAuthoringApiChat: generateRecoverableChat } = await import('../authoringInference')
    const onReset = vi.fn()
    const onChunk = vi.fn()

    await expect(generateRecoverableChat([
      { role: 'system', content: `scene:${'a'.repeat(5_000)}` },
      { role: 'user', content: `latest:${'b'.repeat(2_000)}` },
    ], {
      maxContextCharacters: 6_000,
      recoveryMaxContextCharacters: 2_000,
      maxNewTokens: 160,
      onReset,
      onChunk,
    })).resolves.toBe('Recovered in-character reply.')

    const initialBody = JSON.parse(String(fetchMock.mock.calls[1][1]?.body))
    const recoveredBody = JSON.parse(String(fetchMock.mock.calls[2][1]?.body))
    expect(initialBody.max_tokens).toBe(160)
    expect(recoveredBody.max_tokens).toBe(48)
    expect(JSON.stringify(recoveredBody.messages).length)
      .toBeLessThan(JSON.stringify(initialBody.messages).length)
    expect(onReset).toHaveBeenCalledTimes(1)
    expect(onChunk).toHaveBeenCalledTimes(1)
  })

  it('does not retry client configuration failures', async () => {
    vi.resetModules()
    const fetchMock = vi.fn()
      .mockResolvedValueOnce(new Response(JSON.stringify({
        schema: 'monogatari-authoring-inference-runtime/v1',
        provider: 'api',
        endpoint: '/authoring-api/chat/completions',
        model: 'protected-roleplay-model',
      }), { status: 200 }))
      .mockResolvedValueOnce(new Response(JSON.stringify({
        error: { message: 'invalid credential' },
      }), { status: 401 }))
    vi.stubGlobal('fetch', fetchMock)
    const { generateAuthoringApiChat: generateProtectedChat } = await import('../authoringInference')

    await expect(generateProtectedChat([
      { role: 'user', content: 'Hello?' },
    ])).rejects.toThrow('invalid credential')
    expect(fetchMock).toHaveBeenCalledTimes(2)
  })
})
