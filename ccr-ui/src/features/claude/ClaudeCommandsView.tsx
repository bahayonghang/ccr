import { claudeCommandsConfig } from '@/configs/commands'
import { BaseCommands } from '@/features/platform'

export function ClaudeCommandsView() {
  return <BaseCommands config={claudeCommandsConfig} />
}
