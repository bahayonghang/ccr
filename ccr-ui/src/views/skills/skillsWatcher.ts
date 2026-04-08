export interface SkillsChangedPayload {
  affectsInventory?: boolean
  affectsSources?: boolean
  affectsMarketplace?: boolean
}

interface SkillsChangedHandlerDeps {
  currentTab: string
  loadOnboardingCandidates: (force?: boolean) => Promise<unknown>
  refresh: (includeMarketplace?: boolean) => Promise<unknown>
}

export async function handleSkillsChangedPayload(
  payload: SkillsChangedPayload | undefined,
  { currentTab, loadOnboardingCandidates, refresh }: SkillsChangedHandlerDeps
) {
  if (payload?.affectsInventory) {
    await loadOnboardingCandidates(true)
  }

  await refresh(Boolean(payload?.affectsMarketplace || currentTab === 'marketplace' || currentTab === 'explore'))
}
