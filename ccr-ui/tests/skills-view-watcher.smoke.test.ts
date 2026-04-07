import { describe, expect, it, vi } from 'vitest'
import { handleSkillsChangedPayload } from '@/views/skills/skillsWatcher'

describe('SkillsView watcher smoke', () => {
  it('reloads onboarding candidates on inventory-only watcher events', async () => {
    const loadOnboardingCandidates = vi.fn(async () => [])
    const refresh = vi.fn(async () => undefined)

    await handleSkillsChangedPayload(
      {
        affectsInventory: true,
        affectsSources: false,
        affectsMarketplace: false,
      },
      {
        currentTab: 'inventory',
        loadOnboardingCandidates,
        refresh,
      }
    )

    expect(loadOnboardingCandidates).toHaveBeenCalledTimes(1)
    expect(loadOnboardingCandidates).toHaveBeenCalledWith(true)
    expect(refresh).toHaveBeenCalledTimes(1)
    expect(refresh).toHaveBeenCalledWith(false)
  })

  it('still refreshes marketplace data when the marketplace tab is active', async () => {
    const loadOnboardingCandidates = vi.fn(async () => [])
    const refresh = vi.fn(async () => undefined)

    await handleSkillsChangedPayload(
      {
        affectsInventory: false,
        affectsSources: false,
        affectsMarketplace: false,
      },
      {
        currentTab: 'marketplace',
        loadOnboardingCandidates,
        refresh,
      }
    )

    expect(loadOnboardingCandidates).not.toHaveBeenCalled()
    expect(refresh).toHaveBeenCalledWith(true)
  })
})
