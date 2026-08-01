import { createApp, defineComponent, h, nextTick, ref } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const getGrokDashboardOverviewMock = vi.fn()
const getCurrentEnvironmentMock = vi.fn()
const getCliVersionMock = vi.fn()

vi.mock('@/api', () => ({
  grokApi: {
    getGrokDashboardOverview: (...args: unknown[]) => getGrokDashboardOverviewMock(...args),
  },
}))

vi.mock('@/api/runtime/environment', () => ({
  getCurrentEnvironment: (...args: unknown[]) => getCurrentEnvironmentMock(...args),
}))

vi.mock('@/api/runtime/system', () => ({
  getCliVersion: (...args: unknown[]) => getCliVersionMock(...args),
}))

vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string, params?: Record<string, unknown>) => (
      params ? `${key}:${JSON.stringify(params)}` : key
    ),
    locale: ref('en-US'),
  }),
}))

const localEnvironment = {
  id: 'local',
  env_type: 'local' as const,
  name: 'local',
  display_name: 'Local',
  description: 'Local environment',
  is_active: true,
}

const activeOverviewResponse = {
  status: 'ok' as const,
  activation: 'active' as const,
  activation_name: 'work',
  current_profile: 'work',
  auth_mode: 'env_key' as const,
  profiles_total: 2,
  profiles_enabled: 2,
  config_exists: true,
  config_path_display: '~/.grok/config.toml',
}

const cliVersionResponse = {
  platform: 'grok',
  installed: true,
  version: '1.2.3',
  status: 'ok',
  elapsed_ms: 20,
}

const mountComposable = async () => {
  const { useGrokDashboard } = await import('@/composables/useGrokDashboard')
  let state: ReturnType<typeof useGrokDashboard> | null = null
  const element = document.createElement('div')
  document.body.appendChild(element)

  const app = createApp(defineComponent({
    setup() {
      state = useGrokDashboard()
      return () => h('div')
    },
  }))

  app.mount(element)
  await nextTick()

  return {
    state: state!,
    unmount: () => {
      app.unmount()
      element.remove()
    },
  }
}

beforeEach(() => {
  vi.resetModules()
  document.body.innerHTML = ''
  getGrokDashboardOverviewMock.mockReset()
  getCurrentEnvironmentMock.mockReset()
  getCliVersionMock.mockReset()
  getCurrentEnvironmentMock.mockResolvedValue(localEnvironment)
  getGrokDashboardOverviewMock.mockResolvedValue(activeOverviewResponse)
  getCliVersionMock.mockResolvedValue(cliVersionResponse)
})

afterEach(() => {
  document.body.innerHTML = ''
})

describe('Grok dashboard smoke', () => {
  it('stops before Grok and version calls in a non-local environment', async () => {
    getCurrentEnvironmentMock.mockResolvedValueOnce({
      ...localEnvironment,
      id: 'wsl:Ubuntu',
      env_type: 'wsl',
    })
    const mounted = await mountComposable()

    try {
      await mounted.state.refresh(false)

      expect(mounted.state.localOnly.value).toBe(true)
      expect(mounted.state.localOnlyEnvType.value).toBe('wsl')
      expect(mounted.state.overview.value).toBeNull()
      expect(getGrokDashboardOverviewMock).not.toHaveBeenCalled()
      expect(getCliVersionMock).not.toHaveBeenCalled()
    } finally {
      mounted.unmount()
    }
  })

  it('treats the backend unsupported envelope as local-only and skips version detection', async () => {
    getGrokDashboardOverviewMock.mockResolvedValueOnce({
      status: 'unsupported_environment',
      env_type: 'ssh',
    })
    const mounted = await mountComposable()

    try {
      await mounted.state.refresh(false)

      expect(mounted.state.localOnly.value).toBe(true)
      expect(mounted.state.localOnlyEnvType.value).toBe('ssh')
      expect(mounted.state.loadError.value).toBeNull()
      expect(getCliVersionMock).not.toHaveBeenCalled()
    } finally {
      mounted.unmount()
    }
  })

  it('does not restore cached local data after an unsupported envelope', async () => {
    const mounted = await mountComposable()

    try {
      await mounted.state.refresh(false)
      getGrokDashboardOverviewMock.mockResolvedValue({
        status: 'unsupported_environment',
        env_type: 'ssh',
      })

      await mounted.state.refresh(true)
      await mounted.state.refresh(false)

      expect(mounted.state.localOnly.value).toBe(true)
      expect(mounted.state.overview.value).toBeNull()
      expect(getGrokDashboardOverviewMock).toHaveBeenCalledTimes(3)
      expect(getCliVersionMock).toHaveBeenCalledTimes(1)
    } finally {
      mounted.unmount()
    }
  })

  it('reuses overview and version caches across mounts within their TTL windows', async () => {
    const first = await mountComposable()
    try {
      await first.state.refresh(false)
    } finally {
      first.unmount()
    }

    const second = await mountComposable()
    try {
      await second.state.refresh(false)

      expect(getGrokDashboardOverviewMock).toHaveBeenCalledTimes(1)
      expect(getCliVersionMock).toHaveBeenCalledTimes(1)
      expect(second.state.currentProfileLabel.value).toBe('work')
      expect(second.state.readinessItems.value).toHaveLength(3)
    } finally {
      second.unmount()
    }
  })

  it('derives ready, drifted, and not-installed next actions in priority order', async () => {
    const mounted = await mountComposable()

    try {
      await mounted.state.refresh(false)
      expect(mounted.state.nextActions.value[0]).toMatchObject({
        key: 'open-settings',
        to: '/grok/settings',
        tone: 'success',
      })

      mounted.state.overview.value = {
        ...activeOverviewResponse,
        activation: 'drifted',
        current_profile: null,
      }
      await nextTick()

      expect(mounted.state.nextActions.value[0]).toMatchObject({
        key: 'repair-drift',
        to: '/grok/profiles',
        tone: 'danger',
      })
      expect(mounted.state.activationWarning.value?.tone).toBe('warning')
      expect(mounted.state.readinessItems.value.find(item => item.key === 'profiles')?.tone)
        .toBe('danger')

      mounted.state.versionStatus.value = 'not_installed'
      mounted.state.versionLabel.value = 'grok.states.version.notInstalled'
      await nextTick()

      expect(mounted.state.nextActions.value[0]).toMatchObject({
        key: 'install',
        external: true,
        tone: 'danger',
      })
      expect(mounted.state.nextActions.value).toHaveLength(2)
    } finally {
      mounted.unmount()
    }
  })

  it('keeps stale overview data and separates forced refresh errors', async () => {
    const mounted = await mountComposable()

    try {
      await mounted.state.refresh(false)
      getGrokDashboardOverviewMock.mockRejectedValueOnce(new Error('backend down'))

      await mounted.state.refresh(true)

      expect(mounted.state.overview.value?.current_profile).toBe('work')
      expect(mounted.state.loadError.value).toBeNull()
      expect(mounted.state.refreshError.value).toBe('backend down')
    } finally {
      mounted.unmount()
    }
  })

  it('keeps a cached version when version refresh fails', async () => {
    const mounted = await mountComposable()

    try {
      await mounted.state.refresh(false)

      getCliVersionMock.mockRejectedValueOnce(new Error('version unavailable'))
      await mounted.state.refresh(true)

      expect(mounted.state.versionStatus.value).toBe('ok')
      expect(mounted.state.versionLabel.value).toBe('v1.2.3')
      expect(mounted.state.refreshError.value).toBe('version unavailable')
    } finally {
      mounted.unmount()
    }
  })

  it('surfaces an initial overview failure as a load error', async () => {
    getGrokDashboardOverviewMock.mockRejectedValueOnce(new Error('initial read failed'))
    const mounted = await mountComposable()

    try {
      await mounted.state.refresh(false)

      expect(mounted.state.overview.value).toBeNull()
      expect(mounted.state.loadError.value).toBe('initial read failed')
      expect(mounted.state.refreshError.value).toBeNull()
      expect(getCliVersionMock).not.toHaveBeenCalled()
    } finally {
      mounted.unmount()
    }
  })

  it('fails closed when the active environment cannot be established', async () => {
    const first = await mountComposable()
    try {
      await first.state.refresh(false)
    } finally {
      first.unmount()
    }

    getCurrentEnvironmentMock.mockRejectedValueOnce(new Error('environment unavailable'))
    const second = await mountComposable()
    try {
      expect(second.state.overview.value?.current_profile).toBe('work')

      await second.state.refresh(false)

      expect(second.state.overview.value).toBeNull()
      expect(second.state.loadError.value).toBe('environment unavailable')
      expect(second.state.refreshError.value).toBeNull()
      expect(getGrokDashboardOverviewMock).toHaveBeenCalledTimes(1)
    } finally {
      second.unmount()
    }
  })
})
