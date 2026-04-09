import { afterEach, describe, expect, it, vi } from 'vitest'

const { getCurrentWindowSafe, loggerWarn } = vi.hoisted(() => ({
  getCurrentWindowSafe: vi.fn(),
  loggerWarn: vi.fn(),
}))

vi.mock('@/utils/tauriWindow', () => ({
  getCurrentWindowSafe,
}))

vi.mock('@/utils/logger', () => ({
  logger: {
    warn: loggerWarn,
  },
}))

import {
  shouldSyncNativeWindowAppearance,
  syncNativeWindowAppearance,
} from '@/utils/nativeWindowAppearance'

const originalPlatformDescriptor = Object.getOwnPropertyDescriptor(window.navigator, 'platform')

const setNavigatorPlatform = (value: string) => {
  Object.defineProperty(window.navigator, 'platform', {
    configurable: true,
    value,
  })
}

afterEach(() => {
  getCurrentWindowSafe.mockReset()
  loggerWarn.mockReset()

  if (originalPlatformDescriptor) {
    Object.defineProperty(window.navigator, 'platform', originalPlatformDescriptor)
  } else {
    Reflect.deleteProperty(window.navigator, 'platform')
  }
})

describe('native window appearance smoke', () => {
  it('syncs macOS native window theme and background color for light mode', async () => {
    setNavigatorPlatform('MacIntel')

    const setTheme = vi.fn().mockResolvedValue(undefined)
    const setBackgroundColor = vi.fn().mockResolvedValue(undefined)
    getCurrentWindowSafe.mockResolvedValue({
      setTheme,
      setBackgroundColor,
    })

    expect(shouldSyncNativeWindowAppearance()).toBe(true)

    await syncNativeWindowAppearance('light')

    expect(setTheme).toHaveBeenCalledWith('light')
    expect(setBackgroundColor).toHaveBeenCalledWith('#EEF4FF')
    expect(loggerWarn).not.toHaveBeenCalled()
  })

  it('skips native window syncing outside macOS', async () => {
    setNavigatorPlatform('Win32')

    expect(shouldSyncNativeWindowAppearance()).toBe(false)

    await syncNativeWindowAppearance('dark')

    expect(getCurrentWindowSafe).not.toHaveBeenCalled()
  })
})
