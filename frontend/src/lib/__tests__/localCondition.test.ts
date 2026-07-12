import { describe, expect, it } from 'vitest'

import { evaluateLocalCondition } from '../localCondition'

describe('shared local condition evaluator', () => {
  it('reads context, variables, and flags without mutating its scope', () => {
    const scope = {
      context: { relationship: 0.75 },
      variables: { route: 'aoi', score: 2 },
      flags: { introduced: true },
    }

    expect(evaluateLocalCondition(
      'relationship >= 0.5 && getVariable("route") == "aoi" && hasFlag("introduced")',
      scope,
    )).toEqual({ result: true, supported: true, error: null })
    expect(scope).toEqual({
      context: { relationship: 0.75 },
      variables: { route: 'aoi', score: 2 },
      flags: { introduced: true },
    })
  })

  it('returns stable unsupported evidence instead of guessing a branch', () => {
    expect(evaluateLocalCondition('customFunction()')).toMatchObject({
      result: false,
      supported: false,
      error: 'Error: unsupported_condition:customFunction()',
    })
    expect(evaluateLocalCondition('')).toEqual({
      result: false,
      supported: false,
      error: 'condition_empty',
    })
  })
})
