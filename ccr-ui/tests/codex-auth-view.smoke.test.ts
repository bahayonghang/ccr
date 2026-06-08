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

vi.mock('@/components/ui/Card.vue', () => ({
  default: defineComponent({
    setup(_props, { slots }) {
      return () => h('section', { class: 'card-stub' }, slots.default?.())
    },
  }),
}))

vi.mock('@/components/ui/Button.vue', () => ({
  default: defineComponent({
    emits: ['click'],
    setup(_props, { slots, emit, attrs }) {
      return () =>
        h(
          'button',
          {
            ...attrs,
            type: 'button',
            onClick: (event: MouseEvent) => emit('click', event),
          },
          slots.default?.()
        )
    },
  }),
}))

vi.mock('@/components/common/BaseModal.vue', () => ({
  default: defineComponent({
    props: {
      modelValue: { type: Boolean, default: false },
    },
    emits: ['update:modelValue'],
    setup(props, { slots }) {
      return () =>
        props.modelValue
          ? h('div', { 'data-testid': 'base-modal' }, [
              slots.header?.({ titleId: 'modal-title' }),
              slots.default?.(),
              slots.footer?.(),
            ])
          : null
    },
  }),
}))

vi.mock('@/components/ConfirmModal.vue', () => ({
  default: defineComponent({
    props: {
      isOpen: { type: Boolean, default: false },
    },
    setup(props) {
      return () => (props.isOpen ? h('div', { 'data-testid': 'confirm-modal' }) : null)
    },
  }),
}))

vi.mock('@/components/codex/CodexAccountCard.vue', () => ({
  default: defineComponent({
    props: {
      account: { type: Object, required: true },
      quota: { type: Object, default: null },
    },
    setup(props) {
      return () =>
        h(
          'div',
          {
            'data-testid': 'codex-account-card',
            'data-account-name': (props.account as { name: string }).name,
            'data-plan-type':
              (props.quota as { quota?: { plan_type?: string } } | null)?.quota?.plan_type ??
              'unknown',
          },
          (props.account as { name: string }).name
        )
    },
  }),
}))

vi.mock('@/stores/ui', () => ({
  useUIStore: () => ({
    showError: vi.fn(),
    showSuccess: vi.fn(),
    showInfo: vi.fn(),
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

const flush = async () => {
  await Promise.resolve()
  await nextTick()
  await Promise.resolve()
  await nextTick()
}

const mountView = async () => {
  const { default: CodexAuthView } = await import('@/views/CodexAuthView.vue')
  const el = document.createElement('div')
  document.body.appendChild(el)

  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', component: defineComponent({ template: '<div />' }) },
      { path: '/codex', component: defineComponent({ template: '<div />' }) },
    ],
  })

  const app = createApp(
    defineComponent({
      setup() {
        return () => h(CodexAuthView)
      },
    })
  )

  app.use(createPinia())
  app.use(i18n)
  app.use(router)
  await router.push('/')
  await router.isReady()

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
  vi.clearAllMocks()

  apiMocks.listCodexProfiles.mockResolvedValue({
    current_profile: 'default',
    profiles: [
      {
        name: 'default',
        auth_mode: 'openai_chatgpt',
      },
    ],
  })

  apiMocks.listCodexAuthAccounts.mockResolvedValue({
    login_state: { type: 'LoggedInSaved', account_name: 'qq_pro' },
    accounts: [
      {
        name: 'alpha-pro',
        description: 'primary account',
        email: 'alpha@example.com',
        is_current: true,
        is_virtual: false,
        plan_type: 'Pro',
        saved_at: '2026-04-14T08:00:00Z',
        last_used: '2026-04-12T08:00:00Z',
        last_refresh: '2026-04-14T07:00:00Z',
      },
      {
        name: 'beta-plus',
        description: 'backup account',
        email: 'beta@example.com',
        is_current: false,
        is_virtual: false,
        plan_type: 'Plus',
        saved_at: '2026-04-10T08:00:00Z',
        last_used: '2026-04-14T10:30:00Z',
        last_refresh: '2026-04-13T07:00:00Z',
      },
      {
        name: 'default',
        description: 'unsaved runtime session',
        email: 'runtime@example.com',
        is_current: false,
        is_virtual: true,
        saved_at: undefined,
        last_used: undefined,
        last_refresh: '2026-04-13T07:00:00Z',
      },
    ],
  })

  apiMocks.getCodexAuthCurrent.mockResolvedValue({
    logged_in: true,
    info: {
      account_id: 'acct_123',
      email: 'alpha@example.com',
      plan_type: 'Pro',
      last_refresh: '2026-04-14T07:00:00Z',
    },
  })

  apiMocks.detectCodexProcess.mockResolvedValue({ has_running_process: false, pids: [] })
  apiMocks.getCodexAllQuotas.mockResolvedValue([
    {
      account_name: 'alpha-pro',
      fetched_at: '2026-04-14T08:30:00Z',
      quota: { plan_type: 'Pro', hourly_percentage: 88, weekly_percentage: 76 },
    },
    {
      account_name: 'beta-plus',
      fetched_at: '2026-04-14T08:30:00Z',
      quota: { plan_type: 'Plus', hourly_percentage: 30, weekly_percentage: 22 },
    },
  ])
  apiMocks.codexIsOAuthPortInUse.mockResolvedValue(false)
  apiMocks.codexListModelProviders.mockResolvedValue({ providers: [] })
})

afterEach(() => {
  document.body.innerHTML = ''
})

describe('CodexAuthView smoke', () => {
  it('renders interpolated login state instead of leaking placeholders', async () => {
    const { el, unmount } = await mountView()

    try {
      expect(el.textContent).toContain('Logged in (qq_pro)')
      expect(el.textContent).not.toContain('{name}')
      expect(el.querySelectorAll('[data-testid="codex-account-card"]')).toHaveLength(3)
    } finally {
      unmount()
    }
  })

  it('filters accounts by search keyword and status pills', async () => {
    const { el, unmount } = await mountView()

    try {
      const searchInput = el.querySelector<HTMLInputElement>('.codex-auth-view__search-box input')
      expect(searchInput).not.toBeNull()

      searchInput!.value = 'beta'
      searchInput!.dispatchEvent(new Event('input', { bubbles: true }))
      await flush()

      let cards = Array.from(el.querySelectorAll<HTMLElement>('[data-testid="codex-account-card"]'))
      expect(cards.map((card) => card.dataset.accountName)).toEqual(['beta-plus'])

      const allButton = Array.from(
        el.querySelectorAll<HTMLButtonElement>('.codex-auth-view__filter-pill')
      ).find((button) => button.textContent?.includes('All'))
      const currentButton = Array.from(
        el.querySelectorAll<HTMLButtonElement>('.codex-auth-view__filter-pill')
      ).find((button) => button.textContent?.includes('Current'))

      expect(allButton).not.toBeNull()
      expect(currentButton).not.toBeNull()

      allButton!.click()
      searchInput!.value = ''
      searchInput!.dispatchEvent(new Event('input', { bubbles: true }))
      currentButton!.click()
      await flush()

      cards = Array.from(el.querySelectorAll<HTMLElement>('[data-testid="codex-account-card"]'))
      expect(cards.map((card) => card.dataset.accountName)).toEqual(['alpha-pro'])
    } finally {
      unmount()
    }
  })

  it('reorders cards when sort changes to recently used', async () => {
    const { el, unmount } = await mountView()

    try {
      let cards = Array.from(el.querySelectorAll<HTMLElement>('[data-testid="codex-account-card"]'))
      expect(cards.map((card) => card.dataset.accountName)).toEqual([
        'alpha-pro',
        'beta-plus',
        'default',
      ])

      const sortSelect = el.querySelector<HTMLSelectElement>('#codex-auth-sort')
      expect(sortSelect).not.toBeNull()

      sortSelect!.value = 'used_desc'
      sortSelect!.dispatchEvent(new Event('change', { bubbles: true }))
      await flush()

      cards = Array.from(el.querySelectorAll<HTMLElement>('[data-testid="codex-account-card"]'))
      expect(cards.map((card) => card.dataset.accountName)).toEqual([
        'beta-plus',
        'alpha-pro',
        'default',
      ])
    } finally {
      unmount()
    }
  })

  it('passes preferredAccountName through the API key add flow', async () => {
    apiMocks.codexAddAuthWithApiKey.mockResolvedValue({
      success: true,
      account_name: 'editorial_api',
      switched: false,
    })

    const { el, unmount } = await mountView()

    try {
      const openButton = Array.from(el.querySelectorAll<HTMLButtonElement>('button')).find(
        (button) => button.textContent?.includes('Add account')
      )
      expect(openButton).not.toBeNull()
      openButton!.click()
      await flush()

      const apiTab = Array.from(el.querySelectorAll<HTMLButtonElement>('button')).find((button) =>
        button.textContent?.includes('API Key')
      )
      expect(apiTab).not.toBeNull()
      apiTab!.click()
      await flush()

      const nameInput = el.querySelector<HTMLInputElement>(
        '[data-testid="codex-add-account-name-input"]'
      )
      const apiKeyInput = Array.from(el.querySelectorAll<HTMLInputElement>('input')).find(
        (input) => input.placeholder === 'sk-...'
      )
      expect(nameInput).not.toBeNull()
      expect(apiKeyInput).not.toBeNull()

      nameInput!.value = 'editorial_api'
      nameInput!.dispatchEvent(new Event('input', { bubbles: true }))
      apiKeyInput!.value = 'sk-live-test'
      apiKeyInput!.dispatchEvent(new Event('input', { bubbles: true }))
      await flush()

      const saveButton = Array.from(el.querySelectorAll<HTMLButtonElement>('button')).find(
        (button) => button.textContent?.includes('Save API account')
      )
      expect(saveButton).not.toBeNull()
      saveButton!.click()
      await flush()

      expect(apiMocks.codexAddAuthWithApiKey).toHaveBeenCalledWith(
        expect.objectContaining({
          apiKey: 'sk-live-test',
          preferredAccountName: 'editorial_api',
        })
      )
    } finally {
      unmount()
    }
  })

  it('applies a Codex provider template in the API key add flow without pre-filling secrets', async () => {
    apiMocks.codexAddAuthWithApiKey.mockResolvedValue({
      success: true,
      account_name: 'openrouter_api',
      switched: false,
    })

    const { el, unmount } = await mountView()

    try {
      const openButton = Array.from(el.querySelectorAll<HTMLButtonElement>('button')).find(
        (button) => button.textContent?.includes('Add account')
      )
      expect(openButton).not.toBeNull()
      openButton!.click()
      await flush()

      const apiTab = Array.from(el.querySelectorAll<HTMLButtonElement>('button')).find((button) =>
        button.textContent?.includes('API Key')
      )
      expect(apiTab).not.toBeNull()
      apiTab!.click()
      await flush()

      const trigger = el.querySelector<HTMLButtonElement>('[data-testid="provider-template-trigger"]')
      expect(trigger).not.toBeNull()
      trigger!.click()
      await flush()

      const searchInput = document.body.querySelector<HTMLInputElement>('[data-testid="provider-template-search"]')
      expect(searchInput).not.toBeNull()
      searchInput!.value = 'openrouter'
      searchInput!.dispatchEvent(new Event('input', { bubbles: true }))
      await flush()

      const option = Array.from(
        document.body.querySelectorAll<HTMLButtonElement>('[data-testid="provider-template-option"]')
      ).find((button) => button.textContent?.includes('OpenRouter'))
      expect(option).not.toBeNull()
      option!.click()
      await flush()

      const apiKeyInput = Array.from(el.querySelectorAll<HTMLInputElement>('input')).find(
        (input) => input.placeholder === 'sk-...'
      )
      expect(apiKeyInput).not.toBeNull()
      expect(apiKeyInput!.value).toBe('')

      apiKeyInput!.value = 'sk-user-entered'
      apiKeyInput!.dispatchEvent(new Event('input', { bubbles: true }))
      await flush()

      const saveButton = Array.from(el.querySelectorAll<HTMLButtonElement>('button')).find(
        (button) => button.textContent?.includes('Save API account')
      )
      expect(saveButton).not.toBeNull()
      saveButton!.click()
      await flush()

      expect(apiMocks.codexAddAuthWithApiKey).toHaveBeenCalledWith(
        expect.objectContaining({
          apiKey: 'sk-user-entered',
          apiBaseUrl: 'https://openrouter.ai/api/v1',
          providerName: 'OpenRouter',
        })
      )
    } finally {
      unmount()
    }
  })

  it('passes preferredAccountName for single JSON imports and locks naming for bundles', async () => {
    apiMocks.codexImportAuthPayload.mockResolvedValue({
      success: true,
      imported: 1,
      switched: false,
      results: [],
    })

    const { el, unmount } = await mountView()

    try {
      const openButton = Array.from(el.querySelectorAll<HTMLButtonElement>('button')).find(
        (button) => button.textContent?.includes('Add account')
      )
      expect(openButton).not.toBeNull()
      openButton!.click()
      await flush()

      const tokenTab = Array.from(el.querySelectorAll<HTMLButtonElement>('button')).find((button) =>
        button.textContent?.includes('Token / JSON')
      )
      expect(tokenTab).not.toBeNull()
      tokenTab!.click()
      await flush()

      const nameInput = el.querySelector<HTMLInputElement>(
        '[data-testid="codex-add-account-name-input"]'
      )
      const helper = el.querySelector<HTMLElement>('[data-testid="codex-add-account-name-helper"]')
      const jsonTextarea = el.querySelector<HTMLTextAreaElement>('.codex-auth-view__textarea--mono')
      expect(nameInput).not.toBeNull()
      expect(helper).not.toBeNull()
      expect(jsonTextarea).not.toBeNull()

      nameInput!.value = 'single_import'
      nameInput!.dispatchEvent(new Event('input', { bubbles: true }))
      jsonTextarea!.value = JSON.stringify({ OPENAI_API_KEY: 'sk-from-json' })
      jsonTextarea!.dispatchEvent(new Event('input', { bubbles: true }))
      await flush()

      const importButton = Array.from(el.querySelectorAll<HTMLButtonElement>('button')).find(
        (button) => button.textContent?.includes('Import payload')
      )
      expect(importButton).not.toBeNull()
      importButton!.click()
      await flush()

      expect(apiMocks.codexImportAuthPayload).toHaveBeenCalledWith(
        expect.objectContaining({
          preferredAccountName: 'single_import',
        })
      )

      jsonTextarea!.value = JSON.stringify({
        version: '1.0',
        accounts: {
          alpha: { account_id: 'a1' },
        },
      })
      jsonTextarea!.dispatchEvent(new Event('input', { bubbles: true }))
      await flush()

      expect(nameInput!.disabled).toBe(true)
      expect(helper!.textContent).toContain('embedded account names')
    } finally {
      unmount()
    }
  })
})
