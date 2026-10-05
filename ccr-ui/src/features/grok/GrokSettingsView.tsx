import { grokSettingsConfig } from '@/configs/settings'
import { BaseSettings } from '@/features/platform'

export function GrokSettingsView() {
  return <BaseSettings config={grokSettingsConfig} />
}
