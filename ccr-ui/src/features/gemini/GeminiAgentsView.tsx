import { geminiAgentsConfig } from '@/configs/agents'
import { BaseAgents } from '@/features/platform'

export function GeminiAgentsView() {
  return <BaseAgents config={geminiAgentsConfig} />
}
