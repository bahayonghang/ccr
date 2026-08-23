import { claudeCodeConfig } from '@/configs/slashCommands'
import { BaseSlashCommands } from './BaseSlashCommands'

export function SlashCommandsView() {
  return <BaseSlashCommands config={claudeCodeConfig} />
}
