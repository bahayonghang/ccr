import { createI18n } from 'vue-i18n'
import { createApp, defineComponent, h, nextTick } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import enUS from '@/i18n/locales/en-US'

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

vi.mock('@/composables/useCodexTrayPanel', async () => {
  const { computed, ref } = await import('vue')
  type TraySnapshot = import('@/types').CodexTraySnapshot

  const snapshot = ref<TraySnapshot | null>(null)
  const screen = ref<'overview' | 'switch'>('overview')
  const loading = ref(false)
  const busyAccount = ref<string | null>(null)
  const error = ref<string | null>(null)
  const isDragging = ref(false)

  const currentAccount = computed(() => snapshot.value?.current_account ?? null)
  const accounts = computed(() => snapshot.value?.accounts ?? [])
  const canManageAccounts = computed(() => snapshot.value?.can_manage_accounts ?? false)

  const loadSnapshot = vi.fn()
  const openMain = vi.fn()
  const openUsage = vi.fn()
  const openAuth = vi.fn()
  const quit = vi.fn()
  const startPanelDrag = vi.fn()
  const switchAccount = vi.fn()
  const goToSwitchScreen = vi.fn(() => {
    if (canManageAccounts.value) {
      screen.value = 'switch'
    }
  })
  const goToOverview = vi.fn(() => {
    screen.value = 'overview'
  })

  const seed = (nextSnapshot: TraySnapshot) => {
    snapshot.value = nextSnapshot
    screen.value = 'overview'
    loading.value = false
    busyAccount.value = null
    error.value = null
  }

  return {
    useCodexTrayPanel: () => ({
      accounts,
      busyAccount,
      canManageAccounts,
      currentAccount,
      error,
      goToOverview,
      goToSwitchScreen,
      isDragging,
      loadSnapshot,
      loading,
      openAuth,
      openMain,
      openUsage,
      quit,
      screen,
      snapshot,
      startPanelDrag,
      switchAccount,
    }),
    __trayPanelTestState: {
      seed,
      screen,
      openAuth,
      openMain,
      openUsage,
      quit,
      startPanelDrag,
      switchAccount,
    },
  }
})

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

const sampleSnapshot = (canManageAccounts = true) => ({
  fetched_at: '2026-04-16T10:00:00Z',
  runtime_mode: 'profile_with_auth',
  runtime_description: 'Profile route + Auth identity',
  profile_label: 'main_pro · openai_chatgpt',
  auth_label: 'OpenAI / ChatGPT',
  current_profile_name: 'main_pro',
  current_profile_provider: 'openai',
  current_profile_auth_mode: 'openai_chatgpt',
  current_auth_name: 'qq-pro',
  login_state: { type: 'LoggedInSaved', account_name: 'qq-pro' },
  can_manage_accounts: canManageAccounts,
  current_account: {
    name: 'qq-pro',
    email: '103***@qq.com',
    is_current: true,
    is_virtual: false,
    last_refresh: '2026-04-16T09:40:00Z',
    can_switch: false,
    quota: {
      hourly_percentage: 87,
      weekly_percentage: 46,
      hourly_reset_time: Math.floor(Date.now() / 1000) + 3600,
      weekly_reset_time: Math.floor(Date.now() / 1000) + 3600 * 24 * 3,
      plan_type: 'PRO',
    },
  },
  accounts: [
    {
      name: 'qq-pro',
      email: '103***@qq.com',
      is_current: true,
      is_virtual: false,
      last_refresh: '2026-04-16T09:40:00Z',
      can_switch: false,
    },
    {
      name: 'backup-plus',
      email: 'backup@example.com',
      is_current: false,
      is_virtual: false,
      last_refresh: '2026-04-15T18:00:00Z',
      can_switch: canManageAccounts,
    },
  ],
})

const flush = async () => {
  await Promise.resolve()
  await nextTick()
  await Promise.resolve()
  await nextTick()
}

// mock 工厂额外暴露的 __trayPanelTestState 不存在于真实模块类型上，用局部类型断言取回。
type TrayPanelTestState = {
  seed: (snapshot: ReturnType<typeof sampleSnapshot>) => void
  openAuth: ReturnType<typeof vi.fn>
  openMain: ReturnType<typeof vi.fn>
  openUsage: ReturnType<typeof vi.fn>
}

const getTrayPanelTestState = async (): Promise<TrayPanelTestState> => {
  const trayPanelModule = await import('@/composables/useCodexTrayPanel')
  return (trayPanelModule as unknown as { __trayPanelTestState: TrayPanelTestState })
    .__trayPanelTestState
}

const mountView = async () => {
  const { default: CodexTrayPanelView } = await import('@/views/tray/CodexTrayPanelView.vue')
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(
    defineComponent({
      setup() {
        return () => h(CodexTrayPanelView)
      },
    })
  )

  app.use(i18n)
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

beforeEach(async () => {
  vi.clearAllMocks()
  const testState = await getTrayPanelTestState()
  testState.seed(sampleSnapshot(true))
})

afterEach(() => {
  vi.clearAllMocks()
  document.body.innerHTML = ''
})

describe('codex tray panel', () => {
  it('keeps overview focused on current status and actions', async () => {
    const { el, unmount } = await mountView()

    try {
      expect(el.querySelector('[data-testid="tray-overview"]')).not.toBeNull()
      expect(el.querySelector('[data-testid="tray-switch-screen"]')).toBeNull()
      expect(el.querySelector('[data-testid="tray-switch-row-backup-plus"]')).toBeNull()
      expect(el.textContent).toContain('Switch Account')
      expect(el.textContent).toContain('Open Usage')
      expect(el.textContent).toContain('Open CCR')
    } finally {
      unmount()
    }
  })

  it('enters the in-panel switch screen from the primary switch action', async () => {
    const { el, unmount } = await mountView()

    try {
      const trigger = el.querySelector('[data-testid="tray-action-switch"]') as HTMLButtonElement
      trigger.click()
      await flush()

      expect(el.querySelector('[data-testid="tray-overview"]')).toBeNull()
      expect(el.querySelector('[data-testid="tray-switch-screen"]')).not.toBeNull()
      expect(el.querySelector('[data-testid="tray-switch-row-backup-plus"]')).not.toBeNull()
      expect(el.textContent).toContain('Open Auth in CCR')
    } finally {
      unmount()
    }
  })

  it('routes overview action buttons through the existing shell helpers', async () => {
    const testState = await getTrayPanelTestState()
    const { el, unmount } = await mountView()

    try {
      ;(el.querySelector('[data-testid="tray-action-open-usage"]') as HTMLButtonElement).click()
      ;(el.querySelector('[data-testid="tray-action-open-main"]') as HTMLButtonElement).click()
      await flush()

      expect(testState.openUsage).toHaveBeenCalledTimes(1)
      expect(testState.openMain).toHaveBeenCalledTimes(1)
    } finally {
      unmount()
    }
  })

  it('disables the switch action when account management is unavailable', async () => {
    const testState = await getTrayPanelTestState()
    testState.seed(sampleSnapshot(false))

    const { el, unmount } = await mountView()

    try {
      const trigger = el.querySelector('[data-testid="tray-action-switch"]') as HTMLButtonElement
      expect(trigger.disabled).toBe(true)
      expect(el.textContent).toContain('Switching is unavailable for the current profile.')
      ;(el.querySelector('[data-testid="tray-action-open-auth"]') as HTMLButtonElement).click()
      await flush()
      expect(testState.openAuth).toHaveBeenCalledTimes(1)
    } finally {
      unmount()
    }
  })
})
