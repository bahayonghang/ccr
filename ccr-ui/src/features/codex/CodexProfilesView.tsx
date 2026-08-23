import { codexProfilesConfig } from '@/configs/profiles'
import { BaseProfiles } from '@/features/platform'

export function CodexProfilesView() {
  return <BaseProfiles config={codexProfilesConfig} />
}
