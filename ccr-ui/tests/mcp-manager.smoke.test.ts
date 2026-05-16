import { createApp, defineComponent, h, nextTick } from 'vue'
import { createPinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { useUIStore } from '@/stores/ui'

const apiMocks = vi.hoisted(() => ({
  listUnifiedMcp: vi.fn(),
  addUnifiedMcp: vi.fn(),
  updateUnifiedMcp: vi.fn(),
  deleteUnifiedMcp: vi.fn(),
  toggleUnifiedMcp: vi.fn(),
  importUnifiedMcpServers: vi.fn(),
}))

const confirmMock = vi.hoisted(() => vi.fn())

vi.mock('@/api', () => apiMocks)

vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string) => {
      const labels: Record<string, string> = {
        'mcp.searchServers': 'Search MCP servers',
      }
      return labels[key] ?? key
    },
  }),
}))

const flushPromises = async () => {
  await Promise.resolve()
  await Promise.resolve()
  await nextTick()
  await nextTick()
}

const createMcpResponse = () => ({
  servers: [
    {
      platform: 'claude',
      name: 'exa',
      url: 'https://local.example/mcp',
      command: null,
      args: [],
      env: {},
      headers: {
        Authorization: 'Bearer••••cret',
      },
      disabled: false,
      scope: 'local',
      source_path: 'C:/Users/test/.claude.json',
      approval_state: null,
      effective: true,
      hidden_by: null,
    },
    {
      platform: 'claude',
      name: 'exa',
      url: 'https://project.example/mcp',
      command: null,
      args: [],
      env: {},
      headers: {},
      disabled: false,
      scope: 'project',
      source_path: 'D:/repo/.mcp.json',
      approval_state: 'approved',
      effective: false,
      hidden_by: 'local:exa',
    },
    {
      platform: 'claude',
      name: 'memory',
      command: 'npx',
      url: null,
      args: ['-y', '@modelcontextprotocol/server-memory'],
      env: {
        EXA_API_KEY: 'sk-r••••••cret',
      },
      headers: {},
      disabled: false,
      scope: 'project',
      source_path: 'D:/repo/.mcp.json',
      approval_state: 'pending',
      effective: false,
      hidden_by: null,
    },
  ],
  capabilities: [
    {
      platform: 'claude',
      supports_toggle: true,
      supports_url: true,
      supports_headers: true,
      supports_timeout: true,
      supports_cwd: true,
      supports_trust: true,
      supports_include_tools: true,
    },
  ],
  diagnostics: [
    {
      level: 'info',
      message: 'Matched Claude local project key: D:/repo',
      source_path: 'C:/Users/test/.claude.json',
      scope: 'local',
      matched: true,
    },
  ],
})

const mountMcpManager = async () => {
  const McpManagerView = (await import('@/views/mcp/McpManagerView.vue')).default
  const el = document.createElement('div')
  document.body.appendChild(el)
  const pinia = createPinia()

  const app = createApp(defineComponent({
    setup() {
      return () => h(McpManagerView)
    },
  }))
  app.use(pinia)
  app.config.globalProperties.$t = (key: string) => {
    const labels: Record<string, string> = {
      'mcp.searchServers': 'Search MCP servers',
    }
    return labels[key] ?? key
  }
  app.mount(el)
  await flushPromises()

  return {
    el,
    pinia,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

beforeEach(() => {
  vi.clearAllMocks()
  apiMocks.listUnifiedMcp.mockResolvedValue(createMcpResponse())
  apiMocks.importUnifiedMcpServers.mockResolvedValue([])
  confirmMock.mockReturnValue(true)
  Object.defineProperty(window, 'confirm', {
    configurable: true,
    value: confirmMock,
  })
})

afterEach(() => {
  document.body.innerHTML = ''
  vi.restoreAllMocks()
})

describe('MCP manager smoke', () => {
  it('renders the unified { servers } response without empty cards and shows scope diagnostics', async () => {
    const { el, unmount } = await mountMcpManager()

    try {
      expect(apiMocks.listUnifiedMcp).toHaveBeenCalledTimes(1)
      expect(el.textContent).toContain('exa')
      expect(el.textContent).toContain('Effective config')
      expect(el.textContent).toContain('Source & precedence')
      expect(el.textContent).toContain('C:/Users/test/.claude.json')
      expect(el.textContent).toContain('Hidden by local:exa')
      expect(el.textContent).toContain('Bearer••••cret')
      expect(el.textContent).not.toContain('No MCP servers configured')
      expect(el.textContent).not.toContain('[object Object]')
    } finally {
      unmount()
    }
  })

  it('filters hidden and pending project servers while retaining source diagnostics', async () => {
    const { el, unmount } = await mountMcpManager()

    try {
      const hiddenButton = Array.from(el.querySelectorAll('button'))
        .find(button => button.textContent?.includes('Hidden')) as HTMLButtonElement
      hiddenButton.click()
      await nextTick()

      const memoryButton = Array.from(el.querySelectorAll('button'))
        .find(button => button.textContent?.includes('memory')) as HTMLButtonElement
      memoryButton.click()
      await nextTick()

      expect(el.textContent).toContain('memory')
      expect(el.textContent).toContain('Pending')
      expect(el.textContent).toContain('D:/repo/.mcp.json')
      expect(el.textContent).toContain('Matched Claude local project key')
    } finally {
      unmount()
    }
  })

  it('imports servers one by one, reports partial failures, and refreshes after import', async () => {
    apiMocks.importUnifiedMcpServers.mockResolvedValue([
      { name: 'exa', ok: true, message: 'added' },
      { name: 'bad', ok: false, error: 'invalid config' },
    ])

    const { el, pinia, unmount } = await mountMcpManager()

    try {
      const importButton = Array.from(el.querySelectorAll('button'))
        .find(button => button.textContent?.includes('Import')) as HTMLButtonElement
      importButton.click()
      await nextTick()

      const textarea = el.querySelector('textarea') as HTMLTextAreaElement
      textarea.value = JSON.stringify({
        mcpServers: {
          exa: { type: 'http', url: 'https://mcp.exa.ai/mcp' },
          bad: { command: 'npx', args: ['bad'] },
        },
      })
      textarea.dispatchEvent(new Event('input'))
      await nextTick()

      const submitButton = Array.from(el.querySelectorAll('button'))
        .find(button => button.textContent?.includes('Import 2 server')) as HTMLButtonElement
      submitButton.click()
      await flushPromises()

      expect(apiMocks.importUnifiedMcpServers).toHaveBeenCalledWith([
        expect.objectContaining({
          platform: 'claude',
          scope: 'user',
          name: 'exa',
          url: 'https://mcp.exa.ai/mcp',
        }),
        expect.objectContaining({
          platform: 'claude',
          scope: 'user',
          name: 'bad',
          command: 'npx',
          args: ['bad'],
        }),
      ])
      expect(apiMocks.listUnifiedMcp).toHaveBeenCalledTimes(2)
      expect(useUIStore(pinia).toasts.at(-1)?.message).toContain('failed: bad: invalid config')
    } finally {
      unmount()
    }
  })
})
