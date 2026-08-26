import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, render, renderHook, screen, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { createMemoryRouter, RouterProvider } from 'react-router'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { grokProfileFixtures } from './fixtures/profiles'
import { useProfilesQuickSwitchStore } from '@/features/profiles/stores'

const notify = vi.hoisted(() => ({
  confirm: vi.fn(async () => true),
  success: vi.fn(),
  error: vi.fn(),
  warning: vi.fn(),
}))

const grokMocks = vi.hoisted(() => ({
  env: { env_type: 'local', id: 'local' },
  listGrokProfiles: vi.fn(),
  deleteGrokProfile: vi.fn(),
  applyGrokProfile: vi.fn(),
  updateGrokProfile: vi.fn(),
  grokProfileOff: vi.fn(),
}))

vi.mock('@/configs/surfaceNotify', () => ({ surfaceNotify: notify }))

vi.mock('@/api/runtime/environment', () => ({
  getCurrentEnvironment: vi.fn(async () => grokMocks.env),
}))

vi.mock('@/api', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/api')>()
  return {
    ...actual,
    getCurrentEnvironment: vi.fn(async () => grokMocks.env),
    grokApi: {
      ...actual.grokApi,
      listGrokProfiles: grokMocks.listGrokProfiles,
      deleteGrokProfile: grokMocks.deleteGrokProfile,
      applyGrokProfile: grokMocks.applyGrokProfile,
      updateGrokProfile: grokMocks.updateGrokProfile,
      grokProfileOff: grokMocks.grokProfileOff,
    },
  }
})

const { useGrokProfilesPage, runProfileRecovery } = await import(
  '@/features/grok/profiles/useGrokProfilesPage'
)
const { GrokProfilesView } = await import('@/features/grok/GrokProfilesView')

const wrapHook = (children: ReactNode) => {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>
}

const renderView = () => {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  const router = createMemoryRouter([{ path: '*', element: <GrokProfilesView /> }], {
    initialEntries: ['/'],
  })
  return render(
    <QueryClientProvider client={client}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  )
}

describe('grok profiles view lock', () => {
  beforeEach(() => {
    grokMocks.env = { env_type: 'local', id: 'local' }
    grokMocks.listGrokProfiles.mockReset()
    grokMocks.deleteGrokProfile.mockReset()
    grokMocks.applyGrokProfile.mockReset()
    notify.confirm.mockReset()
    notify.confirm.mockResolvedValue(true)
    notify.error.mockReset()
    notify.success.mockReset()
    grokMocks.listGrokProfiles.mockResolvedValue({
      status: 'ok',
      profiles: grokProfileFixtures,
      current_profile: 'grok-current',
      activation: 'active',
    })
    localStorage.clear()
    useProfilesQuickSwitchStore.setState({ pinnedByPlatform: {}, recentByPlatform: {} })
  })

  it('keeps local-only fail-closed and retains pins', async () => {
    grokMocks.env = { env_type: 'remote', id: 'ssh-1' }
    useProfilesQuickSwitchStore.getState().pin('grok', 'grok-current')
    const hook = renderHook(() => useGrokProfilesPage(), {
      wrapper: ({ children }) => wrapHook(children),
    })
    await waitFor(() => {
      expect(hook.result.current.localOnly).toBe(true)
    })
    expect(hook.result.current.records).toEqual([])
    expect(useProfilesQuickSwitchStore.getState().pinnedByPlatform.grok).toEqual(['grok-current'])
  })

  it('force-deletes active or drifted once and does not loop', async () => {
    grokMocks.deleteGrokProfile
      .mockResolvedValueOnce({
        status: 'blocked',
        reason: 'active',
        message: 'active profile',
      })
      .mockResolvedValueOnce({
        status: 'blocked',
        reason: 'active',
        message: 'still blocked',
      })
    const hook = renderHook(() => useGrokProfilesPage(), {
      wrapper: ({ children }) => wrapHook(children),
    })
    await waitFor(() => {
      expect(hook.result.current.profiles.length).toBeGreaterThan(0)
    })
    await act(async () => {
      await hook.result.current.handleDelete('grok-current')
    })
    expect(grokMocks.deleteGrokProfile).toHaveBeenCalledTimes(2)
    expect(grokMocks.deleteGrokProfile.mock.calls[0]?.[1]).toEqual({ force: false })
    expect(grokMocks.deleteGrokProfile.mock.calls[1]?.[1]).toEqual({ force: true })
    expect(notify.error).toHaveBeenCalled()
  })

  it('does not offer force for unsafe_missing_entry_state', async () => {
    grokMocks.deleteGrokProfile.mockResolvedValue({
      status: 'blocked',
      reason: 'unsafe_missing_entry_state',
      message: 'unsafe',
    })
    const hook = renderHook(() => useGrokProfilesPage(), {
      wrapper: ({ children }) => wrapHook(children),
    })
    await waitFor(() => {
      expect(hook.result.current.profiles.length).toBeGreaterThan(0)
    })
    await act(async () => {
      await hook.result.current.handleDelete('grok-current')
    })
    expect(grokMocks.deleteGrokProfile).toHaveBeenCalledTimes(1)
    expect(notify.confirm).toHaveBeenCalledTimes(1)
    expect(notify.error).toHaveBeenCalled()
  })

  it('runs rename recovery apply then cleanup without looping', async () => {
    grokMocks.applyGrokProfile.mockResolvedValue({ status: 'applied', profile: 'new' })
    grokMocks.deleteGrokProfile.mockResolvedValue({ status: 'deleted' })
    await runProfileRecovery({
      status: 'rename_apply_failed',
      oldName: 'old',
      newName: 'new',
    })
    expect(grokMocks.applyGrokProfile).toHaveBeenCalledWith('new')
    expect(grokMocks.deleteGrokProfile).not.toHaveBeenCalled()
    await runProfileRecovery({
      status: 'rename_cleanup_failed',
      oldName: 'old',
      newName: 'new',
    })
    expect(grokMocks.deleteGrokProfile).toHaveBeenCalledWith('old')
  })

  it('maps recovery onto hook notice state', async () => {
    const hook = renderHook(() => useGrokProfilesPage(), {
      wrapper: ({ children }) => wrapHook(children),
    })
    await waitFor(() => {
      expect(hook.result.current.profiles.length).toBeGreaterThan(0)
    })
    act(() => {
      hook.result.current.handleEditorDone({
        status: 'recovery',
        kind: 'rename_cleanup_failed',
        message: 'cleanup failed',
        oldName: 'old',
        newName: 'new',
      })
    })
    expect(hook.result.current.recovery?.status).toBe('rename_cleanup_failed')
  })

  it('renders enabled/total summary from projected records', async () => {
    renderView()
    await waitFor(() => {
      expect(screen.getByTestId('profiles-surface')).toBeTruthy()
    })
    expect(screen.getByTestId('profiles-stat-total').textContent).toBe(
      String(grokProfileFixtures.length),
    )
    expect(screen.getByTestId('profiles-card-grid').textContent).toMatch(
      /official|third_party|官方|第三方/,
    )
  })
})
