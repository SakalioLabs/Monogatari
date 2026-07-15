import { describe, expect, it } from 'vitest'
import { reactive } from 'vue'

import { cloneJsonRecord, cloneJsonValue, isJsonRecord, parseJsonRecord } from '../jsonValue'

describe('JSON value boundaries', () => {
  it('clones nested Vue proxies without retaining mutable references', () => {
    const source = reactive({ nested: { ready: true }, list: [{ score: 1 }] })
    const cloned = cloneJsonRecord(source)

    ;(cloned.nested as { ready: boolean }).ready = false
    ;(cloned.list as Array<{ score: number }>)[0].score = 2
    expect(source).toEqual({ nested: { ready: true }, list: [{ score: 1 }] })
  })

  it('recognizes and parses JSON objects without accepting arrays or scalars', () => {
    expect(isJsonRecord({ ready: true })).toBe(true)
    expect(isJsonRecord([])).toBe(false)
    expect(parseJsonRecord('{"ready":true}')).toEqual({ ready: true })
    expect(parseJsonRecord('[]')).toBeNull()
    expect(parseJsonRecord('null')).toBeNull()
    expect(parseJsonRecord('{')).toBeNull()
  })

  it('clones arrays and leaves JSON primitives unchanged', () => {
    expect(cloneJsonValue([{ value: 1 }, null, 'text'])).toEqual([{ value: 1 }, null, 'text'])
    expect(cloneJsonValue(false)).toBe(false)
  })
})
