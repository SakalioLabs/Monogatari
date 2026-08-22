import { describe, expect, it } from 'vitest'
import {
  buildControlledDialogueMessages,
  resolveControlledDialogueText,
} from '../controlledDialogueResponse'

const character = {
  id: 'aqua',
  name: 'Aqua',
  description: 'A proud water goddess who masks worry with confidence.',
  emotion: 'confident',
  portrait_path: null,
  sprite_path: null,
  personality: { speech_style: 'bright and theatrical' },
}

const generation = {
  context: 'The party has just accepted the toad quest and will leave for the field.',
  grounding_markers: ['toad', 'quest'],
  forbidden_markers: ['system', 'route'],
  max_characters: 120,
  max_sentences: 2,
}

describe('controlled dialogue response', () => {
  it('builds a character prompt that explicitly preserves the authored route', () => {
    const messages = buildControlledDialogueMessages(character, 'en', generation)

    expect(messages).toHaveLength(2)
    expect(messages[0].content).toContain('fixed-route visual novel')
    expect(messages[0].content).toContain('The author owns every story branch')
    expect(messages[0].content).toContain(generation.context)
    expect(messages[1]).toEqual({
      role: 'user',
      content: 'Write only the visible in-character reply for this node.',
    })
  })

  it('keeps only a grounded, bounded visible reply', () => {
    const reply = resolveControlledDialogueText(
      '<think>ignore this</think>The toad quest is beneath a goddess, but I will make it shine. We leave at once. Extra sentence.',
      'Fallback line.',
      generation,
    )

    expect(reply).toBe('The toad quest is beneath a goddess, but I will make it shine. We leave at once.')
  })

  it('uses the authored line when the model violates a marker contract', () => {
    expect(resolveControlledDialogueText(
      'I can alter the route for you.',
      'Authored fallback.',
      generation,
    )).toBe('Authored fallback.')
    expect(resolveControlledDialogueText(
      'We should leave now.',
      'Authored fallback.',
      generation,
    )).toBe('Authored fallback.')
  })
})
