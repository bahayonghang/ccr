import { codexSettingsConfig } from '@/configs/settings'
import { BaseSettings } from '@/features/platform'

export function CodexSettingsView() {
  return <BaseSettings config={codexSettingsConfig} />
}
