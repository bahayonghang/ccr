import { opencodeSettingsConfig } from '@/configs/settings'
import { BaseSettings } from '@/features/platform'

export function OpenCodeSettingsView() {
  return <BaseSettings config={opencodeSettingsConfig} />
}
