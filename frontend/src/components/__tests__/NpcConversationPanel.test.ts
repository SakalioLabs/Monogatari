import { flushPromises, mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({
  detectWebGpuSupport: vi.fn(),
  generateWebGpuChat: vi.fn(),
  invokeCommand: vi.fn(),
  loadKnowledgeAuthoringCatalog: vi.fn(),
}))

vi.mock('../../lib/tauri', () => ({
  invokeCommand: mocks.invokeCommand,
}))

vi.mock('../../lib/webgpuInference', () => ({
  detectWebGpuSupport: mocks.detectWebGpuSupport,
  generateWebGpuChat: mocks.generateWebGpuChat,
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
    mocks.loadKnowledgeAuthoringCatalog.mockResolvedValue(knowledgeCatalog)
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
