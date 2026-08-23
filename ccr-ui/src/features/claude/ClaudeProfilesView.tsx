import { claudeProfilesConfig } from '@/configs/profiles'
import { BaseProfiles } from '@/features/platform'

export function ClaudeProfilesView() {
  return <BaseProfiles config={claudeProfilesConfig} />
}
