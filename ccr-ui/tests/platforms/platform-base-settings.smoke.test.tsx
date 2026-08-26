import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { describe, expect, it, vi } from 'vitest'
import type { SettingsConfig } from '@/configs/settings'
import { BaseSettings } from '@/features/platform'

const notify = {
  success: vi.fn(),
  error: vi.fn(),
  warning: vi.fn(),
  confirm: vi.fn().mockResolvedValue(true),
}

const makeConfig = (cacheKey: string, model: string): SettingsConfig => ({
  cacheKey,
  homePath: '/',
  module: 'test',
  i18nPrefix: 'test',
  titleKey: `title-${cacheKey}`,
  subtitleKey: `sub-${cacheKey}`,
  tabs: [{ id: 'model', labelKey: 'model-tab' }],
  fields: [{ id: 'model', tab: 'model', kind: 'text', labelKey: 'model-label' }],
  features: {},
  notify,
  load: async () => ({ model }),
  save: async () => undefined,
})

const renderWithQuery = (node: ReactNode) => {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  })
  return render(<QueryClientProvider client={client}>{node}</QueryClientProvider>)
}

describe('BaseSettings shared implementation', () => {
  it('renders the same base for two platform configs', async () => {
    const first = makeConfig('settings-a', 'sonnet')
    const second = makeConfig('settings-b', 'gpt')
    const { unmount } = renderWithQuery(<BaseSettings config={first} t={(key) => key} />)
    await waitFor(() => {
      expect(screen.getByText('title-settings-a')).toBeTruthy()
    })
    unmount()
    renderWithQuery(<BaseSettings config={second} t={(key) => key} />)
    await waitFor(() => {
      expect(screen.getByText('title-settings-b')).toBeTruthy()
    })
  })
})
