import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { createMemoryRouter, RouterProvider } from 'react-router'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { CLAUDE_SECRET_KEYS, stripCredentials } from '@/configs/profileCredentials'
import { profilesConfigs } from '@/configs/profiles'
import {
  claudeProfilePresentation,
  profilePresentations,
} from '@/configs/profilePresentation'
import { ProfilesSurface } from '@/features/platform/profiles/ProfilesSurface'
import { ProfilesNotice } from '@/features/platform/profiles/shared'
import { claudeProfileFixtures, claudeDisplayRecords } from './fixtures/profiles'

const SENTINEL = 'sentinel-auth-token-9f3c2a1b'

const apiMocks = vi.hoisted(() => ({
  listClaudeProfiles: vi.fn(),
  listCodexProfiles: vi.fn(),
  exportClaudeProfiles: vi.fn(),
  getClaudeProfilesRaw: vi.fn(),
  saveClaudeProfilesRaw: vi.fn(),
  getCodexProfilesRaw: vi.fn(),
  saveCodexProfilesRaw: vi.fn(),
  getCurrentEnvironment: vi.fn(async () => ({ env_type: 'local', id: 'local' })),
  listGrokProfiles: vi.fn(),
}))

vi.mock('@/utils/download', () => ({
  downloadTextFile: vi.fn(),
}))

vi.mock('@/api/runtime/environment', () => ({
  getCurrentEnvironment: () => apiMocks.getCurrentEnvironment(),
}))

vi.mock('@/api/domains/claude', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/api/domains/claude')>()
  return {
    ...actual,
    getClaudeProfilesRaw: apiMocks.getClaudeProfilesRaw,
    saveClaudeProfilesRaw: apiMocks.saveClaudeProfilesRaw,
  }
})

vi.mock('@/api/domains/codex', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/api/domains/codex')>()
  return {
    ...actual,
    getCodexProfilesRaw: apiMocks.getCodexProfilesRaw,
    saveCodexProfilesRaw: apiMocks.saveCodexProfilesRaw,
  }
})

vi.mock('@/api', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/api')>()
  return {
    ...actual,
    getCurrentEnvironment: apiMocks.getCurrentEnvironment,
    listClaudeProfiles: apiMocks.listClaudeProfiles,
    listCodexProfiles: apiMocks.listCodexProfiles,
    exportClaudeProfiles: apiMocks.exportClaudeProfiles,
    grokApi: {
      ...actual.grokApi,
      listGrokProfiles: apiMocks.listGrokProfiles,
    },
  }
})

const { ClaudeProfilesView } = await import('@/features/claude/ClaudeProfilesView')
const { CodexProfilesView } = await import('@/features/codex/CodexProfilesView')
const { GrokProfilesView } = await import('@/features/grok/GrokProfilesView')

const wrap = (node: ReactNode) => {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  const router = createMemoryRouter([{ path: '*', element: node }], { initialEntries: ['/'] })
  return (
    <QueryClientProvider client={client}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  )
}

describe('profiles platform wiring', () => {
  beforeEach(() => {
    localStorage.clear()
    apiMocks.listClaudeProfiles.mockResolvedValue({
      profiles: claudeProfileFixtures.map((item) => ({
        ...item,
        auth_token: SENTINEL,
      })),
      current_profile: 'claude-current',
      can_off: true,
    })
    apiMocks.listCodexProfiles.mockResolvedValue({
      profiles: [],
      current_profile: null,
      can_off: false,
    })
    apiMocks.listGrokProfiles.mockResolvedValue({
      status: 'ok',
      profiles: [],
      current_profile: null,
      activation: 'inactive',
    })
    apiMocks.exportClaudeProfiles.mockResolvedValue({ content: 'name = "x"', filename: 'profiles.toml' })
    apiMocks.getClaudeProfilesRaw.mockResolvedValue({
      status: 'ok',
      content: '',
      token: 'tok',
      path: 'profiles.toml',
      exists: true,
    })
    apiMocks.getCodexProfilesRaw.mockResolvedValue({
      status: 'ok',
      content: '',
      token: 'tok',
      path: 'config.toml',
      exists: true,
    })
  })

  it('renders Claude with canOff and raw-source, hiding the sentinel', async () => {
    const log = vi.spyOn(console, 'log').mockImplementation(() => undefined)
    const info = vi.spyOn(console, 'info').mockImplementation(() => undefined)
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => undefined)
    const error = vi.spyOn(console, 'error').mockImplementation(() => undefined)
    render(wrap(<ClaudeProfilesView />))
    await waitFor(() => {
      expect(screen.getByTestId('profiles-surface').getAttribute('data-can-off')).toBe('true')
    })
    expect(screen.getByTestId('profiles-edit-source')).toBeTruthy()
    expect(document.body.textContent ?? '').not.toContain(SENTINEL)
    const projected = claudeProfilePresentation.project(
      stripCredentials({ ...claudeProfileFixtures[0], auth_token: SENTINEL }, CLAUDE_SECRET_KEYS),
      { current: 'claude-current' },
    )
    expect(JSON.stringify(projected)).not.toContain(SENTINEL)
    fireEvent.click(screen.getByText('profilesSurface.export'))
    await waitFor(() => {
      expect(apiMocks.exportClaudeProfiles).toHaveBeenCalledWith(false)
    })
    expect(log.mock.calls.flat().join(' ')).not.toContain(SENTINEL)
    expect(info.mock.calls.flat().join(' ')).not.toContain(SENTINEL)
    expect(warn.mock.calls.flat().join(' ')).not.toContain(SENTINEL)
    expect(error.mock.calls.flat().join(' ')).not.toContain(SENTINEL)
    log.mockRestore()
    info.mockRestore()
    warn.mockRestore()
    error.mockRestore()
  })

  it('renders Codex without Off when can_off is false, with raw-source', async () => {
    render(wrap(<CodexProfilesView />))
    await waitFor(() => {
      expect(screen.getByTestId('profiles-surface').getAttribute('data-can-off')).toBe('false')
    })
    expect(screen.getByTestId('profiles-edit-source')).toBeTruthy()
    expect(screen.queryByTestId('profiles-off-banner')).toBeNull()
  })

  it('renders Grok without raw-source and shows notice', async () => {
    render(wrap(<GrokProfilesView />))
    await waitFor(() => {
      expect(screen.getByTestId('profiles-surface')).toBeTruthy()
    })
    expect(screen.queryByTestId('profiles-edit-source')).toBeNull()
    const router = createMemoryRouter(
      [
        {
          path: '*',
          element: (
            <ProfilesSurface
              platformKey="grok-notice"
              presentation={profilePresentations.grok}
              records={claudeDisplayRecords}
              current={null}
              environmentLabel="本机"
              environmentOk
              canOff={false}
              onAdd={vi.fn()}
              onEdit={vi.fn()}
              onApply={vi.fn()}
              onOff={vi.fn(async () => undefined)}
              onReload={vi.fn()}
              notice={<ProfilesNotice tone="warning" message="rename_apply_failed" />}
            />
          ),
        },
      ],
      { initialEntries: ['/'] },
    )
    render(<RouterProvider router={router} />)
    expect(screen.getByTestId('profiles-notice').textContent).toContain('rename_apply_failed')
  })

  it('renders antigravity from the real registry with an empty snapshot', async () => {
    const snapshot = await profilesConfigs.antigravity.list()
    expect(snapshot).toEqual({ profiles: [], current: null })
    const presentation = profilePresentations.antigravity
    const router = createMemoryRouter(
      [
        {
          path: '*',
          element: (
            <ProfilesSurface
              platformKey="antigravity"
              presentation={presentation}
              records={[]}
              current={null}
              environmentLabel="本机"
              environmentOk
              canOff={false}
              onAdd={vi.fn()}
              onEdit={vi.fn()}
              onApply={vi.fn()}
              onOff={vi.fn(async () => undefined)}
              onReload={vi.fn()}
            />
          ),
        },
      ],
      { initialEntries: ['/'] },
    )
    render(<RouterProvider router={router} />)
    expect(screen.getByTestId('profiles-surface')).toBeTruthy()
    expect(screen.getByTestId('profiles-page-header')).toBeTruthy()
    expect(screen.getByTestId('profiles-stat-strip')).toBeTruthy()
    expect(screen.getByTestId('profiles-toolbar')).toBeTruthy()
    expect(screen.getByTestId('profiles-empty')).toBeTruthy()
  })
})
