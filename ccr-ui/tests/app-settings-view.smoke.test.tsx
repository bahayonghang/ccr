import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
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

describe('AppSettingsView', () => {
  it('renders settings sections', () => {
    render(<AppSettingsView />)
    expect(screen.getByTestId('settings-section-appearance')).toBeTruthy()
    expect(screen.getByTestId('settings-theme-system')).toBeTruthy()
    expect(screen.getByTestId('settings-confirm-exit-toggle')).toBeTruthy()
  })
})
