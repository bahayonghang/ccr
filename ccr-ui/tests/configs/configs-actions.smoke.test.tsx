import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { createMemoryRouter, RouterProvider } from 'react-router'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ConfigsView } from '@/features/configs/ConfigsView'
import { useConfigsViewStore } from '@/features/configs/stores'
import { setLocale } from '@/i18n'
import zhCN from '@/i18n/locales/zh-CN'
import enUS from '@/i18n/locales/en-US'
import { configsKeys } from '@/features/configs/queries'
import type { ConfigInfo } from '@/types/generated/config/ConfigInfo'
import type { ProfileOutcome } from '@/types/generated/profiles/ProfileOutcome'

const bridge = vi.hoisted(() => ({
  invoke: vi.fn(),
  confirm: vi.fn(),
  success: vi.fn(),
  warning: vi.fn(),
  error: vi.fn(),
  events: [] as string[],
}))

vi.mock('@tauri-apps/api/core', () => ({ invoke: bridge.invoke }))
vi.mock('@/api', async () => ({
  ...await vi.importActual<typeof import('@/api/domains/config')>('@/api/domains/config'),
  getUsageByProviderV2: vi.fn().mockResolvedValue([]),
  getCurrentEnvironment: vi.fn().mockResolvedValue({ env_type: 'local' }),
}))
vi.mock('@/api/runtime/environment', () => ({
  isTauriEnvironment: () => false,
  TauriRuntimeApi: { getTauriVersion: vi.fn() },
}))
vi.mock('@/configs/surfaceNotify', () => ({
  surfaceNotify: {
    confirm: bridge.confirm, success: bridge.success,
    warning: bridge.warning, error: bridge.error,
  },
}))

const row = (name: string, enabled = true): ConfigInfo => ({
  version: 'source-version', name, description: name,
  base_url: 'https://fixture.invalid', auth_token: 'masked',
  is_current: false, is_default: false, usage_count: 0, enabled,
})

const outcome = (status: ProfileOutcome['status']): ProfileOutcome => ({
  operation_id: 'fixture-operation', platform: 'claude', profile: 'ready',
  previous_profile: null, status,
  activation_committed: status === 'applied' || status === 'applied_with_warning',
  warnings: status === 'applied_with_warning' ? ['history_failed'] : [],
  recovery_paths: status === 'recovery_required' ? ['synthetic-recovery-path'] : [],
})

function renderPage() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  const router = createMemoryRouter([{ path: '/configs', element: <ConfigsView /> }], {
    initialEntries: ['/configs'],
  })
  return { ...render(<QueryClientProvider client={client}><RouterProvider router={router} /></QueryClientProvider>), client }
}

describe('config page actions through the generated client', () => {
  let rows: ConfigInfo[]
  let result: ProfileOutcome

  beforeEach(async () => {
    await setLocale('en-US')
    vi.clearAllMocks()
    useConfigsViewStore.setState(useConfigsViewStore.getInitialState())
    rows = [row('ready'), row('disabled-profile', false)]
    result = outcome('applied')
    bridge.events.length = 0
    bridge.confirm.mockResolvedValue(true)
    bridge.success.mockImplementation(() => bridge.events.push('success'))
    bridge.warning.mockImplementation(() => bridge.events.push('warning'))
    bridge.error.mockImplementation(() => bridge.events.push('error'))
    bridge.invoke.mockImplementation(async (command: string, args: { name?: string }) => {
      bridge.events.push(command)
      if (command === 'list_configs') return rows
      if (command !== 'switch_config') throw new Error(`Unexpected fixture command: ${command}`)
      if (result.activation_committed) {
        rows = rows.map(config => ({
          ...config, is_current: config.name === args.name,
          enabled: config.name === args.name ? true : config.enabled,
        }))
      }
      return { platform: 'claude', name: args.name, outcome: { ...result, profile: args.name } }
    })
  })

  it('updates tabs, summaries and mounted cards without changing query data or remounting', async () => {
    rows = [{ ...row('ready'), description: '', is_current: true, is_default: true }, { ...row('disabled-profile', false), description: '' }]
    await setLocale('zh-CN')
    const { client } = renderPage()
    await screen.findByRole('button', { name: zhCN.configs.enable + ': disabled-profile' })
    const pageElement = document.querySelector('.configs-page')
    const card = document.querySelector('[data-config-name="ready"]')!
    const snapshot = client.getQueryData(configsKeys.list())
    const summary = document.querySelector('.configs-workspace > div > button p')!
    expect(summary.textContent).toBe(zhCN.configs.filters.all)
    expect(card.textContent).toContain(zhCN.configs.noDescription)
    expect(card.textContent).toContain(zhCN.configs.currentBadge)
    await act(async () => { await setLocale('en-US') })
    expect(screen.getByRole('button', { name: enUS.configs.tabs.configList })).toBeTruthy()
    expect(screen.getByRole('button', { name: enUS.configs.tabs.history })).toBeTruthy()
    expect(summary.textContent).toBe(enUS.configs.filters.all)
    expect(card.textContent).toContain(enUS.configs.noDescription)
    expect(card.textContent).toContain(enUS.configs.currentBadge)
    expect(card.textContent).toContain(enUS.configs.defaultBadge)
    expect(card.textContent).toContain(enUS.configs.inUse)
    expect(screen.getByRole('button', { name: enUS.configs.enable + ': disabled-profile' })).toBeTruthy()
    expect(document.querySelector('.configs-page')).toBe(pageElement)
    expect(document.querySelector('[data-config-name="ready"]')).toBe(card)
    expect(client.getQueryData(configsKeys.list())).toBe(snapshot)
    expect(bridge.invoke.mock.calls.filter(([command]) => command === 'list_configs')).toHaveLength(1)
    await act(async () => { await setLocale('zh-CN') })
    expect(summary.textContent).toBe(zhCN.configs.filters.all)
    expect(card.textContent).toContain(zhCN.configs.noDescription)
  })

  it('switches an enabled row with an explicit platform and refreshes committed state', async () => {
    renderPage()
    fireEvent.click(await screen.findByRole('button', { name: /^(切换|Switch): ready$/ }))
    await waitFor(() => expect(bridge.invoke).toHaveBeenCalledWith('switch_config', {
      platform: 'claude', name: 'ready', enable: false,
      confirmationToken: 'desktop-confirm:switch_config',
    }))
    await waitFor(() => expect(bridge.success).toHaveBeenCalledOnce())
    expect(bridge.events).toEqual(['list_configs', 'switch_config', 'list_configs', 'success'])
    expect(useConfigsViewStore.getState().currentConfig).toBe('ready')
  })

  it('exposes an enable action for a disabled row and sends enable true', async () => {
    renderPage()
    fireEvent.click(await screen.findByRole('button', { name: /^(启用|Enable): disabled-profile$/ }))
    await waitFor(() => expect(bridge.invoke).toHaveBeenCalledWith('switch_config', {
      platform: 'claude', name: 'disabled-profile', enable: true,
      confirmationToken: 'desktop-confirm:switch_config',
    }))
    await waitFor(() => expect(bridge.success).toHaveBeenCalledOnce())
    expect(bridge.invoke.mock.calls.filter(([command]) => command === 'switch_config')).toHaveLength(1)
    expect(useConfigsViewStore.getState().currentConfig).toBe('disabled-profile')
  })

  it('does not send a mutation when switch confirmation is cancelled', async () => {
    bridge.confirm.mockResolvedValue(false)
    renderPage()
    fireEvent.click(await screen.findByRole('button', { name: /^(切换|Switch): ready$/ }))
    await waitFor(() => expect(bridge.confirm).toHaveBeenCalledOnce())
    expect(bridge.invoke.mock.calls.filter(([command]) => command === 'switch_config')).toHaveLength(0)
    expect(bridge.success).not.toHaveBeenCalled()
  })

  it('does not enable a disabled row when confirmation is cancelled', async () => {
    bridge.confirm.mockResolvedValue(false)
    renderPage()
    fireEvent.click(await screen.findByRole('button', { name: /^Enable: disabled-profile$/ }))
    await waitFor(() => expect(bridge.confirm).toHaveBeenCalledOnce())
    expect(bridge.invoke.mock.calls.filter(([command]) => command === 'switch_config')).toHaveLength(0)
    expect(bridge.success).not.toHaveBeenCalled()
  })

  it('refreshes a committed warning and does not repeat activation or show success', async () => {
    result = outcome('applied_with_warning')
    renderPage()
    fireEvent.click(await screen.findByRole('button', { name: /^(切换|Switch): ready$/ }))
    await waitFor(() => expect(bridge.warning).toHaveBeenCalledOnce())
    expect(bridge.events).toEqual(['list_configs', 'switch_config', 'list_configs', 'warning'])
    expect(bridge.success).not.toHaveBeenCalled()
    expect(bridge.error).not.toHaveBeenCalled()
  })

  it.each(['recovery_required', 'unchanged'] as const)('refreshes %s without a false success or current marker', async status => {
    result = outcome(status)
    renderPage()
    fireEvent.click(await screen.findByRole('button', { name: /^(切换|Switch): ready$/ }))
    await waitFor(() => expect(bridge.error).toHaveBeenCalledOnce())
    expect(bridge.events).toEqual(['list_configs', 'switch_config', 'list_configs', 'error'])
    expect(bridge.success).not.toHaveBeenCalled()
    expect(bridge.warning).not.toHaveBeenCalled()
    expect(useConfigsViewStore.getState().currentConfig).toBeNull()
  })
})
