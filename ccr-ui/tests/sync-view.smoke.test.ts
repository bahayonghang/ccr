
import { createApp, defineComponent, h, nextTick } from 'vue'
import { createI18n } from 'vue-i18n'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import enUS from '@/i18n/locales/en-US'

const apiMocks = vi.hoisted(() => ({
  getSyncStatus: vi.fn(),
  listSyncFolders: vi.fn(),
  addSyncFolder: vi.fn(),
  updateSyncFolder: vi.fn(),
  deleteSyncFolder: vi.fn(),
  pushSync: vi.fn(),
  pullSync: vi.fn(),
  pushSyncFolder: vi.fn(),
  pullSyncFolder: vi.fn(),
}))

vi.mock('@/api', () => apiMocks)

vi.mock('vue-router', () => ({
  RouterLink: defineComponent({
    props: { to: { type: [String, Object], required: true } },
    setup(_props, { slots }) {
      return () => h('a', {}, slots.default?.())
    },
  }),
}))

vi.mock('@/components/ui/SIcon.vue', () => ({
  default: defineComponent({
    props: {
      name: { type: String, required: true },
      size: { type: String, default: '' },
      class: { type: String, default: '' },
    },
    setup(props) {
      return () => h('span', { 'data-icon': props.name, class: [props.size, props.class] })
    },
  }),
}))

vi.mock('@/components/PageHeaderCard.vue', () => ({
  default: defineComponent({
    setup(_props, { slots }) {
      return () => h('section', { 'data-stub': 'PageHeaderCard' }, slots.default?.())
    },
  }),
}))

vi.mock('@/components/ui/AsyncStatePanel.vue', () => ({
  default: defineComponent({
    props: { state: { type: String, required: true }, title: { type: String, default: '' } },
    setup(props) {
      return () => h('div', { 'data-state': props.state }, props.title)
    },
  }),
}))

vi.mock('@/components/sync/SyncInfoSidebar.vue', () => ({
  default: defineComponent({
    props: { syncStatus: { type: Object, default: null } },
    setup(props) {
      return () => h('aside', { 'data-stub': 'SyncInfoSidebar' }, JSON.stringify(props.syncStatus))
    },
  }),
}))

vi.mock('@/components/sync/SyncOperationOutputPanel.vue', () => ({
  default: defineComponent({
    props: { output: { type: String, default: '' } },
    setup(props) {
      return () => h('pre', { 'data-output': '' }, props.output)
    },
  }),
}))

const lastSelectionProps = vi.hoisted(() => ({ value: null as null | Record<string, unknown> }))
const lastBatchProps = vi.hoisted(() => ({ value: null as null | Record<string, unknown> }))
const lastFoldersProps = vi.hoisted(() => ({ value: null as null | Record<string, unknown> }))

vi.mock('@/components/sync/SyncSelectionPanel.vue', () => ({
  default: defineComponent({
    props: [
      'addCustomFolder', 'addingCustom', 'applying', 'applySelection', 'customFolder', 'hasChanges',
      'optionalItems', 'presetConfig', 'toggleItem', 'updateCustomField', 'updateOptionalLocalPath',
      'updateOptionalRemotePath', 'updatePresetLocalPath',
    ],
    setup(props) {
      return () => {
        lastSelectionProps.value = props as unknown as Record<string, unknown>
        return h('section', { 'data-stub': 'SyncSelectionPanel' }, String(props.hasChanges))
      }
    },
  }),
}))

vi.mock('@/components/sync/SyncEnabledFoldersPanel.vue', () => ({
  default: defineComponent({
    props: [
      'folders', 'getFolderStatus', 'pullFolder', 'pushFolder', 'refreshFolders', 'refreshingFolders',
      'removeFolder', 'toggleFolder',
    ],
    setup(props) {
      return () => {
        lastFoldersProps.value = props as unknown as Record<string, unknown>
        return h('section', { 'data-stub': 'SyncEnabledFoldersPanel' }, JSON.stringify(props.folders))
      }
    },
  }),
}))

vi.mock('@/components/sync/SyncBatchOperationsPanel.vue', () => ({
  default: defineComponent({
    props: ['batchOperating', 'foldersCount', 'getAllFoldersStatus', 'pullAllFolders', 'pushAllFolders'],
    setup(props) {
      return () => {
        lastBatchProps.value = props as unknown as Record<string, unknown>
        return h('section', { 'data-stub': 'SyncBatchOperationsPanel' }, String(props.foldersCount))
      }
    },
  }),
}))

const i18n = createI18n({
  legacy: false,
  locale: 'en-US',
  fallbackLocale: 'en-US',
  missingWarn: false,
  fallbackWarn: false,
  messages: { 'en-US': enUS },
})

const flushAsync = async () => {
  await Promise.resolve()
  await new Promise(resolve => setTimeout(resolve, 0))
}

const mountView = async () => {
  const { default: SyncView } = await import('@/views/SyncView.vue')
  const el = document.createElement('div')
  document.body.appendChild(el)
  const app = createApp(SyncView)
  app.use(i18n)
  app.mount(el)
  await nextTick()
  await flushAsync()
  await nextTick()

  return {
    el,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

beforeEach(() => {
  apiMocks.getSyncStatus.mockResolvedValue({ configured: true, enabled: true })
  apiMocks.listSyncFolders.mockResolvedValue([])
  apiMocks.addSyncFolder.mockResolvedValue({ name: 'config', local_path: '~/.ccr/platforms/', remote_path: '/ccr/platforms', enabled: true })
  apiMocks.updateSyncFolder.mockResolvedValue({ name: 'config', local_path: '~/.ccr/platforms/', remote_path: '/ccr/platforms', enabled: true })
  apiMocks.deleteSyncFolder.mockResolvedValue({ success: true, message: 'deleted' })
  apiMocks.pushSync.mockResolvedValue({ success: true, message: 'pushed', total: 1, successCount: 1, failed: [] })
  apiMocks.pullSync.mockResolvedValue({ success: true, message: 'pulled', total: 1, successCount: 1, failed: [] })
  apiMocks.pushSyncFolder.mockResolvedValue({ success: true, message: 'folder pushed', total: 1, successCount: 1, failed: [] })
  apiMocks.pullSyncFolder.mockResolvedValue({ success: true, message: 'folder pulled', total: 1, successCount: 1, failed: [] })
  lastSelectionProps.value = null
  lastBatchProps.value = null
  lastFoldersProps.value = null
})

afterEach(() => {
  document.body.innerHTML = ''
  vi.clearAllMocks()
})

describe('SyncView WebDAV folder selection flow', () => {
  it('enables apply selection when WebDAV is configured but no default folder exists', async () => {
    const { unmount } = await mountView()

    try {
      expect(lastSelectionProps.value?.hasChanges).toBe(true)
      expect(lastBatchProps.value?.foldersCount).toBe(0)

      await (lastSelectionProps.value?.applySelection as () => Promise<void>)()
      await flushAsync()
      await nextTick()

      expect(apiMocks.addSyncFolder).toHaveBeenCalledWith(
        'config',
        '~/.ccr/platforms/',
        'platforms',
        'CCR 供应商配置（API地址、密钥等）',
      )
    } finally {
      unmount()
    }
  })

  it('normalizes snake_case folders and counts only enabled folders for batch actions', async () => {
    apiMocks.listSyncFolders.mockResolvedValue([
      { name: 'config', local_path: '~/.ccr/platforms/', remote_path: 'platforms', enabled: false },
      { name: 'claude', local_path: '~/.claude/', remote_path: '/ccr/claude', enabled: true },
    ])

    const { unmount } = await mountView()

    try {
      expect(lastFoldersProps.value?.folders).toEqual([
        { name: 'config', enabled: false, description: undefined, localPath: '~/.ccr/platforms/', remotePath: 'platforms' },
        { name: 'claude', enabled: true, description: undefined, localPath: '~/.claude/', remotePath: '/ccr/claude' },
      ])
      expect(lastBatchProps.value?.foldersCount).toBe(1)
    } finally {
      unmount()
    }
  })

  it('uses single-folder sync commands for per-folder upload and download', async () => {
    apiMocks.listSyncFolders.mockResolvedValue([
      { name: 'config', localPath: '~/.ccr/platforms/', remotePath: 'platforms', enabled: true },
    ])

    const { unmount } = await mountView()

    try {
      await (lastFoldersProps.value?.pushFolder as (name: string) => Promise<void>)('config')
      await (lastFoldersProps.value?.pullFolder as (name: string) => Promise<void>)('config')

      expect(apiMocks.pushSyncFolder).toHaveBeenCalledWith('config', false)
      expect(apiMocks.pullSyncFolder).toHaveBeenCalledWith('config', false)
    } finally {
      unmount()
    }
  })
})
