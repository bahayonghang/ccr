import { codexAgentsConfig } from '@/configs/agents'
import { BaseAgents } from '@/features/platform'

export function CodexAgentsView() {
  return <BaseAgents config={codexAgentsConfig} />
}
