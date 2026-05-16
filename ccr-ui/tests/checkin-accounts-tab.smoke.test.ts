import { createApp, defineComponent, h, nextTick } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import type { AccountInfo, CheckinProvider } from '@/types/checkin'
import * as checkinApi from '@/api'
import CheckinAccountsTab from '@/views/checkin/tabs/CheckinAccountsTab.vue'
import { createI18nStub } from './helpers/i18n-stub'

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

vi.mock('@/components/common/BaseModal.vue', () => ({
  default: defineComponent({
    props: {
      modelValue: { type: Boolean, required: true },
    },
    setup(props, { slots }) {
      return () =>
        props.modelValue
          ? h('div', [
              slots.header?.({ titleId: 'modal-title' }),
              slots.default?.(),
              slots.footer?.(),
            ])
          : null
    },
  }),
}))

vi.mock('@/stores/ui', () => ({
  useUIStore: () => ({
    showError: vi.fn(),
    requestConfirm: vi.fn().mockResolvedValue(true),
  }),
}))

vi.mock('@/api', () => ({
  createCheckinAccount: vi.fn(),
  updateCheckinAccount: vi.fn(),
  deleteCheckinAccount: vi.fn(),
  getCheckinAccountCookies: vi.fn(),
}))

const mockedGetCheckinAccountCookies = checkinApi.getCheckinAccountCookies as ReturnType<typeof vi.fn>

const providers: CheckinProvider[] = [
  {
    id: 'provider-1',
    name: 'Elysiver',
    base_url: 'https://example.com',
    checkin_path: '/checkin',
    balance_path: '/balance',
    user_info_path: '/user',
    auth_header: 'Authorization',
    auth_prefix: 'Bearer ',
  } as CheckinProvider,
]

const accounts: AccountInfo[] = [
  {
    id: 'account-1',
    provider_id: 'provider-1',
    provider_name: 'Elysiver',
    name: 'elysiver_main',
    enabled: true,
    latest_balance: 12.34,
    total_quota: 56.78,
    total_consumed: 44.44,
    last_checkin_at: '2026-03-26T08:56:14Z',
    api_user: '12345',
  } as AccountInfo,
]

const openAccountEditor = async (el: HTMLElement, editLabel = 'Edit') => {
  const trigger = el.querySelector<HTMLButtonElement>('.checkin-accounts-tab__menu-trigger')
  expect(trigger).not.toBeNull()

  Object.defineProperty(trigger!, 'getBoundingClientRect', {
    configurable: true,
    value: () => ({
      width: 40,
      height: 40,
      top: 140,
      right: 980,
      bottom: 180,
      left: 940,
      x: 940,
      y: 140,
      toJSON: () => {},
    }),
  })

  trigger!.dispatchEvent(new MouseEvent('click', { bubbles: true }))
  await nextTick()

  const editButton = Array.from(
    document.body.querySelectorAll<HTMLButtonElement>('.checkin-accounts-tab__menu-item')
  ).find(button => button.textContent?.includes(editLabel))

  expect(editButton).not.toBeNull()
  editButton!.dispatchEvent(new MouseEvent('click', { bubbles: true }))
  await Promise.resolve()
  await nextTick()
}

const getCookiesTextarea = (el: HTMLElement) =>
  el.querySelector<HTMLTextAreaElement>(
    '.checkin-accounts-tab__control--textarea.checkin-accounts-tab__control--mono'
  )

const getApiUserInput = (el: HTMLElement) =>
  el.querySelector<HTMLInputElement>('input[placeholder="12345"]')

const mountTab = async (locale: 'en-US' | 'zh-CN' = 'en-US') => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(
    defineComponent({
      setup() {
        return () =>
          h(CheckinAccountsTab, {
            providers,
            accounts,
            builtinProviders: [],
            checkinLoading: false,
            onRefresh: () => {},
            onCheckin: () => {},
            onRefreshBalance: () => {},
            onNavigate: () => {},
            onShowOauthWizard: () => {},
          })
      },
    })
  )

  app.use(createI18nStub(locale))
  app.mount(el)
  await nextTick()

  return {
    el,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

afterEach(() => {
  vi.clearAllMocks()
  document.body.innerHTML = ''
})

describe('CheckinAccountsTab smoke', () => {
  it('renders account actions in English with no-wrap single account button', async () => {
    const { el, unmount } = await mountTab('en-US')

    try {
      expect(el.textContent).toContain('Account Management')
      expect(el.textContent).toContain('OAuth Login')
      expect(el.textContent).toContain('Check in')
      expect(el.textContent).not.toContain('签到账号')
      expect(el.textContent).not.toContain('OAuth 登录')

      const miniButton = el.querySelector<HTMLButtonElement>('.checkin-accounts-tab__mini-button')
      const miniLabel = el.querySelector<HTMLElement>('.checkin-accounts-tab__mini-button-label')
      expect(miniButton).not.toBeNull()
      expect(miniLabel).not.toBeNull()
      expect(miniButton?.className).toContain('checkin-accounts-tab__mini-button')
      expect(miniLabel?.textContent).toBe('Check in')
    } finally {
      unmount()
    }
  })

  it('renders account actions in Chinese', async () => {
    const { el, unmount } = await mountTab('zh-CN')

    try {
      expect(el.textContent).toContain('签到账号')
      expect(el.textContent).toContain('OAuth 登录')
      expect(el.textContent).toContain('签到')
    } finally {
      unmount()
    }
  })

  it('teleports the account action menu to body to avoid table clipping', async () => {
    const { el, unmount } = await mountTab('zh-CN')

    try {
      const trigger = el.querySelector<HTMLButtonElement>('.checkin-accounts-tab__menu-trigger')
      expect(trigger).not.toBeNull()

      Object.defineProperty(trigger!, 'getBoundingClientRect', {
        configurable: true,
        value: () => ({
          width: 40,
          height: 40,
          top: 140,
          right: 980,
          bottom: 180,
          left: 940,
          x: 940,
          y: 140,
          toJSON: () => {},
        }),
      })

      trigger!.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await nextTick()

      const teleportedMenu = document.body.querySelector<HTMLElement>(
        '.checkin-accounts-tab__menu--floating'
      )

      expect(teleportedMenu).not.toBeNull()
      expect(teleportedMenu?.textContent).toContain('刷新余额')
      expect(teleportedMenu?.textContent).toContain('编辑')
      expect(teleportedMenu?.textContent).toContain('删除')
      expect(teleportedMenu?.className).toContain('checkin-accounts-tab__menu--bottom')
      expect(el.querySelector('.checkin-accounts-tab__menu--floating')).toBeNull()
    } finally {
      unmount()
    }
  })

  it('backfills api user when opening editor for an existing account', async () => {
    mockedGetCheckinAccountCookies.mockResolvedValue({
      cookies_json: '{"session":"abc123"}',
      api_user: '67890',
    })

    const { el, unmount } = await mountTab()

    try {
      await openAccountEditor(el)

      const apiUserInput = getApiUserInput(document.body)
      const cookiesTextarea = getCookiesTextarea(document.body)

      expect(apiUserInput).not.toBeNull()
      expect(cookiesTextarea).not.toBeNull()
      expect(apiUserInput?.value).toBe('67890')
      expect(cookiesTextarea?.value).toBe('abc123')
    } finally {
      unmount()
    }
  })

  it('preserves full cookies JSON when opening editor for an existing account', async () => {
    const fullCookiesJson = '{"session":"abc123","cf_clearance":"token-1"}'
    mockedGetCheckinAccountCookies.mockResolvedValue({
      cookies_json: fullCookiesJson,
      api_user: '67890',
    })

    const { el, unmount } = await mountTab()

    try {
      await openAccountEditor(el)

      const cookiesTextarea = getCookiesTextarea(document.body)
      expect(cookiesTextarea).not.toBeNull()
      expect(cookiesTextarea?.value).toBe(fullCookiesJson)
    } finally {
      unmount()
    }
  })
})
