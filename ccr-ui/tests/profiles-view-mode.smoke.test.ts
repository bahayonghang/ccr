import { act, renderHook } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { useProfilesViewStore } from '@/features/profiles/stores'
import { useProfilesSurface } from '@/features/platform/profiles/useProfilesSurface'
import { claudeDisplayRecords } from './fixtures/profiles'

const renderSurface = (platformKey: string) =>
  renderHook(() =>
    useProfilesSurface({
      platformKey,
      records: claudeDisplayRecords,
      current: 'claude-current',
    }),
  )

describe('profiles view mode persistence', () => {
  beforeEach(() => {
    localStorage.clear()
    useProfilesViewStore.setState({ viewByPlatform: {} })
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('keeps the selected view after unmount and remount', () => {
    const first = renderSurface('claude')
    act(() => {
      first.result.current.setViewMode('table')
    })
    expect(first.result.current.viewMode).toBe('table')
    expect(localStorage.getItem('ccr:profiles:view:claude')).toBe('table')
    first.unmount()

    const second = renderSurface('claude')
    expect(second.result.current.viewMode).toBe('table')
    second.unmount()
  })

  it('isolates view mode by platform key', () => {
    const claude = renderSurface('claude')
    const codex = renderSurface('codex')
    act(() => {
      claude.result.current.setViewMode('table')
    })
    expect(claude.result.current.viewMode).toBe('table')
    expect(codex.result.current.viewMode).toBe('card')
    claude.unmount()
    codex.unmount()
  })

  it('still switches view when localStorage.setItem throws', () => {
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
      throw new Error('quota')
    })
    const hook = renderSurface('claude')
    expect(() => {
      act(() => {
        hook.result.current.setViewMode('table')
      })
    }).not.toThrow()
    expect(hook.result.current.viewMode).toBe('table')
    hook.unmount()
  })
})
