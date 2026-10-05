import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { createMemoryRouter, RouterProvider } from 'react-router'
import { describe, expect, it, vi } from 'vitest'

vi.mock('@/api', () => {
  const runtimeSummary = {
    mode: 'runtime_only',
    official_login_state: { type: 'LoggedInUnsaved' },
    login_state: { type: 'LoggedInUnsaved' },
    auth_diagnosis: {
      observations: [],
      presumed_effective_source: null,
      custom_api_key_responses_present: false,
      unobservable: [],
    },
  }
  return {
    getCurrentEnvironment: vi.fn().mockResolvedValue({ env_type: 'local' }),
    listClaudeAuthAccounts: vi.fn().mockResolvedValue({
      accounts: [],
      login_state: { type: 'LoggedInUnsaved' },
      runtime_summary: runtimeSummary,
      current_profile_auth_mode: null,
      can_auth_off: true,
    }),
    getClaudeAuthCurrent: vi.fn().mockResolvedValue({
      logged_in: true,
      info: {
        email: 'a@b.c',
        account_uuid: 'u1',
        billing_type: null,
        subscription_type: null,
        rate_limit_tier: null,
        expires_at: null,
      },
      runtime_summary: runtimeSummary,
      login_state: { type: 'LoggedInUnsaved' },
      can_auth_off: true,
    }),
    saveClaudeAuth: vi.fn(),
    switchClaudeAuth: vi.fn(),
    deleteClaudeAuth: vi.fn(),
  }
})

vi.mock('@/api/domains/claude', () => ({
  claudeAuthOff: vi.fn().mockResolvedValue({ changed: true, warnings: [] }),
  claudeProfileOff: vi.fn().mockResolvedValue({ remaining_suppressors: [], warnings: [] }),
  listClaudeProfiles: vi.fn().mockResolvedValue({ profiles: [], current_profile: null, can_off: true }),
}))

import { ClaudeAuthView } from '@/features/claude/ClaudeAuthView'

const renderView = (node: ReactNode) => {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  const router = createMemoryRouter([{ path: '/', Component: () => node as React.ReactElement }], {
    initialEntries: ['/'],
  })
  return render(
    <QueryClientProvider client={client}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  )
}

describe('claude-auth-view', () => {
  it('renders diagnosis and auth-off after load', async () => {
    renderView(<ClaudeAuthView />)
    await waitFor(() => {
      expect(screen.getByTestId('claude-auth-diagnosis')).toBeTruthy()
    })
    expect(screen.getByTestId('claude-auth-off')).toBeTruthy()
    expect(screen.getByTestId('claude-auth-presumed-source')).toBeTruthy()
  })
})
