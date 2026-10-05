import { describe, expect, it } from 'vitest'
import { getErrorMessage } from '@/types/api'

describe('getErrorMessage', () => {
  it('passes Tauri string rejections through unchanged', () => {
    // Tauri v2 invoke 对 Result<_, String> 以纯字符串 reject，不能丢给 fallback
    expect(getErrorMessage('检测到 WAF 挑战页面（响应为 HTML）', '未知错误')).toBe(
      '检测到 WAF 挑战页面（响应为 HTML）'
    )
    expect(getErrorMessage('HTTP 401: 签到失败', '未知错误')).toBe('HTTP 401: 签到失败')
  })

  it('extracts message from Error instances', () => {
    expect(getErrorMessage(new Error('boom'), '未知错误')).toBe('boom')
  })

  it('falls back for non-string non-Error values', () => {
    expect(getErrorMessage(42, '未知错误')).toBe('未知错误')
    expect(getErrorMessage(null, '未知错误')).toBe('未知错误')
    expect(getErrorMessage(undefined, '未知错误')).toBe('未知错误')
  })

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

  it('keeps global error toasts from ending with an empty reason', () => {
    const toastMessage = `应用错误: ${getErrorMessage(new Error(''), '未知错误')}`

    expect(toastMessage).toBe('应用错误: 未知错误')
  })
})
