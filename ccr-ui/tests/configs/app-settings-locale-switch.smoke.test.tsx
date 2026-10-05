import { act, render, screen } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { DEFAULT_LOCALE, setLocale } from '@/i18n'
import { AppSettingsView } from '@/features/configs/AppSettingsView'

vi.mock('@/api/runtime/environment', async () => {
  const actual = await vi.importActual<typeof import('@/api/runtime/environment')>('@/api/runtime/environment')
  return {
    ...actual,
    isTauriEnvironment: () => false,
    getEnvironmentName: () => 'web',
    getTauriVersion: vi.fn().mockResolvedValue(null),
    shellGetPreferences: vi.fn().mockResolvedValue({
      confirm_before_exit: true,
      close_to_tray: false,
      open_panel_on_tray_click: true,
    }),
    shellSetPreferences: vi.fn(),
  }
})

const heroMeta = (container: HTMLElement): string =>
  container.querySelector('.app-settings-hero__meta')?.textContent ?? ''

describe('AppSettingsView live locale switch', () => {
  beforeEach(async () => {
    await setLocale(DEFAULT_LOCALE)
  })

  afterEach(async () => {
    await setLocale(DEFAULT_LOCALE)
  })

  it('refreshes section captions and hero meta on a zh → en → zh round trip without remount', async () => {
    const { container } = render(<AppSettingsView />)

    const appearanceButton = screen.getByTestId('settings-section-appearance')
    const shellButton = screen.getByTestId('settings-section-shell')

    expect(appearanceButton.textContent).toContain('主题与视觉基调')
    expect(shellButton.textContent).toContain('退出行为与布局尺寸')
    expect(heroMeta(container)).toContain('Web 预览')

    await act(async () => {
      await setLocale('en-US')
    })

    expect(screen.getByTestId('settings-section-appearance').textContent).toContain(
      'Theme and visual tone',
    )
    expect(screen.getByTestId('settings-section-shell').textContent).toContain(
      'Exit behavior and layout size',
    )
    expect(heroMeta(container)).toContain('Web preview')

    await act(async () => {
      await setLocale('zh-CN')
    })

    expect(screen.getByTestId('settings-section-appearance').textContent).toContain('主题与视觉基调')
    expect(screen.getByTestId('settings-section-shell').textContent).toContain('退出行为与布局尺寸')
    expect(heroMeta(container)).toContain('Web 预览')
    expect(heroMeta(container)).not.toContain('Web preview')
  })
})
