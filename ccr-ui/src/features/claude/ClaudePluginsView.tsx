import { claudePluginsConfig } from '@/configs/plugins'
import { BasePlugins } from '@/features/platform'

export function ClaudePluginsView() {
  return <BasePlugins config={claudePluginsConfig} />
}
