import { act, renderHook, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { PROFILES_PIN_CAP, useProfilesQuickSwitch } from '@/composables/useProfilesQuickSwitch'
import { useProfilesQuickSwitchStore } from '@/features/profiles/stores'

const PINNED_KEY = 'ccr:profiles:pinned:claude'
const RECENT_KEY = 'ccr:profiles:recent:claude'

const setup = (names: string[] = [], onPinLimit?: () => void) =>
  renderHook(() =>
    useProfilesQuickSwitch({
      platform: 'claude',
      getProfileNames: () => names,
      onPinLimit,
    }),
  )

describe('useProfilesQuickSwitch smoke', () => {
  beforeEach(() => {
    localStorage.clear()
    useProfilesQuickSwitchStore.setState({
      pinnedByPlatform: {},
      recentByPlatform: {},
    })
  })

  it('persists pinned and recent lists to localStorage and reads them back', () => {
    const first = setup(['a', 'b', 'c'])
    act(() => {
      expect(first.result.current.pin('a')).toBe(true)
      expect(first.result.current.pin('b')).toBe(true)
      first.result.current.recordUse('c')
    })
    expect(JSON.parse(localStorage.getItem(PINNED_KEY) ?? '[]')).toEqual(['a', 'b'])
    expect(JSON.parse(localStorage.getItem(RECENT_KEY) ?? '[]')).toEqual(['c'])
    first.unmount()

    const second = setup(['a', 'b', 'c'])
    expect(second.result.current.pinned).toEqual(['a', 'b'])
    expect(second.result.current.recent).toEqual(['c'])
    expect(second.result.current.stableTargets).toEqual(['a', 'b'])
    second.unmount()
  })

  it('cleans stale names against the current list and writes back storage', async () => {
    useProfilesQuickSwitchStore.setState({
      pinnedByPlatform: { claude: ['keep', 'gone', 'renamed-old'] },
      recentByPlatform: { claude: ['keep', 'gone-recent'] },
    })
    localStorage.setItem(PINNED_KEY, JSON.stringify(['keep', 'gone', 'renamed-old']))
    localStorage.setItem(RECENT_KEY, JSON.stringify(['keep', 'gone-recent']))
    const switcher = setup(['keep'])
    await waitFor(() => expect(switcher.result.current.pinned).toEqual(['keep']))
    expect(switcher.result.current.recent).toEqual(['keep'])
    expect(JSON.parse(localStorage.getItem(PINNED_KEY) ?? '[]')).toEqual(['keep'])
    expect(JSON.parse(localStorage.getItem(RECENT_KEY) ?? '[]')).toEqual(['keep'])
    switcher.unmount()
  })

  it('preserves persisted names until the first profile snapshot is ready', async () => {
    useProfilesQuickSwitchStore.setState({
      pinnedByPlatform: { claude: ['keep', 'gone'] },
      recentByPlatform: { claude: ['keep', 'gone-recent'] },
    })
    localStorage.setItem(PINNED_KEY, JSON.stringify(['keep', 'gone']))
    localStorage.setItem(RECENT_KEY, JSON.stringify(['keep', 'gone-recent']))
    let names: string[] | null = null
    const switcher = renderHook(() =>
      useProfilesQuickSwitch({
        platform: 'claude',
        getProfileNames: () => names,
      }),
    )
    expect(switcher.result.current.pinned).toEqual(['keep', 'gone'])
    names = ['keep']
    switcher.rerender()
    await waitFor(() => expect(switcher.result.current.pinned).toEqual(['keep']))
    expect(JSON.parse(localStorage.getItem(PINNED_KEY) ?? '[]')).toEqual(['keep'])
    switcher.unmount()
  })

  it('rejects the 9th pin without evicting existing pins and fires onPinLimit', () => {
    const names = Array.from({ length: PROFILES_PIN_CAP + 1 }, (_, index) => `p${index + 1}`)
    const onPinLimit = vi.fn()
    const switcher = setup(names, onPinLimit)
    act(() => {
      for (const name of names.slice(0, PROFILES_PIN_CAP)) {
        expect(switcher.result.current.pin(name)).toBe(true)
      }
    })
    expect(switcher.result.current.canPin).toBe(false)
    act(() => {
      expect(switcher.result.current.pin(names[PROFILES_PIN_CAP] ?? '')).toBe(false)
    })
    expect(onPinLimit).toHaveBeenCalledTimes(1)
    expect(switcher.result.current.pinned).toEqual(names.slice(0, PROFILES_PIN_CAP))
    switcher.unmount()
  })

  it('never numbers the recent list and recordUse never touches numbering', () => {
    const switcher = setup(['a', 'b', 'c', 'd'])
    act(() => {
      switcher.result.current.pin('a')
      switcher.result.current.pin('b')
      switcher.result.current.recordUse('c')
      switcher.result.current.recordUse('d')
      switcher.result.current.recordUse('c')
    })
    expect(switcher.result.current.stableTargets).toEqual(['a', 'b'])
    expect(switcher.result.current.recent.slice(0, 2)).toEqual(['c', 'd'])
    act(() => {
      switcher.result.current.recordUse('a')
    })
    expect(switcher.result.current.recentNotPinned).toEqual(['c', 'd'])
    expect(switcher.result.current.stableTargets).toEqual(['a', 'b'])
    switcher.unmount()
  })

  it('follows renames in both pinned and recent lists', () => {
    let names = ['old', 'other']
    const switcher = renderHook(() =>
      useProfilesQuickSwitch({
        platform: 'claude',
        getProfileNames: () => names,
      }),
    )
    act(() => {
      switcher.result.current.pin('old')
      switcher.result.current.recordUse('old')
    })
    act(() => {
      names = ['new', 'other']
      switcher.result.current.renamePinned('old', 'new')
    })
    expect(switcher.result.current.pinned).toEqual(['new'])
    expect(switcher.result.current.recent).toEqual(['new'])
    switcher.unmount()
  })
})
