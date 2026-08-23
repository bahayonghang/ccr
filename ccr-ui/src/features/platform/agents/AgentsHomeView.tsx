import { claudeAgentsConfig } from '@/configs/agents'
import { BaseAgents } from '@/features/platform/agents/BaseAgents'

/** `/agents` 通用列表。原 generic/AgentsView，config 走 Claude agents。 */
export function AgentsHomeView() {
  return <BaseAgents config={claudeAgentsConfig} />
}
