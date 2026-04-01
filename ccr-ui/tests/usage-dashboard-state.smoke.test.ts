import { createApp, defineComponent, h, nextTick, reactive } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const usageStore = reactive({
  summary: null,
  trends: [],
  modelStats: [],
  logs: null as null | { records: unknown[] },
  logsModelFilter: undefined as string | undefined,
  lastImportResults: [] as Array<{ platform: string; error?: string }>,
  warning: '',
  currentImportJob: null as null | { warnings: string[]; status: string; files_total: number; files_scanned: number; records_imported: number },
  hasNoUsageData: false,
  isBootstrapping: false,
  importing: false,
  lastImportSummary: null as null | { processed_files: number; imported_records: number },
  initializeDashboard: vi.fn(async () => undefined),
  startAutoRefresh: vi.fn(),
  stopAutoRefresh: vi.fn(),
  setFilters: vi.fn(),
  startImportJob: vi.fn(async () => undefined),
  triggerImport: vi.fn(async () => undefined),
  fetchLogs: vi.fn(),
})

let tauriRuntime = false

vi.mock('@/stores/usage', () => ({
  useUsageStore: () => usageStore,
}))

vi.mock('@/utils/tauriRuntime', () => ({
  isTauriRuntime: () => tauriRuntime,
}))

vi.mock('vue-i18n', async () => {
  const { ref } = await import('vue')
  return {
    useI18n: () => ({
      t: (key: string) => key,
      locale: ref('en-US'),
    }),
  }
})

vi.mock('@tanstack/vue-virtual', () => ({
  useVirtualizer: vi.fn(() => ({
    getVirtualItems: () => [],
    getTotalSize: () => 0,
  })),
}))

const mountComposable = async () => {
  const { useUsageDashboardState } = await import('@/views/usage/useUsageDashboardState')
  let state: ReturnType<typeof useUsageDashboardState> | null = null
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(defineComponent({
    setup() {
      state = useUsageDashboardState()
      return () => h('div')
    },
  }))

  app.mount(el)
  await Promise.resolve()
  await nextTick()
  await nextTick()

  return {
    el,
    state: state!,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

beforeEach(() => {
  tauriRuntime = false
  usageStore.summary = null
  usageStore.trends = []
  usageStore.modelStats = []
  usageStore.logs = null
  usageStore.logsModelFilter = undefined
  usageStore.lastImportResults = []
  usageStore.warning = ''
  usageStore.currentImportJob = null
  usageStore.hasNoUsageData = false
  usageStore.isBootstrapping = false
  usageStore.importing = false
  usageStore.lastImportSummary = null
  usageStore.initializeDashboard.mockClear()
  usageStore.startAutoRefresh.mockClear()
  usageStore.stopAutoRefresh.mockClear()
  usageStore.setFilters.mockClear()
  usageStore.startImportJob.mockClear()
  usageStore.triggerImport.mockClear()
  usageStore.fetchLogs.mockClear()
})

afterEach(() => {
  document.body.innerHTML = ''
  vi.restoreAllMocks()
})

describe('usage dashboard state smoke', () => {
  it('starts store auto refresh without registering its own visibility listener', async () => {
    tauriRuntime = true
    const addEventListenerSpy = vi.spyOn(document, 'addEventListener')
    const { unmount } = await mountComposable()

    try {
      expect(usageStore.initializeDashboard).toHaveBeenCalledTimes(1)
      expect(usageStore.startAutoRefresh).toHaveBeenCalledTimes(1)
      expect(addEventListenerSpy).not.toHaveBeenCalledWith(
        'visibilitychange',
        expect.any(Function),
      )
    } finally {
      unmount()
    }

    expect(usageStore.stopAutoRefresh).toHaveBeenCalledTimes(1)
  })

  it('loads logs when the logs tab becomes active', async () => {
    tauriRuntime = true
    const { state, unmount } = await mountComposable()

    try {
      state.activeTab.value = 'logs'
      await nextTick()

      expect(usageStore.fetchLogs).toHaveBeenCalledWith('reset')
    } finally {
      unmount()
    }
  })

  it('skips desktop initialization in web runtime mode', async () => {
    tauriRuntime = false
    const { state, unmount } = await mountComposable()

    try {
      expect(state.runtimeUnavailable.value).toBe(true)
      expect(usageStore.initializeDashboard).not.toHaveBeenCalled()
      expect(usageStore.startAutoRefresh).not.toHaveBeenCalled()
    } finally {
      unmount()
    }

    expect(usageStore.stopAutoRefresh).toHaveBeenCalled()
  })

  it('starts a background import job with the current recent-days window', async () => {
    tauriRuntime = true
    const { state, unmount } = await mountComposable()

    try {
      state.selectedDays.value = 90
      await state.doImport()

      expect(usageStore.startImportJob).toHaveBeenCalledWith({
        platform: undefined,
        reason: 'manual',
        recentDays: 90,
      })
    } finally {
      unmount()
    }
  })
})
