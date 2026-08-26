import { render } from '@testing-library/react'
import { useForm } from 'react-hook-form'
import { describe, expect, it, vi } from 'vitest'
import { ProfileCardGrid } from '@/components/profiles'
import { claudeProfilePresentation } from '@/configs/profilePresentation'
import { AccountDashboardCalendar } from '@/features/checkin/components/AccountDashboardCalendar'
import { AccountDashboardTrend } from '@/features/checkin/components/AccountDashboardTrend'
import { CheckinResultPanel } from '@/features/checkin/components/CheckinResultPanel'
import { CheckinImportExportTab } from '@/features/checkin/tabs/CheckinImportExportTab'
import { CheckinProvidersTab } from '@/features/checkin/tabs/CheckinProvidersTab'
import { CommandList } from '@/features/commands/CommandList'
import { Titlebar } from '@/shell/Titlebar'
import { OAuthWizardBody } from '@/features/checkin/components/OAuthWizardBody'
import { OAuthWizardModal } from '@/features/checkin/components/OAuthWizardModal'
import {
  initialOAuthWizardState,
  oauthWizardReducer,
  type OAuthType,
  type OAuthWizardState,
} from '@/features/checkin/lib/oauthWizardReducer'
import { extractApiUserFromCredentials, parseCookies } from '@/features/checkin/lib/parseCredentials'
import type { BuiltinProvider } from '@/types/checkin'
import { claudeDisplayRecords } from '../fixtures/profiles'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async () => ({})),
}))

const provider: BuiltinProvider = {
  id: 'p1',
  name: 'Prov',
  description: 'd',
  domain: 'example.com',
  base_url: 'https://example.com',
  balance_path: '/b',
  user_info_path: '/u',
  auth_header: 'Authorization',
  auth_prefix: 'Bearer',
  supports_checkin: true,
  requires_waf_bypass: false,
  requires_cf_clearance: false,
  checkin_bugged: false,
  icon: 'i',
  category: 'c',
}

function WizardHarness({ state }: { state: OAuthWizardState }) {
  const form = useForm({
    defaultValues: {
      provider_id: 'p1',
      oauth_type: 'linuxdo' as OAuthType,
      credentials: 'a=b',
      api_user: 'u',
      account_name: 'n',
    },
  })
  return (
    <OAuthWizardBody
      state={state}
      form={form}
      oauthProviders={[provider]}
      selectedProvider={provider}
      defaultAccountName="n"
      parsedCookieCount={1}
      t={(key) => key}
      onSelectProvider={() => undefined}
      onSelectOAuthType={() => undefined}
      onCopyUrl={() => undefined}
      onBackToSelection={() => undefined}
    />
  )
}

describe('check-in oauth wizard and import/export', () => {
  it('reduces wizard state and renders each step', () => {
    let state = initialOAuthWizardState()
    state = oauthWizardReducer(state, { type: 'SELECT_PROVIDER', id: 'p1', oauthType: 'github' })
    state = oauthWizardReducer(state, { type: 'SELECT_OAUTH_TYPE', oauthType: 'linuxdo' })
    state = oauthWizardReducer(state, { type: 'FETCH_URL_START' })
    state = oauthWizardReducer(state, { type: 'FETCH_URL_SUCCESS', url: 'https://auth.example', guide: ['g'] })
    state = oauthWizardReducer(state, { type: 'COPIED' })
    state = oauthWizardReducer(state, { type: 'CLEAR_COPIED' })
    state = oauthWizardReducer(state, { type: 'GOTO_CREDENTIALS' })
    state = oauthWizardReducer(state, { type: 'PARSE_ERROR', message: 'bad' })
    state = oauthWizardReducer(state, { type: 'CLEAR_PARSE_ERROR' })
    state = oauthWizardReducer(state, { type: 'GOTO_CONFIRM' })
    state = oauthWizardReducer(state, { type: 'CREATE_START' })
    state = oauthWizardReducer(state, { type: 'CREATE_SUCCESS' })
    state = oauthWizardReducer(state, { type: 'CREATE_ERROR', message: 'fail' })
    state = oauthWizardReducer(state, { type: 'BACK' })
    state = oauthWizardReducer(state, { type: 'FETCH_URL_ERROR', message: 'err' })
    state = oauthWizardReducer(state, { type: 'RESET' })
    expect(state.step).toBe(0)

    for (const step of [0, 1, 2, 3] as const) {
      render(
        <WizardHarness
          state={{
            ...initialOAuthWizardState(),
            step,
            authorizeUrl: 'https://auth.example',
            extractionGuide: ['copy'],
            oauthError: step === 1 ? 'err' : '',
            parseError: step === 2 ? 'parse' : '',
            createError: step === 3 ? 'create' : '',
            creatingAccount: false,
            createSuccess: step === 3,
          }}
        />,
      )
    }
    render(
      <OAuthWizardModal isOpen builtinProviders={[provider]} onClose={() => undefined} onSuccess={() => undefined} />,
    )
    render(<CheckinImportExportTab onRefresh={() => undefined} />)
    expect(parseCookies('sid=abc; token=xyz').sid).toBe('abc')
    expect(extractApiUserFromCredentials('{"api_user":"u1"}')).toBe('u1')

    render(<AccountDashboardCalendar calendar={null} />)
    render(
      <AccountDashboardCalendar
        calendar={{
          account_id: 'a1',
          year: 2026,
          month: 1,
          days: [
            { date: '2026-01-01', is_checked_in: true, reward_amount: 1, income_increment: 0 },
            { date: '2026-01-02', is_checked_in: false, reward_amount: 0, income_increment: 0 },
          ],
          month_stats: { check_in_rate: 50 },
        } as never}
      />,
    )
    render(
      <AccountDashboardTrend
        trend={{
          data_points: [
            { date: '2026-01-01', current_balance: 1, reward_amount: 1, is_checked_in: true },
            { date: '2026-01-02', current_balance: 2, reward_amount: 0, is_checked_in: false },
          ],
        } as never}
      />,
    )
    const item = {
      account_id: 'a1',
      account_name: 'acc',
      provider_name: 'prov',
      status: 'success',
    }
    render(
      <CheckinResultPanel
        result={{ results: [item], summary: { success: 1, failed: 1, skipped: 1, already_checked_in: 1 } } as never}
        phase="finished"
        resultRef={() => undefined}
        wafRunning={false}
        wafMessage={null}
        wafProviderName={null}
        successItems={[item as never]}
        failedItems={[{ ...item, status: 'failed', account_id: 'a2' } as never]}
        skippedItems={[{ ...item, status: 'skipped', account_id: 'a3' } as never]}
        alreadyItems={[{ ...item, status: 'already_checked_in', account_id: 'a4' } as never]}
        t={(key) => key}
        getSuccessDetail={() => 'ok'}
        getFailedDetail={() => 'fail'}
        getSkippedDetail={() => 'skip'}
        getAlreadyDetail={() => 'already'}
        getErrorLabel={() => null}
        onOpenProviders={() => undefined}
        onFixCookie={() => undefined}
        onClose={() => undefined}
      />,
    )
    render(<CheckinProvidersTab providers={[]} builtinProviders={[provider]} />)
    render(
      <ProfileCardGrid
        records={claudeDisplayRecords.slice(0, 1)}
        presentation={claudeProfilePresentation}
        inspectorOpen={false}
        onSelect={() => undefined}
        onEdit={() => undefined}
        onApply={() => undefined}
      />,
    )
    render(
      <CommandList
        loading={false}
        commands={[{ name: 'c1', enabled: true, description: 'd' } as never]}
        onEdit={() => undefined}
        onDelete={() => undefined}
        onToggle={() => undefined}
      />,
    )
    render(<Titlebar />)
  })
})
