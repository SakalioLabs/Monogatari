import { describe, expect, it } from 'vitest'

import {
  cleanRendererPathMap,
  rendererAssetValidationMessage,
  selectCharacterRendererAsset,
} from '../rendererAssets'

describe('renderer asset selection', () => {
  it('cleans expression maps without retaining blank or non-object values', () => {
    expect(cleanRendererPathMap(null)).toEqual({})
    expect(cleanRendererPathMap(['sprite.png'])).toEqual({})
    expect(cleanRendererPathMap({
      ' happy ': ' characters/aoi_happy.png ',
      blank: '  ',
      empty: null,
    })).toEqual({ happy: 'characters/aoi_happy.png' })
  })

  it.each([
    ['C:/private/model.glb', ['.glb'], 'Absolute paths are not portable'],
    ['https://example.test/model.glb', ['.glb'], 'Use a project-relative path'],
    ['assets/../private/model.glb', ['.glb'], 'Parent traversal is not allowed'],
    ['assets/model.exe', ['.glb'], 'Expected .glb'],
  ])('rejects unsafe or unsupported path %s', (path, extensions, message) => {
    expect(rendererAssetValidationMessage(path, extensions)).toBe(message)
  })

  it('selects renderer sources in runtime fallback order', () => {
    const character = {
      live2d_model_path: 'assets/aoi.model3.json',
      model_3d_path: 'assets/aoi.glb',
      sprite_path: 'assets/aoi.png',
    }
    expect(selectCharacterRendererAsset(character).mode).toBe('live2d')
    expect(selectCharacterRendererAsset(character, {
      blockedPaths: ['assets/aoi.model3.json'],
    }).mode).toBe('model3d')
    expect(selectCharacterRendererAsset(character, {
      blockedPaths: ['assets/aoi.model3.json', 'assets/aoi.glb'],
    }).mode).toBe('sprite')
  })

  it('uses expression sprites and rejects invalid sources when validation is requested', () => {
    const choice = selectCharacterRendererAsset({
      live2d_model_path: '../escape.model3.json',
      sprite_paths: {
        neutral: 'assets/neutral.png',
        happy: 'assets/happy.png',
      },
      emotion: 'neutral',
    }, { expression: 'happy', validatePaths: true })

    expect(choice).toMatchObject({ mode: 'sprite', path: 'assets/happy.png' })
    expect(selectCharacterRendererAsset({ sprite_path: '../escape.png' }, {
      validatePaths: true,
    }).mode).toBe('placeholder')
  })
})
