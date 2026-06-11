import { createApp, defineComponent, h, nextTick } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import type { AccountInfo, CheckinProvider, CheckinRecordInfo } from '@/types/checkin'
import * as checkinApi from '@/api'
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
          ? h('div', { class: 'mock-base-modal' }, [
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
    showInfo: vi.fn(),
    requestConfirm: vi.fn().mockResolvedValue(true),
  }),
}))

vi.mock('@/api', () => ({
  createCheckinAccount: vi.fn(),
  updateCheckinAccount: vi.fn(),
  deleteCheckinAccount: vi.fn(),
  getCheckinAccountCookies: vi.fn(),
  listCheckinRecords: vi.fn(),
  exportCheckinRecords: vi.fn(),
}))

import CheckinRecordsTab from '@/views/checkin/tabs/CheckinRecordsTab.vue'
import CheckinAccountsTab from '@/views/checkin/tabs/CheckinAccountsTab.vue'

const mockedListCheckinRecords = checkinApi.listCheckinRecords as ReturnType<typeof vi.fn>
const mockedGetCheckinAccountCookies = checkinApi.getCheckinAccountCookies as ReturnType<
  typeof vi.fn
>

const providers: CheckinProvider[] = [
  {
    id: 'provider-1',
    name: 'AnyRouter',
    base_url: 'https://anyrouter.example.com',
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
    provider_name: 'AnyRouter',
    name: 'anyrouter_stumail',
    enabled: true,
    api_user: '12345',
  } as AccountInfo,
]

const cookieExpiredRecord: CheckinRecordInfo = {
  id: 'record-1',
  account_id: 'account-1',
  status: 'failed',
  message: 'Cookie 过期',
  error_code: 'cookie_expired',
  checked_in_at: '2026-06-10T08:00:00.000Z',
} as CheckinRecordInfo

const mountComponent = async (component: unknown, props: Record<string, unknown>) => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(
    defineComponent({
      setup() {
        return () => h(component as never, props as never)
      },
    })
  )

  app.use(createI18nStub('zh-CN'))
  app.mount(el)
  await nextTick()
  // 等待 onMounted 异步加载（失败历史 / cookies 回填）
  await Promise.resolve()
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

describe('checkin cookie_expired quick fix smoke', () => {
  it('renders update-cookie entry on cookie_expired record rows and emits account id on click', async () => {
    mockedListCheckinRecords.mockResolvedValue({ records: [], total: 0 })
    const onUpdateCookie = vi.fn()

    const { el, unmount } = await mountComponent(CheckinRecordsTab, {
      records: [cookieExpiredRecord],
      providers,
      accounts,
      todayStats: null,
      onUpdateCookie,
    })

    try {
      const fixButton = el.querySelector<HTMLButtonElement>('.checkin-records__fix-button')
      expect(fixButton).not.toBeNull()
      expect(fixButton?.textContent).toContain('更新 Cookie')

      fixButton!.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await nextTick()

      expect(onUpdateCookie).toHaveBeenCalledWith('account-1')
    } finally {
      unmount()
    }
  })

  it('does not render update-cookie entry for non cookie_expired failures', async () => {
    mockedListCheckinRecords.mockResolvedValue({ records: [], total: 0 })

    const { el, unmount } = await mountComponent(CheckinRecordsTab, {
      records: [{ ...cookieExpiredRecord, error_code: 'waf_blocked' }],
      providers,
      accounts,
      todayStats: null,
    })

    try {
      expect(el.querySelector('.checkin-records__fix-button')).toBeNull()
    } finally {
      unmount()
    }
  })

  it('opens the account editor with focused cookies field when pendingEditAccountId is set', async () => {
    mockedGetCheckinAccountCookies.mockResolvedValue({
      cookies_json: '{"session":"expired-session"}',
      api_user: '12345',
    })
    const onPendingEditConsumed = vi.fn()

    const { unmount } = await mountComponent(CheckinAccountsTab, {
      providers,
      accounts,
      builtinProviders: [],
      checkinLoading: false,
      pendingEditAccountId: 'account-1',
      onPendingEditConsumed,
    })

    try {
      // 等待编辑弹窗打开（cookies 回填 + 聚焦是异步链路）
      await vi.waitFor(() => {
        expect(document.body.querySelector('.mock-base-modal')).not.toBeNull()
      })

      expect(onPendingEditConsumed).toHaveBeenCalled()
      expect(mockedGetCheckinAccountCookies).toHaveBeenCalledWith('account-1')

      const textarea = document.body.querySelector<HTMLTextAreaElement>(
        '.checkin-accounts-tab__control--credential'
      )
      expect(textarea).not.toBeNull()
      expect(textarea?.value).toBe('expired-session')
      await vi.waitFor(() => {
        expect(document.activeElement).toBe(textarea)
      })
    } finally {
      unmount()
    }
  })
})
