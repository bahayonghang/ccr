import { createApp, defineComponent, h, nextTick } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const apiMocks = vi.hoisted(() => ({
  convertConfig: vi.fn(),
  listSourceMcpServers: vi.fn(),
  syncMcpServer: vi.fn(),
  syncAllMcpServers: vi.fn(),
  listMcpPresets: vi.fn(),
  installMcpPreset: vi.fn(),
  sshListHosts: vi.fn(),
  refreshEnvironments: vi.fn(),
  sshAddHost: vi.fn(),
  sshConnect: vi.fn(),
  sshConfirmHostFingerprint: vi.fn(),
  sshDetectCli: vi.fn(),
  sshDisconnect: vi.fn(),
  sshGetConnectionState: vi.fn(),
  sshListKeys: vi.fn(),
  sshProbeHostFingerprint: vi.fn(),
  sshReadConfig: vi.fn(),
  sshReconnect: vi.fn(),
  sshTestConnection: vi.fn(),
  sshWriteConfig: vi.fn(),
}))

vi.mock('@/components/ui/SIcon.vue', () => ({
  default: defineComponent({
    props: {
      name: { type: String, required: true },
      size: { type: String, default: '' },
    },
    setup(props) {
      return () => h('span', { 'data-icon': props.name, class: props.size })
    },
  }),
}))

vi.mock('@/components/ModuleSubnav.vue', () => ({
  default: defineComponent({
    setup(_props, { slots }) {
      return () => h('div', { 'data-module-subnav': 'true' }, slots.default?.())
    },
  }),
}))

vi.mock('vue-router', () => ({
  RouterLink: defineComponent({
    props: {
      to: { type: [String, Object], required: true },
    },
    setup(_props, { slots }) {
      return () => h('a', {}, slots.default?.())
    },
  }),
}))

vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string, params?: Record<string, string | number>) => {
      if (params?.format) {
        return `${key}:${params.format}`
      }
      return key
    },
  }),
}))

vi.mock('@/stores/ui', () => ({
  useUIStore: () => ({
    showWarning: vi.fn(),
    showError: vi.fn(),
    showSuccess: vi.fn(),
  }),
}))

vi.mock('@/api', () => ({
  ...apiMocks,
}))

const flush = async () => {
  await Promise.resolve()
  await Promise.resolve()
  await nextTick()
}

const mountComponent = async (component: object, props?: Record<string, unknown>) => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(defineComponent({
    setup() {
      return () => h(component as never, props)
    },
  }))

  app.config.globalProperties.$t = (key: string) => key
  app.mount(el)
  await flush()

  return {
    el,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

beforeEach(() => {
  document.body.innerHTML = ''
  Object.values(apiMocks).forEach((mock) => mock.mockReset())

  apiMocks.listSourceMcpServers.mockResolvedValue([])
  apiMocks.syncMcpServer.mockResolvedValue({ results: [] })
  apiMocks.syncAllMcpServers.mockResolvedValue({ servers: {} })
  apiMocks.listMcpPresets.mockResolvedValue([
    {
      id: 'filesystem',
      name: 'Filesystem',
      description: 'Preset for filesystem access',
      tags: ['local'],
      requires_api_key: false,
      command: 'npx',
      args: ['-y', '@modelcontextprotocol/server-filesystem'],
    },
  ])
  apiMocks.installMcpPreset.mockResolvedValue({ results: [] })
  apiMocks.sshListHosts.mockResolvedValue([])
  apiMocks.refreshEnvironments.mockResolvedValue(undefined)
  apiMocks.sshAddHost.mockResolvedValue(undefined)
  apiMocks.sshConnect.mockResolvedValue({ connected: true })
  apiMocks.sshConfirmHostFingerprint.mockResolvedValue(undefined)
  apiMocks.sshDetectCli.mockResolvedValue({})
  apiMocks.sshDisconnect.mockResolvedValue({ connected: false })
  apiMocks.sshGetConnectionState.mockResolvedValue({ connected: false })
  apiMocks.sshListKeys.mockResolvedValue([])
  apiMocks.sshProbeHostFingerprint.mockResolvedValue({ status: 'ok' })
  apiMocks.sshReadConfig.mockResolvedValue('')
  apiMocks.sshReconnect.mockResolvedValue({ connected: true })
  apiMocks.sshTestConnection.mockResolvedValue({ success: true, latency_ms: 10 })
  apiMocks.sshWriteConfig.mockResolvedValue(undefined)
})

afterEach(() => {
  document.body.innerHTML = ''
})

describe('platform target pruning smoke', () => {
  it('keeps converter choices free of Droid', async () => {
    const ConverterView = (await import('@/views/ConverterView.vue')).default
    const { el, unmount } = await mountComponent(ConverterView)

    try {
      expect(el.textContent).toContain('Claude Code')
      expect(el.textContent).toContain('Codex')
      expect(el.textContent).toContain('Gemini')
      expect(el.textContent).not.toContain('Droid')
    } finally {
      unmount()
    }
  })

  it('keeps MCP sync and preset target pickers free of Droid', async () => {
    const [{ default: McpSyncPanel }, { default: McpPresetsPanel }] = await Promise.all([
      import('@/components/McpSyncPanel.vue'),
      import('@/components/McpPresetsPanel.vue'),
    ])

    const syncMount = await mountComponent(McpSyncPanel)
    const presetMount = await mountComponent(McpPresetsPanel)

    try {
      expect(syncMount.el.textContent).toContain('Codex')
      expect(syncMount.el.textContent).toContain('Gemini')
      expect(syncMount.el.textContent).not.toContain('Droid')

      const presetCard = presetMount.el.querySelector('div[class*="cursor-pointer"]')
      presetCard?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await flush()

      expect(presetMount.el.textContent).toContain('Claude')
      expect(presetMount.el.textContent).toContain('Codex')
      expect(presetMount.el.textContent).toContain('Gemini')
      expect(presetMount.el.textContent).not.toContain('Droid')
    } finally {
      syncMount.unmount()
      presetMount.unmount()
    }
  })

  it('keeps SSH platform selector free of Droid', async () => {
    const SshManagementView = (await import('@/views/SshManagementView.vue')).default
    const { el, unmount } = await mountComponent(SshManagementView)

    try {
      const options = Array.from(el.querySelectorAll('option')).map((option) => option.textContent?.trim())
      expect(options).toContain('claude')
      expect(options).toContain('codex')
      expect(options).toContain('gemini')
      expect(options).toContain('opencode')
      expect(options).not.toContain('droid')
    } finally {
      unmount()
    }
  })
})
