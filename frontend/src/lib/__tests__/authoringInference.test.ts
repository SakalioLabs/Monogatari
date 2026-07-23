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
})
