import { claudeSettingsConfig } from '@/configs/settings'
import { BaseSettings } from '@/features/platform'

export function ClaudeSettingsView() {
  return <BaseSettings config={claudeSettingsConfig} />
}
