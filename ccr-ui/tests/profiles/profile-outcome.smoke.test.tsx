import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, renderHook, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { ProfileEditorAdapter } from '@/configs/profileEditorAdapter'
import { useClaudeProfilesPage } from '@/features/claude/profiles/useClaudeProfilesPage'
import { useCodexProfilesPage } from '@/features/codex/profiles/useCodexProfilesPage'
import { useGrokProfilesPage } from '@/features/grok/profiles/useGrokProfilesPage'
import { useProfileEditor } from '@/features/platform/profiles/useProfileEditor'
import type { ProfileOutcome } from '@/types/generated/profiles/ProfileOutcome'
import { profileOutcome, profileOutcomeWarning, requireProfileOutcome } from '@/utils/profileOutcome'
import { claudeProfileFixtures, codexProfileFixtures, grokProfileFixtures } from '../fixtures/profiles'

const api = vi.hoisted(() => ({
  listClaude: vi.fn(), listCodex: vi.fn(), listGrok: vi.fn(),
  applyClaude: vi.fn(), applyCodex: vi.fn(), applyGrok: vi.fn(),
  environment: vi.fn(async () => ({ env_type: 'local', id: 'local' })),
}))
const notify = vi.hoisted(() => ({
  success: vi.fn(), warning: vi.fn(), error: vi.fn(), confirm: vi.fn(async () => true),
}))

vi.mock('@/configs/surfaceNotify', () => ({ surfaceNotify: notify }))
vi.mock('@/api/runtime/environment', () => ({ getCurrentEnvironment: api.environment }))
vi.mock('@/api', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/api')>()
  return {
    ...actual,
    getCurrentEnvironment: api.environment,
    listClaudeProfiles: api.listClaude, listCodexProfiles: api.listCodex,
    applyClaudeProfile: api.applyClaude, applyCodexProfile: api.applyCodex,
    grokApi: { ...actual.grokApi, listGrokProfiles: api.listGrok, applyGrokProfile: api.applyGrok },
  }
})

const outcome = (overrides: Partial<ProfileOutcome> = {}): ProfileOutcome => ({
  operation_id: 'operation-fixture', platform: 'claude', profile: 'target', previous_profile: 'old',
  status: 'applied_with_warning', activation_committed: true, warnings: ['history_failed'], recovery_paths: [],
  ...overrides,
})

const wrapper = () => {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  )
}

describe('profile operation outcomes', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    api.listClaude.mockResolvedValue({ profiles: claudeProfileFixtures, current_profile: 'old', can_off: true })
    api.listCodex.mockResolvedValue({ profiles: codexProfileFixtures, current_profile: 'old', can_off: true })
    api.listGrok.mockResolvedValue({ status: 'ok', profiles: grokProfileFixtures, current_profile: 'old', activation: 'active' })
  })

  it('retains legacy payload compatibility and rejects malformed or failed outcomes', () => {
    expect(profileOutcome({ success: true })).toBeNull()
    expect(profileOutcome({ outcome: { ...outcome(), future_field: 1 } })?.activation_committed).toBe(true)
    expect(() => requireProfileOutcome({ outcome: { status: 'unknown' } })).toThrow()
    expect(() => requireProfileOutcome({ outcome: outcome({ status: 'unchanged', activation_committed: false }) })).toThrow()
    expect(() => requireProfileOutcome({ outcome: outcome({ status: 'recovery_required', activation_committed: false }) })).toThrow()
    expect(profileOutcomeWarning({ outcome: outcome({ activation_committed: false }) })).toBeTruthy()
  })

  const cases = [
    { name: 'Claude', hook: useClaudeProfilesPage, apply: api.applyClaude, list: api.listClaude, target: claudeProfileFixtures[0].name },
    { name: 'Codex', hook: useCodexProfilesPage, apply: api.applyCodex, list: api.listCodex, target: codexProfileFixtures[0].name },
    { name: 'Grok', hook: useGrokProfilesPage, apply: api.applyGrok, list: api.listGrok, target: grokProfileFixtures[0].name },
  ]

  for (const item of cases) {
    it(item.name + ' shows a committed warning once and refreshes without reactivation', async () => {
      item.apply.mockResolvedValue({ status: 'applied', outcome: outcome() })
      const { result } = renderHook<{ loading: boolean; onApply: (name: string) => Promise<void> }, void>(item.hook, { wrapper: wrapper() })
      await waitFor(() => expect(result.current.loading).toBe(false))
      await act(async () => result.current.onApply(item.target))
      expect(item.apply).toHaveBeenCalledTimes(1)
      expect(notify.warning).toHaveBeenCalledTimes(1)
      expect(notify.success).not.toHaveBeenCalled()
      expect(notify.error).not.toHaveBeenCalled()
      expect(item.list).toHaveBeenCalledTimes(2)
    })

    it(item.name + ' shows recovery as an error and refreshes without success or retry', async () => {
      item.apply.mockResolvedValue({ status: 'applied', outcome: outcome({ status: 'recovery_required', activation_committed: false }) })
      const { result } = renderHook<{ loading: boolean; onApply: (name: string) => Promise<void> }, void>(item.hook, { wrapper: wrapper() })
      await waitFor(() => expect(result.current.loading).toBe(false))
      await act(async () => result.current.onApply(item.target))
      expect(item.apply).toHaveBeenCalledTimes(1)
      expect(notify.error).toHaveBeenCalledTimes(1)
      expect(notify.success).not.toHaveBeenCalled()
      expect(item.list).toHaveBeenCalledTimes(2)
    })
  }

  it('save-and-apply does not activate a rename that the backend already committed', async () => {
    const submit = vi.fn(async () => ({ status: 'ok' as const, appliedName: 'renamed', activationCommitted: true, warning: 'audit pending' }))
    const adapter: ProfileEditorAdapter<{ name: string }, { name: string }> = {
      createEmpty: () => ({ name: 'old' }), fromRecord: (record) => record, sections: [], validate: () => [], submit,
    }
    const onApply = vi.fn()
    const onDone = vi.fn()
    const { result } = renderHook(() => useProfileEditor({
      adapter, target: null, originalName: 'old', existingNames: ['old'], hasExistingBaseUrl: true, onApply, onDone,
    }))
    await act(async () => result.current.submit(true))
    expect(submit).toHaveBeenCalledTimes(1)
    expect(onApply).not.toHaveBeenCalled()
    expect(onDone).toHaveBeenCalledWith(expect.objectContaining({ warning: 'audit pending' }), true)
  })
})
