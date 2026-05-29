
import { createApp, defineComponent, h, nextTick } from 'vue'
import { createI18n } from 'vue-i18n'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import enUS from '@/i18n/locales/en-US'

const apiMocks = vi.hoisted(() => ({
  getSyncStatus: vi.fn(),
  listSyncAssets: vi.fn(),
  pushSyncAsset: vi.fn(),
  pullSyncAsset: vi.fn(),
  syncSingleAsset: vi.fn(),
  syncAllAssets: vi.fn(),
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
      return () => h('section', { 'data-stub': 'PageHeaderCard' }, [
        h('div', { 'data-slot': 'actions' }, slots.actions?.()),
        h('div', { 'data-slot': 'default' }, slots.default?.()),
      ])
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

const i18n = createI18n({
  legacy: false,
  locale: 'en-US',
  fallbackLocale: 'en-US',
  missingWarn: false,
  fallbackWarn: false,
  messages: { 'en-US': enUS },
})

const assetFixtures = [
  {
    id: 'ccr-platforms',
    group: 'ccr',
    name: 'CCR Platforms',
    description: 'All platform configuration',
    kind: 'directory',
    sensitive: true,
    local_path: '~/.ccr/platforms/',
    resolved_local_path: 'C:/Users/test/.ccr/platforms',
    remote_path: '/ccr/platforms/',
    local_exists: true,
    remote_exists: false,
    canonical_name: null,
  },
  {
    id: 'claude-memory',
    group: 'claude',
    name: 'CLAUDE.md',
    description: 'Claude global memory',
    kind: 'file',
    sensitive: false,
    localPath: '~/.claude/CLAUDE.md',
    resolvedLocalPath: 'C:/Users/test/.claude/CLAUDE.md',
    remotePath: '/ccr/claude/CLAUDE.md',
    localExists: true,
    remoteExists: true,
    canonicalName: 'CLAUDE.md',
  },
  {
    id: 'codex-config',
    group: 'codex',
    name: 'config.toml',
    description: 'Codex user config',
    kind: 'file',
    sensitive: true,
    localPath: '~/.codex/config.toml',
    resolvedLocalPath: 'C:/Users/test/.codex/config.toml',
    remotePath: '/ccr/codex/config.toml',
    localExists: false,
    remoteExists: null,
    canonicalName: 'config.toml',
  },
]

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

const findButton = (el: Element, label: string): HTMLButtonElement => {
  const button = [...el.querySelectorAll('button')]
    .find(item => item.textContent?.includes(label)) as HTMLButtonElement | undefined
  expect(button).toBeTruthy()
  return button as HTMLButtonElement
}

beforeEach(() => {
  apiMocks.getSyncStatus.mockResolvedValue({ configured: true, enabled: true, webdav_url: 'https://dav.example.com', username: 'tester' })
  apiMocks.listSyncAssets.mockResolvedValue(assetFixtures)
  apiMocks.pushSyncAsset.mockResolvedValue({ success: true, message: 'pushed', total: 1, successCount: 1, failed: [] })
  apiMocks.pullSyncAsset.mockResolvedValue({ success: true, message: 'pulled', total: 1, successCount: 1, failed: [] })
  apiMocks.syncSingleAsset.mockResolvedValue({ success: true, message: 'synced', total: 1, successCount: 1, failed: [] })
  apiMocks.syncAllAssets.mockResolvedValue({ success: true, message: 'synced all', total: 3, successCount: 3, failed: [] })
})

afterEach(() => {
  document.body.innerHTML = ''
  vi.clearAllMocks()
})

describe('SyncView configuration asset console', () => {
  it('loads the fixed manifest assets and excludes legacy folder-selection panels', async () => {
    const { el, unmount } = await mountView()

    try {
      expect(apiMocks.listSyncAssets).toHaveBeenCalledTimes(1)
      expect(el.textContent).toContain('Configuration Asset Console')
      expect(el.textContent).toContain('CCR Platforms')
      expect(el.textContent).toContain('CLAUDE.md')
      expect(el.textContent).toContain('config.toml')
      expect(el.textContent).toContain('Local missing')
      expect(el.textContent).not.toContain('Select Sync Platforms')
      expect(el.textContent).not.toContain('Custom Folder')
      expect(el.textContent).not.toContain('Gemini')
      expect(el.textContent).not.toContain('Antigravity')
    } finally {
      unmount()
    }
  })

  it('runs per-asset push, pull, sync and whole-manifest sync commands', async () => {
    const { el, unmount } = await mountView()

    try {
      const firstCard = [...el.querySelectorAll('.sync-asset-card')]
        .find(card => card.textContent?.includes('CCR Platforms')) as HTMLElement
      expect(firstCard).toBeTruthy()

      findButton(firstCard, 'Push').click()
      await flushAsync()
      expect(apiMocks.pushSyncAsset).toHaveBeenCalledWith('ccr-platforms', false)

      findButton(firstCard, 'Pull').click()
      await flushAsync()
      expect(apiMocks.pullSyncAsset).toHaveBeenCalledWith('ccr-platforms', false)

      findButton(firstCard, 'Sync').click()
      await flushAsync()
      expect(apiMocks.syncSingleAsset).toHaveBeenCalledWith('ccr-platforms', false)

      findButton(el, 'Sync all once').click()
      await flushAsync()
      expect(apiMocks.syncAllAssets).toHaveBeenCalledWith(false)
    } finally {
      unmount()
    }
  })

  it('allows sync for remote-only assets so missing local config can be restored', async () => {
    const { el, unmount } = await mountView()

    try {
      const missingCard = [...el.querySelectorAll('.sync-asset-card')]
        .find(card => card.textContent?.includes('config.toml')) as HTMLElement
      expect(missingCard).toBeTruthy()

      const syncButton = findButton(missingCard, 'Sync')
      expect(syncButton.disabled).toBe(false)

      syncButton.click()
      await flushAsync()

      expect(apiMocks.syncSingleAsset).toHaveBeenCalledWith('codex-config', false)
    } finally {
      unmount()
    }
  })

  it('offers force retry for overwrite conflicts and masks operation output secrets', async () => {
    apiMocks.pushSyncAsset
      .mockRejectedValueOnce('Remote asset already exists; rerun with force to overwrite. api_key=sk-testsecret123456')
      .mockResolvedValueOnce({ success: true, message: 'forced api_key=sk-testsecret123456', total: 1, successCount: 1, failed: [] })

    const { el, unmount } = await mountView()

    try {
      const firstCard = [...el.querySelectorAll('.sync-asset-card')]
        .find(card => card.textContent?.includes('CCR Platforms')) as HTMLElement
      findButton(firstCard, 'Push').click()
      await flushAsync()
      await nextTick()

      expect(el.textContent).toContain('Force retry')
      expect(el.querySelector('[data-output]')?.textContent).toContain('api_key=••••••')
      expect(el.querySelector('[data-output]')?.textContent).not.toContain('sk-testsecret')

      findButton(firstCard, 'Force retry').click()
      await flushAsync()
      expect(apiMocks.pushSyncAsset).toHaveBeenLastCalledWith('ccr-platforms', true)
    } finally {
      unmount()
    }
  })

  it('offers force retry when sync finds both local and remote assets', async () => {
    apiMocks.syncAllAssets
      .mockResolvedValueOnce({
        success: false,
        message: 'Completed sync for 2/3 sync asset(s); 1 failed.',
        total: 3,
        successCount: 2,
        failed: [
          {
            folder: 'claude-settings',
            message: 'Remote asset already exists; rerun with force to overwrite.',
          },
        ],
      })
      .mockResolvedValueOnce({ success: true, message: 'forced sync all', total: 3, successCount: 3, failed: [] })

    const { el, unmount } = await mountView()

    try {
      findButton(el, 'Sync all once').click()
      await flushAsync()
      await nextTick()

      expect(apiMocks.syncAllAssets).toHaveBeenCalledWith(false)
      expect(el.textContent).toContain('Force sync all')

      findButton(el, 'Force sync all').click()
      await flushAsync()
      expect(apiMocks.syncAllAssets).toHaveBeenLastCalledWith(true)
    } finally {
      unmount()
    }
  })
})
