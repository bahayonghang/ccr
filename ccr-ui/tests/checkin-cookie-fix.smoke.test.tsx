import { fireEvent, render, waitFor } from '@testing-library/react'
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest'
import type { AccountInfo, CheckinProvider, CheckinRecordInfo } from '@/types/checkin'
import * as checkinApi from '@/api'
import { CheckinAccountsTab } from '@/features/checkin/tabs/CheckinAccountsTab'
import { CheckinRecordsTab } from '@/features/checkin/tabs/CheckinRecordsTab'

vi.mock('@/api', () => ({
  createCheckinAccount: vi.fn(),
  updateCheckinAccount: vi.fn(),
  deleteCheckinAccount: vi.fn(),
  getCheckinAccountCookies: vi.fn(),
  listCheckinRecords: vi.fn(),
  exportCheckinRecords: vi.fn(),
}))

vi.mock('@/configs/surfaceNotify', () => ({
  surfaceNotify: {
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
    confirm: vi.fn().mockResolvedValue(true),
  },
}))

const mockedListCheckinRecords = checkinApi.listCheckinRecords as ReturnType<typeof vi.fn>
const mockedGetCheckinAccountCookies = checkinApi.getCheckinAccountCookies as ReturnType<typeof vi.fn>

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
    enabled: true,
    created_at: '2026-06-10T00:00:00Z',
  },
]

const accounts: AccountInfo[] = [
  {
    id: 'account-1',
    provider_id: 'provider-1',
    provider_name: 'AnyRouter',
    name: 'anyrouter_stumail',
    enabled: true,
    api_user: '12345',
    cookies_masked: '***',
    created_at: '2026-06-10T00:00:00Z',
  },
]

const cookieExpiredRecord: CheckinRecordInfo = {
  id: 'record-1',
  account_id: 'account-1',
  status: 'failed',
  message: 'Cookie 过期',
  error_code: 'cookie_expired',
  checked_in_at: '2026-06-10T08:00:00.000Z',
}

beforeAll(() => {
  class ResizeObserverStub {
    observe(): void {}
    unobserve(): void {}
    disconnect(): void {}
  }
  if (typeof globalThis.ResizeObserver === 'undefined') {
    globalThis.ResizeObserver = ResizeObserverStub as unknown as typeof ResizeObserver
  }
  const mouseEventCtor = (globalThis.MouseEvent ?? Event) as unknown as typeof MouseEvent
  class PointerEventStub extends mouseEventCtor {
    readonly pointerId: number
    readonly pointerType: string
    readonly isPrimary: boolean
    constructor(type: string, params: PointerEventInit = {}) {
      super(type, {
        bubbles: params.bubbles,
        cancelable: params.cancelable,
        button: params.button ?? 0,
      })
      this.pointerId = params.pointerId ?? 0
      this.pointerType = params.pointerType ?? 'mouse'
      this.isPrimary = params.isPrimary ?? true
    }
  }
  if (typeof globalThis.PointerEvent === 'undefined') {
    const stub = PointerEventStub as unknown as typeof PointerEvent
    globalThis.PointerEvent = stub
    window.PointerEvent = stub
  }
})

afterEach(() => {
  vi.clearAllMocks()
})

describe('checkin cookie_expired quick fix smoke', () => {
  it('renders update-cookie entry on cookie_expired record rows and emits account id on click', async () => {
    localStorage.setItem('ccr-ui-locale', 'zh-CN')
    mockedListCheckinRecords.mockResolvedValue({ records: [], total: 0 })
    const onUpdateCookie = vi.fn()
    const { container } = render(
      <CheckinRecordsTab
        records={[cookieExpiredRecord]}
        recordsLoadError={null}
        providers={providers}
        accounts={accounts}
        todayStats={null}
        onUpdateCookie={onUpdateCookie}
      />,
    )
    const fixButton = container.querySelector<HTMLButtonElement>('.checkin-records__fix-button')
    expect(fixButton?.textContent).toContain('更新 Cookie')
    fireEvent.click(fixButton!)
    expect(onUpdateCookie).toHaveBeenCalledWith('account-1')
  })

  it('does not render update-cookie entry for non cookie_expired failures', () => {
    localStorage.setItem('ccr-ui-locale', 'zh-CN')
    mockedListCheckinRecords.mockResolvedValue({ records: [], total: 0 })
    const { container } = render(
      <CheckinRecordsTab
        records={[{ ...cookieExpiredRecord, error_code: 'waf_blocked' }]}
        recordsLoadError={null}
        providers={providers}
        accounts={accounts}
        todayStats={null}
      />,
    )
    expect(container.querySelector('.checkin-records__fix-button')).toBeNull()
  })

  it('shows a records load error instead of an empty state when records fail to load', () => {
    localStorage.setItem('ccr-ui-locale', 'zh-CN')
    mockedListCheckinRecords.mockResolvedValue({ records: [], total: 0 })
    const { container } = render(
      <CheckinRecordsTab
        records={[]}
        recordsLoadError="加载签到记录失败"
        providers={providers}
        accounts={accounts}
        todayStats={null}
      />,
    )
    expect(container.querySelector('.checkin-records__error')).not.toBeNull()
    expect(container.querySelector('.checkin-records__empty')).toBeNull()
    expect(container.textContent).toContain('加载签到记录失败')
  })

  it('opens the account editor with focused cookies field when pendingEditAccountId is set', async () => {
    localStorage.setItem('ccr-ui-locale', 'zh-CN')
    mockedGetCheckinAccountCookies.mockResolvedValue({
      cookies_json: '{"session":"expired-session"}',
      api_user: '12345',
    })
    const onPendingEditConsumed = vi.fn()
    render(
      <CheckinAccountsTab
        providers={providers}
        accounts={accounts}
        builtinProviders={[]}
        checkinLoading={false}
        pendingEditAccountId="account-1"
        onPendingEditConsumed={onPendingEditConsumed}
      />,
    )
    await waitFor(() => {
      expect(document.body.querySelector('#checkin-account-form')).not.toBeNull()
    })
    expect(onPendingEditConsumed).toHaveBeenCalled()
    expect(mockedGetCheckinAccountCookies).toHaveBeenCalledWith('account-1')
    const textarea = document.body.querySelector<HTMLTextAreaElement>(
      '.checkin-accounts-tab__control--credential',
    )
    expect(textarea?.value).toBe('expired-session')
    await waitFor(() => {
      expect(document.activeElement).toBe(textarea)
    })
  })
})
