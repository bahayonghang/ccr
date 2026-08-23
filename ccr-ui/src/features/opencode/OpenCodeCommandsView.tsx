import { opencodeCommandsConfig } from '@/configs/commands'
import { BaseCommands } from '@/features/platform'

export function OpenCodeCommandsView() {
  return <BaseCommands config={opencodeCommandsConfig} />
}
