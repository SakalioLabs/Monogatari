import { flushPromises, mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({
  detectWebGpuSupport: vi.fn(),
  generateAuthoringApiChat: vi.fn(),
  generateWebGpuChat: vi.fn(),
  invokeCommand: vi.fn(),
  loadAuthoringApiRuntime: vi.fn(),
  loadKnowledgeAuthoringCatalog: vi.fn(),
}))

vi.mock('../../lib/tauri', () => ({
  invokeCommand: mocks.invokeCommand,
}))

vi.mock('../../lib/webgpuInference', () => ({
  detectWebGpuSupport: mocks.detectWebGpuSupport,
  generateWebGpuChat: mocks.generateWebGpuChat,
}))

vi.mock('../../lib/authoringInference', () => ({
  generateAuthoringApiChat: mocks.generateAuthoringApiChat,
  loadAuthoringApiRuntime: mocks.loadAuthoringApiRuntime,
}))

vi.mock('../../lib/knowledgeContent', () => ({
  loadKnowledgeAuthoringCatalog: mocks.loadKnowledgeAuthoringCatalog,
}))

import NpcConversationPanel from '../NpcConversationPanel.vue'
import type { StoryCharacterInfo } from '../../lib/storyContent'

const character: StoryCharacterInfo = {
  id: 'echo_nine',
  name: '九号回声',
  description: '从四十七年后的潮汐干涉中传来的求救声音。',
  background: '由九份幸存者语音样本重建。',
  emotion: 'fragmented',
  personality: { speech_style: '短促的感官碎片' },
  portrait_path: null,
  sprite_path: null,
  knowledge_refs: ['echo_protocol'],
}

const knowledgeCatalog = {
  schema: 'monogatari-knowledge-authoring/v1',
  catalog_fingerprint: 'fixture',
  entries: [{
    id: 'echo_protocol',
    category: 'lore',
    title: '九号回声协议',
    content: '讯号只能传递坐标和少量记忆。',
    tags: [],
    importance: 1,
    metadata: {},
    related_entries: [],
  }],
}

describe('NpcConversationPanel', () => {
  beforeEach(() => {
    mocks.detectWebGpuSupport.mockReturnValue({ available: true, reason: 'available' })
    mocks.loadAuthoringApiRuntime.mockResolvedValue(null)
    mocks.loadKnowledgeAuthoringCatalog.mockResolvedValue(knowledgeCatalog)
    mocks.generateAuthoringApiChat.mockReset()
    mocks.generateWebGpuChat.mockReset()
    mocks.invokeCommand.mockReset()
  })

  it('runs an in-story browser conversation with pinned character context', async () => {
    mocks.generateWebGpuChat.mockImplementation(async (_messages, options) => {
      options.onChunk('<think>隐藏推理</think>雨声还在。')
      return '<think>隐藏推理</think>雨声还在。'
    })
    const wrapper = mount(NpcConversationPanel, {
      props: { open: true, character, desktopRuntime: false, locale: 'zh-CN' },
    })
    await flushPromises()

    expect(wrapper.get('[data-testid="npc-panel"]').attributes('data-npc-runtime')).toBe('webgpu')
    expect(wrapper.text()).toContain('九号回声')
    await wrapper.get('[data-testid="npc-input"]').setValue('你还记得什么？')
    await wrapper.get('[data-testid="npc-send"]').trigger('click')
    await flushPromises()

    const modelMessages = mocks.generateWebGpuChat.mock.calls[0][0]
    expect(modelMessages[0].content).toContain('九号回声协议')
    expect(modelMessages.at(-1)).toEqual({ role: 'user', content: '你还记得什么？' })
    expect(wrapper.text()).toContain('雨声还在。')
    expect(wrapper.text()).not.toContain('隐藏推理')
    expect(wrapper.findAll('[data-npc-message-role]')).toHaveLength(2)
    expect(wrapper.emitted('emotion')?.[0]).toEqual(['fragmented'])
  })

  it('reuses the guarded desktop command and reports story-state evidence', async () => {
    mocks.invokeCommand.mockImplementation(async (command) => {
      if (command === 'get_chat_history') {
        return [{ role: 'character', content: '旧讯号仍在。', emotion: 'distant', timestamp: '1' }]
      }
      if (command === 'send_chat_message') {
        return {
          character_response: '坐标没有改变。',
          emotion: 'steady',
          relationship_delta: 0.05,
          event_applications: [{}],
          safety_trace: { pinned_knowledge_ref_count: 5, response_guard_applied: true },
        }
      }
      return undefined
    })
    const wrapper = mount(NpcConversationPanel, {
      props: { open: true, character, desktopRuntime: true, locale: 'zh-CN' },
    })
    await flushPromises()

    expect(wrapper.text()).toContain('旧讯号仍在。')
    await wrapper.get('[data-testid="npc-input"]').setValue('确认坐标。')
    await wrapper.get('[data-testid="npc-send"]').trigger('click')
    await flushPromises()

    expect(mocks.invokeCommand).toHaveBeenCalledWith('send_chat_message', {
      characterId: 'echo_nine',
      message: '确认坐标。',
    })
    expect(wrapper.text()).toContain('坐标没有改变。')
    expect(wrapper.text()).toContain('5 pinned knowledge refs')
    expect(wrapper.emitted('emotion')?.[0]).toEqual(['steady'])
    expect(wrapper.emitted('storyProgress')).toHaveLength(1)

    await wrapper.get('[data-testid="npc-clear"]').trigger('click')
    await flushPromises()
    expect(mocks.invokeCommand).toHaveBeenCalledWith('clear_chat_history', { characterId: 'echo_nine' })
  })

  it('keeps a chapter free-talk turn outside the persistent NPC and story channels', async () => {
    mocks.invokeCommand.mockResolvedValue({
      character_response: '先把这段路走完，之后再问我。',
      emotion: 'steady',
      used_fallback: false,
    })
    const wrapper = mount(NpcConversationPanel, {
      props: {
        open: true,
        character,
        desktopRuntime: true,
        locale: 'zh-CN',
        containedTalk: {
          character_id: 'echo_nine',
          context: '玩家正在穿过潮汐站的外廊。',
          title: '章节短谈',
          opening_text: '别停下，潮声会掩盖脚步。',
          fallback_text: '她望向潮水，没有再补充。',
          max_turns: 1,
          max_characters: 80,
        },
      },
    })
    await flushPromises()

    expect(wrapper.text()).toContain('别停下，潮声会掩盖脚步。')
    await wrapper.get('[data-testid="npc-input"]').setValue('你害怕吗？')
    await wrapper.get('[data-testid="npc-send"]').trigger('click')
    await flushPromises()

    expect(mocks.invokeCommand).toHaveBeenCalledWith('send_dialogue_free_talk_message', {
      message: '你害怕吗？',
    })
    expect(mocks.invokeCommand).not.toHaveBeenCalledWith('send_chat_message', expect.anything())
    expect(wrapper.text()).toContain('先把这段路走完，之后再问我。')
    expect(wrapper.emitted('storyProgress')).toBeUndefined()
    expect(wrapper.get('[data-testid="npc-input"]').attributes()).toHaveProperty('disabled')

    await wrapper.get('.npc-return').trigger('click')
    expect(wrapper.emitted('close')).toHaveLength(1)
  })

  it('prefers the configured API runtime over the local browser model', async () => {
    mocks.loadAuthoringApiRuntime.mockResolvedValue({
      schema: 'monogatari-authoring-inference-runtime/v1',
      provider: 'api',
      endpoint: '/authoring-api/chat/completions',
      model: 'story-model',
      max_new_tokens: 96,
      temperature: 0.7,
      top_p: 0.9,
    })
    mocks.generateAuthoringApiChat.mockResolvedValue('The signal is still here.')
    const wrapper = mount(NpcConversationPanel, {
      props: { open: true, character, desktopRuntime: false, locale: 'en' },
    })
    await flushPromises()

    expect(wrapper.get('[data-testid="npc-panel"]').attributes('data-npc-runtime')).toBe('api')
    await wrapper.get('[data-testid="npc-input"]').setValue('Can you hear me?')
    await wrapper.get('[data-testid="npc-send"]').trigger('click')
    await flushPromises()

    expect(mocks.generateAuthoringApiChat).toHaveBeenCalledOnce()
    expect(mocks.generateWebGpuChat).not.toHaveBeenCalled()
    expect(wrapper.text()).toContain('The signal is still here.')
  })

  it('blocks an unreachable configured API without falling back to WebGPU', async () => {
    mocks.loadAuthoringApiRuntime.mockResolvedValue({
      schema: 'monogatari-authoring-inference-runtime/v1',
      provider: 'api',
      endpoint: '/authoring-api/chat/completions',
      model: 'offline-roleplay-model',
      ready: false,
      issue: 'upstream_unreachable',
      max_new_tokens: 96,
      temperature: 0.7,
      top_p: 0.9,
    })
    const wrapper = mount(NpcConversationPanel, {
      props: { open: true, character, desktopRuntime: false, locale: 'en' },
    })
    await flushPromises()

    expect(wrapper.get('[data-testid="npc-panel"]').attributes('data-npc-runtime')).toBe('api')
    expect(wrapper.get('[role="alert"]').text()).toContain('offline-roleplay-model')
    expect(wrapper.get('[data-testid="npc-input"]').attributes()).toHaveProperty('disabled')
    expect(wrapper.get('[data-testid="npc-send"]').attributes()).toHaveProperty('disabled')
    expect(mocks.generateAuthoringApiChat).not.toHaveBeenCalled()
    expect(mocks.generateWebGpuChat).not.toHaveBeenCalled()
  })

  it('rechecks the API runtime before falling back to WebGPU', async () => {
    const apiRuntime = {
      schema: 'monogatari-authoring-inference-runtime/v1' as const,
      provider: 'api' as const,
      endpoint: '/authoring-api/chat/completions',
      model: 'story-model',
      max_new_tokens: 96,
      temperature: 0.7,
      top_p: 0.9,
    }
    mocks.loadAuthoringApiRuntime
      .mockResolvedValueOnce(null)
      .mockResolvedValue(apiRuntime)
    mocks.generateAuthoringApiChat.mockResolvedValue('The remote signal recovered.')
    const wrapper = mount(NpcConversationPanel, {
      props: { open: true, character, desktopRuntime: false, locale: 'en' },
    })
    await flushPromises()

    expect(wrapper.get('[data-testid="npc-panel"]').attributes('data-npc-runtime')).toBe('webgpu')
    await wrapper.get('[data-testid="npc-input"]').setValue('Try the signal again.')
    await wrapper.get('[data-testid="npc-send"]').trigger('click')
    await flushPromises()

    expect(mocks.loadAuthoringApiRuntime).toHaveBeenCalledTimes(2)
    expect(mocks.generateAuthoringApiChat).toHaveBeenCalledOnce()
    expect(mocks.generateWebGpuChat).not.toHaveBeenCalled()
    expect(wrapper.text()).toContain('The remote signal recovered.')
  })

  it('rolls back an out-of-memory browser turn and restores the retryable input', async () => {
    mocks.generateWebGpuChat.mockRejectedValue(
      new Error('failed to call OrtRun(). ERROR_CODE: 6, ERROR_MESSAGE: std::bad_alloc'),
    )
    const wrapper = mount(NpcConversationPanel, {
      props: { open: true, character, desktopRuntime: false, locale: 'zh-CN' },
    })
    await flushPromises()

    await wrapper.get('[data-testid="npc-input"]').setValue('请再确认一次坐标。')
    await wrapper.get('[data-testid="npc-send"]').trigger('click')
    await flushPromises()

    expect(wrapper.findAll('[data-npc-message-role]')).toHaveLength(0)
    expect((wrapper.get('[data-testid="npc-input"]').element as HTMLTextAreaElement).value)
      .toBe('请再确认一次坐标。')
    expect(wrapper.get('[role="alert"]').text()).toContain('not committed')
    expect(wrapper.get('[role="alert"]').text()).not.toContain('OrtRun')
    expect(wrapper.get('[role="alert"]').text()).not.toContain('std::bad_alloc')
  })

  it('disables the composer when WebGPU is unavailable', async () => {
    mocks.detectWebGpuSupport.mockReturnValue({ available: false, reason: 'webgpu-unavailable' })
    const wrapper = mount(NpcConversationPanel, {
      props: { open: true, character, desktopRuntime: false, locale: 'en' },
    })
    await flushPromises()

    expect(wrapper.get('[role="alert"]').text()).toContain('WebGPU is unavailable')
    expect(wrapper.get('[data-testid="npc-input"]').attributes()).toHaveProperty('disabled')
    expect(wrapper.get('[data-testid="npc-send"]').attributes()).toHaveProperty('disabled')
  })
})
