import { geminiMcpConfig } from '@/configs/mcp'
import { BaseMcp } from '@/features/platform'

export function GeminiMcpView() {
  return <BaseMcp config={geminiMcpConfig} />
}
