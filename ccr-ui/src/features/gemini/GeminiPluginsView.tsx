import { geminiPluginsConfig } from '@/configs/plugins'
import { BasePlugins } from '@/features/platform'

export function GeminiPluginsView() {
  return <BasePlugins config={geminiPluginsConfig} />
}
