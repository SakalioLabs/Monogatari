import { describe, expect, it } from 'vitest'

import {
  DEFAULT_SCENE_PRESENTATION,
  normalizeScenePresentation,
  scenePresentationPreset,
  scenePresentationStyles,
} from '../scenePresentation'

describe('scene presentation', () => {
  it('keeps legacy scenes on the cinematic bottom layout', () => {
    expect(normalizeScenePresentation(null)).toEqual(DEFAULT_SCENE_PRESENTATION)
    expect(scenePresentationStyles(null).dialogue).toMatchObject({
      bottom: '4%',
      left: '50%',
      width: '92%',
      height: '38%',
    })
  })

  it('clamps agent-authored numeric values and rejects unknown enums', () => {
    expect(normalizeScenePresentation({
      dialogue_position: 'unknown' as never,
      dialogue_width_percent: 200,
      character_anchor: 'unknown' as never,
      character_offset_x_percent: -90,
      background_dim_percent: 101,
    })).toMatchObject({
      dialogue_position: 'bottom',
      dialogue_width_percent: 96,
      character_anchor: 'center',
      character_offset_x_percent: -30,
      background_dim_percent: 80,
    })
  })

  it('creates isolated presets and maps side dialogue to stage coordinates', () => {
    const preset = scenePresentationPreset('split_left')
    const second = scenePresentationPreset('split_left')
    preset.dialogue_width_percent = 60
    expect(second.dialogue_width_percent).toBe(44)
    expect(scenePresentationStyles(second)).toMatchObject({
      character: { left: '25%', width: '50%' },
      dialogue: { right: '3%', top: '50%', height: '80%' },
    })
  })
})
