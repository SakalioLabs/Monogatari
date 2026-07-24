import { describe, expect, it } from 'vitest'
import {
  advanceBrowserRoleplayCampaign,
  startBrowserRoleplayCampaign,
  validateRoleplayCampaign,
  type RoleplayCampaignDefinition,
} from '../roleplayCampaign'
import type { SceneRoleplayDefinition, SceneRoleplaySession } from '../sceneRoleplay'

const roleplay = (id: string, endingId: string): SceneRoleplayDefinition => ({
  schema: 'monogatari-scene-roleplay/v1',
  id,
  title: id,
  start_node_id: 'start',
  exhaustion_ending_id: endingId,
  max_total_turns: 1,
  score_dimensions: [{ id: 'trust', label: 'Trust', description: 'Trust', min: -2, max: 2, initial: 0 }],
  nodes: [{
    id: 'start',
    scene_id: 'room',
    character_id: 'aqua',
    supporting_character_ids: [],
    opening_narration: 'Start.',
    situation: 'A decision.',
    player_goal: 'Decide.',
    character_goal: 'Respond.',
    knowledge_refs: [],
    min_turns: 1,
    max_turns: 1,
    score_rules: [{ dimension_id: 'trust', guidance: 'Trust.', max_delta_per_turn: 1 }],
    relationship_rule: { guidance: 'Respect.', max_delta_per_turn: 0.2 },
    evidence_rules: [],
    transitions: [],
    timeout_target: { kind: 'ending', ending_id: endingId },
  }],
  inference: { max_context_characters: 8000, max_recent_turns: 6, npc_max_tokens: 128, evaluator_max_tokens: 128 },
})

const campaign: RoleplayCampaignDefinition = {
  schema: 'monogatari-roleplay-campaign/v1',
  id: 'volume_one',
  title: 'Volume One',
  start_entry_id: 'chapter_one',
  entries: [
    {
      id: 'chapter_one',
      roleplay_id: 'chapter1',
      routes: [{ ending_id: 'chapter1_done', target: { kind: 'entry', entry_id: 'chapter_two' } }],
    },
    {
      id: 'chapter_two',
      roleplay_id: 'chapter2',
      routes: [{ ending_id: 'chapter2_done', target: { kind: 'complete' } }],
    },
  ],
}

const completed = (roleplayId: string, endingId: string, relationship: number): SceneRoleplaySession => ({
  roleplay_id: roleplayId,
  current_node_id: 'start',
  node_turns: 1,
  total_turns: 1,
  scores: { trust: 1 },
  initial_relationships: { aqua: 0 },
  relationships: { aqua: relationship },
  observed_evidence: [],
  status: 'completed',
  ending_id: endingId,
  transcript: [],
  archived_turn_count: 0,
})

describe('roleplay campaign', () => {
  const roleplays = [roleplay('chapter1', 'chapter1_done'), roleplay('chapter2', 'chapter2_done')]

  it('carries relationships but starts each chapter with fresh local scores', () => {
    const started = startBrowserRoleplayCampaign(campaign, roleplays)
    const advanced = advanceBrowserRoleplayCampaign(started, roleplays, completed('chapter1', 'chapter1_done', 0.35))

    expect(advanced.session.current_entry_id).toBe('chapter_two')
    expect(advanced.session.relationships.aqua).toBe(0.35)
    expect(advanced.active_roleplay?.session.initial_relationships?.aqua).toBe(0.35)
    expect(advanced.active_roleplay?.session.scores.trust).toBe(0)
    expect(advanced.session.completed_entries[0].scores.trust).toBe(1)
  })

  it('rejects unfinished sessions, wrong roleplays, and unrouted endings', () => {
    const started = startBrowserRoleplayCampaign(campaign, roleplays)
    expect(() => advanceBrowserRoleplayCampaign(started, roleplays, {
      ...completed('chapter1', 'chapter1_done', 0),
      status: 'active',
      ending_id: null,
    })).toThrow(/does not match/)
    expect(() => advanceBrowserRoleplayCampaign(started, roleplays, completed('chapter2', 'chapter2_done', 0))).toThrow(/does not match/)
    expect(() => advanceBrowserRoleplayCampaign(started, roleplays, completed('chapter1', 'forged', 0))).toThrow(/no campaign route/)
  })

  it('rejects cycles, unreachable entries, and missing ending routes', () => {
    const cyclic = structuredClone(campaign)
    cyclic.entries[1].routes[0].target = { kind: 'entry', entry_id: 'chapter_one' }
    expect(() => validateRoleplayCampaign(cyclic, roleplays)).toThrow(/cycle/)

    const missingRoute = structuredClone(campaign)
    missingRoute.entries[0].routes = []
    expect(() => validateRoleplayCampaign(missingRoute, roleplays)).toThrow(/routes/)
  })
})
