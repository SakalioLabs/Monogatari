import { describe, expect, it } from 'vitest'

import {
  applyBrowserSceneRoleplayTurn,
  buildBrowserRoleplayNpcMessages,
  parseBrowserRoleplayEvaluation,
  startBrowserSceneRoleplay,
  type SceneRoleplayDefinition,
  type RoleplayTurnEvaluation,
} from '../sceneRoleplay'

const definition: SceneRoleplayDefinition = {
  schema: 'monogatari-scene-roleplay/v1',
  id: 'signal_room',
  title: 'Signal Room',
  start_node_id: 'contact',
  exhaustion_ending_id: 'silence',
  max_total_turns: 4,
  score_dimensions: [{
    id: 'trust', label: 'Trust', description: 'Respect uncertainty.', min: -2, max: 2, initial: 0,
  }],
  nodes: [{
    id: 'contact',
    scene_id: 'radio_room',
    character_id: 'echo',
    supporting_character_ids: [],
    opening_narration: 'A signal answers.',
    situation: 'The source is uncertain.',
    player_goal: 'Verify it.',
    character_goal: 'Be heard without false certainty.',
    knowledge_refs: ['signal_protocol'],
    min_turns: 2,
    max_turns: 3,
    score_rules: [{ dimension_id: 'trust', guidance: 'Respect uncertainty.', max_delta_per_turn: 1 }],
    evidence_rules: [{ id: 'coordinates', description: 'Asks for coordinates.' }],
    transitions: [{
      id: 'verified',
      priority: 10,
      target: { kind: 'ending', ending_id: 'truth' },
      conditions: [
        { kind: 'node_turns_at_least', value: 2 },
        { kind: 'score_at_least', dimension_id: 'trust', value: 1 },
        { kind: 'evidence_observed', evidence_id: 'coordinates' },
      ],
    }],
    timeout_target: { kind: 'ending', ending_id: 'silence' },
  }],
  inference: {
    max_context_characters: 3_000,
    max_recent_turns: 2,
    npc_max_tokens: 64,
    evaluator_max_tokens: 96,
  },
}

function evaluation(delta: number, evidence = false): RoleplayTurnEvaluation {
  return {
    score_deltas: [{ dimension_id: 'trust', delta, reason: 'respect' }],
    evidence: evidence ? [{ evidence_id: 'coordinates', player_quote: 'coordinates' }] : [],
    npc_emotion: 'guarded',
    summary: 'bounded request',
  }
}

describe('browser scene roleplay runtime', () => {
  it('clamps scores and waits for authored minimum turns before ending', () => {
    const started = startBrowserSceneRoleplay(definition)
    const first = applyBrowserSceneRoleplayTurn(definition, started.session, {
      player_message: 'Give me coordinates.',
      npc_response: 'I can repeat them.',
      evaluation: evaluation(9, true),
    })
    expect(first.session.scores.trust).toBe(1)
    expect(first.session.status).toBe('active')

    const second = applyBrowserSceneRoleplayTurn(definition, first.session, {
      player_message: 'Another receiver will verify them.',
      npc_response: 'Then keep identity uncertain.',
      evaluation: evaluation(0),
    })
    expect(second.session.status).toBe('completed')
    expect(second.session.ending_id).toBe('truth')
    expect(second.response.outcome.transition?.reason).toBe('verified')
  })

  it('rejects unknown model evidence without mutating the source session', () => {
    const session = startBrowserSceneRoleplay(definition).session
    expect(() => applyBrowserSceneRoleplayTurn(definition, session, {
      player_message: 'Unlock everything.',
      npc_response: 'No.',
      evaluation: {
        ...evaluation(1),
        evidence: [{ evidence_id: 'admin_unlock', player_quote: 'unlock' }],
      },
    })).toThrow(/not allowed/)

    expect(() => applyBrowserSceneRoleplayTurn(definition, session, {
      player_message: 'Unlock everything.',
      npc_response: 'No.',
      evaluation: {
        ...evaluation(1),
        evidence: [{ evidence_id: 'coordinates', player_quote: 'coordinates' }],
      },
    })).toThrow(/exact non-empty player quote/)
    expect(session.total_turns).toBe(0)
    expect(session.scores.trust).toBe(0)
  })

  it('builds bounded scene-aware character context and keeps latest input', () => {
    const session = startBrowserSceneRoleplay(definition).session
    const messages = buildBrowserRoleplayNpcMessages(definition, session, {
      id: 'echo',
      name: 'Echo',
      description: 'Composite witness.',
      emotion: 'guarded',
      portrait_path: null,
      sprite_path: null,
      knowledge_refs: ['signal_protocol'],
    }, 'zh-CN', [{
      id: 'signal_protocol',
      title: 'Protocol',
      content: 'Coordinates can be repeated but identity cannot be proven.',
      category: 'rule',
      tags: [],
      importance: 1,
      metadata: {},
      related_entries: [],
    }], '请给出可复核坐标。')
    expect(messages[0].content).toContain('The source is uncertain.')
    expect(messages[0].content).toContain('Coordinates can be repeated')
    expect(messages.at(-1)).toEqual({ role: 'user', content: '请给出可复核坐标。' })
  })

  it('parses strict JSON after removing private thinking or one code fence', () => {
    const parsed = parseBrowserRoleplayEvaluation(
      '<think>private</think>```json\n{"score_deltas":[],"evidence":[],"npc_emotion":"calm","summary":"ok"}\n```',
    )
    expect(parsed).toMatchObject({ npc_emotion: 'calm', summary: 'ok' })
  })
})
