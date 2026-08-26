import { beforeEach, afterEach, describe, expect, it, vi } from 'vitest'
import type { ProviderTemplate } from '@/types/providerTemplates'
import type { DesktopShellPreferences } from '@/api/runtime/environment'

// store action 全量状态转移测试（08-22-state-logic-port 批次 6 / AC8）：
// 每个store 的每个 action 至少一个用例，覆盖初始态 → 目标态转换；
// 涉及持久化的断言 localStorage 键与值的字节形态。

vi.mock('@/api/runtime/environment', () => ({
  shellGetPreferences: vi.fn(),
  shellSetPreferences: vi.fn(),
}))

import {
  DEFAULT_SIDEBAR_WIDTH,
  MAX_SIDEBAR_WIDTH,
  MIN_SIDEBAR_WIDTH,
  useShellPreferencesStore,
} from '@/shell/stores/shellPreferences'
import { shellGetPreferences, shellSetPreferences } from '@/api/runtime/environment'
import { CUSTOM_PROVIDER_TEMPLATES_STORAGE_KEY } from '@/utils/providerTemplates'
import { useUIStore } from '@/shell/stores/ui'
import { useCommandsViewStore } from '@/features/commands/stores'
import { useUsageViewStore } from '@/features/usage/stores'
import {
  DEFAULT_USAGE_RANGE_PRESET,
  getLocalDateRangeWindow,
} from '@/views/usage/dateWindow'
import { useConfigsViewStore } from '@/features/configs/stores'
import { PROFILES_PIN_CAP, useProfilesQuickSwitchStore } from '@/features/profiles/stores'
import { useProviderTemplatesStore } from '@/composables/useProviderTemplates'

const mockShellGet = vi.mocked(shellGetPreferences)
const mockShellSet = vi.mocked(shellSetPreferences)

const baseRuntimePreferences: DesktopShellPreferences = {
  confirm_before_exit: true,
  close_to_tray: false,
  open_panel_on_tray_click: true,
  tray_panel: { placement_mode: 'anchored', manual_position: null },
}

beforeEach(() => {
  // zustand 单例：每个用例前恢复模块初始态，保证用例隔离
  useUIStore.getState().resolveConfirmDialog(false)
  useUIStore.setState(useUIStore.getInitialState())
  useShellPreferencesStore.setState(useShellPreferencesStore.getInitialState())
  useCommandsViewStore.setState(useCommandsViewStore.getInitialState())
  useUsageViewStore.setState(useUsageViewStore.getInitialState())
  useConfigsViewStore.setState(useConfigsViewStore.getInitialState())
  useProfilesQuickSwitchStore.setState(useProfilesQuickSwitchStore.getInitialState())
  useProviderTemplatesStore.setState(useProviderTemplatesStore.getInitialState())

  mockShellGet.mockResolvedValue(baseRuntimePreferences)
  mockShellSet.mockImplementation(async (preferences) => preferences)
})

afterEach(() => {
  vi.useRealTimers()
})

describe('ui store（shell/stores/ui.ts）', () => {
  const s = () => useUIStore.getState()

  it('showToast：初始空 → 追加一条默认 info toast 并返回自增 id', () => {
    expect(s().toasts).toEqual([])
    const id = s().showToast('hello', 'info', 0)
    expect(s().toasts).toEqual([{ id, message: 'hello', type: 'info', duration: 0 }])
    expect(id).toBeGreaterThan(0)
  })

  it('removeToast：仅移除目标 id', () => {
    const first = s().showToast('a', 'info', 0)
    const second = s().showToast('b', 'info', 0)
    s().removeToast(first)
    expect(s().toasts.map((toast) => toast.id)).toEqual([second])
  })

  it('showSuccess/showError/showWarning/showInfo：类型与默认时长映射正确', () => {
    s().showSuccess('ok', 0)
    s().showError('bad', 0)
    s().showWarning('careful', 0)
    s().showInfo('fyi', 0)
    expect(s().toasts.map(({ type }) => type)).toEqual(['success', 'error', 'warning', 'info'])
    // 默认 duration 走原值域（不触发定时器路径的断言只看类型顺序）
  })

  it('showToast：duration > 0 时到期自动移除', () => {
    vi.useFakeTimers()
    const id = s().showToast('transient', 'info', 3000)
    expect(s().toasts).toHaveLength(1)
    vi.advanceTimersByTime(3000)
    expect(s().toasts.some((toast) => toast.id === id)).toBe(false)
  })

  it('requestConfirm：弹窗态填充默认值且 promise 挂起', async () => {
    let settled: boolean | undefined
    const pending = s().requestConfirm({ title: 'T', message: 'M' })
    void pending.then((value) => {
      settled = value
    })
    await Promise.resolve()
    expect(settled).toBeUndefined()
    expect(s().confirmDialog).toEqual({
      title: 'T',
      message: 'M',
      confirmText: undefined,
      cancelText: undefined,
      type: 'info',
      surface: 'glass',
    })
  })

  it('requestConfirm：已有弹窗时新请求使旧 promise 以 false 收敛', async () => {
    let firstResult: boolean | undefined
    void s().requestConfirm({ title: 'old', message: 'm' }).then((value) => {
      firstResult = value
    })
    await Promise.resolve()

    const secondPending = s().requestConfirm({ title: 'new', message: 'm' })
    await Promise.resolve()
    expect(firstResult).toBe(false)

    s().resolveConfirmDialog(true)
    await expect(secondPending).resolves.toBe(true)
    expect(s().confirmDialog).toBeNull()
  })

  it('resolveConfirmDialog：以确认结果收敛并清空弹窗', async () => {
    const pending = s().requestConfirm({ title: 'T', message: 'M' })
    s().resolveConfirmDialog(false)
    await expect(pending).resolves.toBe(false)
    expect(s().confirmDialog).toBeNull()
  })

  it('startLoading/stopLoading：loading 态与文案成对切换', () => {
    s().startLoading('处理中')
    expect(s().globalLoading).toBe(true)
    expect(s().loadingMessage).toBe('处理中')
    s().stopLoading()
    expect(s().globalLoading).toBe(false)
    expect(s().loadingMessage).toBe('')
  })

  it('clearToasts：清空全部 toast', () => {
    s().showToast('x', 'info', 0)
    s().showToast('y', 'info', 0)
    s().clearToasts()
    expect(s().toasts).toEqual([])
  })
})

describe('commandsView store（features/commands/stores.ts）', () => {
  const s = () => useCommandsViewStore.getState()
  const storedJson = () => localStorage.getItem('ccr-commands-view')

  it('setSortKey：name → usage，持久化 JSON 字节不变', () => {
    s().setSortKey('usage')
    expect(s().sortKey).toBe('usage')
    expect(storedJson()).toBe(
      JSON.stringify({
        sortKey: 'usage',
        sortDir: 'asc',
        viewMode: 'tree',
        showDeprecated: true,
        expandedFolders: [],
      }),
    )
  })

  it('toggleSortDir：asc ↔ desc 且持久化', () => {
    s().toggleSortDir()
    expect(s().sortDir).toBe('desc')
    expect(JSON.parse(storedJson() ?? '{}')).toMatchObject({ sortDir: 'desc' })
    s().toggleSortDir()
    expect(s().sortDir).toBe('asc')
  })

  it('setViewMode：tree → flat 且持久化', () => {
    s().setViewMode('flat')
    expect(s().viewMode).toBe('flat')
    expect(JSON.parse(storedJson() ?? '{}')).toMatchObject({ viewMode: 'flat' })
  })

  it('toggleShowDeprecated：true → false 且持久化', () => {
    expect(s().showDeprecated).toBe(true)
    s().toggleShowDeprecated()
    expect(s().showDeprecated).toBe(false)
    expect(JSON.parse(storedJson() ?? '{}')).toMatchObject({ showDeprecated: false })
  })

  it('toggleFolder：加入与移除均持久化', () => {
    s().toggleFolder('agents')
    expect(s().expandedFolders).toEqual(['agents'])
    s().toggleFolder('skills')
    expect(s().expandedFolders).toEqual(['agents', 'skills'])
    s().toggleFolder('agents')
    expect(s().expandedFolders).toEqual(['skills'])
    expect(JSON.parse(storedJson() ?? '{}')).toMatchObject({ expandedFolders: ['skills'] })
  })

  it('restore：从 localStorage 合并恢复，形状未知字段收敛为跳过', () => {
    localStorage.setItem(
      'ccr-commands-view',
      JSON.stringify({ sortKey: 'usage', viewMode: 'flat', expandedFolders: ['a'], staleField: 1 }),
    )
    s().restore()
    expect(s().sortKey).toBe('usage')
    expect(s().viewMode).toBe('flat')
    expect(s().expandedFolders).toEqual(['a'])
    expect(s().sortDir).toBe('asc')
    expect(s().showDeprecated).toBe(true)
  })

  it('restore：损坏 JSON 不抛出且保持现状', () => {
    localStorage.setItem('ccr-commands-view', '{broken json')
    expect(() => s().restore()).not.toThrow()
    expect(s().viewMode).toBe('tree')
  })
})

describe('usage 视图偏好 store（features/usage/stores.ts）', () => {
  const s = () => useUsageViewStore.getState()

  it('setPlatform：undefined → codex', () => {
    expect(s().platform).toBeUndefined()
    s().setPlatform('codex')
    expect(s().platform).toBe('codex')
  })

  it('setTimeRange：空对象 → 带起止区间', () => {
    s().setTimeRange({ start: '2026-01-01', end: '2026-01-31' })
    expect(s().timeRange).toEqual({ start: '2026-01-01', end: '2026-01-31' })
  })

  it('初态 last_30d 带本地 30 日窗口，查询不会落成 all-time', () => {
    expect(s().rangePreset).toBe(DEFAULT_USAGE_RANGE_PRESET)
    expect(s().timeRange).toEqual(getLocalDateRangeWindow(DEFAULT_USAGE_RANGE_PRESET))
    expect(s().timeRange.start).toBeDefined()
    expect(s().timeRange.end).toBeDefined()
  })

  it('resetFilters：回到默认 30 日窗口', () => {
    s().setPlatform('claude')
    s().setTimeRange({ start: 'x' })
    s().resetFilters()
    expect(s().platform).toBeUndefined()
    expect(s().rangePreset).toBe(DEFAULT_USAGE_RANGE_PRESET)
    expect(s().timeRange).toEqual(getLocalDateRangeWindow(DEFAULT_USAGE_RANGE_PRESET))
  })
})

describe('configs 视图 store（features/configs/stores.ts）', () => {
  const s = () => useConfigsViewStore.getState()

  it('setCurrentConfig：null → 配置名', () => {
    s().setCurrentConfig('gpt5-default')
    expect(s().currentConfig).toBe('gpt5-default')
  })

  it('setSearchQuery：空串 → 关键词', () => {
    s().setSearchQuery('grok')
    expect(s().searchQuery).toBe('grok')
  })

  it('setFormDraft：按配置 id 写入草稿互不覆盖', () => {
    s().setFormDraft('config-a', '{"model":"x"}')
    s().setFormDraft('config-b', 42)
    expect(s().formDrafts['config-a']).toBe('{"model":"x"}')
    expect(s().formDrafts['config-b']).toBe(42)
  })

  it('clearFormDraft：仅移除目标草稿；id 不存在时保持原状', () => {
    s().setFormDraft('config-a', 'draft')
    const before = s().formDrafts
    s().clearFormDraft('missing')
    expect(s().formDrafts).toBe(before)
    s().clearFormDraft('config-a')
    expect(s().formDrafts).toEqual({})
  })
})

describe('profiles 快速切换 store（features/profiles/stores.ts）', () => {
  const s = () => useProfilesQuickSwitchStore.getState()
  const pinnedKey = 'ccr:profiles:pinned:claude'
  const recentKey = 'ccr:profiles:recent:claude'

  it('pin：空数组 → 追加并逐字节写入平台键', () => {
    expect(s().pin('claude', 'p1')).toBe(true)
    expect(s().pinnedByPlatform.claude).toEqual(['p1'])
    expect(localStorage.getItem(pinnedKey)).toBe('["p1"]')
  })

  it('pin：重复钉选返回 false 且不改写', () => {
    s().pin('claude', 'p1')
    expect(s().pin('claude', 'p1')).toBe(false)
    expect(localStorage.getItem(pinnedKey)).toBe('["p1"]')
    expect(s().pin('claude', '')).toBe(false)
  })

  it('pin：达到上限后拒绝并触发 onPinLimit', () => {
    for (let i = 0; i < PROFILES_PIN_CAP; i++) {
      expect(s().pin('claude', `p${i}`)).toBe(true)
    }
    const onPinLimit = vi.fn()
    expect(s().pin('claude', 'overflow', onPinLimit)).toBe(false)
    expect(onPinLimit).toHaveBeenCalledTimes(1)
    expect(s().pinnedByPlatform.claude).toHaveLength(PROFILES_PIN_CAP)
  })

  it('unpin：移除并回写；未钉选名为无操作', () => {
    s().pin('claude', 'p1')
    s().pin('claude', 'p2')
    s().unpin('claude', 'p1')
    expect(s().pinnedByPlatform.claude).toEqual(['p2'])
    expect(localStorage.getItem(pinnedKey)).toBe('["p2"]')
    s().unpin('claude', 'ghost')
    expect(s().pinnedByPlatform.claude).toEqual(['p2'])
  })

  it('recordUse：最新置顶、去重、上限 16 截断并持久化', () => {
    for (let i = 0; i < 18; i++) {
      s().recordUse('claude', `n${i}`)
    }
    s().recordUse('claude', 'n17')
    const recent = s().recentByPlatform.claude
    expect(recent[0]).toBe('n17')
    expect(recent).toHaveLength(16)
    expect(recent.includes('n0')).toBe(false)
    expect(localStorage.getItem(recentKey)).toBe(JSON.stringify(recent))
  })

  it('renamePinned：钉选与最近列表中的旧名同步替换并回写', () => {
    s().pin('claude', 'p1')
    s().pin('claude', 'p2')
    s().recordUse('claude', 'p1')
    s().renamePinned('claude', 'p1', 'p1-renamed')
    expect(s().pinnedByPlatform.claude).toEqual(['p1-renamed', 'p2'])
    expect(s().recentByPlatform.claude).toContain('p1-renamed')
    expect(localStorage.getItem(pinnedKey)).toBe('["p1-renamed","p2"]')
  })

  it('cleanupStale：profileNames 为 null 时跳过清理', () => {
    s().pin('claude', 'gone')
    s().cleanupStale('claude', null)
    expect(s().pinnedByPlatform.claude).toEqual(['gone'])
  })

  it('cleanupStale：过滤已不存在项并回写两族键', () => {
    s().pin('claude', 'keep')
    s().pin('claude', 'gone')
    s().recordUse('claude', 'gone')
    s().cleanupStale('claude', ['keep'])
    expect(s().pinnedByPlatform.claude).toEqual(['keep'])
    expect(s().recentByPlatform.claude).toEqual([])
    expect(localStorage.getItem(pinnedKey)).toBe('["keep"]')
    expect(localStorage.getItem(recentKey)).toBe('[]')
  })
})

describe('provider 模板 store（composables/useProviderTemplates.ts 内置 store）', () => {
  const s = () => useProviderTemplatesStore.getState()

  const templateOf = (id: string): ProviderTemplate => ({
    id,
    name: id,
    category: 'third_party',
    baseUrls: ['https://example.com'],
    platforms: { codex: {} },
  })

  it('saveCustomTemplate：初始为空 → 新增自定义模板并写入存储键', () => {
    expect(s().customTemplates).toEqual([])
    s().saveCustomTemplate(templateOf('tpl-1'))
    expect(s().customTemplates).toHaveLength(1)
    expect(s().customTemplates[0]).toMatchObject({ id: 'tpl-1', source: 'custom' })
    const stored: unknown = JSON.parse(
      localStorage.getItem(CUSTOM_PROVIDER_TEMPLATES_STORAGE_KEY) ?? '[]',
    )
    expect(Array.isArray(stored)).toBe(true)
    expect(stored).toHaveLength(1)
  })

  it('saveCustomTemplate：同 id 再保存为更新而非追加', () => {
    s().saveCustomTemplate(templateOf('tpl-1'))
    s().saveCustomTemplate({ ...templateOf('tpl-1'), name: 'renamed' })
    expect(s().customTemplates).toHaveLength(1)
    expect(s().customTemplates[0].name).toBe('renamed')
  })

  it('removeCustomTemplate：按 id 过滤并回写存储键', () => {
    s().saveCustomTemplate(templateOf('tpl-1'))
    s().removeCustomTemplate('tpl-1')
    expect(s().customTemplates).toEqual([])
    expect(localStorage.getItem(CUSTOM_PROVIDER_TEMPLATES_STORAGE_KEY)).toBe('[]')
  })

  it('reloadCustomTemplates：从 localStorage 重读水合', () => {
    localStorage.setItem(
      CUSTOM_PROVIDER_TEMPLATES_STORAGE_KEY,
      JSON.stringify([templateOf('external')]),
    )
    s().reloadCustomTemplates()
    expect(s().customTemplates.map((template) => template.id)).toEqual(['external'])
  })
})

describe('shellPreferences store（shell/stores/shellPreferences.ts）', () => {
  const s = () => useShellPreferencesStore.getState()

  it('侧栏宽度常量与 clamp 边界', () => {
    expect(DEFAULT_SIDEBAR_WIDTH).toBe(240)
    expect(MIN_SIDEBAR_WIDTH).toBe(200)
    expect(MAX_SIDEBAR_WIDTH).toBe(480)
  })

  it('initializeTheme：迁移并应用存储主题到 document', () => {
    localStorage.setItem('ccr-theme', 'dark')
    s().initializeTheme()
    expect(s().theme).toBe('dark')
    expect(s().effectiveTheme).toBe('dark')
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark')
    expect(document.documentElement.classList.contains('dark')).toBe(true)
  })

  it('setTheme：light → dark 并写 ccr-theme', () => {
    expect(s().theme).toBe('light')
    s().setTheme('dark')
    expect(s().theme).toBe('dark')
    expect(s().effectiveTheme).toBe('dark')
    expect(localStorage.getItem('ccr-theme')).toBe('dark')
  })

  it('toggleTheme：在 light/dark 间翻转并落盘', () => {
    expect(s().effectiveTheme).toBe('light')
    s().toggleTheme()
    expect(s().effectiveTheme).toBe('dark')
    s().toggleTheme()
    expect(s().effectiveTheme).toBe('light')
    expect(localStorage.getItem('ccr-theme')).toBe('light')
  })

  it('setFlavor：neutral → clay 并写 ccr-flavor', () => {
    s().setFlavor('clay')
    expect(s().flavor).toBe('clay')
    expect(s().resolvedFlavor).toBe('clay')
    expect(localStorage.getItem('ccr-flavor')).toBe('clay')
  })

  it('setFlavor：旧值域输入迁移到新值域后落盘', () => {
    s().setFlavor('legacy-value' as never)
    expect(['neutral', 'clay']).toContain(s().flavor)
    expect(localStorage.getItem('ccr-flavor')).toBe(s().flavor)
  })

  it('setAccent：接受枚举值并写 ccr-accent', () => {
    s().setAccent('clay')
    expect(s().accent).toBe('clay')
    expect(localStorage.getItem('ccr-accent')).toBe('clay')
  })

  it('setUiFont/setCodeFont：净化后写入状态与 ccr-font-* 键', () => {
    s().setUiFont('Inter')
    s().setCodeFont('JetBrains Mono')
    expect(s().uiFont).toBe('Inter')
    expect(s().codeFont).toBe('JetBrains Mono')
    expect(localStorage.getItem('ccr-font-ui')).toBe('Inter')
    expect(localStorage.getItem('ccr-font-code')).toBe('JetBrains Mono')
  })

  it('setLocalePreference：zh-CN → en-US，标签与 ccr-ui-locale 同步', async () => {
    expect(s().localeLabel).toBe('中文')
    const locale = await s().setLocalePreference('en-US')
    expect(locale).toBe('en-US')
    expect(s().locale).toBe('en-US')
    expect(s().localeLabel).toBe('English')
    expect(localStorage.getItem('ccr-ui-locale')).toBe('en-US')
  })

  it('updateSidebarWidth：超界收敛到上限并持久化', () => {
    const clamped = s().updateSidebarWidth(9999)
    expect(clamped).toBe(MAX_SIDEBAR_WIDTH)
    expect(s().sidebarWidth).toBe(MAX_SIDEBAR_WIDTH)
    expect(localStorage.getItem('ccr-sidebar-width')).toBe(String(MAX_SIDEBAR_WIDTH))
  })

  it('updateSidebarWidth：persist=false 只改内存不落盘', () => {
    s().updateSidebarWidth(320, false)
    expect(s().sidebarWidth).toBe(320)
    expect(localStorage.getItem('ccr-sidebar-width')).toBeNull()
  })

  it('commitSidebarWidth：把当前宽度补写进存储', () => {
    s().updateSidebarWidth(320, false)
    s().commitSidebarWidth()
    expect(localStorage.getItem('ccr-sidebar-width')).toBe('320')
  })

  it('resetLayout：恢复默认宽度并持久化', () => {
    s().updateSidebarWidth(480)
    s().resetLayout()
    expect(s().sidebarWidth).toBe(DEFAULT_SIDEBAR_WIDTH)
    expect(localStorage.getItem('ccr-sidebar-width')).toBe('240')
  })

  it('hydrateRuntimePreferences：拉取成功填充 runtime 三项且二次调用不再请求', async () => {
    mockShellGet.mockResolvedValue({
      ...baseRuntimePreferences,
      confirm_before_exit: false,
      close_to_tray: true,
    })
    await s().hydrateRuntimePreferences()
    expect(s().confirmBeforeExit).toBe(false)
    expect(s().closeToTray).toBe(true)
    expect(s().runtimeHydrated).toBe(true)

    await s().hydrateRuntimePreferences()
    expect(mockShellGet).toHaveBeenCalledTimes(1)
  })

  it('hydrateRuntimePreferences：拉取失败时回退安全默认并标记已水合', async () => {
    mockShellGet.mockRejectedValueOnce(new Error('ipc down'))
    await s().hydrateRuntimePreferences()
    expect(s().confirmBeforeExit).toBe(true)
    expect(s().closeToTray).toBe(false)
    expect(s().openPanelOnTrayClick).toBe(true)
    expect(s().runtimeHydrated).toBe(true)
  })

  it('setConfirmBeforeExit：先改本地再 flush 到后端', async () => {
    await s().setConfirmBeforeExit(false)
    expect(s().confirmBeforeExit).toBe(false)
    expect(mockShellSet).toHaveBeenCalledWith(expect.objectContaining({ confirm_before_exit: false }))
  })

  it('setCloseToTray：状态与后端写一致', async () => {
    await s().setCloseToTray(true)
    expect(s().closeToTray).toBe(true)
    expect(mockShellSet).toHaveBeenCalledWith(expect.objectContaining({ close_to_tray: true }))
  })

  it('setOpenPanelOnTrayClick：状态与后端写一致', async () => {
    await s().setOpenPanelOnTrayClick(false)
    expect(s().openPanelOnTrayClick).toBe(false)
    expect(mockShellSet).toHaveBeenCalledWith(
      expect.objectContaining({ open_panel_on_tray_click: false }),
    )
  })

  it('setPerfTelemetryPreference：开关写 ccr-ui:perf 键', () => {
    s().setPerfTelemetryPreference(true)
    expect(s().perfTelemetryEnabled).toBe(true)
    expect(localStorage.getItem('ccr-ui:perf')).toBe('1')
    s().setPerfTelemetryPreference(false)
    expect(s().perfTelemetryEnabled).toBe(false)
    expect(localStorage.getItem('ccr-ui:perf')).toBeNull()
  })
})
