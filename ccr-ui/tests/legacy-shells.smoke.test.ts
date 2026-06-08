import { createPinia } from 'pinia'
import { createI18n } from 'vue-i18n'
import { createApp, defineComponent, h, nextTick } from 'vue'
import { createMemoryHistory, createRouter } from 'vue-router'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import enUS from '@/i18n/locales/en-US'

const apiMocks = vi.hoisted(() => ({
  listCodexProfiles: vi.fn(),
  listCodexAuthAccounts: vi.fn(),
  getCodexAuthCurrent: vi.fn(),
  saveCodexAuth: vi.fn(),
  switchCodexAuth: vi.fn(),
  deleteCodexAuth: vi.fn(),
  detectCodexProcess: vi.fn(),
  getCodexAllQuotas: vi.fn(),
  codexOAuthLoginStart: vi.fn(),
  codexOAuthLoginCompleted: vi.fn(),
  codexOAuthLoginCancel: vi.fn(),
  codexOAuthSubmitCallbackUrl: vi.fn(),
  codexIsOAuthPortInUse: vi.fn(),
  codexReleaseOAuthPort: vi.fn(),
  codexOpenExternalUrl: vi.fn(),
  codexImportAuthPayload: vi.fn(),
  codexImportAuthFromLocal: vi.fn(),
  codexAddAuthWithApiKey: vi.fn(),
  codexListModelProviders: vi.fn(),
  codexSaveModelProvider: vi.fn(),
  codexDeleteModelProvider: vi.fn(),
  listCodexModels: vi.fn(),
  getCodexProfile: vi.fn(),
  addCodexProfile: vi.fn(),
  updateCodexProfile: vi.fn(),
  deleteCodexProfile: vi.fn(),
  applyCodexProfile: vi.fn(),
  exportCodexProfiles: vi.fn(),
  addCodexCustomModel: vi.fn(),
  listOpenCodeProviders: vi.fn(),
  getOpenCodeConfig: vi.fn(),
  getOpenCodeTuiSettings: vi.fn(),
  addOpenCodeProvider: vi.fn(),
  updateOpenCodeProvider: vi.fn(),
  deleteOpenCodeProvider: vi.fn(),
  listOpenCodeMcpServers: vi.fn(),
  addOpenCodeMcpServer: vi.fn(),
  updateOpenCodeMcpServer: vi.fn(),
  deleteOpenCodeMcpServer: vi.fn(),
  listOpenCodeAgents: vi.fn(),
  addOpenCodeAgent: vi.fn(),
  updateOpenCodeAgent: vi.fn(),
  deleteOpenCodeAgent: vi.fn(),
  listOpenCodeCommands: vi.fn(),
  addOpenCodeCommand: vi.fn(),
  updateOpenCodeCommand: vi.fn(),
  deleteOpenCodeCommand: vi.fn(),
  listOpenCodePlugins: vi.fn(),
  listOpenCodeLocalPlugins: vi.fn(),
  listOpenCodeThemes: vi.fn(),
  addOpenCodePlugin: vi.fn(),
  deleteOpenCodePlugin: vi.fn(),
  updateOpenCodeConfig: vi.fn(),
  updateOpenCodeTuiSettings: vi.fn(),
}))

vi.mock('@/api', () => ({
  ...apiMocks,
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
    template: '<div data-testid="module-subnav" />',
  }),
}))

vi.mock('@/components/common/AnimatedBackground.vue', () => ({
  default: defineComponent({
    template: '<div data-testid="animated-background" />',
  }),
}))

vi.mock('@/components/codex/CodexAccountCard.vue', () => ({
  default: defineComponent({
    template: '<div data-testid="codex-account-card" />',
  }),
}))

vi.mock('@/components/codex/CodexProfileEditorModal.vue', () => ({
  default: defineComponent({
    props: {
      modelValue: { type: Boolean, default: false },
    },
    setup(props) {
      return () => (props.modelValue ? h('div', { 'data-testid': 'codex-profile-editor-modal' }) : null)
    },
  }),
}))

vi.mock('@/components/common/BaseModal.vue', () => ({
  default: defineComponent({
    props: {
      modelValue: { type: Boolean, default: false },
      title: { type: String, default: '' },
      contentClass: { type: String, default: '' },
    },
    emits: ['update:modelValue'],
    setup(props, { slots }) {
      return () => (props.modelValue
        ? h('div', { class: props.contentClass || '', 'data-testid': 'base-modal' }, [
            slots.header?.({ titleId: 'modal-title' }),
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

const mountView = async (component: unknown) => {
  const el = document.createElement('div')
  document.body.appendChild(el)
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', component: defineComponent({ template: '<div />' }) },
      { path: '/codex', component: defineComponent({ template: '<div />' }) },
      { path: '/opencode', component: defineComponent({ template: '<div />' }) },
    ],
  })

  const app = createApp(defineComponent({
    setup() {
      return () => h(component as never)
    },
  }))

  app.use(createPinia())
  app.use(i18n)
  app.use(router)
  await router.push('/')
  await router.isReady()

  app.mount(el)
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

const flushView = async () => {
  await Promise.resolve()
  await nextTick()
  await Promise.resolve()
  await nextTick()
}

const setControlValue = async (
  control: HTMLInputElement | HTMLTextAreaElement,
  value: string,
) => {
  control.value = value
  control.dispatchEvent(new Event('input', { bubbles: true }))
  await nextTick()
}

const findButtonByText = (root: ParentNode, text: string) => {
  return Array.from(root.querySelectorAll<HTMLButtonElement>('button'))
    .find((button) => button.textContent?.includes(text)) ?? null
}

beforeEach(() => {
  apiMocks.listCodexProfiles.mockReset()
  apiMocks.listCodexAuthAccounts.mockReset()
  apiMocks.getCodexAuthCurrent.mockReset()
  apiMocks.saveCodexAuth.mockReset()
  apiMocks.switchCodexAuth.mockReset()
  apiMocks.deleteCodexAuth.mockReset()
  apiMocks.detectCodexProcess.mockReset()
  apiMocks.getCodexAllQuotas.mockReset()
  apiMocks.listCodexModels.mockReset()
  apiMocks.getCodexProfile.mockReset()
  apiMocks.addCodexProfile.mockReset()
  apiMocks.updateCodexProfile.mockReset()
  apiMocks.deleteCodexProfile.mockReset()
  apiMocks.applyCodexProfile.mockReset()
  apiMocks.exportCodexProfiles.mockReset()
  apiMocks.addCodexCustomModel.mockReset()
  apiMocks.listOpenCodeProviders.mockReset()
  apiMocks.getOpenCodeConfig.mockReset()
  apiMocks.getOpenCodeTuiSettings.mockReset()
  apiMocks.addOpenCodeProvider.mockReset()
  apiMocks.updateOpenCodeProvider.mockReset()
  apiMocks.deleteOpenCodeProvider.mockReset()
  apiMocks.listOpenCodeMcpServers.mockReset()
  apiMocks.addOpenCodeMcpServer.mockReset()
  apiMocks.updateOpenCodeMcpServer.mockReset()
  apiMocks.deleteOpenCodeMcpServer.mockReset()
  apiMocks.listOpenCodeAgents.mockReset()
  apiMocks.addOpenCodeAgent.mockReset()
  apiMocks.updateOpenCodeAgent.mockReset()
  apiMocks.deleteOpenCodeAgent.mockReset()
  apiMocks.listOpenCodeCommands.mockReset()
  apiMocks.addOpenCodeCommand.mockReset()
  apiMocks.updateOpenCodeCommand.mockReset()
  apiMocks.deleteOpenCodeCommand.mockReset()
  apiMocks.listOpenCodePlugins.mockReset()
  apiMocks.listOpenCodeLocalPlugins.mockReset()
  apiMocks.listOpenCodeThemes.mockReset()
  apiMocks.addOpenCodePlugin.mockReset()
  apiMocks.deleteOpenCodePlugin.mockReset()
  apiMocks.updateOpenCodeConfig.mockReset()
  apiMocks.updateOpenCodeTuiSettings.mockReset()

  apiMocks.listCodexProfiles.mockResolvedValue({
    current_profile: 'default',
    profiles: [
      {
        name: 'default',
        enabled: true,
        model: 'gpt-5.4',
        auth_mode: 'openai_chatgpt',
        base_url: '',
        description: 'default profile',
        tags: [],
        provider: 'openai',
      },
    ],
  })
  apiMocks.listCodexAuthAccounts.mockResolvedValue({
    accounts: [],
    login_state: { type: 'NotLoggedIn' },
  })
  apiMocks.getCodexAuthCurrent.mockResolvedValue({ logged_in: false, info: null })
  apiMocks.exportCodexProfiles.mockResolvedValue({
    content: '[profiles.default]\nauth_token = "secret"\n',
    filename: 'ccr-codex-profiles-test.toml',
  })
  apiMocks.detectCodexProcess.mockResolvedValue({ has_running_process: false, pids: [] })
  apiMocks.getCodexAllQuotas.mockResolvedValue([])
  apiMocks.codexIsOAuthPortInUse.mockResolvedValue(false)
  apiMocks.codexListModelProviders.mockResolvedValue({ providers: [] })
  apiMocks.listCodexModels.mockResolvedValue({ builtin_models: ['gpt-5.4'], custom_models: [] })
  apiMocks.getOpenCodeConfig.mockResolvedValue({})
  apiMocks.getOpenCodeTuiSettings.mockResolvedValue({})
  apiMocks.listOpenCodeProviders.mockResolvedValue([])
  apiMocks.listOpenCodeMcpServers.mockResolvedValue([])
  apiMocks.listOpenCodeAgents.mockResolvedValue([])
  apiMocks.listOpenCodeCommands.mockResolvedValue([])
  apiMocks.listOpenCodePlugins.mockResolvedValue([])
  apiMocks.listOpenCodeLocalPlugins.mockResolvedValue([])
  apiMocks.listOpenCodeThemes.mockResolvedValue([])
  Object.defineProperty(URL, 'createObjectURL', {
    configurable: true,
    value: vi.fn(() => 'blob:ccr-codex-profiles-test'),
  })
  Object.defineProperty(URL, 'revokeObjectURL', {
    configurable: true,
    value: vi.fn(),
  })
  vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(() => undefined)
})

afterEach(() => {
  document.body.innerHTML = ''
  vi.clearAllMocks()
})

describe('legacy shell pages smoke', () => {
  it('renders Codex auth shell', async () => {
    const { default: CodexAuthView } = await import('@/views/CodexAuthView.vue')
    const { el, unmount } = await mountView(CodexAuthView)

    try {
      expect(el.textContent).toContain('Auth')
      expect(el.querySelectorAll('[data-testid="codex-account-card"]')).toHaveLength(0)
    } finally {
      unmount()
    }
  })

  it('renders Codex profiles shell', async () => {
    const { default: CodexProfilesView } = await import('@/views/CodexProfilesView.vue')
    const { el, unmount } = await mountView(CodexProfilesView)

    try {
      expect(el.textContent).toContain('Profile')
      expect(el.textContent).toContain('Official Config')
    } finally {
      unmount()
    }
  })

  it('exports Codex profiles TOML from the profiles shell', async () => {
    const { default: CodexProfilesView } = await import('@/views/CodexProfilesView.vue')
    const { el, unmount } = await mountView(CodexProfilesView)

    try {
      await Promise.resolve()
      await nextTick()
      await nextTick()

      const exportButton = el.querySelector('[data-icon="Download"]')?.closest('button') as HTMLButtonElement | null

      expect(exportButton).not.toBeNull()
      expect(exportButton!.disabled).toBe(false)

      exportButton!.click()
      await Promise.resolve()
      await nextTick()
      await nextTick()

      expect(apiMocks.exportCodexProfiles).toHaveBeenCalledWith(true)
      expect(URL.createObjectURL).toHaveBeenCalled()
      expect(HTMLAnchorElement.prototype.click).toHaveBeenCalled()
      expect(URL.revokeObjectURL).toHaveBeenCalledWith('blob:ccr-codex-profiles-test')
    } finally {
      unmount()
    }
  })

  it('renders OpenCode providers shell', async () => {
    const { default: OpenCodeProvidersView } = await import('@/views/OpenCodeProvidersView.vue')
    const { el, unmount } = await mountView(OpenCodeProvidersView)

    try {
      expect(el.textContent).toContain('Provider')
      expect(el.textContent).toContain('暂无 Provider')
    } finally {
      unmount()
    }
  })

  it('creates OpenCode OpenAI-compatible providers with root npm config', async () => {
    apiMocks.addOpenCodeProvider.mockResolvedValue({ id: 'openai' })
    apiMocks.updateOpenCodeConfig.mockResolvedValue({ disabled_providers: [] })

    const { default: OpenCodeProvidersView } = await import('@/views/OpenCodeProvidersView.vue')
    const { el, unmount } = await mountView(OpenCodeProvidersView)

    try {
      el.querySelector<HTMLButtonElement>('[data-testid="provider-template-trigger"]')?.click()
      await flushView()

      const searchInput = el.querySelector<HTMLInputElement>('[data-testid="provider-template-search"]')
      expect(searchInput).not.toBeNull()
      await setControlValue(searchInput!, 'openai compatible')

      const option = Array.from(el.querySelectorAll<HTMLButtonElement>('[data-testid="provider-template-option"]'))
        .find(button => button.textContent?.includes('OpenAI Compatible'))
      expect(option).not.toBeNull()

      option!.click()
      await flushView()

      const textInputs = Array.from(el.querySelectorAll<HTMLInputElement>('input:not([type="checkbox"])'))
      const providerInputs = textInputs.slice(-5)
      expect(providerInputs[0]?.value).toBe('openai')
      expect(providerInputs[1]?.value).toBe('OpenAI Compatible')
      expect(providerInputs[2]?.value).toBe('@ai-sdk/openai-compatible')

      await setControlValue(providerInputs[3], '<YOUR_API_KEY>')
      await setControlValue(providerInputs[4], 'https://api.example.com/v1')

      const textareas = Array.from(el.querySelectorAll<HTMLTextAreaElement>('textarea'))
      await setControlValue(textareas[0], JSON.stringify({
        'gpt-5.2': {
          name: 'GPT-5.2',
          limit: {
            context: 400000,
            output: 128000,
          },
          options: {
            store: false,
          },
          variants: {
            low: {},
            medium: {},
            high: {},
            xhigh: {},
          },
        },
      }))

      findButtonByText(el, '保存')?.click()
      await flushView()

      expect(apiMocks.addOpenCodeProvider).toHaveBeenCalledWith('openai', {
        name: 'OpenAI Compatible',
        npm: '@ai-sdk/openai-compatible',
        options: {
          apiKey: '<YOUR_API_KEY>',
          baseURL: 'https://api.example.com/v1',
        },
        models: {
          'gpt-5.2': {
            name: 'GPT-5.2',
            limit: {
              context: 400000,
              output: 128000,
            },
            options: {
              store: false,
            },
            variants: {
              low: {},
              medium: {},
              high: {},
              xhigh: {},
            },
          },
        },
      })
    } finally {
      unmount()
    }
  })

  it('preserves OpenCode provider root extras when saving edits', async () => {
    apiMocks.listOpenCodeProviders.mockResolvedValue([
      {
        id: 'openai',
        name: 'OpenAI Compatible',
        npm: '@ai-sdk/openai-compatible',
        api: 'chat',
        whitelist: ['gpt-5.2'],
        options: {
          apiKey: '{env:OPENAI_API_KEY}',
          baseURL: 'https://api.example.com/v1',
          timeout: 600000,
        },
        models: {
          'gpt-5.2': {
            name: 'GPT-5.2',
          },
        },
      },
    ])
    apiMocks.addOpenCodeProvider.mockResolvedValue({ id: 'openai' })
    apiMocks.updateOpenCodeConfig.mockResolvedValue({ disabled_providers: [] })

    const { default: OpenCodeProvidersView } = await import('@/views/OpenCodeProvidersView.vue')
    const { el, unmount } = await mountView(OpenCodeProvidersView)

    try {
      findButtonByText(el, '编辑')?.click()
      await flushView()

      findButtonByText(el, '保存')?.click()
      await flushView()

      expect(apiMocks.addOpenCodeProvider).toHaveBeenCalledWith('openai', {
        name: 'OpenAI Compatible',
        npm: '@ai-sdk/openai-compatible',
        api: 'chat',
        whitelist: ['gpt-5.2'],
        options: {
          timeout: 600000,
          apiKey: '{env:OPENAI_API_KEY}',
          baseURL: 'https://api.example.com/v1',
        },
        models: {
          'gpt-5.2': {
            name: 'GPT-5.2',
          },
        },
      })
    } finally {
      unmount()
    }
  })

  it('renders OpenCode mcp shell', async () => {
    const { default: OpenCodeMcpView } = await import('@/views/OpenCodeMcpView.vue')
    const { el, unmount } = await mountView(OpenCodeMcpView)

    try {
      expect(el.textContent).toContain('MCP')
      expect(el.textContent).toContain('暂无 MCP 服务器')
    } finally {
      unmount()
    }
  })

  it('renders OpenCode agents shell', async () => {
    const { default: OpenCodeAgentsView } = await import('@/views/OpenCodeAgentsView.vue')
    const { el, unmount } = await mountView(OpenCodeAgentsView)

    try {
      expect(el.textContent).toContain('Built-in layout')
      expect(el.textContent).toContain('暂无自定义 Agent')
    } finally {
      unmount()
    }
  })

  it('renders OpenCode commands shell', async () => {
    const { default: OpenCodeCommandsView } = await import('@/views/OpenCodeCommandsView.vue')
    const { el, unmount } = await mountView(OpenCodeCommandsView)

    try {
      expect(el.textContent).toContain('Built-in behavior')
      expect(el.textContent).toContain('暂无自定义 Command')
    } finally {
      unmount()
    }
  })

  it('renders OpenCode plugins shell', async () => {
    const { default: OpenCodePluginsView } = await import('@/views/OpenCodePluginsView.vue')
    const { el, unmount } = await mountView(OpenCodePluginsView)

    try {
      expect(el.textContent).toContain('Plugins')
      expect(el.textContent).toContain('暂无 npm 插件配置')
    } finally {
      unmount()
    }
  })

  it('renders OpenCode settings shell', async () => {
    const { default: OpenCodeSettingsView } = await import('@/views/OpenCodeSettingsView.vue')
    const { el, unmount } = await mountView(OpenCodeSettingsView)

    try {
      expect(el.textContent).toContain('Runtime config')
      expect(el.textContent).toContain('TUI config')
    } finally {
      unmount()
    }
  })
})
