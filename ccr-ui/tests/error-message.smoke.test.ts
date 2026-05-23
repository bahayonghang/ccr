import { describe, expect, it } from 'vitest'
import { getErrorMessage } from '@/types/api'

describe('getErrorMessage', () => {
  it('falls back when Error.message is empty', () => {
    expect(getErrorMessage(new Error(''), '未知错误')).toBe('未知错误')
  })

  it('falls back when an object message is whitespace', () => {
    expect(getErrorMessage({ message: '   ' }, '未知错误')).toBe('未知错误')
  })

  it('falls back when a string error is whitespace', () => {
    expect(getErrorMessage('   ', '未知错误')).toBe('未知错误')
  })

  it('trims readable error messages', () => {
    expect(getErrorMessage(new TypeError('bad'), '未知错误')).toBe('bad')
    expect(getErrorMessage(' bad ', '未知错误')).toBe('bad')
  })

  it('uses non-generic error names before fallback', () => {
    expect(getErrorMessage(new TypeError(''), '未知错误')).toBe('TypeError')
  })

  it('keeps global Vue error toasts from ending with an empty reason', () => {
    const toastMessage = `应用错误: ${getErrorMessage(new Error(''), '未知错误')}`

    expect(toastMessage).toBe('应用错误: 未知错误')
  })
})
