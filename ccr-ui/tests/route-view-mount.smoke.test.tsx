import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render } from '@testing-library/react'
import { beforeAll, beforeEach, describe, expect, it, vi } from 'vitest'
import { I18nextProvider } from 'react-i18next'
import { createMemoryRouter, RouterProvider } from 'react-router'

const apiStubs = vi.hoisted(() => {
  const resolved = (value: unknown) => vi.fn().mockResolvedValue(value)

  const stubForName = (name: string) => {
    if (name === 'isTauriEnvironment') return () => false
    if (name === 'getEnvironmentName') return () => 'web'
    if (name === 'getTauriVersion') return resolved(null)
    if (name === 'getHomeUsageOverviewV2' || name === 'getCodexTraySnapshot') return resolved(null)
    if (name === 'getCodexDashboardOverview') {
      return resolved({
        auth: { logged_in: false, saved_accounts_total: 0, current: null },
        profiles: {
          total: 0,
          enabled_total: 0,
          disabled_total: 0,
          current_profile: null,
          current: null,
        },
        config: {},
        inventory: {
          mcp_servers_total: 0,
          agents_total: 0,
          sessions_total: 0,
          config_profiles_total: 0,
        },
      })
    }
    if (name === 'getCodexDashboardUsageSummary') {
      return resolved({
        freshness: 'empty',
        freshness_description: '',
        five_hour: { total_requests: 0, total_input_tokens: 0, total_output_tokens: 0 },
        seven_day: { total_requests: 0, total_input_tokens: 0, total_output_tokens: 0 },
        all_time: { total_requests: 0, total_input_tokens: 0, total_output_tokens: 0 },
      })
    }
    if (name === 'getCurrentEnvironment') {
      return resolved({ env_type: 'local', id: 'local' })
    }
    if (name === 'getCliVersion') {
      return resolved({ status: 'ok', installed: true, version: '1.0.0' })
    }
    if (name === 'getCliVersions') {
      return resolved({ entries: [] })
    }
    if (name === 'shellGetPreferences' || name === 'shellSetPreferences') {
      return resolved({
        confirm_before_exit: true,
        close_to_tray: false,
        open_panel_on_tray_click: true,
      })
    }
    if (/^is[A-Z]/.test(name) || /^has[A-Z]/.test(name)) return vi.fn(() => false)
    const lower = name.toLowerCase()
    if (
      lower.includes('trend')
      || lower.includes('heatmap')
      || lower.includes('logs')
      || lower.includes('assets')
      || lower.includes('events')
      || lower.includes('feed')
      || lower.includes('records')
      || lower.includes('breakdown')
      || lower.includes('session')
      || lower.includes('tools')
      || lower.includes('bymodel')
      || lower.includes('byproject')
      || lower.includes('byprovider')
    ) {
      return resolved([])
    }
    return resolved({})
  }

  const wrapValue = (name: string, value: unknown): unknown => {
    if (typeof value === 'function') return stubForName(name)
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      const nested: Record<string, unknown> = {}
      for (const [key, child] of Object.entries(value as Record<string, unknown>)) {
        nested[key] = wrapValue(key, child)
      }
      return nested
    }
    return value
  }

  const invokeResult = (command: string) => {
    if (command.includes('home_usage_overview') || command.includes('codex_tray_snapshot')) {
      return null
    }
    if (command.includes('codex_dashboard_overview')) {
      return {
        auth: { logged_in: false, saved_accounts_total: 0, current: null },
        profiles: { total: 0, enabled_total: 0, disabled_total: 0, current_profile: null, current: null },
        config: {},
        inventory: { mcp_servers_total: 0, agents_total: 0, sessions_total: 0, config_profiles_total: 0 },
      }
    }
    if (command.includes('codex_dashboard_usage_summary')) {
      return {
        freshness: 'empty',
        freshness_description: '',
        five_hour: { total_requests: 0, total_input_tokens: 0, total_output_tokens: 0 },
        seven_day: { total_requests: 0, total_input_tokens: 0, total_output_tokens: 0 },
        all_time: { total_requests: 0, total_input_tokens: 0, total_output_tokens: 0 },
      }
    }
    if (/list_|_list$|trends|by_model|by_project|by_provider|logs|events|feed|assets/.test(command)) {
      return []
    }
    return {}
  }

  return { wrapValue, invokeResult }
})

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (command: string) => apiStubs.invokeResult(command)),
  InvokeArgs: {},
  InvokeOptions: {},
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}))

vi.mock('@tauri-apps/api/app', () => ({
  getVersion: vi.fn().mockResolvedValue('1.0.0'),
}))

vi.mock('react-apexcharts', () => ({
  default: () => <div data-testid="mock-apex-chart" />,
}))

vi.mock('@uiw/react-codemirror', () => ({
  default: () => <textarea data-testid="mock-codemirror" />,
}))

vi.mock('@/api', async (importOriginal) => {
  const actual = await importOriginal<Record<string, unknown>>()
  const wrapped: Record<string, unknown> = {}
  for (const [key, value] of Object.entries(actual)) {
    wrapped[key] = apiStubs.wrapValue(key, value)
  }
  return wrapped
})

import i18n, { ensureLocaleLoaded, setLocale, type SupportedLocale } from '@/i18n'
import zhCN from '@/i18n/locales/zh-CN'
import enUS from '@/i18n/locales/en-US'
import { flattenCatalog } from '@/shell/routeCatalog'
import { appRoutes } from '@/shell/router'

/** 与 scripts/i18n-utils.mjs 相同的叶子 key 收集。 */
function* leafKeys(obj: unknown, prefix = ''): Generator<string> {
  if (!obj || typeof obj !== 'object' || Array.isArray(obj)) return
  for (const [key, value] of Object.entries(obj)) {
    const path = prefix ? `${prefix}.${key}` : key
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      yield* leafKeys(value, path)
    } else {
      yield path
    }
  }
}

const KEY_SHAPE_RE = /[A-Za-z][A-Za-z0-9_]*(?:\.[A-Za-z0-9_]+)+/g

/** 与 scripts/i18n-utils.mjs 相同：粗筛后用叶子集合判定。 */
function findLeakedKeys(text: string, keySet: Set<string>): string[] {
  const hits: string[] = []
  const seen = new Set<string>()
  KEY_SHAPE_RE.lastIndex = 0
  let match: RegExpExecArray | null
  while ((match = KEY_SHAPE_RE.exec(text))) {
    const token = match[0]
    if (keySet.has(token) && !seen.has(token)) {
      seen.add(token)
      hits.push(token)
    }
  }
  return hits
}

const zhKeySet = new Set(leafKeys(zhCN))
const enKeySet = new Set(leafKeys(enUS))

const catalogRoutes = flattenCatalog()

/** 把目录里的动态段换成可导航路径。 */
const materializePath = (path: string): string => {
  let next = path.replace('/:client?', '').replace(':client?', '')
  next = next.replace(/:accountId/g, 'acc-1')
  next = next.replace(/:platform/g, 'claude')
  next = next.replace(/:name/g, 'sample')
  next = next.replace(/\/{2,}/g, '/')
  if (next.length > 1) next = next.replace(/\/$/, '')
  return next || '/'
}

const isHydrateSpinner = (node: Element): boolean => {
  const parent = node.parentElement
  return Boolean(parent?.className.includes('min-h-[12.5rem]'))
}

const BOUNDARY_TEXT = ['页面渲染失败', '托盘面板渲染失败']

const routeFailureText = (container: HTMLElement): string | null => {
  const text = container.innerText || container.textContent || ''
  return BOUNDARY_TEXT.find((marker) => text.includes(marker)) ?? null
}

const routeMounted = (container: HTMLElement): boolean => {
  if (routeFailureText(container)) return false
  if (container.querySelector('[data-testid="route-placeholder"]')) return false
  if ([...container.querySelectorAll('.loading-spinner')].some(isHydrateSpinner)) return false
  return Boolean(
    container.querySelector('#main-content')
    || container.querySelector('.codex-tray-panel'),
  )
}

const assertNoBoundary = (container: HTMLElement, path: string) => {
  const failure = routeFailureText(container)
  if (failure) {
    const detail = container.querySelector('[role="alert"]')?.textContent ?? failure
    throw new Error(`${path}: ${detail}`)
  }
}

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms))

const mountCatalogPath = async (path: string) => {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  })
  const router = createMemoryRouter(appRoutes, { initialEntries: [path] })
  const view = render(
    <I18nextProvider i18n={i18n}>
      <QueryClientProvider client={client}>
        <RouterProvider router={router} />
      </QueryClientProvider>
    </I18nextProvider>,
  )
  const deadline = Date.now() + 4_000
  while (Date.now() < deadline) {
    assertNoBoundary(view.container, path)
    if (routeMounted(view.container)) break
    await sleep(40)
  }
  assertNoBoundary(view.container, path)
  if (!routeMounted(view.container)) {
    throw new Error(`${path}: 未挂载（无 #main-content / .codex-tray-panel）`)
  }
  await sleep(80)
  assertNoBoundary(view.container, path)
  return view
}

const scanLeaks = (text: string, locale: SupportedLocale, path: string) => {
  const keySet = locale === 'en-US' ? enKeySet : zhKeySet
  const hits = findLeakedKeys(text, keySet)
  return hits.map((key) => `${path} → ${key}`)
}

beforeAll(async () => {
  class ResizeObserverStub {
    observe(): void {}
    unobserve(): void {}
    disconnect(): void {}
  }
  if (typeof globalThis.ResizeObserver === 'undefined') {
    globalThis.ResizeObserver = ResizeObserverStub as unknown as typeof ResizeObserver
  }
  if (typeof window.scrollTo !== 'function') {
    window.scrollTo = () => {}
  }
  HTMLElement.prototype.scrollTo = () => {}
  await ensureLocaleLoaded('zh-CN')
  await ensureLocaleLoaded('en-US')
  await setLocale('zh-CN')
})

beforeEach(async () => {
  await setLocale('zh-CN')
})

describe('findLeakedKeys fixtures', () => {
  it('matches the four detector fixtures from detect-i18n-key-leak.mjs', () => {
    expect(findLeakedKeys('checkin.stats.total_accounts', zhKeySet)).toEqual([
      'checkin.stats.total_accounts',
    ])
    expect(findLeakedKeys('common.save', zhKeySet)).toEqual(['common.save'])
    expect(findLeakedKeys('保存', zhKeySet)).toEqual([])
    expect(findLeakedKeys('package.json example.com', zhKeySet)).toEqual([])
    expect(zhKeySet.has('checkin.stats.total_accounts')).toBe(true)
    expect(zhKeySet.has('common.save')).toBe(true)
    expect(zhKeySet.has('package.json')).toBe(false)
    expect(zhKeySet.has('example.com')).toBe(false)
  })
})

describe('catalog route view mount', () => {
  it('locks flattenCatalog at 75 records', () => {
    expect(catalogRoutes).toHaveLength(75)
  })

  it('mounts each catalog path in zh-CN without leaked i18n keys', async () => {
    const leaks: string[] = []
    const failures: string[] = []
    for (const route of catalogRoutes) {
      const path = materializePath(route.redirect ?? route.path)
      let view: ReturnType<typeof render> | null = null
      try {
        view = await mountCatalogPath(path)
        leaks.push(...scanLeaks(view.container.innerText, 'zh-CN', path))
      } catch (error) {
        failures.push(error instanceof Error ? error.message : String(error))
      } finally {
        view?.unmount()
      }
    }
    expect(failures, failures.join('\n')).toEqual([])
    expect(leaks, leaks.join('\n')).toEqual([])
  }, 180_000)

  it('mounts each catalog path in en-US without leaked i18n keys', async () => {
    await setLocale('en-US')
    const leaks: string[] = []
    const failures: string[] = []
    for (const route of catalogRoutes) {
      const path = materializePath(route.redirect ?? route.path)
      let view: ReturnType<typeof render> | null = null
      try {
        view = await mountCatalogPath(path)
        leaks.push(...scanLeaks(view.container.innerText, 'en-US', path))
      } catch (error) {
        failures.push(error instanceof Error ? error.message : String(error))
      } finally {
        view?.unmount()
      }
    }
    expect(failures, failures.join('\n')).toEqual([])
    expect(leaks, leaks.join('\n')).toEqual([])
  }, 180_000)
})
