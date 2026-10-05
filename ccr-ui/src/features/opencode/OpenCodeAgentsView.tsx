import { opencodeAgentsConfig } from '@/configs/agents'
import { BaseAgents } from '@/features/platform'

export function OpenCodeAgentsView() {
  return <BaseAgents config={opencodeAgentsConfig} />
}
