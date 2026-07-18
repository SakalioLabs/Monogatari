import { flushPromises, mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({
  detectWebGpuSupport: vi.fn(),
  generateWebGpuChat: vi.fn(),
  loadKnowledgeAuthoringCatalog: vi.fn(),
}))

vi.mock('../../lib/webgpuInference', () => ({
  detectWebGpuSupport: mocks.detectWebGpuSupport,
  generateWebGpuChat: mocks.generateWebGpuChat,
}))

vi.mock('../../lib/knowledgeContent', () => ({
  loadKnowledgeAuthoringCatalog: mocks.loadKnowledgeAuthoringCatalog,
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

describe('SceneRoleplayPanel', () => {
  beforeEach(() => {
    mocks.detectWebGpuSupport.mockReturnValue({ available: true, reason: 'available' })
    mocks.loadKnowledgeAuthoringCatalog.mockResolvedValue({ entries: [] })
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

    const update = wrapper.emitted('update')?.at(-1)?.[0] as ReturnType<typeof startBrowserSceneRoleplay>
    expect(update.session.scores.trust).toBe(1)
    expect(update.session.observed_evidence).toEqual(['verification'])
    expect(update.session.transcript[0].npc_response)
      .toBe('The receiver keeps the coordinates inside the signal.')
    expect(wrapper.text()).not.toContain('OrtRun')
    expect(wrapper.text()).not.toContain('std::bad_alloc')
  })
})
