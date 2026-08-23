import { fireEvent, render, waitFor } from '@testing-library/react'
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest'
import type { AccountInfo, CheckinProvider } from '@/types/checkin'
import * as checkinApi from '@/api'
import { CheckinAccountsTab } from '@/features/checkin/tabs/CheckinAccountsTab'
import { setLocale } from '@/i18n'

vi.mock('@/api', () => ({
  createCheckinAccount: vi.fn(),
  updateCheckinAccount: vi.fn(),
  deleteCheckinAccount: vi.fn(),
  getCheckinAccountCookies: vi.fn(),
}))

vi.mock('@/configs/surfaceNotify', () => ({
  surfaceNotify: {
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
    confirm: vi.fn().mockResolvedValue(true),
  },
}))

const mockedGetCheckinAccountCookies = checkinApi.getCheckinAccountCookies as ReturnType<typeof vi.fn>
const mockedUpdateCheckinAccount = checkinApi.updateCheckinAccount as ReturnType<typeof vi.fn>

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
    enabled: true,
    created_at: '2026-03-26T00:00:00Z',
  },
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
    cookies_masked: '***',
    created_at: '2026-03-26T00:00:00Z',
  },
]

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
        ctrlKey: params.ctrlKey ?? false,
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

const mountTab = async (locale: 'en-US' | 'zh-CN' = 'en-US', accountList: AccountInfo[] = accounts) => {
  await setLocale(locale)
  return render(
    <CheckinAccountsTab
      providers={providers}
      accounts={accountList}
      builtinProviders={[]}
      checkinLoading={false}
    />,
  )
}

const openAccountEditor = async (editLabel = 'Edit') => {
  const trigger = document.body.querySelector<HTMLButtonElement>('.checkin-accounts-tab__menu-trigger')
  expect(trigger).not.toBeNull()
  fireEvent.pointerDown(trigger!, { button: 0, ctrlKey: false })
  const editButton = await waitFor(() => {
    const found = Array.from(
      document.body.querySelectorAll<HTMLElement>('.checkin-accounts-tab__menu-item'),
    ).find((button) =>
      Boolean(button.textContent && /Edit|编辑|checkin\.accounts\.edit/.test(button.textContent)),
    )
    expect(found).toBeTruthy()
    return found!
  })
  fireEvent.click(editButton)
  await waitFor(() => {
    expect(document.body.querySelector('#checkin-account-form')).not.toBeNull()
  })
}

afterEach(() => {
  vi.clearAllMocks()
})

describe('CheckinAccountsTab smoke', () => {
  it('renders account actions in English with no-wrap single account button', async () => {
    const { container } = await mountTab('en-US')
    expect(container.textContent).toContain('Account Management')
    expect(container.textContent).toContain('OAuth Login')
    expect(container.textContent).toContain('Check in')
    expect(container.textContent).not.toContain('签到账号')
    const miniLabel = container.querySelector('.checkin-accounts-tab__mini-button-label')
    expect(miniLabel?.textContent).toBe('Check in')
  })

  it('renders account actions in Chinese', async () => {
    const { container } = await mountTab('zh-CN')
    expect(container.textContent).toContain('签到账号')
    expect(container.textContent).toContain('OAuth 登录')
    expect(container.textContent).toContain('签到')
  })

  it('teleports the account action menu to body to avoid table clipping', async () => {
    const { container } = await mountTab('zh-CN')
    const trigger = container.querySelector<HTMLButtonElement>('.checkin-accounts-tab__menu-trigger')
    expect(trigger).not.toBeNull()
    fireEvent.pointerDown(trigger!, { button: 0, ctrlKey: false })
    const teleportedMenu = await waitFor(() => {
      const menu = document.body.querySelector<HTMLElement>('.checkin-accounts-tab__menu--floating')
      expect(menu).not.toBeNull()
      return menu!
    })
    expect(teleportedMenu.textContent).toMatch(/checkin\.actions\.refreshBalance|刷新余额/)
    expect(teleportedMenu.textContent).toMatch(/checkin\.accounts\.edit|编辑/)
    expect(teleportedMenu.textContent).toMatch(/checkin\.accounts\.delete|删除/)
    expect(container.querySelector('.checkin-accounts-tab__menu--floating')).toBeNull()
  })

  it('backfills api user when opening editor for an existing account', async () => {
    mockedGetCheckinAccountCookies.mockResolvedValue({
      cookies_json: '{"session":"abc123"}',
      api_user: '67890',
    })
    await mountTab()
    await openAccountEditor()
    const apiUserInput = document.body.querySelector<HTMLInputElement>('input[placeholder="12345"]')
    const cookiesTextarea = document.body.querySelector<HTMLTextAreaElement>(
      '.checkin-accounts-tab__control--textarea.checkin-accounts-tab__control--mono',
    )
    expect(apiUserInput?.value).toBe('67890')
    expect(cookiesTextarea?.value).toBe('abc123')
  })

  it('opens the editor with a plain API User hint that avoids i18n-t vnode interpolation', async () => {
    mockedGetCheckinAccountCookies.mockResolvedValue({
      cookies_json: '{"session":"abc123"}',
      api_user: '67890',
    })
    await mountTab()
    await openAccountEditor()
    expect(document.body.textContent).toContain('Required for session / cookies login. Prefer')
    expect(document.body.textContent).toContain('from Local Storage, or find')
    expect(document.body.textContent).toContain('in request headers.')
    const hintCodes = Array.from(document.body.querySelectorAll<HTMLElement>('code')).map(
      (code) => code.textContent,
    )
    expect(hintCodes).toContain('user.id')
    expect(hintCodes).toContain('new-api-user')
  })

  it('submits enabled true when enabling an existing account from the editor', async () => {
    mockedGetCheckinAccountCookies.mockResolvedValue({
      cookies_json: '{"session":"abc123"}',
      api_user: '67890',
    })
    await mountTab('en-US', [{ ...accounts[0], enabled: false }])
    await openAccountEditor()
    const enabledCheckbox = document.body.querySelector<HTMLInputElement>('#account-enabled')
    expect(enabledCheckbox?.checked).toBe(false)
    fireEvent.click(enabledCheckbox!)
    const form = document.body.querySelector<HTMLFormElement>('#checkin-account-form')
    fireEvent.submit(form!)
    await waitFor(() => {
      expect(mockedUpdateCheckinAccount).toHaveBeenCalledWith(
        'account-1',
        expect.objectContaining({
          enabled: true,
          api_user: '67890',
        }),
      )
    })
  })

  it('keeps the account editor as a readable credential workbench with form-backed footer actions', async () => {
    mockedGetCheckinAccountCookies.mockResolvedValue({
      cookies_json: '{"session":"abc123"}',
      api_user: '67890',
    })
    await mountTab()
    await openAccountEditor()
    expect(document.body.querySelector('.checkin-accounts-tab__modal-body')).not.toBeNull()
    expect(document.body.querySelector('.checkin-accounts-tab__form-section--identity')).not.toBeNull()
    expect(document.body.querySelector('.checkin-accounts-tab__form-section--credentials')).not.toBeNull()
    const credentialTextarea = document.body.querySelector<HTMLTextAreaElement>(
      '.checkin-accounts-tab__control--credential',
    )
    expect(credentialTextarea?.getAttribute('rows')).toBe('7')
    const submitButton = document.body.querySelector<HTMLButtonElement>('button[type="submit"][form="checkin-account-form"]')
    expect(submitButton).not.toBeNull()
  })

  it('preserves full cookies JSON when opening editor for an existing account', async () => {
    const fullCookiesJson = '{"session":"abc123","cf_clearance":"token-1"}'
    mockedGetCheckinAccountCookies.mockResolvedValue({
      cookies_json: fullCookiesJson,
      api_user: '67890',
    })
    await mountTab()
    await openAccountEditor()
    const cookiesTextarea = document.body.querySelector<HTMLTextAreaElement>(
      '.checkin-accounts-tab__control--textarea.checkin-accounts-tab__control--mono',
    )
    expect(cookiesTextarea?.value).toBe(fullCookiesJson)
  })
})
