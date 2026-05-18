import { createApp, nextTick } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const apiMocks = vi.hoisted(() => ({
  getOpenCodeConfig: vi.fn(),
  getOpenCodeTuiSettings: vi.fn(),
  listOpenCodeProviders: vi.fn(),
  listOpenCodeMcpServers: vi.fn(),
  listOpenCodeAgents: vi.fn(),
  listOpenCodeCommands: vi.fn(),
  listOpenCodePlugins: vi.fn(),
  listOpenCodeLocalPlugins: vi.fn(),
}))

vi.mock('@/api', () => apiMocks)

vi.mock('vue-router', async () => {
  const { defineComponent, h } = await vi.importActual<typeof import('vue')>('vue')

  return {
    RouterLink: defineComponent({
      props: {
        to: {
          type: String,
          required: true,
        },
      },
      setup(props, { slots }) {
        return () => h('a', { href: props.to, 'data-router-link': props.to }, slots.default?.())
      },
    }),
  }
})

const flush = async () => {
  await Promise.resolve()
  await nextTick()
  await Promise.resolve()
  await nextTick()
}

const resetApiMocks = () => {
  apiMocks.getOpenCodeConfig.mockResolvedValue({
    model: 'anthropic/claude-sonnet-4',
    default_agent: 'planner',
    share: 'manual',
    server: {
      hostname: '127.0.0.1',
      port: 4096,
      cors: ['http://localhost:1420'],
    },
  })
  apiMocks.getOpenCodeTuiSettings.mockResolvedValue({ theme: 'catppuccin-mocha' })
  apiMocks.listOpenCodeProviders.mockResolvedValue([
    { id: 'anthropic', npm: '@ai-sdk/anthropic' },
    { id: 'openai', npm: '@ai-sdk/openai' },
  ])
  apiMocks.listOpenCodeMcpServers.mockResolvedValue([
    { id: 'docs', type: 'local', command: 'node' },
    { id: 'browser', type: 'remote', url: 'http://localhost:3000' },
  ])
  apiMocks.listOpenCodeAgents.mockResolvedValue([
    { name: 'planner', scope: 'global', mode: 'primary' },
    { name: 'reviewer', scope: 'project', mode: 'subagent' },
  ])
  apiMocks.listOpenCodeCommands.mockResolvedValue([
    { name: 'ship', scope: 'project', content: 'ship it' },
    { name: 'audit', scope: 'global', content: 'audit it' },
  ])
  apiMocks.listOpenCodePlugins.mockResolvedValue(['opencode-plugin-a'])
  apiMocks.listOpenCodeLocalPlugins.mockResolvedValue([
    { name: 'local-a', scope: 'project', path: '.opencode/plugins/local-a.ts' },
  ])
}

const mountView = async () => {
  const { default: OpenCodeView } = await import('@/views/OpenCodeView.vue')
  const el = document.createElement('div')
  document.body.appendChild(el)
  const app = createApp(OpenCodeView)
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

describe('OpenCodeView smoke', () => {
  beforeEach(() => {
    vi.resetModules()
    vi.clearAllMocks()
    resetApiMocks()
  })

  afterEach(() => {
    document.body.innerHTML = ''
  })

  it('renders a high-density ops board with runtime chips, six entries, and live counts', async () => {
    const { el, unmount } = await mountView()

    try {
      expect(el.textContent).toContain('Operational console')
      expect(el.textContent).toContain('OpenCode operator deck')
      expect(el.textContent).toContain('127.0.0.1:4096')
      expect(el.textContent).toContain('cors configured')
      expect(el.textContent).toContain('planner')

      const capabilityLinks = el.querySelectorAll('.opencode-capability-link')
      expect(capabilityLinks).toHaveLength(6)
      expect(el.textContent).toContain('Providers')
      expect(el.textContent).toContain('2 live')
      expect(el.textContent).toContain('MCP')
      expect(el.textContent).toContain('Agents')
      expect(el.textContent).toContain('Commands')
      expect(el.textContent).toContain('Plugins')
      expect(el.textContent).toContain('Settings')
      expect(el.textContent).toContain('Runtime intelligence')
      expect(el.textContent).toContain('opencode agent')
    } finally {
      unmount()
    }
  })

  it('switches the compact inspector without leaving the landing page', async () => {
    const { el, unmount } = await mountView()

    try {
      const toolsTab = Array.from(el.querySelectorAll<HTMLButtonElement>('.opencode-inspector-tab'))
        .find((button) => button.textContent?.includes('Built-in tools'))
      expect(toolsTab).toBeTruthy()

      toolsTab?.click()
      await flush()

      expect(el.textContent).toContain('websearch')
      expect(el.textContent).toContain('webfetch')
      expect(el.textContent).not.toContain('opencode stats')

      const discoveryTab = Array.from(el.querySelectorAll<HTMLButtonElement>('.opencode-inspector-tab'))
        .find((button) => button.textContent?.includes('Local discovery'))
      discoveryTab?.click()
      await flush()

      expect(el.textContent).toContain('local-a')
      expect(el.textContent).toContain('1 primary')
    } finally {
      unmount()
    }
  })

  it('refreshes overview data from the existing OpenCode API wrappers', async () => {
    const { el, unmount } = await mountView()

    try {
      apiMocks.listOpenCodeProviders.mockResolvedValueOnce([
        { id: 'anthropic', npm: '@ai-sdk/anthropic' },
        { id: 'openai', npm: '@ai-sdk/openai' },
        { id: 'google', npm: '@ai-sdk/google' },
      ])

      const refresh = Array.from(el.querySelectorAll<HTMLButtonElement>('button'))
        .find((button) => button.textContent?.includes('Refresh'))
      expect(refresh).toBeTruthy()

      refresh?.click()
      await flush()

      expect(apiMocks.getOpenCodeConfig).toHaveBeenCalledTimes(2)
      expect(apiMocks.listOpenCodeProviders).toHaveBeenCalledTimes(2)
      expect(el.textContent).toContain('3 live')
    } finally {
      unmount()
    }
  })

  it('keeps usable data visible and marks the failed source when one API rejects', async () => {
    apiMocks.listOpenCodeMcpServers.mockRejectedValueOnce(new Error('mcp config unreadable'))

    const { el, unmount } = await mountView()

    try {
      expect(el.textContent).toContain('Operational console')
      expect(el.textContent).toContain('Providers')
      expect(el.textContent).toContain('2 live')
      expect(el.textContent).toContain('mcp warning')
      expect(el.textContent).toContain('1 degraded source(s)')
    } finally {
      unmount()
    }
  })
})
