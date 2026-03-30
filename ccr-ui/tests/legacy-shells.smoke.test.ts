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
  listCodexModels: vi.fn(),
  getCodexProfile: vi.fn(),
  addCodexProfile: vi.fn(),
  updateCodexProfile: vi.fn(),
  deleteCodexProfile: vi.fn(),
  applyCodexProfile: vi.fn(),
  addCodexCustomModel: vi.fn(),
  listOpenCodeProviders: vi.fn(),
  addOpenCodeProvider: vi.fn(),
  updateOpenCodeProvider: vi.fn(),
  deleteOpenCodeProvider: vi.fn(),
  listOpenCodePlugins: vi.fn(),
  addOpenCodePlugin: vi.fn(),
  deleteOpenCodePlugin: vi.fn(),
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
  apiMocks.addCodexCustomModel.mockReset()
  apiMocks.listOpenCodeProviders.mockReset()
  apiMocks.addOpenCodeProvider.mockReset()
  apiMocks.updateOpenCodeProvider.mockReset()
  apiMocks.deleteOpenCodeProvider.mockReset()
  apiMocks.listOpenCodePlugins.mockReset()
  apiMocks.addOpenCodePlugin.mockReset()
  apiMocks.deleteOpenCodePlugin.mockReset()

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
  apiMocks.detectCodexProcess.mockResolvedValue({ has_running_process: false, pids: [] })
  apiMocks.getCodexAllQuotas.mockResolvedValue([])
  apiMocks.listCodexModels.mockResolvedValue({ builtin_models: ['gpt-5.4'], custom_models: [] })
  apiMocks.listOpenCodeProviders.mockResolvedValue([])
  apiMocks.listOpenCodePlugins.mockResolvedValue([])
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

  it('renders OpenCode plugins shell', async () => {
    const { default: OpenCodePluginsView } = await import('@/views/OpenCodePluginsView.vue')
    const { el, unmount } = await mountView(OpenCodePluginsView)

    try {
      expect(el.textContent).toContain('插件管理')
      expect(el.textContent).toContain('暂无插件')
    } finally {
      unmount()
    }
  })
})
