import { describe, expect, it } from 'vitest'

import { resolveWindowChromeMode } from '@/utils/windowChrome'

describe('window chrome mode smoke', () => {
  it('uses native window chrome for all Tauri desktop platforms', () => {
    expect(resolveWindowChromeMode(true, 'windows')).toBe('native')
    expect(resolveWindowChromeMode(true, 'macos')).toBe('native')
    expect(resolveWindowChromeMode(true, 'linux')).toBe('native')
  })

  it('keeps custom chrome only for non-Tauri browser preview', () => {
    expect(resolveWindowChromeMode(false, 'windows')).toBe('custom')
    expect(resolveWindowChromeMode(false, 'macos')).toBe('custom')
    expect(resolveWindowChromeMode(false, 'linux')).toBe('custom')
  })
})
