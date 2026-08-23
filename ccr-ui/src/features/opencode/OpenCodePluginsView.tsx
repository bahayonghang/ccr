import { opencodePluginsConfig } from '@/configs/plugins'
import { BasePlugins } from '@/features/platform'

export function OpenCodePluginsView() {
  return <BasePlugins config={opencodePluginsConfig} />
}
