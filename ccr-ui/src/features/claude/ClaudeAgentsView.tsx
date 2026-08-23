import { claudeAgentsConfig } from '@/configs/agents'
import { BaseAgents } from '@/features/platform'

export function ClaudeAgentsView() {
  return <BaseAgents config={claudeAgentsConfig} />
}
