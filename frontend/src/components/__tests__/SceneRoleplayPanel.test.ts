import { flushPromises, mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({
  detectWebGpuSupport: vi.fn(),
  generateWebGpuChat: vi.fn(),
  generateAuthoringApiChat: vi.fn(),
  loadAuthoringApiRuntime: vi.fn(),
  loadKnowledgeAuthoringCatalog: vi.fn(),
}))

vi.mock('../../lib/webgpuInference', () => ({
  detectWebGpuSupport: mocks.detectWebGpuSupport,
  generateWebGpuChat: mocks.generateWebGpuChat,
}))

vi.mock('../../lib/knowledgeContent', () => ({
  loadKnowledgeAuthoringCatalog: mocks.loadKnowledgeAuthoringCatalog,
}))

vi.mock('../../lib/authoringInference', () => ({
  generateAuthoringApiChat: mocks.generateAuthoringApiChat,
  loadAuthoringApiRuntime: mocks.loadAuthoringApiRuntime,
}))

import SceneRoleplayPanel from '../SceneRoleplayPanel.vue'
import {
  startBrowserSceneRoleplay,
  type SceneRoleplayDefinition,
} from '../../lib/sceneRoleplay'
import type { StoryCharacterInfo } from '../../lib/storyContent'

const definition: SceneRoleplayDefinition = {
  schema: 'monogatari-scene-roleplay/v1',
  id: 'memory_failure_recovery',
  title: 'Memory Failure Recovery',
  start_node_id: 'contact',
  exhaustion_ending_id: 'silence',
  max_total_turns: 4,
  score_dimensions: [{
    id: 'trust', label: 'Trust', description: 'Verifiable care.', min: -2, max: 2, initial: 0,
  }],
  nodes: [{
    id: 'contact',
    scene_id: 'station',
    character_id: 'echo',
    supporting_character_ids: [],
    opening_narration: 'The receiver opens.',
    situation: 'A signal needs independent verification.',
    player_goal: 'Verify the signal.',
    character_goal: 'Keep the exchange grounded.',
    knowledge_refs: [],
    response_guard: {
      forbidden_markers: [],
      grounding_markers: ['receiver', 'coordinates', 'signal'],
      min_grounding_matches: 3,
      recoveries: ['The receiver keeps the coordinates inside the signal.'],
      max_characters: 100,
      max_sentences: 2,
    },
    fallback_evaluation: {
      score_signals: [{
        dimension_id: 'trust',
        positive_markers: ['second receiver'],
        negative_markers: [],
        delta: 1,
      }],
      evidence_signals: [{ evidence_id: 'verification', markers: ['coordinates'] }],
    },
    min_turns: 1,
    max_turns: 2,
    score_rules: [{ dimension_id: 'trust', guidance: 'Reward independent checks.', max_delta_per_turn: 1 }],
    evidence_rules: [{ id: 'verification', description: 'A repeatable check.' }],
    transitions: [],
    timeout_target: { kind: 'ending', ending_id: 'silence' },
  }],
  inference: {
    max_context_characters: 3_000,
    max_recent_turns: 2,
    npc_max_tokens: 64,
    evaluator_max_tokens: 64,
  },
}

const character: StoryCharacterInfo = {
  id: 'echo',
  name: 'Echo',
  description: 'A bounded station signal.',
  background: 'The source remains uncertain.',
  emotion: 'steady',
  personality: { speech_style: 'concise' },
  portrait_path: null,
  sprite_path: null,
  knowledge_refs: [],
}

const keeper: StoryCharacterInfo = {
  id: 'keeper',
  name: 'Keeper',
  description: 'The station archive keeper.',
  background: 'Keeps only independently verifiable records.',
  emotion: 'focused',
  personality: { speech_style: 'precise' },
  portrait_path: null,
  sprite_path: null,
  knowledge_refs: [],
}

describe('SceneRoleplayPanel', () => {
  beforeEach(() => {
    mocks.detectWebGpuSupport.mockReturnValue({ available: true, reason: 'available' })
    mocks.loadKnowledgeAuthoringCatalog.mockResolvedValue({ entries: [] })
    mocks.loadAuthoringApiRuntime.mockResolvedValue(null)
    mocks.generateAuthoringApiChat.mockReset()
    mocks.generateWebGpuChat.mockReset()
  })

  it('commits an authored in-world turn when ORT exhausts memory', async () => {
    mocks.generateWebGpuChat.mockRejectedValue(new Error(
      'failed to call OrtRun(). ERROR_CODE: 6, ERROR_MESSAGE: std::bad_alloc',
    ))
    const wrapper = mount(SceneRoleplayPanel, {
      props: {
        snapshot: startBrowserSceneRoleplay(definition),
        desktopRuntime: false,
        characters: [character],
        endings: [],
        locale: 'en',
        sceneName: 'Station',
      },
    })
    await flushPromises()

    await wrapper.get('textarea').setValue('Use a second receiver to verify the coordinates.')
    await wrapper.get('.send-button').trigger('click')
    await flushPromises()

    expect(mocks.generateWebGpuChat).toHaveBeenCalledTimes(1)
    expect(wrapper.find('.roleplay-error').exists()).toBe(false)
    expect(wrapper.get('[data-testid="scene-roleplay"]').attributes('data-evaluation-source'))
      .toBe('authored_fallback_npc_inference_error')
    expect(wrapper.get('[data-testid="roleplay-degraded"]').text())
      .toContain('Part of live inference was unavailable')

    const update = wrapper.emitted('update')?.at(-1)?.[0] as ReturnType<typeof startBrowserSceneRoleplay>
    expect(update.session.scores.trust).toBe(1)
    expect(update.session.observed_evidence).toEqual(['verification'])
    expect(update.session.transcript[0].npc_response)
      .toBe('The receiver keeps the coordinates inside the signal.')
    expect(wrapper.text()).not.toContain('OrtRun')
    expect(wrapper.text()).not.toContain('std::bad_alloc')

    await wrapper.setProps({ snapshot: update })
    await flushPromises()
    await wrapper.setProps({ snapshot: startBrowserSceneRoleplay(definition) })
    await flushPromises()

    expect(wrapper.find('[data-testid="roleplay-degraded"]').exists()).toBe(false)
    expect(wrapper.get('[data-testid="scene-roleplay"]').attributes('data-evaluation-source'))
      .toBeUndefined()
  })

  it('uses the project authoring API for both NPC generation and evaluation', async () => {
    mocks.loadAuthoringApiRuntime.mockResolvedValue({
      schema: 'monogatari-authoring-inference-runtime/v1',
      provider: 'api',
      endpoint: '/authoring-api/chat/completions',
      model: 'remote-roleplay-model',
      max_new_tokens: 256,
      temperature: 0.7,
      top_p: 0.9,
    })
    mocks.generateAuthoringApiChat
      .mockResolvedValueOnce('The receiver holds the coordinates inside the signal.')
      .mockResolvedValueOnce(JSON.stringify({
        score_deltas: { trust: 1 },
        evidence: { verification: 'coordinates' },
        npc_emotion: 'steady',
        summary: 'The player requested an independent check.',
      }))
    const wrapper = mount(SceneRoleplayPanel, {
      props: {
        snapshot: startBrowserSceneRoleplay(definition),
        desktopRuntime: false,
        characters: [character],
        endings: [],
        locale: 'en',
        sceneName: 'Station',
      },
    })
    await flushPromises()

    await wrapper.get('textarea').setValue('Use a second receiver to verify the coordinates.')
    await wrapper.get('.send-button').trigger('click')
    await flushPromises()

    expect(mocks.generateAuthoringApiChat).toHaveBeenCalledTimes(2)
    expect(mocks.generateWebGpuChat).not.toHaveBeenCalled()
    expect(wrapper.get('[data-testid="roleplay-runtime"]').text())
      .toBe('LLM NPC · remote-roleplay-model API')
    expect(wrapper.get('[data-testid="roleplay-runtime"]').attributes('data-runtime-kind'))
      .toBe('api')
    expect(wrapper.get('[data-testid="scene-roleplay"]').attributes('data-evaluation-source'))
      .toBe('authoring_api_model')
  })

  it('lets the player address every NPC present in the scene', async () => {
    const ensembleDefinition = structuredClone(definition)
    ensembleDefinition.nodes[0].supporting_character_ids = ['keeper']
    ensembleDefinition.nodes[0].relationship_rule = {
      guidance: 'Reward grounded cooperation.',
      max_delta_per_turn: 0.1,
    }
    mocks.loadAuthoringApiRuntime.mockResolvedValue({
      schema: 'monogatari-authoring-inference-runtime/v1',
      provider: 'api',
      endpoint: '/authoring-api/chat/completions',
      model: 'ensemble-model',
      max_new_tokens: 256,
      temperature: 0.7,
      top_p: 0.9,
    })
    mocks.generateAuthoringApiChat
      .mockResolvedValueOnce('The receiver keeps the coordinates inside the signal.')
      .mockResolvedValueOnce(JSON.stringify({
        score_deltas: { trust: 0 },
        evidence: {},
        relationship_delta: 0.1,
        relationship_reason: 'The player addressed the keeper directly.',
        npc_emotion: 'focused',
        summary: 'The keeper answered.',
      }))
    const wrapper = mount(SceneRoleplayPanel, {
      props: {
        snapshot: startBrowserSceneRoleplay(ensembleDefinition, { echo: 0.2, keeper: 0.4 }),
        desktopRuntime: false,
        characters: [character, keeper],
        endings: [],
        locale: 'en',
        sceneName: 'Station',
      },
    })
    await flushPromises()

    const participants = wrapper.findAll('.participant-button')
    expect(participants.map(button => button.text())).toEqual(['Echo', 'Keeper'])
    await participants[1].trigger('click')
    expect(wrapper.text()).toContain('Keeper')
    expect(wrapper.emitted('speakerChange')?.at(-1)).toEqual(['keeper'])
    await wrapper.get('textarea').setValue('Keeper, preserve only the repeatable record.')
    await wrapper.get('.send-button').trigger('click')
    await flushPromises()

    const npcPrompt = mocks.generateAuthoringApiChat.mock.calls[0][0]
      .map((message: { content: string }) => message.content)
      .join('\n')
    expect(npcPrompt).toContain('active_speaker=keeper')
    expect(npcPrompt).toContain('The station archive keeper.')
    const update = wrapper.emitted('update')?.at(-1)?.[0] as ReturnType<typeof startBrowserSceneRoleplay>
    expect(update.session.transcript[0].speaker_id).toBe('keeper')
    expect(update.session.relationships?.keeper).toBeCloseTo(0.5)
    expect(update.session.relationships?.echo).toBeCloseTo(0.2)
  })

  it('switches to a recovered project API before the next clean turn', async () => {
    mocks.loadAuthoringApiRuntime
      .mockResolvedValueOnce(null)
      .mockResolvedValueOnce({
        schema: 'monogatari-authoring-inference-runtime/v1',
        provider: 'api',
        endpoint: '/authoring-api/chat/completions',
        model: 'recovered-roleplay-model',
        max_new_tokens: 256,
        temperature: 0.7,
        top_p: 0.9,
      })
    mocks.generateAuthoringApiChat
      .mockResolvedValueOnce('The receiver holds the coordinates inside the signal.')
      .mockResolvedValueOnce(JSON.stringify({
        score_deltas: { trust: 1 },
        evidence: { verification: 'coordinates' },
        npc_emotion: 'steady',
        summary: 'The project API recovered before the turn.',
      }))
    const wrapper = mount(SceneRoleplayPanel, {
      props: {
        snapshot: startBrowserSceneRoleplay(definition),
        desktopRuntime: false,
        characters: [character],
        endings: [],
        locale: 'en',
        sceneName: 'Station',
      },
    })
    await flushPromises()

    expect(wrapper.get('[data-testid="roleplay-runtime"]').attributes('data-runtime-kind'))
      .toBe('webgpu')
    await wrapper.get('textarea').setValue('Use a second receiver to verify the coordinates.')
    await wrapper.get('.send-button').trigger('click')
    await flushPromises()

    expect(mocks.loadAuthoringApiRuntime).toHaveBeenCalledTimes(2)
    expect(mocks.generateAuthoringApiChat).toHaveBeenCalledTimes(2)
    expect(mocks.generateWebGpuChat).not.toHaveBeenCalled()
    expect(wrapper.get('[data-testid="roleplay-runtime"]').text())
      .toBe('LLM NPC · recovered-roleplay-model API')
    expect(wrapper.get('[data-testid="scene-roleplay"]').attributes('data-evaluation-source'))
      .toBe('authoring_api_model')
  })
})
