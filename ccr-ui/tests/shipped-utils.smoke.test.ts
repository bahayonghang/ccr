import { act, renderHook } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { copyText } from '@/utils/clipboard'
import {
  CodexCardStyles,
  CodexTheme,
  formatRelativeTime,
  formatTimestamp,
  generateId,
  getModelDisplayName,
  getProviderColor,
  handleCardHover,
  isValidGitHubToken,
  isValidUrl,
  maskToken,
} from '@/utils/codexHelpers'
import { downloadTextFile } from '@/utils/download'
import { extractErrorMessage, getErrorMessage, showErrorSafe } from '@/utils/errorHandler'
import {
  formatJsonInput,
  maskSecret,
  normalizeStringListInput,
  parseJsonInput,
  splitCommandInput,
  stringifyCommandInput,
} from '@/utils/opencode'
import { useFuzzySearch } from '@/features/mcp/useFuzzySearch'

describe('clipboard copyText', () => {
  it('uses the async Clipboard API then falls back to execCommand', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined)
    Object.defineProperty(navigator, 'clipboard', {
      configurable: true,
      value: { writeText },
    })
    await expect(copyText('hello')).resolves.toBe(true)
    expect(writeText).toHaveBeenCalledWith('hello')

    writeText.mockRejectedValueOnce(new Error('denied'))
    const exec = vi.fn().mockReturnValue(true)
    Object.defineProperty(document, 'execCommand', {
      configurable: true,
      value: exec,
    })
    await expect(copyText('fallback')).resolves.toBe(true)
    expect(exec).toHaveBeenCalledWith('copy')
  })
})

describe('downloadTextFile', () => {
  it('creates an object URL and clicks an anchor', () => {
    const createObjectURL = vi.fn(() => 'blob:mock')
    const revokeObjectURL = vi.fn()
    vi.stubGlobal('URL', { createObjectURL, revokeObjectURL })
    const click = vi.fn()
    HTMLAnchorElement.prototype.click = click
    downloadTextFile('notes.txt', 'body')
    expect(createObjectURL).toHaveBeenCalled()
    expect(click).toHaveBeenCalled()
    expect(revokeObjectURL).toHaveBeenCalledWith('blob:mock')
    vi.unstubAllGlobals()
  })
})

describe('codexHelpers', () => {
  it('masks tokens, colors providers, and formats time', () => {
    expect(maskToken('')).toBe('')
    expect(maskToken('abcd')).toBe('****')
    expect(maskToken('abcdefghij')).toBe('abcd****ghij')
    expect(getProviderColor('OpenAI')).toBe('#10a37f')
    expect(getProviderColor('Unknown')).toBe('#8b5cf6')
    expect(formatTimestamp('not-a-date')).toBe('-')
    expect(formatTimestamp('2026-01-02T03:04:05Z')).toMatch(/2026/)
    expect(formatRelativeTime(Date.now() - 30_000)).toBe('刚刚')
    expect(formatRelativeTime(Date.now() - 120_000)).toContain('分钟前')
    expect(isValidUrl('https://example.com')).toBe(true)
    expect(isValidUrl('not a url')).toBe(false)
    expect(isValidGitHubToken('ghp_abcdefghijklmnopqrstuvwxyz0123456789')).toBe(true)
    expect(isValidGitHubToken('nope')).toBe(false)
    expect(getModelDisplayName('gpt-4')).toBe('GPT-4')
    expect(getModelDisplayName('custom-model')).toBe('custom-model')
    expect(generateId('t')).toMatch(/^t-/)
    expect(CodexCardStyles.base.borderRadius).toBe('var(--radius-2xl)')
    expect(CodexTheme.primary).toBe('#6366f1')
    const el = document.createElement('div')
    handleCardHover(el, true)
    expect(el.style.transform).toBe('translateY(-0.25rem)')
    handleCardHover(el, false)
    expect(el.style.transform).toBe('translateY(0)')
  })
})

describe('opencode helpers', () => {
  it('formats, parses, and masks command input', () => {
    expect(formatJsonInput(null)).toBe('{}')
    expect(formatJsonInput({ a: 1 })).toContain('"a"')
    expect(parseJsonInput('{"a":1}', { a: 0 })).toEqual({ a: 1 })
    expect(parseJsonInput('', { a: 0 })).toEqual({ a: 0 })
    expect(splitCommandInput('npx -y "foo bar"')).toEqual(['npx', '-y', 'foo bar'])
    expect(stringifyCommandInput(['npx', '-y'])).toBe('npx -y')
    expect(maskSecret(undefined)).toBe('not configured')
    expect(maskSecret('{env:TOKEN}')).toBe('{env:TOKEN}')
    expect(maskSecret('abcd')).toMatch(/••••/)
    expect(maskSecret('abcdefghijklmnop')).toMatch(/^abcd/)
    expect(normalizeStringListInput('a\n b \n\nc')).toEqual(['a', 'b', 'c'])
  })
})

describe('errorHandler', () => {
  it('extracts messages and forwards showError', () => {
    expect(getErrorMessage(new Error('boom'))).toBe('boom')
    expect(getErrorMessage('plain')).toBe('plain')
    expect(getErrorMessage({ message: 'obj' })).toBe('obj')
    expect(getErrorMessage(1)).toBe('发生未知错误')
    expect(extractErrorMessage('plain')).toBe('plain')
    expect(extractErrorMessage({ error: 'nested' })).toBe('nested')
    expect(extractErrorMessage({ cause: 'cause' })).toBe('cause')
    expect(extractErrorMessage({})).toBeNull()
    const showError = vi.fn()
    showErrorSafe({ showError }, new Error('x'), 'fallback')
    expect(showError).toHaveBeenCalledWith('x')
    showErrorSafe({}, new Error('x'), 'fallback')
    expect(showError).toHaveBeenCalledTimes(1)
  })
})

describe('mcp useFuzzySearch', () => {
  it('returns all items then filters by query', () => {
    const { result } = renderHook(() =>
      useFuzzySearch([{ name: 'filesystem' }, { name: 'github' }], ['name'], { threshold: 0.4 }),
    )
    expect(result.current.results).toHaveLength(2)
    act(() => {
      result.current.setQuery('file')
    })
    expect(result.current.results.map((item) => item.name)).toEqual(['filesystem'])
  })
})
