import { reactive } from 'vue'
import { describe, expect, it } from 'vitest'

import {
  countKnowledgeEntriesByCategory,
  createKnowledgeEntryEditForm,
  filterKnowledgeEntries,
  knowledgeEntryDefinitionFromEditForm,
  knowledgeEntryEditFormFromDefinition,
  knowledgeEntryEditFormSnapshot,
  knowledgeTaxonomy,
  nextKnowledgeEntryId,
  parseKnowledgeCommaList,
  validateKnowledgeEntryEditForm,
  type KnowledgeEntryEditForm,
} from '../knowledgeEditing'
import type { KnowledgeEntryDefinition } from '../knowledgeContent'

function entry(overrides: Partial<KnowledgeEntryDefinition> = {}): KnowledgeEntryDefinition {
  return {
    id: 'springtown_archive',
    title: 'Springtown Archive',
    category: 'world_lore',
    content: 'Canonical town history for character and Agent context.',
    tags: ['history', 'town'],
    related_entries: ['location_park'],
    importance: 0.8,
    metadata: { source: { kind: 'project' } },
    ...overrides,
  }
}

function form(overrides: Partial<KnowledgeEntryEditForm> = {}): KnowledgeEntryEditForm {
  return {
    id: 'agent_context',
    title: 'Agent Context',
    category: 'world_lore',
    content: 'Stable context for automated visual novel authoring.',
    tagsText: 'agent, delivery',
    relatedText: 'location_park',
    importance: 0.7,
    metadata: {},
    ...overrides,
  }
}

describe('knowledge editing domain', () => {
  it('clones reactive metadata without retaining nested references', () => {
    const source = reactive(entry())
    const draft = knowledgeEntryEditFormFromDefinition(source)

    expect(draft).toMatchObject({
      id: 'springtown_archive',
      tagsText: 'history, town',
      relatedText: 'location_park',
    })
    ;(draft.metadata.source as Record<string, unknown>).kind = 'draft'
    expect((source.metadata.source as Record<string, unknown>).kind).toBe('project')
  })

  it('builds a trimmed, bounded, isolated persistence definition', () => {
    const draft = form({
      id: ' agent_context ',
      title: ' Agent Context ',
      category: ' WORLD_LORE ',
      content: ' Canonical context. ',
      tagsText: 'agent, delivery, agent, ',
      relatedText: 'location_park, location_park',
      importance: 2,
      metadata: { nested: { enabled: true } },
    })

    const definition = knowledgeEntryDefinitionFromEditForm(draft)
    expect(definition).toEqual({
      id: 'agent_context',
      title: 'Agent Context',
      category: 'world_lore',
      content: 'Canonical context.',
      tags: ['agent', 'delivery'],
      related_entries: ['location_park'],
      importance: 1,
      metadata: { nested: { enabled: true } },
    })
    ;(definition.metadata.nested as Record<string, unknown>).enabled = false
    expect((draft.metadata.nested as Record<string, unknown>).enabled).toBe(true)
  })

  it('allocates deterministic IDs and isolated create forms', () => {
    expect(nextKnowledgeEntryId(['new_entry_1', 'new_entry_3'])).toBe('new_entry_2')
    expect(nextKnowledgeEntryId([], ' agent_note ')).toBe('agent_note_1')

    const first = createKnowledgeEntryEditForm(['new_entry_1'])
    const second = createKnowledgeEntryEditForm(['new_entry_1'])
    first.metadata.owner = 'first'
    expect(first.id).toBe('new_entry_2')
    expect(second.metadata).toEqual({})
  })

  it('parses stable comma lists and snapshots form state', () => {
    expect(parseKnowledgeCommaList(' world, route, world, , ending '))
      .toEqual(['world', 'route', 'ending'])
    const draft = form()
    const baseline = knowledgeEntryEditFormSnapshot(draft)
    draft.title = 'Changed'
    expect(knowledgeEntryEditFormSnapshot(draft)).not.toBe(baseline)
  })

  it('filters entries and derives sorted taxonomy evidence', () => {
    const entries = [
      entry(),
      entry({ id: 'agent_route', title: 'Agent Route', category: 'route', tags: ['agent'], related_entries: [] }),
    ]
    expect(knowledgeTaxonomy(entries)).toEqual({
      categories: ['route', 'world_lore'],
      tags: ['agent', 'history', 'town'],
    })
    expect(countKnowledgeEntriesByCategory(entries, 'world_lore')).toBe(1)
    expect(filterKnowledgeEntries(entries, { query: 'CANONICAL', category: 'world_lore', tag: 'history' }))
      .toEqual([entries[0]])
    expect(filterKnowledgeEntries(entries, { query: 'agent', category: null, tag: null }))
      .toEqual([entries[0], entries[1]])
  })

  it('reports identity and required-field failures by stable code', () => {
    expect(validateKnowledgeEntryEditForm(form({
      id: 'Bad ID',
      title: ' ',
      category: 'bad category',
      content: ' ',
      relatedText: '',
    }), {
      original_entry_id: null,
      existing_entry_ids: ['Bad ID'],
    })).toEqual([
      { code: 'invalid_id' },
      { code: 'title_required' },
      { code: 'content_required' },
      { code: 'category_format' },
      { code: 'duplicate_id', target_id: 'Bad ID' },
    ])
  })

  it('reports invalid, self, and missing relations while accepting known references', () => {
    expect(validateKnowledgeEntryEditForm(form({
      relatedText: 'bad relation, agent_context, missing_entry, location_park',
    }), {
      original_entry_id: null,
      existing_entry_ids: ['location_park'],
    })).toEqual([
      { code: 'related_id', target_id: 'bad relation' },
      { code: 'related_self', target_id: 'agent_context' },
      { code: 'related_missing', target_id: 'missing_entry' },
    ])
  })
})
