import { createPinia } from 'pinia'
import { createI18n } from 'vue-i18n'
import { createApp, defineComponent, h, nextTick } from 'vue'
import { createMemoryHistory, createRouter } from 'vue-router'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const apiMocks = vi.hoisted(() => ({
  deleteClaudeAuth: vi.fn(),
  getClaudeAuthCurrent: vi.fn(),
  listClaudeAuthAccounts: vi.fn(),
  saveClaudeAuth: vi.fn(),
  switchClaudeAuth: vi.fn(),
  listClaudeProfiles: vi.fn(),
  claudeProfileOff: vi.fn(),
}))

const uiMocks = vi.hoisted(() => ({
  requestConfirm: vi.fn(),
  showError: vi.fn(),
  showSuccess: vi.fn(),
  showWarning: vi.fn(),
}))

vi.mock('@/api', () => ({ ...apiMocks }))

vi.mock('@/api/domains/claude', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/api/domains/claude')>()
  return {
    ...actual,
    listClaudeProfiles: apiMocks.listClaudeProfiles,
    claudeProfileOff: apiMocks.claudeProfileOff,
  }
})

vi.mock('@/stores/ui', () => ({
  useUIStore: () => uiMocks,
}))

vi.mock('@/components/ModuleSubnav.vue', () => ({
  default: defineComponent({
    template: '<div data-testid="module-subnav" />',
  }),
}))

vi.mock('@/components/common/BaseModal.vue', () => ({
  default: defineComponent({
    props: {
      modelValue: { type: Boolean, default: false },
    },
    setup(props, { slots }) {
      return () => props.modelValue ? h('div', { 'data-testid': 'base-modal' }, slots.default?.()) : null
    },
  }),
}))

vi.mock('@/components/ui/EmptyState.vue', () => ({
  default: defineComponent({
    template: '<div data-testid="empty-state" />',
  }),
}))

const source = {
  kind: 'anthropic_api_key',
  location: 'settings_env',
  confidence: 'potential',
  evidence: 'official_contract',
  ownership: 'user_owned',
  suppresses_subscription: true,
} as const

const runtimeSummary = {
  mode: 'runtime_only',
  official_login_state: { type: 'LoggedInSaved', account_name: 'work' },
  current_auth_name: 'work',
  login_state: { type: 'LoggedInSaved', account_name: 'work' },
  auth_diagnosis: {
    observations: [source],
    presumed_effective_source: source,
    custom_api_key_responses_present: true,
    unobservable: [
      'other_shell_environment',
      'project_settings_for_unknown_working_directories',
    ],
  },
}

const flush = async () => {
  await Promise.resolve()
  await nextTick()
  await Promise.resolve()
  await nextTick()
}

const mountView = async () => {
  const { default: ClaudeAuthView } = await import('@/views/ClaudeAuthView.vue')
  const el = document.createElement('div')
  document.body.appendChild(el)

  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', component: defineComponent({ template: '<div />' }) },
      { path: '/claude-code', component: defineComponent({ template: '<div />' }) },
    ],
  })
  const i18n = createI18n({
    legacy: false,
    locale: 'en-US',
    fallbackLocale: 'en-US',
    messages: { 'en-US': {} },
  })
  const app = createApp(ClaudeAuthView)
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
  uiMocks.requestConfirm.mockResolvedValue(true)
  apiMocks.listClaudeAuthAccounts.mockResolvedValue({
    accounts: [
      {
        name: 'work',
        description: 'Primary subscription',
        email: 'wor***@example.com',
        billing_type: 'subscription',
        subscription_type: 'pro',
        rate_limit_tier: 'default',
        is_current: false,
        is_logged_in: true,
        saved_at: '2026-07-29T00:00:00Z',
        last_used: null,
        expires_at: '2027-07-29T00:00:00Z',
      },
    ],
    login_state: runtimeSummary.login_state,
    runtime_summary: runtimeSummary,
    current_profile_auth_mode: null,
  })
  apiMocks.getClaudeAuthCurrent.mockResolvedValue({
    logged_in: true,
    info: {
      account_uuid: 'account-work',
      email: 'work@example.com',
      billing_type: 'subscription',
      subscription_type: 'pro',
      rate_limit_tier: 'default',
      expires_at: '2027-07-29T00:00:00Z',
    },
    runtime_summary: runtimeSummary,
    login_state: runtimeSummary.login_state,
  })
  apiMocks.switchClaudeAuth.mockResolvedValue({
    success: true,
    message: 'switched',
    cleared_managed_sources: ['ANTHROPIC_AUTH_TOKEN'],
    remaining_suppressors: [source],
    warnings: ['fallback warning'],
  })
  apiMocks.saveClaudeAuth.mockResolvedValue({ success: true, message: 'saved' })
  apiMocks.deleteClaudeAuth.mockResolvedValue({ success: true, message: 'deleted' })
  apiMocks.listClaudeProfiles.mockResolvedValue({
    profiles: [],
    current_profile: null,
    can_off: true,
  })
  apiMocks.claudeProfileOff.mockResolvedValue({
    ok: true,
    changed: true,
    previous_profile: 'work',
    runtime_mode: 'official_auth',
    warnings: [],
    remaining_suppressors: [],
    cleared_managed_sources: [],
  })
})

afterEach(() => {
  document.body.innerHTML = ''
})

describe('ClaudeAuthView diagnosis', () => {
  it('renders the presumed source, confidence, ownership, and observable boundary', async () => {
    const { el, unmount } = await mountView()

    try {
      const diagnosis = el.querySelector('[data-testid="claude-auth-diagnosis"]')
      expect(diagnosis).not.toBeNull()
      expect(diagnosis?.textContent).toContain('Auth source diagnosis')
      expect(diagnosis?.textContent).toContain('ANTHROPIC_API_KEY')
      expect(diagnosis?.textContent).toContain('Potential')
      expect(diagnosis?.textContent).toContain('User-owned')
      expect(diagnosis?.textContent).toContain('Present, context only')
      expect(diagnosis?.textContent).toContain('2 unobservable layer(s)')
    } finally {
      unmount()
    }
  })

  it('shows structured remaining suppressors as warning toasts after switching', async () => {
    const { el, unmount } = await mountView()

    try {
      const switchButton = Array.from(el.querySelectorAll<HTMLButtonElement>('button')).find(
        button => button.textContent?.trim() === 'Switch'
      )
      expect(switchButton).not.toBeUndefined()
      switchButton?.click()

      await vi.waitFor(() => {
        expect(apiMocks.switchClaudeAuth).toHaveBeenCalledWith('work')
        expect(uiMocks.showWarning).toHaveBeenCalledWith(
          expect.stringContaining('ANTHROPIC_API_KEY'),
          6000,
        )
      })
      expect(uiMocks.showWarning).toHaveBeenCalledWith(
        expect.stringContaining('Potential'),
        6000,
      )
      expect(uiMocks.showSuccess).toHaveBeenCalledWith(
        expect.stringContaining('cleared 1 CCR-managed setting'),
      )
    } finally {
      unmount()
    }
  })

  it('shows the diagnosis off button when the backend reports can_off', async () => {
    const { el, unmount } = await mountView()

    try {
      expect(el.querySelector('[data-testid="claude-auth-profile-off"]')).not.toBeNull()
    } finally {
      unmount()
    }
  })

  it('does not write when the diagnosis off confirmation is cancelled', async () => {
    uiMocks.requestConfirm.mockResolvedValue(false)
    const { el, unmount } = await mountView()

    try {
      el.querySelector<HTMLButtonElement>('[data-testid="claude-auth-profile-off"]')?.click()
      await vi.waitFor(() => {
        expect(uiMocks.requestConfirm).toHaveBeenCalledWith(
          expect.objectContaining({ type: 'warning' }),
        )
      })
      expect(apiMocks.claudeProfileOff).not.toHaveBeenCalled()
    } finally {
      unmount()
    }
  })
})
