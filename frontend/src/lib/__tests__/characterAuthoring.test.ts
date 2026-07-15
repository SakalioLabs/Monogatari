import { describe, expect, it } from 'vitest'

import {
  buildStoryCharacter,
  characterFormFromStory,
  characterFormSnapshot,
  characterSpritePlaceholder,
  characterSummaryFromStory,
  createCharacterForm,
  fillMissingCharacterSpritePaths,
  filterCharacterSummaries,
  parseCharacterKnowledgeRefs,
  toggleCharacterKnowledgeRef,
  validateCharacterForm,
  type CharacterForm,
} from '../characterAuthoring'
import type { StoryCharacterInfo } from '../storyContent'

function validForm(): CharacterForm {
  return {
    ...createCharacterForm(),
    id: 'agent_guide',
    name: 'Agent Guide',
  }
}

function storyCharacter(): StoryCharacterInfo {
  return {
    id: 'guide',
    name: ' Guide ',
    description: 'Description',
    background: 'Background',
    emotion: '',
    currentEmotion: 'happy',
    personality: {
      openness: 2,
      conscientiousness: -1,
      extraversion: '0.75',
      agreeableness: null,
      neuroticism: 0.25,
      speech_style: 'Measured',
    },
    portrait_path: 'assets/portraits/guide.png',
    sprite_path: null,
    sprite_paths: { ' happy ': ' assets/sprites/guide_happy.png ', empty: ' ' },
    live2d_model_path: null,
    model_3d_path: 'assets/models/guide.glb',
    relationships: { ' player ': 2, rival: -2 },
    knowledge_entries: [{ topic: 'Origin', content: 'Stable lore' }],
    knowledge_refs: [],
    knowledge: ['legacy_lore', 'legacy_lore'],
    emotion_modifiers: { ' happy ': 'Bright' },
  }
}

describe('Character authoring form domain', () => {
  it('creates isolated forms with runtime-compatible defaults', () => {
    const first = createCharacterForm()
    const second = createCharacterForm()

    first.relationships.player = 1
    first.knowledge_entries.push({ topic: 'A', content: 'B' })
    expect(second).toEqual({
      id: '', name: '', description: '', background: '', speech_style: '',
      default_emotion: 'neutral', live2d_model_path: '', model_3d_path: '', portrait_path: '',
      sprite_path: '', sprite_paths: {}, openness: 0.5, conscientiousness: 0.5,
      extraversion: 0.5, agreeableness: 0.5, neuroticism: 0.5, relationships: {},
      knowledge_entries: [], knowledge_refs: '', emotion_modifiers: {},
    })
  })

  it('normalizes loaded runtime characters into isolated editable forms', () => {
    const source = storyCharacter()
    const form = characterFormFromStory(source)

    expect(form).toMatchObject({
      id: 'guide',
      default_emotion: 'happy',
      openness: 1,
      conscientiousness: 0,
      extraversion: 0.75,
      agreeableness: 0.5,
      neuroticism: 0.25,
      relationships: { player: 1, rival: -1 },
      sprite_paths: { happy: 'assets/sprites/guide_happy.png' },
      knowledge_refs: 'legacy_lore',
      emotion_modifiers: { happy: 'Bright' },
    })
    form.knowledge_entries[0].content = 'Changed'
    expect(source.knowledge_entries?.[0].content).toBe('Stable lore')
  })

  it('maps complete story records to stable browser summaries', () => {
    expect(characterSummaryFromStory(storyCharacter())).toEqual({
      id: 'guide',
      name: ' Guide ',
      description: 'Description',
      emotion: 'happy',
      live2d_model_path: null,
      portrait_path: 'assets/portraits/guide.png',
      sprite_path: null,
    })
  })

  it('filters summaries by trimmed case-insensitive identity text without aliasing input', () => {
    const summaries = [characterSummaryFromStory(storyCharacter()), {
      id: 'rival', name: 'Rival', description: 'A challenger', emotion: 'angry',
      live2d_model_path: null, portrait_path: null, sprite_path: null,
    }]

    expect(filterCharacterSummaries(summaries, '  GUIDE ')).toEqual([summaries[0]])
    expect(filterCharacterSummaries(summaries, 'challenger')).toEqual([summaries[1]])
    expect(filterCharacterSummaries(summaries, '')).toEqual(summaries)
    expect(filterCharacterSummaries(summaries, '')).not.toBe(summaries)
  })
})

describe('Character authoring validation and serialization', () => {
  it('returns stable validation codes and rejects case-folded portable ID collisions', () => {
    const form = validForm()
    const context = { isNew: true, existingCharacterIds: ['Agent_Guide'], knownKnowledgeIds: ['world'] }

    expect(validateCharacterForm({ ...form, name: '' }, context)?.code).toBe('required')
    expect(validateCharacterForm({ ...form, id: '../guide' }, context)?.code).toBe('invalid_id')
    expect(validateCharacterForm(form, context)?.code).toBe('duplicate_id')
    expect(validateCharacterForm({ ...form, id: 'new_guide', knowledge_refs: 'world, missing' }, context))
      .toEqual({ code: 'unknown_knowledge', knowledge_refs: ['missing'] })
    expect(validateCharacterForm({
      ...form,
      id: 'new_guide',
      knowledge_entries: [{ topic: ' ', content: 'Lore' }],
    }, context)?.code).toBe('invalid_private_knowledge')
    expect(validateCharacterForm({ ...form, id: 'new_guide', knowledge_refs: 'unavailable' }, {
      ...context,
      knownKnowledgeIds: null,
    })).toBeNull()
  })

  it('deduplicates and toggles ordered knowledge references', () => {
    expect(parseCharacterKnowledgeRefs('world, route, world, , ending')).toEqual(['world', 'route', 'ending'])
    expect(toggleCharacterKnowledgeRef('world, route', 'world')).toBe('route')
    expect(toggleCharacterKnowledgeRef('world, route', 'ending')).toBe('world, route, ending')
    expect(toggleCharacterKnowledgeRef('world', '  ')).toBe('world')
  })

  it('builds the exact trimmed and bounded story payload without mutating its form', () => {
    const form: CharacterForm = {
      ...validForm(),
      id: ' agent_guide ',
      name: ' Agent Guide ',
      description: ' Description ',
      background: ' Background ',
      speech_style: ' Measured ',
      default_emotion: ' happy ',
      live2d_model_path: ' ',
      model_3d_path: ' assets/models/guide.glb ',
      portrait_path: ' assets/portraits/guide.png ',
      sprite_path: ' assets/sprites/guide.png ',
      sprite_paths: { ' happy ': ' assets/sprites/guide_happy.png ', empty: '' },
      openness: 2,
      conscientiousness: -1,
      extraversion: Number.NaN,
      relationships: { ' player ': 2, rival: -2 },
      knowledge_entries: [{ topic: ' Origin ', content: ' Stable lore ' }],
      knowledge_refs: 'world, world, route',
      emotion_modifiers: { ' happy ': ' Bright ' },
    }
    const original = structuredClone(form)
    const character = buildStoryCharacter(form)

    expect(form).toEqual(original)
    expect(character).toMatchObject({
      id: 'agent_guide',
      name: 'Agent Guide',
      description: 'Description',
      background: 'Background',
      personality: { openness: 1, conscientiousness: 0, extraversion: 0, speech_style: 'Measured' },
      emotion: 'happy',
      live2d_model_path: null,
      model_3d_path: 'assets/models/guide.glb',
      portrait_path: 'assets/portraits/guide.png',
      sprite_path: 'assets/sprites/guide.png',
      sprite_paths: { happy: 'assets/sprites/guide_happy.png' },
      relationships: { player: 1, rival: -1 },
      knowledge_entries: [{ topic: 'Origin', content: 'Stable lore' }],
      knowledge_refs: ['world', 'route'],
      emotion_modifiers: { happy: ' Bright ' },
    })
  })

  it('fingerprints canonical payloads for dirty-state comparisons', () => {
    const form = validForm()
    const baseline = characterFormSnapshot(form)
    form.openness = 2
    expect(characterFormSnapshot(form)).not.toBe(baseline)
    form.openness = 1
    expect(characterFormSnapshot(form)).toBe(characterFormSnapshot({ ...form, openness: 3 }))
  })

  it('fills missing expression sprites and creates portable placeholders without aliasing input', () => {
    const paths = { happy: 'assets/sprites/happy.png', empty: ' ' }
    expect(fillMissingCharacterSpritePaths(paths, ' assets/sprites/base.png ', ['happy', 'sad']))
      .toEqual({ happy: 'assets/sprites/happy.png', sad: 'assets/sprites/base.png' })
    expect(paths).toEqual({ happy: 'assets/sprites/happy.png', empty: ' ' })
    expect(characterSpritePlaceholder('guide', 'sad', ' assets/sprites/base.png ')).toBe('assets/sprites/base.png')
    expect(characterSpritePlaceholder(' ', 'sad', '')).toBe('assets/sprites/character_sad.png')
  })
})
