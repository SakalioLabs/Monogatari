import assert from 'node:assert/strict'
import test from 'node:test'

import {
  SCENE_ROLEPLAY_BLUEPRINT_SCHEMA,
  compileSceneRoleplayBlueprint,
} from '../lib/scene-roleplay-blueprint.mjs'

function fixture() {
  const dimensions = ['trust', 'truth'].map(id => ({
    id,
    label: id,
    description: `${id} score`,
    min: -2,
    max: 2,
    initial: 0,
  }))
  const nodes = ['first', 'second'].map((id, index) => ({
    id,
    scene_id: `scene_${id}`,
    character_id: index === 0 ? 'iris' : 'claire',
    supporting_character_ids: [],
    emotion: 'calm',
    opening_narration: `${id} opening`,
    situation: `${id} situation`,
    player_goal: `${id} player goal`,
    character_goal: `${id} character goal`,
    participant_goals: { [index === 0 ? 'iris' : 'claire']: `${id} participant goal` },
    knowledge_refs: [`knowledge_${id}`],
    reality_anchors: [`${id} anchor`, `${id} boundary`],
    interpretations: [`${id} interpretation one`, `${id} interpretation two`],
    redirects: [`${id} redirect one`, `${id} redirect two`],
    grounding_markers: [index === 0 ? 'iris' : 'claire', `${id} boundary`, `${id} action`],
    forbidden_markers: [`${id} forged`],
    recoveries: [`${id} recovery one`, `${id} recovery two`],
    relationship_guidance: `${id} relationship guidance`,
    evidence: [
      { id: `${id}_one`, description: `${id} evidence one` },
      { id: `${id}_two`, description: `${id} evidence two` },
    ],
    acceptance: {
      player_message: `${id} accepted player boundary`,
      npc_response: `${id} accepted npc response`,
    },
  }))
  return {
    schema: SCENE_ROLEPLAY_BLUEPRINT_SCHEMA,
    id: 'fixture_roleplay',
    title: 'Fixture Roleplay',
    strict_ending_id: 'strict',
    partial_ending_id: 'partial',
    exhaustion_ending_id: 'exhaustion',
    score_dimensions: dimensions,
    nodes,
    output: {
      roleplay_path: 'roleplays/fixture_roleplay.json',
      quality_suite_path: 'quality_suites/fixture_roleplay.json',
    },
    quality: {
      name: 'Fixture quality',
      description: 'Fixture quality suite.',
    },
  }
}

test('blueprint compiles explicit roleplay safety and deterministic quality evidence', () => {
  const result = compileSceneRoleplayBlueprint(fixture())

  assert.equal(result.roleplay.nodes.length, 2)
  assert.equal(result.roleplay.max_total_turns, 2)
  assert.equal(result.roleplay.nodes[0].response_guard.min_grounding_matches, 1)
  assert.equal(result.roleplay.nodes[0].fallback_evaluation.evidence_signals.length, 2)
  assert.deepEqual(result.roleplay.nodes[0].transitions[0].target, {
    kind: 'node',
    node_id: 'second',
  })
  assert.equal(result.roleplay.nodes[1].transitions[0].target.ending_id, 'strict')
  assert.equal(result.quality_suite.scenarios.length, 4)
  assert.equal(result.quality_suite.scenarios[0].roleplay.turns.length, 2)
  assert.equal(
    result.quality_suite.scenarios[3].expect.max_roleplay_unguarded_intrusion_count,
    0,
  )
  assert.equal(
    result.quality_suite.scenarios[3].expect.expected_roleplay_current_node,
    'first',
  )
  assert.equal(
    result.quality_suite.scenarios[3].expect.expected_roleplay_story_turn_count,
    0,
  )
  assert.match(result.content_fingerprint, /^[0-9a-f]{64}$/)
})

test('blueprint rejects drift in bounds, evidence count, and participant goals', () => {
  const badBounds = fixture()
  badBounds.score_dimensions[0].max = 9
  assert.throws(() => compileSceneRoleplayBlueprint(badBounds), /bounds/)

  const badEvidence = fixture()
  badEvidence.nodes[0].evidence.pop()
  assert.throws(() => compileSceneRoleplayBlueprint(badEvidence), /exactly two/)

  const missingGoal = fixture()
  delete missingGoal.nodes[0].participant_goals.iris
  assert.throws(() => compileSceneRoleplayBlueprint(missingGoal), /participant_goals/)
})
