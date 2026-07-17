import { describe, expect, it } from 'vitest'

import type { StoryDialogueInfo, WebDialogueNode } from '../storyContent'
import {
  advanceBrowserDialogue,
  applyBrowserRelationshipChanges,
  selectBrowserDialogueChoice,
  startBrowserDialogue,
  StoryPlaytestError,
} from '../storyPlaytest'

function dialogue(nodes: Record<string, WebDialogueNode>, startNodeId = 'start'): StoryDialogueInfo {
  return {
    id: 'intro',
    title: 'Intro',
    start_node_id: startNodeId,
    node_count: Object.keys(nodes).length,
    nodes,
    access: {
      content_type: 'dialogue',
      content_id: 'intro',
      gated: false,
      unlocked: true,
      unlock_event_ids: [],
    },
  }
}

describe('browser story playtest dialogue cursor', () => {
  it('starts at the declared node and exposes stable choice indices', () => {
    const transition = startBrowserDialogue(dialogue({
      start: {
        speaker_id: 'aoi',
        scene_id: 'archive',
        text: 'Choose.',
        emotion: 'happy',
        choices: [
          { text: 'Left', next_node_id: 'left' },
          { text: 'Right', next_node_id: 'right' },
        ],
      },
      left: { text: 'Left path.' },
      right: { text: 'Right path.' },
    }))

    expect(transition).toMatchObject({
      runtime: { node_id: 'start' },
      completed: false,
      blocked_reason: null,
      state: {
        is_active: true,
        speaker: 'aoi',
        scene_id: 'archive',
        text: 'Choose.',
        emotion: 'happy',
        choices: [{ index: 0, text: 'Left' }, { index: 1, text: 'Right' }],
      },
    })
  })

  it('starts an authoring preview at an explicit existing node', () => {
    const script = dialogue({
      start: { text: 'Start.', next_node_id: 'preview' },
      preview: { text: 'Preview this node.', scene_id: 'archive' },
    })

    expect(startBrowserDialogue(script, {}, ' preview ')).toMatchObject({
      runtime: { node_id: 'preview' },
      state: { text: 'Preview this node.', scene_id: 'archive' },
    })
    expect(() => startBrowserDialogue(script, {}, 'missing')).toThrowError(expect.objectContaining({
      code: 'dialogue_node_missing',
      node_id: 'missing',
    }))
  })

  it('returns choice relationship effects and moves to a verified target', () => {
    const script = dialogue({
      start: {
        text: 'Choose.',
        choices: [{
          text: 'Be kind',
          next_node_id: 'kind',
          relationship_changes: { ' aoi ': 0.4, invalid: Number.NaN },
        }],
      },
      kind: { text: 'Thank you.' },
    })

    const started = startBrowserDialogue(script)
    expect(selectBrowserDialogueChoice(script, started.runtime, 0)).toMatchObject({
      runtime: { node_id: 'kind' },
      relationship_changes: { aoi: 0.4 },
      state: { text: 'Thank you.' },
    })
  })

  it('applies browser relationship effects immutably and clamps scores', () => {
    const characters = [
      { id: 'aoi', relationships: { player: 0.8 } },
      { id: 'mio', relationships: { player: -0.9 } },
      { id: 'sora' },
    ]
    const updated = applyBrowserRelationshipChanges(characters, { aoi: 0.5, mio: -0.5 })

    expect(updated.map((character) => character.relationships?.player ?? 0)).toEqual([1, -1, 0])
    expect(updated[0]).not.toBe(characters[0])
    expect(updated[2]).toBe(characters[2])
    expect(characters[0]!.relationships?.player).toBe(0.8)
  })

  it('rejects unknown relationship targets before changing any character', () => {
    const characters = [{ id: 'aoi', relationships: { player: 0.2 } }]

    expect(() => applyBrowserRelationshipChanges(characters, { aoi: 0.4, missing: 0.1 }))
      .toThrowError(expect.objectContaining({
        code: 'relationship_target_missing',
        target_node_id: 'missing',
      }))
    expect(characters[0]!.relationships.player).toBe(0.2)
  })

  it('blocks linear advance while a choice is required and completes terminal nodes', () => {
    const script = dialogue({
      start: { text: 'Choose.', choices: [{ text: 'Continue', next_node_id: 'end' }] },
      end: { text: 'Done.', is_ending: true },
    })

    const started = startBrowserDialogue(script)
    expect(advanceBrowserDialogue(script, started.runtime)).toMatchObject({
      runtime: { node_id: 'start' },
      completed: false,
      blocked_reason: 'choice_required',
    })
    const ending = selectBrowserDialogueChoice(script, started.runtime, 0)
    expect(advanceBrowserDialogue(script, ending.runtime)).toMatchObject({
      runtime: { node_id: null },
      completed: true,
      state: { is_active: false, text: '' },
    })
  })

  it('keeps authored choice indices while filtering choices from dialogue-local state', () => {
    const script = dialogue({
      start: {
        text: 'Choose.',
        script: "setFlag('met_aoi', true)",
        choices: [
          { text: 'Hidden', next_node_id: 'hidden', condition: "hasFlag('missing')" },
          { text: 'Visible', next_node_id: 'visible', condition: "hasFlag('met_aoi')" },
        ],
      },
      hidden: { text: 'Hidden.' },
      visible: { text: 'Visible.' },
    })

    const started = startBrowserDialogue(script)
    expect(started.state.choices).toEqual([{ index: 1, text: 'Visible' }])
    expect(started.runtime.flags).toEqual({ met_aoi: true })
    expect(() => selectBrowserDialogueChoice(script, started.runtime, 0)).toThrowError(expect.objectContaining({
      code: 'choice_unavailable',
    }))
    expect(selectBrowserDialogueChoice(script, started.runtime, 1).state.text).toBe('Visible.')
  })

  it('skips disabled linear nodes and carries script variables into later conditions', () => {
    const script = dialogue({
      start: { text: 'Start.', script: "setVariable('score', 2)", next_node_id: 'skip' },
      skip: { text: 'Skip.', condition: "getVariable('score') < 2", next_node_id: 'shown' },
      shown: { text: 'Shown.', condition: "getVariable('score') == 2" },
    })

    const started = startBrowserDialogue(script)
    const advanced = advanceBrowserDialogue(script, started.runtime)
    expect(advanced.runtime).toMatchObject({ node_id: 'shown', variables: { score: 2 } })
    expect(advanced.state.text).toBe('Shown.')
  })

  it('rejects blocked conditional terminals and unsupported browser expressions explicitly', () => {
    const blocked = dialogue({ start: { text: 'Blocked.', condition: 'false' } })
    expect(() => startBrowserDialogue(blocked)).toThrowError(expect.objectContaining({
      code: 'node_condition_blocked',
    }))

    const unsupported = dialogue({ start: { text: 'Unsupported.', condition: 'unknownFunction()' } })
    expect(() => startBrowserDialogue(unsupported)).toThrowError(expect.objectContaining({
      code: 'condition_unsupported',
    }))
  })

  it.each([
    ['', 'dialogue_start_missing'],
    ['missing', 'dialogue_node_missing'],
  ])('rejects invalid active node %s with code %s', (nodeId, code) => {
    const script = dialogue({ start: { text: 'Start.' } }, nodeId)
    expect(() => startBrowserDialogue(script)).toThrowError(expect.objectContaining({ code }))
  })

  it('rejects invalid choice indices and missing graph targets explicitly', () => {
    const script = dialogue({
      start: { text: 'Choose.', choices: [{ text: 'Broken', next_node_id: 'missing' }] },
    })
    const runtime = startBrowserDialogue(script).runtime

    for (const run of [
      () => selectBrowserDialogueChoice(script, runtime, 2),
      () => selectBrowserDialogueChoice(script, runtime, 0),
    ]) {
      try {
        run()
        throw new Error('Expected StoryPlaytestError')
      } catch (error) {
        expect(error).toBeInstanceOf(StoryPlaytestError)
      }
    }
    expect(() => selectBrowserDialogueChoice(script, runtime, 2)).toThrowError(expect.objectContaining({
      code: 'choice_index_invalid',
    }))
    expect(() => selectBrowserDialogueChoice(script, runtime, 0)).toThrowError(expect.objectContaining({
      code: 'choice_target_missing',
      target_node_id: 'missing',
    }))
  })

  it('rejects missing linear targets instead of silently ending playback', () => {
    const script = dialogue({ start: { text: 'Start.', next_node_id: 'missing' } })
    const runtime = startBrowserDialogue(script).runtime
    expect(() => advanceBrowserDialogue(script, runtime)).toThrowError(expect.objectContaining({
      code: 'next_target_missing',
      target_node_id: 'missing',
    }))
  })
})
