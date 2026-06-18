import { createPinia } from 'pinia'
import { createI18n } from 'vue-i18n'
import { createApp, defineComponent, h, nextTick } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import enUS from '@/i18n/locales/en-US'

const apiMocks = vi.hoisted(() => ({
  listCodexAgentSources: vi.fn(),
  getCodexAgentSourceCatalog: vi.fn(),
  addCodexAgentSource: vi.fn(),
  removeCodexAgentSource: vi.fn(),
  syncCodexAgentSource: vi.fn(),
  installCodexSourceAgent: vi.fn(),
  syncCodexSourceInstall: vi.fn(),
  forceSyncCodexSourceInstall: vi.fn(),
  acceptLocalCodexSourceInstall: vi.fn(),
  untrackCodexSourceInstall: vi.fn(),
}))

vi.mock('@/api', () => ({
  ...apiMocks,
}))

vi.mock('@/stores/ui', () => ({
  useUIStore: () => ({
    showSuccess: vi.fn(),
    showError: vi.fn(),
    requestConfirm: vi.fn(async () => true),
  }),
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

vi.mock('@/components/ui/Card.vue', () => ({
  default: defineComponent({
    setup(_props, { slots }) {
      return () => h('div', { 'data-testid': 'card' }, slots.default?.())
    },
  }),
}))

vi.mock('@/components/common/BaseModal.vue', () => ({
  default: defineComponent({
    props: {
      modelValue: { type: Boolean, default: false },
      title: { type: String, default: '' },
    },
    emits: ['update:modelValue'],
    setup(props, { slots }) {
      return () => (props.modelValue
        ? h('div', { 'data-testid': 'base-modal' }, [
            h('div', props.title),
            slots.default?.(),
            slots.footer?.(),
          ])
        : null)
    },
  }),
}))

const i18n = createI18n({
  legacy: false,
  locale: 'en-US',
  fallbackLocale: 'en-US',
  missingWarn: false,
  fallbackWarn: false,
  messages: {
    'en-US': enUS,
  },
})

const mountComponent = async () => {
  const { default: CodexAgentSourcesPanel } = await import('@/components/codex/CodexAgentSourcesPanel.vue')
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(defineComponent({
    setup() {
      return () => h(CodexAgentSourcesPanel, {
        onRefreshInstalled: vi.fn(),
      })
    },
  }))

  app.use(createPinia())
  app.use(i18n)
  app.mount(el)
  await nextTick()
  await nextTick()
  await Promise.resolve()
  await nextTick()
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
  apiMocks.listCodexAgentSources.mockReset()
  apiMocks.getCodexAgentSourceCatalog.mockReset()
  apiMocks.addCodexAgentSource.mockReset()
  apiMocks.removeCodexAgentSource.mockReset()
  apiMocks.syncCodexAgentSource.mockReset()
  apiMocks.installCodexSourceAgent.mockReset()
  apiMocks.syncCodexSourceInstall.mockReset()
  apiMocks.forceSyncCodexSourceInstall.mockReset()
  apiMocks.acceptLocalCodexSourceInstall.mockReset()
  apiMocks.untrackCodexSourceInstall.mockReset()

  apiMocks.listCodexAgentSources.mockResolvedValue({
    sources: [
      {
        id: 'src_1',
        repoUrl: 'https://github.com/VoltAgent/awesome-codex-subagents',
        owner: 'VoltAgent',
        repo: 'awesome-codex-subagents',
        defaultBranch: 'main',
        status: 'ok',
        agentCount: 1,
        diagnosticsCount: 0,
        scanComplete: true,
        isStale: false,
        cacheTtlSeconds: 900,
      },
    ],
  })
  apiMocks.getCodexAgentSourceCatalog.mockResolvedValue({
    source: {
      id: 'src_1',
      repoUrl: 'https://github.com/VoltAgent/awesome-codex-subagents',
      owner: 'VoltAgent',
      repo: 'awesome-codex-subagents',
      defaultBranch: 'main',
      status: 'ok',
      agentCount: 1,
      diagnosticsCount: 0,
      scanComplete: true,
      isStale: false,
      cacheTtlSeconds: 900,
    },
    diagnostics: [],
    installs: [],
    agents: [
      {
        id: 'agent_1',
        sourceId: 'src_1',
        sourcePath: 'categories/01-core-development/frontend-developer.toml',
        fileName: 'frontend-developer.toml',
        blobSha: 'blob',
        contentHash: 'hash',
        category: '01-core-development',
        categoryLabel: 'Core Development',
        name: 'frontend-developer',
        description: 'Frontend specialist',
        rawToml: 'name = "frontend-developer"',
      },
    ],
  })
  apiMocks.installCodexSourceAgent.mockResolvedValue({ ok: true })
  apiMocks.acceptLocalCodexSourceInstall.mockResolvedValue({ ok: true })
  apiMocks.untrackCodexSourceInstall.mockResolvedValue({ ok: true })
})

afterEach(() => {
  document.body.innerHTML = ''
  vi.clearAllMocks()
})

describe('CodexAgentSourcesPanel smoke', () => {
  it('loads a source catalog and installs a remote agent', async () => {
    const { el, unmount } = await mountComponent()

    try {
      expect(el.textContent).toContain('VoltAgent/awesome-codex-subagents')
      expect(el.textContent).toContain('frontend-developer')

      const installButton = Array.from(el.querySelectorAll('button')).find((button) =>
        button.textContent?.includes('Install'))
      expect(installButton).toBeTruthy()
      installButton?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await nextTick()

      const modal = el.querySelector('[data-testid="base-modal"]')
      expect(modal?.textContent).toContain('Install Remote Agent')

      const confirmButton = Array.from(modal?.querySelectorAll('button') ?? []).find((button) =>
        button.textContent?.includes('Install'))
      confirmButton?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await nextTick()

      expect(apiMocks.installCodexSourceAgent).toHaveBeenCalledTimes(1)
      expect(apiMocks.installCodexSourceAgent.mock.calls[0]?.[0]).toMatchObject({
        sourceId: 'src_1',
        agentId: 'agent_1',
      })
    } finally {
      unmount()
    }
  })

  it('renders source error details when the source is unhealthy', async () => {
    apiMocks.listCodexAgentSources.mockResolvedValueOnce({
      sources: [
        {
          id: 'src_error',
          repoUrl: 'https://github.com/example/private-repo',
          owner: 'example',
          repo: 'private-repo',
          defaultBranch: 'main',
          status: 'access-denied',
          lastError: 'GitHub API: GitHub access denied or authentication is required',
          agentCount: 0,
          diagnosticsCount: 1,
          scanComplete: false,
          isStale: false,
          cacheTtlSeconds: 900,
        },
      ],
    })
    apiMocks.getCodexAgentSourceCatalog.mockResolvedValueOnce({
      source: {
        id: 'src_error',
        repoUrl: 'https://github.com/example/private-repo',
        owner: 'example',
        repo: 'private-repo',
        defaultBranch: 'main',
        status: 'access-denied',
        lastError: 'GitHub API: GitHub access denied or authentication is required',
        agentCount: 0,
        diagnosticsCount: 1,
        scanComplete: false,
        isStale: false,
        cacheTtlSeconds: 900,
      },
      diagnostics: [
        {
          path: 'https://github.com/example/private-repo',
          severity: 'error',
          message: 'GitHub API: GitHub access denied or authentication is required',
        },
      ],
      installs: [],
      agents: [],
    })

    const { el, unmount } = await mountComponent()

    try {
      expect(el.textContent).toContain('private-repo')
      expect(el.textContent).toContain('GitHub access denied or authentication is required')
      expect(el.textContent).toContain('access denied')
    } finally {
      unmount()
    }
  })

  it('renders a stale-cache callout when the catalog is stale', async () => {
    apiMocks.listCodexAgentSources.mockResolvedValueOnce({
      sources: [
        {
          id: 'src_stale',
          repoUrl: 'https://github.com/example/stale-repo',
          owner: 'example',
          repo: 'stale-repo',
          defaultBranch: 'main',
          status: 'ok',
          agentCount: 1,
          diagnosticsCount: 0,
          scanComplete: true,
          isStale: true,
          cacheTtlSeconds: 900,
        },
      ],
    })
    apiMocks.getCodexAgentSourceCatalog.mockResolvedValueOnce({
      source: {
        id: 'src_stale',
        repoUrl: 'https://github.com/example/stale-repo',
        owner: 'example',
        repo: 'stale-repo',
        defaultBranch: 'main',
        status: 'ok',
        agentCount: 1,
        diagnosticsCount: 0,
        scanComplete: true,
        isStale: true,
        cacheTtlSeconds: 900,
      },
      diagnostics: [],
      installs: [],
      agents: [
        {
          id: 'agent_stale',
          sourceId: 'src_stale',
          sourcePath: 'agents/reviewer.toml',
          fileName: 'reviewer.toml',
          blobSha: 'blob',
          contentHash: 'hash',
          category: 'agents',
          categoryLabel: 'Agents',
          name: 'reviewer',
          description: 'reviewer',
          rawToml: 'name = "reviewer"',
        },
      ],
    })

    const { el, unmount } = await mountComponent()

    try {
      expect(el.textContent).toContain('stale cache')
      expect(el.textContent).toContain('served from cache')
      expect(el.textContent).toContain('Rescan')
    } finally {
      unmount()
    }
  })

  it('surfaces tracked-install repair actions for local-modified and broken states', async () => {
    apiMocks.listCodexAgentSources.mockResolvedValueOnce({
      sources: [
        {
          id: 'src_actions',
          repoUrl: 'https://github.com/example/actions-repo',
          owner: 'example',
          repo: 'actions-repo',
          defaultBranch: 'main',
          status: 'ok',
          agentCount: 2,
          diagnosticsCount: 0,
          scanComplete: true,
          isStale: false,
          cacheTtlSeconds: 900,
        },
      ],
    })
    apiMocks.getCodexAgentSourceCatalog.mockResolvedValueOnce({
      source: {
        id: 'src_actions',
        repoUrl: 'https://github.com/example/actions-repo',
        owner: 'example',
        repo: 'actions-repo',
        defaultBranch: 'main',
        status: 'ok',
        agentCount: 2,
        diagnosticsCount: 0,
        scanComplete: true,
        isStale: false,
        cacheTtlSeconds: 900,
      },
      diagnostics: [],
      agents: [],
      installs: [
        {
          id: 'install_local',
          sourceId: 'src_actions',
          repoUrl: 'https://github.com/example/actions-repo',
          sourcePath: 'agents/a.toml',
          installedName: 'agent-a',
          targetPath: '/tmp/agent-a.toml',
          status: 'local-modified',
          hasUpstreamUpdate: false,
          hasLocalChanges: true,
        },
        {
          id: 'install_broken',
          sourceId: 'src_actions',
          repoUrl: 'https://github.com/example/actions-repo',
          sourcePath: 'agents/b.toml',
          installedName: 'agent-b',
          targetPath: '/tmp/agent-b.toml',
          status: 'broken',
          hasUpstreamUpdate: false,
          hasLocalChanges: false,
          lastError: 'missing',
        },
      ],
    })

    const { el, unmount } = await mountComponent()

    try {
      const acceptButton = Array.from(el.querySelectorAll('button')).find((button) =>
        button.textContent?.includes('Accept Local'))
      expect(acceptButton).toBeTruthy()
      acceptButton?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await nextTick()

      expect(apiMocks.acceptLocalCodexSourceInstall).toHaveBeenCalledTimes(1)
      expect(apiMocks.acceptLocalCodexSourceInstall.mock.calls[0]?.[0]).toBe('install_local')

      const untrackButton = Array.from(el.querySelectorAll('button')).find((button) =>
        button.textContent?.includes('Untrack'))
      expect(untrackButton).toBeTruthy()
      untrackButton?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await nextTick()

      expect(apiMocks.untrackCodexSourceInstall).toHaveBeenCalledTimes(1)
      expect(apiMocks.untrackCodexSourceInstall.mock.calls[0]?.[0]).toBe('install_broken')
    } finally {
      unmount()
    }
  })
})
