export type {
  SettingsConfig,
  SettingsFeatureName,
  SettingsFeatures,
  SettingsField,
  SettingsFieldKind,
  SettingsFieldOption,
  SettingsScalar,
  SettingsTab,
  SettingsValues,
} from '@/configs/settings-types'
export { claudeSettingsConfig } from '@/configs/settings-claude'
export { grokSettingsConfig } from '@/configs/settings-grok'
export { codexSettingsConfig } from '@/configs/settings-codex'
export { opencodeSettingsConfig } from '@/configs/settings-opencode'

import { claudeSettingsConfig } from '@/configs/settings-claude'
import { grokSettingsConfig } from '@/configs/settings-grok'
import { codexSettingsConfig } from '@/configs/settings-codex'
import { opencodeSettingsConfig } from '@/configs/settings-opencode'

export const settingsConfigs = {
  claude: claudeSettingsConfig,
  grok: grokSettingsConfig,
  codex: codexSettingsConfig,
  opencode: opencodeSettingsConfig,
} as const
