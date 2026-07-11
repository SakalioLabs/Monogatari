import { describe, expect, it } from 'vitest'

import {
  normalizeDialogueDefinition,
  validateDialogueDefinition,
  type DialogueDefinition,
  type DialogueNodeDefinition,
} from '../dialogueAuthoring'

function node(overrides: Partial<DialogueNodeDefinition> = {}): DialogueNodeDefinition {
  return {
    speaker_id: 'aoi',
    text: 'Hello.',
    next_node_id: null,
    choices: [],
    condition: null,
    script: null,
    emotion: null,
    use_llm: false,
    llm_prompt: null,
    llm_system_prompt: null,
    is_ending: false,
    ending_type: null,
    ...overrides,
  }
}

function dialogue(overrides: Partial<DialogueDefinition> = {}): DialogueDefinition {
  return {
    id: 'aoi_intro',
    title: 'Aoi Intro',
    description: null,
    start_node_id: 'start',
    nodes: {
      start: node({ next_node_id: 'end' }),
      end: node({ text: 'Goodbye.', is_ending: true, ending_type: 'normal' }),
    },
    variables: {},
    ...overrides,
  }
}

describe('dialogue authoring contracts', () => {
  it('accepts a reachable graph with known speakers', () => {
    expect(validateDialogueDefinition(dialogue(), ['aoi'])).toEqual([])
  })

  it('normalizes stable maps and authored text', () => {
    const normalized = normalizeDialogueDefinition(dialogue({
      id: ' aoi_intro ',
      title: ' Aoi Intro ',
      description: '  First meeting. ',
      nodes: {
        z_end: node({ text: ' End ', is_ending: true }),
        start: node({ text: ' Start ', next_node_id: 'z_end' }),
      },
      variables: { z: 1, a: 2 },
    }))
    expect(Object.keys(normalized.nodes)).toEqual(['start', 'z_end'])
    expect(Object.keys(normalized.variables)).toEqual(['a', 'z'])
    expect(normalized.nodes.start.text).toBe('Start')
    expect(normalized.description).toBe('First meeting.')
  })

  it('reports broken references, unreachable nodes, unsafe terminal state, and LLM gaps', () => {
    const issues = validateDialogueDefinition(dialogue({
      nodes: {
        start: node({
          speaker_id: 'unknown',
          next_node_id: 'missing',
          use_llm: true,
          choices: [{
            text: 'Choice',
            next_node_id: 'end',
            relationship_changes: { aoi: 2 },
            condition: null,
          }],
        }),
        end: node({ is_ending: true, next_node_id: 'start' }),
        orphan: node(),
      },
    }), ['aoi'])

    expect(issues.some((issue) => issue.includes('unknown speaker'))).toBe(true)
    expect(issues.some((issue) => issue.includes('cannot combine a linear target with choices'))).toBe(true)
    expect(issues.some((issue) => issue.includes('targets missing node'))).toBe(true)
    expect(issues.some((issue) => issue.includes('without a prompt'))).toBe(true)
    expect(issues.some((issue) => issue.includes('must be between -1 and 1'))).toBe(true)
    expect(issues.some((issue) => issue.includes('Ending node'))).toBe(true)
    expect(issues.some((issue) => issue.includes('Unreachable nodes'))).toBe(true)
  })

  it('rejects normalization collisions and non-portable character references', () => {
    expect(() => normalizeDialogueDefinition(dialogue({
      nodes: {
        start: node(),
        ' start ': node({ text: 'Collision' }),
      },
    }))).toThrow('Dialogue node IDs collide')

    const issues = validateDialogueDefinition(dialogue({
      nodes: {
        start: node({
          speaker_id: '../aoi',
          choices: [{
            text: 'Continue',
            next_node_id: 'end',
            relationship_changes: { '../aoi': 0.5 },
            condition: null,
          }],
        }),
        end: node({ is_ending: true }),
      },
    }))
    expect(issues.some((issue) => issue.includes('speaker "../aoi" is not portable'))).toBe(true)
    expect(issues.some((issue) => issue.includes('relationship character "../aoi" is not portable'))).toBe(true)
  })
})
