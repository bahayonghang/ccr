import { grokProfilesConfig } from '@/configs/profiles'
import { BaseProfiles } from '@/features/platform'

export function GrokProfilesView() {
  return <BaseProfiles config={grokProfilesConfig} />
}
