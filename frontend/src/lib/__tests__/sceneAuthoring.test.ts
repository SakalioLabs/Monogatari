import { describe, expect, it } from 'vitest'

import type { SceneDefinition } from '../storyContent'
import {
  hasSceneIdCollision,
  normalizeSceneDefinition,
  validateSceneDefinition,
} from '../sceneAuthoring'

function scene(overrides: Partial<SceneDefinition> = {}): SceneDefinition {
  return {
    id: 'studio_night',
    name: 'Studio Night',
    background_path: 'assets/backgrounds/studio_night.svg',
    bgm_path: 'assets/audio/studio.ogg',
    weather: null,
    time_of_day: 'night',
    tags: ['studio'],
    ...overrides,
  }
}

describe('scene authoring contracts', () => {
  it('normalizes optional text and stable tag ordering', () => {
    expect(normalizeSceneDefinition(scene({
      id: ' studio_night ',
      name: ' Studio Night ',
      weather: '  ',
      tags: [' night ', 'studio', 'night', ''],
    }))).toEqual(scene({
      weather: null,
      tags: ['night', 'studio'],
    }))
  })

  it('accepts a portable scene with supported media', () => {
    expect(validateSceneDefinition(scene())).toEqual([])
  })

  it('rejects traversal, unsupported media, control text, and invalid ids', () => {
    const issues = validateSceneDefinition(scene({
      id: '../studio',
      name: 'Bad\u0000Name',
      background_path: 'assets/../private/studio.svg',
      bgm_path: 'assets/audio/studio.exe',
    }))
    expect(issues).toContain('Scene ID must be a portable 1-128 character id.')
    expect(issues).toContain('Name must contain 1-256 non-control characters.')
    expect(issues).toContain('Background path must use portable project-relative segments.')
    expect(issues).toContain('BGM path must use a supported audio extension.')
  })

  it('rejects whitespace and non-ASCII asset path segments', () => {
    expect(validateSceneDefinition(scene({
      background_path: 'assets/backgrounds/studio night.svg',
      bgm_path: 'assets/audio/\u591c.ogg',
    }))).toEqual(expect.arrayContaining([
      'Background path must use portable project-relative segments.',
      'BGM path must use portable project-relative segments.',
    ]))
  })

  it('detects case-folded portable scene ID collisions', () => {
    expect(hasSceneIdCollision(['studio_night', 'Agent_Route'], ' agent_route ')).toBe(true)
    expect(hasSceneIdCollision(['studio_night'], 'new_route')).toBe(false)
  })
})
