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
})
