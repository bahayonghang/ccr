import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { renderHook, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({
  getGrokDashboardOverview: vi.fn(),
  getCurrentEnvironment: vi.fn(),
  getCliVersion: vi.fn(),
}))

vi.mock('@/api', () => ({
  grokApi: {
    getGrokDashboardOverview: mocks.getGrokDashboardOverview,
  },
}))

vi.mock('@/api/runtime/environment', () => ({
  getCurrentEnvironment: mocks.getCurrentEnvironment,
}))

vi.mock('@/api/runtime/system', () => ({
  getCliVersion: mocks.getCliVersion,
}))

import { useGrokDashboard } from '@/composables/useGrokDashboard'
import { grokKeys } from '@/features/grok/queries'

const t = (key: string) => key

const okOverview = {
  status: 'ok' as const,
  activation: 'inactive' as const,
  activation_name: null,
  current_profile: null,
  auth_mode: null,
  profiles_total: 0,
  profiles_enabled: 0,
  config_exists: false,
  config_path_display: null,
}

const createWrapper = () => {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  })
  const Wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  )
  return Wrapper
}

describe('Grok Local-only dashboard refresh', () => {
  beforeEach(() => {
    mocks.getGrokDashboardOverview.mockReset()
    mocks.getCurrentEnvironment.mockReset()
    mocks.getCliVersion.mockReset()
  })

  it('puts environment id into overview and version Query keys', () => {
    expect(grokKeys.overview('local')).toEqual(['grok', 'overview', 'local'])
    expect(grokKeys.overview('wsl-1')).toEqual(['grok', 'overview', 'wsl-1'])
    expect(grokKeys.version('local')).not.toEqual(grokKeys.version('wsl-1'))
  })

  it('issues no overview or version request for a non-Local environment', async () => {
    mocks.getCurrentEnvironment.mockResolvedValue({ id: 'wsl-1', env_type: 'wsl' })
    const { result } = renderHook(() => useGrokDashboard({ t }), { wrapper: createWrapper() })

    await waitFor(() => {
      expect(result.current.localOnly).toBe(true)
    })
    expect(result.current.overview).toBeNull()
    expect(mocks.getGrokDashboardOverview).not.toHaveBeenCalled()
    expect(mocks.getCliVersion).not.toHaveBeenCalled()
  })

  it('treats unsupported_environment as Local-only and skips version detection', async () => {
    mocks.getCurrentEnvironment.mockResolvedValue({ id: 'local', env_type: 'local' })
    mocks.getGrokDashboardOverview.mockResolvedValue({
      status: 'unsupported_environment',
      env_type: 'ssh',
    })
    const { result } = renderHook(() => useGrokDashboard({ t }), { wrapper: createWrapper() })

    await waitFor(() => {
      expect(result.current.localOnly).toBe(true)
    })
    expect(result.current.overview).toBeNull()
    expect(result.current.localOnlyEnvType).toBe('ssh')
    expect(mocks.getCliVersion).not.toHaveBeenCalled()
  })

  it('fails closed when environment lookup rejects', async () => {
    mocks.getCurrentEnvironment.mockRejectedValue(new Error('environment down'))
    const { result } = renderHook(() => useGrokDashboard({ t }), { wrapper: createWrapper() })

    await waitFor(() => {
      expect(result.current.loadError).toBeTruthy()
    })
    expect(result.current.overview).toBeNull()
    expect(mocks.getGrokDashboardOverview).not.toHaveBeenCalled()
    expect(mocks.getCliVersion).not.toHaveBeenCalled()
  })

  it('runs version detection only after a Local overview succeeds', async () => {
    mocks.getCurrentEnvironment.mockResolvedValue({ id: 'local', env_type: 'local' })
    mocks.getGrokDashboardOverview.mockResolvedValue(okOverview)
    mocks.getCliVersion.mockResolvedValue({ status: 'ok', installed: true, version: '1.0.0' })
    const { result } = renderHook(() => useGrokDashboard({ t }), { wrapper: createWrapper() })

    await waitFor(() => {
      expect(result.current.overview).not.toBeNull()
    })
    await waitFor(() => {
      expect(mocks.getCliVersion).toHaveBeenCalled()
    })
    expect(result.current.localOnly).toBe(false)
  })

  it('clears overview on unsupported_environment and does not restore cached Local data', async () => {
    mocks.getCurrentEnvironment.mockResolvedValue({ id: 'local', env_type: 'local' })
    mocks.getGrokDashboardOverview.mockResolvedValue(okOverview)
    mocks.getCliVersion.mockResolvedValue({ status: 'ok', installed: true, version: '1.0.0' })
    const { result } = renderHook(() => useGrokDashboard({ t }), { wrapper: createWrapper() })
    await waitFor(() => {
      expect(result.current.overview).not.toBeNull()
    })
    await waitFor(() => {
      expect(mocks.getCliVersion).toHaveBeenCalled()
    })

    mocks.getGrokDashboardOverview.mockResolvedValue({
      status: 'unsupported_environment',
      env_type: 'wsl',
    })
    await result.current.refresh(true)
    await waitFor(() => {
      expect(result.current.localOnly).toBe(true)
    })
    expect(result.current.overview).toBeNull()

    const versionCalls = mocks.getCliVersion.mock.calls.length
    await result.current.refresh(true)
    expect(result.current.overview).toBeNull()
    expect(result.current.localOnly).toBe(true)
    expect(mocks.getCliVersion.mock.calls.length).toBe(versionCalls)
  })

  it('preserves overview and sets refreshError when a forced overview refresh rejects', async () => {
    mocks.getCurrentEnvironment.mockResolvedValue({ id: 'local', env_type: 'local' })
    mocks.getGrokDashboardOverview.mockResolvedValue(okOverview)
    mocks.getCliVersion.mockResolvedValue({ status: 'ok', installed: true, version: '1.0.0' })
    const { result } = renderHook(() => useGrokDashboard({ t }), { wrapper: createWrapper() })
    await waitFor(() => {
      expect(result.current.overview).not.toBeNull()
    })

    mocks.getGrokDashboardOverview.mockRejectedValue(new Error('overview down'))
    await result.current.refresh(true)
    await waitFor(() => {
      expect(result.current.refreshError).toBeTruthy()
    })
    expect(result.current.overview).not.toBeNull()
  })

  it('preserves a confirmed version and sets refreshError when version refresh rejects', async () => {
    mocks.getCurrentEnvironment.mockResolvedValue({ id: 'local', env_type: 'local' })
    mocks.getGrokDashboardOverview.mockResolvedValue(okOverview)
    mocks.getCliVersion.mockResolvedValue({ status: 'ok', installed: true, version: '1.0.0' })
    const { result } = renderHook(() => useGrokDashboard({ t }), { wrapper: createWrapper() })
    await waitFor(() => {
      expect(result.current.versionStatus).toBe('ok')
    })

    mocks.getCliVersion.mockRejectedValue(new Error('version down'))
    await result.current.refresh(true)
    await waitFor(() => {
      expect(result.current.refreshError).toBeTruthy()
    })
    expect(result.current.versionStatus).toBe('ok')
  })
})
