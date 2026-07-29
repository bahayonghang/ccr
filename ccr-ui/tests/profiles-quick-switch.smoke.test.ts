import { ref } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const windowChromeMocks = vi.hoisted(() => ({
  getClientPlatform: vi.fn((): 'linux' | 'macos' | 'unknown' | 'windows' => 'windows'),
}))

vi.mock('@/utils/windowChrome', () => ({
  getClientPlatform: windowChromeMocks.getClientPlatform,
}))

import { PROFILES_PIN_CAP, useProfilesQuickSwitch } from '@/composables/useProfilesQuickSwitch'

const PINNED_KEY = 'ccr:profiles:pinned:claude'
const RECENT_KEY = 'ccr:profiles:recent:claude'

const setup = (names: string[] = [], onPinLimit?: () => void) =>
  useProfilesQuickSwitch({
    platform: 'claude',
    getProfileNames: () => names,
    onPinLimit,
  })

describe('useProfilesQuickSwitch smoke', () => {
  beforeEach(() => {
    windowChromeMocks.getClientPlatform.mockReturnValue('windows')
  })

  it('persists pinned and recent lists to localStorage and reads them back', () => {
    const names = ref(['a', 'b', 'c'])
    const first = useProfilesQuickSwitch({
      platform: 'claude',
      getProfileNames: () => names.value,
    })

    expect(first.pin('a')).toBe(true)
    expect(first.pin('b')).toBe(true)
    first.recordUse('c')

    expect(JSON.parse(localStorage.getItem(PINNED_KEY) ?? '[]')).toEqual(['a', 'b'])
    expect(JSON.parse(localStorage.getItem(RECENT_KEY) ?? '[]')).toEqual(['c'])

    const second = setup(['a', 'b', 'c'])
    expect(second.pinned.value).toEqual(['a', 'b'])
    expect(second.recent.value).toEqual(['c'])
    expect(second.stableTargets.value).toEqual(['a', 'b'])
  })

  it('cleans stale names against the current list and writes back storage', () => {
    localStorage.setItem(PINNED_KEY, JSON.stringify(['keep', 'gone', 'renamed-old']))
    localStorage.setItem(RECENT_KEY, JSON.stringify(['keep', 'gone-recent']))

    const switcher = setup(['keep'])

    expect(switcher.pinned.value).toEqual(['keep'])
    expect(switcher.recent.value).toEqual(['keep'])
    expect(JSON.parse(localStorage.getItem(PINNED_KEY) ?? '[]')).toEqual(['keep'])
    expect(JSON.parse(localStorage.getItem(RECENT_KEY) ?? '[]')).toEqual(['keep'])
  })

  it('cleans names that disappear from the list later, but keeps disabled ones', () => {
    const names = ref(['a', 'b'])
    const switcher = useProfilesQuickSwitch({
      platform: 'claude',
      getProfileNames: () => names.value,
    })
    switcher.pin('a')
    switcher.pin('b')

    // 禁用不等于删除：名称仍在列表里，钉选保留
    expect(switcher.pinned.value).toEqual(['a', 'b'])

    names.value = ['a']
    expect(switcher.pinned.value).toEqual(['a'])
    expect(JSON.parse(localStorage.getItem(PINNED_KEY) ?? '[]')).toEqual(['a'])
  })

  it('rejects the 9th pin without evicting existing pins and fires onPinLimit', () => {
    const names = Array.from({ length: PROFILES_PIN_CAP + 1 }, (_, i) => `p${i + 1}`)
    const onPinLimit = vi.fn()
    const switcher = setup(names, onPinLimit)

    for (const name of names.slice(0, PROFILES_PIN_CAP)) {
      expect(switcher.pin(name)).toBe(true)
    }
    expect(switcher.canPin.value).toBe(false)

    expect(switcher.pin(names[PROFILES_PIN_CAP])).toBe(false)
    expect(onPinLimit).toHaveBeenCalledTimes(1)
    expect(switcher.pinned.value).toEqual(names.slice(0, PROFILES_PIN_CAP))
    expect(switcher.stableTargets.value).toHaveLength(PROFILES_PIN_CAP)
  })

  it('never numbers the recent list and recordUse never touches numbering', () => {
    const switcher = setup(['a', 'b', 'c', 'd'])
    switcher.pin('a')
    switcher.pin('b')

    switcher.recordUse('c')
    switcher.recordUse('d')
    switcher.recordUse('c')

    // 编号来源恒等于钉选数组，与 recordUse 顺序无关
    expect(switcher.stableTargets.value).toEqual(['a', 'b'])
    // 最近列表倒序，且已钉选项不再重复出现在 recentNotPinned
    expect(switcher.recent.value.slice(0, 2)).toEqual(['c', 'd'])
    switcher.recordUse('a')
    expect(switcher.recentNotPinned.value).toEqual(['c', 'd'])
    expect(switcher.stableTargets.value).toEqual(['a', 'b'])
  })

  it('follows renames in both pinned and recent lists', () => {
    const switcher = setup(['old', 'other'])
    switcher.pin('old')
    switcher.recordUse('old')

    switcher.renamePinned('old', 'new')

    expect(switcher.pinned.value).toEqual(['new'])
    expect(switcher.recent.value).toEqual(['new'])
    expect(JSON.parse(localStorage.getItem(PINNED_KEY) ?? '[]')).toEqual(['new'])
    expect(JSON.parse(localStorage.getItem(RECENT_KEY) ?? '[]')).toEqual(['new'])
  })

  it('resolves the modifier hint from getClientPlatform', async () => {
    windowChromeMocks.getClientPlatform.mockReturnValue('windows')
    expect(setup(['a']).modifier.value).toBe('Ctrl')

    windowChromeMocks.getClientPlatform.mockReturnValue('linux')
    expect(setup(['a']).modifier.value).toBe('Ctrl')

    windowChromeMocks.getClientPlatform.mockReturnValue('macos')
    expect(setup(['a']).modifier.value).toBe('⌘')
  })
})
