import { describe, expect, it } from 'vitest'
import type { CodexAccountQuota, CodexAuthAccountItem } from '@/types'
import {
  canCustomizeAccountName,
  canSubmitAccountRename,
  detectImportPayloadNamingState,
  filterAndSortCodexAccounts,
  getAccountNameValidationMessage,
  getLoginStateIcon,
  getLoginStateIconClass,
  getLoginStateTone,
  normalizeAccountNameInput,
  resolveAccountPlanType,
  usesOpenAiAuthMode,
} from '@/features/codex/codexAuthAccounts'

const account = (
  overrides: Partial<CodexAuthAccountItem> & { name: string }
): CodexAuthAccountItem => ({
  is_current: false,
  is_virtual: false,
  ...overrides,
})

const quota = (planType?: string, error?: string): CodexAccountQuota => ({
  account_name: 'unused',
  fetched_at: '2026-05-19T00:00:00Z',
  quota: planType ? { hourly_percentage: 0, weekly_percentage: 0, plan_type: planType } : undefined,
  error,
})

describe('codex auth account helpers', () => {
  it('normalizes and validates user-provided account names', () => {
    expect(normalizeAccountNameInput('  editorial_api  ')).toBe('editorial_api')
    expect(normalizeAccountNameInput('   ')).toBeNull()
    expect(getAccountNameValidationMessage('default')).toBe('reserved')
    expect(getAccountNameValidationMessage('a'.repeat(33))).toBe('length')
    expect(getAccountNameValidationMessage('bad name')).toBe('charset')
    expect(getAccountNameValidationMessage('good_name-1')).toBeNull()
  })

  it('detects token payload naming states without coupling to the Vue component', () => {
    expect(detectImportPayloadNamingState('')).toBe('empty')
    expect(detectImportPayloadNamingState('{')).toBe('invalid')
    expect(detectImportPayloadNamingState(JSON.stringify({ OPENAI_API_KEY: 'sk-test' }))).toBe(
      'single'
    )
    expect(detectImportPayloadNamingState(JSON.stringify([{ id: 1 }]))).toBe('single')
    expect(detectImportPayloadNamingState(JSON.stringify([{ id: 1 }, { id: 2 }]))).toBe('multiple')
    expect(detectImportPayloadNamingState(JSON.stringify({ accounts: { alpha: {} } }))).toBe(
      'bundle'
    )
    expect(canCustomizeAccountName('token', 'bundle')).toBe(false)
    expect(canCustomizeAccountName('token', 'single')).toBe(true)
    expect(canCustomizeAccountName('api', 'bundle')).toBe(true)
  })

  it('filters by search/status/plan and preserves sort semantics', () => {
    const accounts = [
      account({
        name: 'alpha-pro',
        email: 'alpha@example.com',
        is_current: true,
        saved_at: '2026-05-01T00:00:00Z',
        last_used: '2026-05-02T00:00:00Z',
      }),
      account({
        name: 'beta-plus',
        description: 'backup',
        saved_at: '2026-05-03T00:00:00Z',
        last_used: '2026-05-04T00:00:00Z',
      }),
      account({ name: 'runtime', is_virtual: true }),
      account({ name: 'needs-attention', saved_at: '2026-05-05T00:00:00Z' }),
    ]
    const quotas = new Map<string, CodexAccountQuota>([
      ['alpha-pro', quota('Pro')],
      ['beta-plus', quota('Plus')],
      ['needs-attention', quota(undefined, 'quota unavailable')],
    ])

    expect(resolveAccountPlanType(accounts[0], quotas)).toBe('pro')
    expect(
      filterAndSortCodexAccounts({
        accounts,
        quotaMap: quotas,
        searchQuery: 'backup',
        statusFilter: 'all',
        planFilter: 'plus',
        sortBy: 'saved_desc',
      }).map((item) => item.name)
    ).toEqual(['beta-plus'])
    expect(
      filterAndSortCodexAccounts({
        accounts,
        quotaMap: quotas,
        searchQuery: '',
        statusFilter: 'attention',
        planFilter: 'all',
        sortBy: 'saved_desc',
      }).map((item) => item.name)
    ).toEqual(['needs-attention'])
    expect(
      filterAndSortCodexAccounts({
        accounts,
        quotaMap: quotas,
        searchQuery: '',
        statusFilter: 'all',
        planFilter: 'all',
        sortBy: 'used_desc',
      }).map((item) => item.name)
    ).toEqual(['beta-plus', 'alpha-pro', 'runtime', 'needs-attention'])
  })

  it('keeps OpenAI auth-mode support explicit', () => {
    expect(usesOpenAiAuthMode('openai_chatgpt')).toBe(true)
    expect(usesOpenAiAuthMode('openai_api_key')).toBe(true)
    expect(usesOpenAiAuthMode('no_auth')).toBe(false)
    expect(usesOpenAiAuthMode(null)).toBe(false)
  })

  it('keeps login state presentation and rename eligibility outside the Vue component', () => {
    expect(getLoginStateTone({ type: 'LoggedInSaved', account_name: 'alpha' })).toBe('success')
    expect(getLoginStateTone({ type: 'LoggedInUnsaved' })).toBe('warning')
    expect(getLoginStateTone({ type: 'ProviderKeyActive', env_key: 'OPENAI_API_KEY' })).toBe(
      'primary'
    )
    expect(getLoginStateTone({ type: 'NotLoggedIn' })).toBe('danger')
    expect(getLoginStateIcon({ type: 'Unknown', raw_type: 'mystery', raw: {} })).toBe(
      'AlertTriangle'
    )
    expect(getLoginStateIconClass({ type: 'ApiKeyActive' })).toContain('text-accent-primary')

    expect(canSubmitAccountRename('old', ' new_name ')).toBe(true)
    expect(canSubmitAccountRename('same', 'same')).toBe(false)
    expect(canSubmitAccountRename('old', 'bad name')).toBe(false)
    expect(canSubmitAccountRename('old', 'a'.repeat(33))).toBe(false)
  })
})
