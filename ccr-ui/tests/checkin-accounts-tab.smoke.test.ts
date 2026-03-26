import { createApp, defineComponent, h, nextTick } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import type { AccountInfo, CheckinProvider } from '@/types/checkin'

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

import CheckinAccountsTab from '@/views/checkin/tabs/CheckinAccountsTab.vue'

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

const mountTab = async () => {
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
  document.body.innerHTML = ''
})

describe('CheckinAccountsTab smoke', () => {
  it('teleports the account action menu to body to avoid table clipping', async () => {
    const { el, unmount } = await mountTab()

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
})
