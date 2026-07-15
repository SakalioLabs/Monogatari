import { describe, expect, it } from 'vitest'
import { reactive } from 'vue'

import type {
  DialogueAuthoringCatalogSnapshot,
  DialogueAuthoringEntry,
  DialogueChoiceDefinition,
  DialogueDefinition,
  DialogueNodeDefinition,
} from '../dialogueAuthoring'
import {
  addDialogueRelationship,
  appendDialogueChoice,
  appendDialogueNode,
  availableDialogueRelationshipCharacters,
  cloneDialogueDefinition,
  createDialogueDraft,
  deleteDialogueNode,
  dialogueDefinitionFromEntry,
  dialogueDraftSnapshot,
  dialogueFlowMode,
  dialogueImplicitTerminalIds,
  dialogueNodeOrder,
  dialogueRelationshipEntries,
  dialogueTerminalCount,
  duplicateDialogueDraft,
  filterDialogueEntries,
  hasDialogueIdCollision,
  mergeDialogueCharacters,
  nextDialogueId,
  parseDialogueVariables,
  removeDialogueChoice,
  removeDialogueRelationship,
  renameDialogueNode,
  renameDialogueRelationship,
  setDialogueNodeFlowMode,
  setDialogueRelationshipDelta,
} from '../dialogueGraphEditing'

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

function choice(overrides: Partial<DialogueChoiceDefinition> = {}): DialogueChoiceDefinition {
  return {
    text: 'Continue',
    next_node_id: 'end',
    relationship_changes: {},
    condition: null,
    ...overrides,
  }
}

function dialogue(overrides: Partial<DialogueDefinition> = {}): DialogueDefinition {
  return {
    id: 'intro',
    title: 'Intro',
    description: null,
    start_node_id: 'start',
    nodes: {
      start: node({ next_node_id: 'middle' }),
      middle: node({ choices: [choice()] }),
      end: node({ text: 'End.', is_ending: true }),
    },
    variables: { route: { open: true } },
    ...overrides,
  }
}

function entry(overrides: Partial<DialogueAuthoringEntry> = {}): DialogueAuthoringEntry {
  return {
    ...dialogue(),
    source_path: 'dialogue/intro.json',
    content_fingerprint: 'sha256',
    access: {
      content_type: 'dialogue',
      content_id: 'intro',
      gated: false,
      unlocked: true,
      unlock_event_ids: [],
    },
    ...overrides,
  }
}

function catalog(dialogues: DialogueAuthoringEntry[]): DialogueAuthoringCatalogSnapshot {
  return {
    schema: 'monogatari-dialogue-authoring-catalog/v1',
    catalog_fingerprint: 'catalog',
    dialogue_count: dialogues.length,
    node_count: dialogues.reduce((count, item) => count + Object.keys(item.nodes).length, 0),
    choice_count: 0,
    llm_node_count: 0,
    dialogues,
  }
}

describe('Dialogue graph draft boundaries', () => {
  it('clones definitions deeply and strips authoring entry metadata', () => {
    const source = entry()
    const cloned = cloneDialogueDefinition(reactive(source))
    const definition = dialogueDefinitionFromEntry(source)

    cloned.nodes.middle.choices[0].text = 'Changed'
    ;(cloned.variables.route as { open: boolean }).open = false
    expect(source.nodes.middle.choices[0].text).toBe('Continue')
    expect(source.variables.route).toEqual({ open: true })
    expect(definition).not.toHaveProperty('source_path')
    expect(definition).not.toHaveProperty('access')
  })

  it('parses only JSON objects and fingerprints the visible variables draft', () => {
    expect(parseDialogueVariables('{"route":"open"}')).toEqual({ route: 'open' })
    expect(parseDialogueVariables('[]')).toBeNull()
    expect(parseDialogueVariables('null')).toBeNull()
    expect(parseDialogueVariables('{')).toBeNull()
    expect(dialogueDraftSnapshot(dialogue(), '{"route":"open"}')).toContain('variablesText')
    expect(dialogueDraftSnapshot(null, '{}')).toBe('')
  })

  it('filters trimmed identity text and allocates portable case-folded dialogue ids', () => {
    const entries = [entry(), entry({ id: 'route_two', title: 'Second Route', description: 'A quiet branch.' })]
    expect(filterDialogueEntries(entries, '  QUIET ')).toEqual([entries[1]])
    expect(filterDialogueEntries(entries, '   ')).toEqual(entries)
    expect(hasDialogueIdCollision(['Agent_Intro'], 'agent_intro')).toBe(true)
    expect(nextDialogueId(['New_Dialogue', 'new_dialogue_2'])).toBe('new_dialogue_3')
  })

  it('creates and duplicates isolated one-node drafts', () => {
    const created = createDialogueDraft(['new_dialogue'], 'New Dialogue', 'Opening line.', 'aoi')
    expect(created).toMatchObject({ id: 'new_dialogue_2', start_node_id: 'start' })
    expect(created.nodes.start).toMatchObject({ speaker_id: 'aoi', text: 'Opening line.' })

    const source = dialogue()
    const duplicated = duplicateDialogueDraft(source, ['intro_copy'], 'Intro Copy')
    duplicated.nodes.start.text = 'Changed'
    expect(duplicated.id).toBe('intro_copy_2')
    expect(source.nodes.start.text).toBe('Hello.')
  })
})

describe('Dialogue graph topology editing', () => {
  it('orders reachable nodes breadth-first and appends unreachable nodes deterministically', () => {
    const source = dialogue({
      nodes: {
        start: node({ choices: [choice({ next_node_id: 'right' }), choice({ next_node_id: 'left' })] }),
        left: node({ next_node_id: 'start' }),
        right: node({ is_ending: true }),
        z_orphan: node(),
        a_orphan: node(),
      },
    })
    expect(dialogueNodeOrder(source)).toEqual(['start', 'right', 'left', 'a_orphan', 'z_orphan'])
    expect(dialogueImplicitTerminalIds(source)).toEqual(['z_orphan', 'a_orphan'])
    expect(dialogueTerminalCount(source)).toBe(3)
  })

  it('appends collision-free nodes without mutating the source graph', () => {
    const source = dialogue({ nodes: { start: node(), node: node(), node_2: node() } })
    const result = appendDialogueNode(source, 'emi')
    expect(result.node_id).toBe('node_3')
    expect(result.dialogue.nodes.node_3.speaker_id).toBe('emi')
    expect(source.nodes).not.toHaveProperty('node_3')
  })

  it('renames nodes and every graph reference without mutating the source', () => {
    const source = dialogue()
    const result = renameDialogueNode(source, 'end', 'finale')
    expect(result).toMatchObject({ node_id: 'finale', changed: true, error: null })
    expect(result.dialogue.nodes).not.toHaveProperty('end')
    expect(result.dialogue.nodes.middle.choices[0].next_node_id).toBe('finale')
    expect(source.nodes.middle.choices[0].next_node_id).toBe('end')

    expect(renameDialogueNode(source, 'end', '../bad').error).toBe('invalid_id')
    expect(renameDialogueNode(source, 'end', 'start').error).toBe('node_exists')
    expect(renameDialogueNode(source, 'missing', 'finale').error).toBe('node_missing')
  })

  it('protects referenced, start, and final nodes and deletes isolated nodes immutably', () => {
    const source = dialogue({ nodes: { ...dialogue().nodes, orphan: node() } })
    expect(deleteDialogueNode(source, 'middle')).toMatchObject({ error: 'node_referenced', references: ['start'] })
    expect(deleteDialogueNode(source, 'start').error).toBe('start_node')
    expect(deleteDialogueNode(dialogue({ nodes: { start: node() } }), 'start').error).toBe('last_node')

    const deleted = deleteDialogueNode(source, 'orphan')
    expect(deleted).toMatchObject({ changed: true, error: null, selected_node_id: 'start' })
    expect(deleted.dialogue.nodes).not.toHaveProperty('orphan')
    expect(source.nodes).toHaveProperty('orphan')
  })

  it('switches flow modes and bounds choice creation without mutating nodes', () => {
    const source = node({ next_node_id: 'end', ending_type: 'legacy' })
    const choices = setDialogueNodeFlowMode(source, 'start', 'choices', ['start', 'end'], 'New choice')
    expect(dialogueFlowMode(choices)).toBe('choices')
    expect(choices.choices).toEqual([choice({ text: 'New choice' })])
    expect(source.next_node_id).toBe('end')

    const ending = setDialogueNodeFlowMode(choices, 'start', 'end', ['start', 'end'], 'Unused')
    expect(ending).toMatchObject({ next_node_id: null, choices: [], is_ending: true })
    const linear = setDialogueNodeFlowMode(ending, 'start', 'linear', ['start', 'end'], 'Unused')
    expect(linear).toMatchObject({ next_node_id: 'end', is_ending: false, ending_type: null })

    const full = node({ choices: Array.from({ length: 32 }, () => choice()) })
    expect(appendDialogueChoice(full, 'start', ['end'], 'Overflow').choices).toHaveLength(32)
    expect(removeDialogueChoice(choices, 0).choices).toEqual([])
  })
})

describe('Dialogue choice relationship editing', () => {
  it('sorts, adds, renames, updates, and removes relationship changes immutably', () => {
    const source = choice({ relationship_changes: { zed: -0.2, aoi: 0.1 } })
    const characters = [{ id: 'aoi', name: 'Aoi' }, { id: 'emi', name: 'Emi' }]
    expect(dialogueRelationshipEntries(source)).toEqual([['aoi', 0.1], ['zed', -0.2]])
    expect(availableDialogueRelationshipCharacters(source, characters)).toEqual([{ id: 'emi', name: 'Emi' }])

    const added = addDialogueRelationship(source, 'emi')
    const renamed = renameDialogueRelationship(added, 'emi', 'hana')
    const updated = setDialogueRelationshipDelta(renamed, 'hana', '-0.45')
    const removed = removeDialogueRelationship(updated, 'zed')
    expect(removed.relationship_changes).toEqual({ aoi: 0.1, hana: -0.45 })
    expect(source.relationship_changes).toEqual({ zed: -0.2, aoi: 0.1 })
  })

  it('derives referenced identities and prefers project display names', () => {
    const source = entry({
      nodes: {
        start: node({ speaker_id: 'agent_guide', choices: [choice({ relationship_changes: { aoi: 0.1 } })] }),
        end: node({ speaker_id: null, is_ending: true }),
      },
    })
    expect(mergeDialogueCharacters(catalog([source]), [{ id: 'agent_guide', name: 'Agent Guide' }]))
      .toEqual([{ id: 'agent_guide', name: 'Agent Guide' }, { id: 'aoi', name: 'Aoi' }])
  })
})
