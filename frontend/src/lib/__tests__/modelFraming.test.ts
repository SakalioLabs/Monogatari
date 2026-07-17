import { describe, expect, it } from 'vitest'
import {
  calculateModelCameraDistance,
  selectPrimaryModelBounds,
  type ModelBounds3,
} from '../modelFraming'

describe('selectPrimaryModelBounds', () => {
  it('keeps connected scene meshes and excludes a small distant export remnant', () => {
    const bounds: ModelBounds3[] = [
      { min: [-19, -1, -44], max: [20, 17, -31] },
      { min: [-16, -1, -10], max: [16, 9, 10] },
      { min: [-15, 0, -8], max: [16, 8, 9] },
      { min: [-184, -10, -23], max: [-182, -10, -21] },
    ]

    const result = selectPrimaryModelBounds(bounds)

    expect(result).toEqual({
      bounds: { min: [-19, -1, -44], max: [20, 17, 10] },
      includedIndices: [0, 1, 2],
      excludedIndices: [3],
    })
  })

  it('selects the dominant distant cluster instead of a tiny origin duplicate', () => {
    const result = selectPrimaryModelBounds([
      { min: [-942, -20, -151], max: [-640, 211, 151] },
      { min: [-938, -20, -147], max: [-644, 211, 147] },
      { min: [-1, -1, -1], max: [1, 1, 1] },
    ])

    expect(result?.includedIndices).toEqual([0, 1])
    expect(result?.excludedIndices).toEqual([2])
    expect(result?.bounds).toEqual({ min: [-942, -20, -151], max: [-640, 211, 151] })
  })

  it('returns null when no candidate has finite visible extent', () => {
    expect(selectPrimaryModelBounds([
      { min: [0, 0, 0], max: [0, 0, 0] },
      { min: [Number.NaN, 0, 0], max: [1, 1, 1] },
    ])).toBeNull()
  })
})

describe('calculateModelCameraDistance', () => {
  it('uses cover framing for a wide scene on a portrait viewport', () => {
    const verticalFov = Math.PI / 4
    const portraitAspect = 390 / 844
    const sceneDistance = calculateModelCameraDistance([2.55, 1.25, 2.8], verticalFov, portraitAspect, 'scene')
    const characterDistance = calculateModelCameraDistance([2.55, 1.25, 2.8], verticalFov, portraitAspect, 'character')

    expect(sceneDistance).toBeCloseTo(2.45, 2)
    expect(characterDistance).toBeGreaterThan(10)
  })
})
