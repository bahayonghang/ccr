import { describe, expect, it } from 'vitest'

import {
  resolveClientPlatform,
  resolveWindowChromeMode,
} from '@/utils/windowChrome'

describe('windowChrome helpers', () => {
  it('detects macOS, Windows, Linux, and unknown platforms', () => {
    expect(resolveClientPlatform('MacIntel')).toBe('macos')
    expect(resolveClientPlatform('Win32')).toBe('windows')
    expect(resolveClientPlatform('Linux x86_64')).toBe('linux')
    expect(resolveClientPlatform('SomethingElse')).toBe('unknown')
  })

  it('uses native chrome only for macOS Tauri windows', () => {
    expect(resolveWindowChromeMode(true, 'macos')).toBe('native')
    expect(resolveWindowChromeMode(true, 'windows')).toBe('custom')
    expect(resolveWindowChromeMode(true, 'linux')).toBe('custom')
    expect(resolveWindowChromeMode(false, 'macos')).toBe('custom')
  })
})
