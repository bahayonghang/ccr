import { describe, expect, it } from 'vitest'

import { toOpenJsonValue } from '@/api/_shared'

describe('typed open JSON boundary', () => {
  it('normalizes nested JSON payloads and omits undefined object fields', () => {
    expect(toOpenJsonValue({ enabled: true, skipped: undefined, nested: [1, 'two', null] }))
      .toEqual({ enabled: true, nested: [1, 'two', null] })
  })

  it.each([BigInt(1), Number.POSITIVE_INFINITY, Symbol('invalid'), undefined])(
    'rejects non-JSON payload value %s',
    value => {
      expect(() => toOpenJsonValue(value, 'Test payload')).toThrow(
        'Test payload must be JSON-compatible',
      )
    },
  )

  it.each([new Date(), new Map([['key', 'value']])])(
    'rejects non-plain object payload %s',
    value => {
      expect(() => toOpenJsonValue(value, 'Test payload')).toThrow(
        'Test payload must contain only plain JSON objects',
      )
    },
  )

  it('rejects circular references', () => {
    const value: Record<string, unknown> = {}
    value.self = value

    expect(() => toOpenJsonValue(value, 'Test payload')).toThrow(
      'Test payload must not contain circular references',
    )
  })
})
