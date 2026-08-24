import { codexMcpConfig } from '@/configs/mcp'
import { BaseMcp } from '@/features/platform/mcp/BaseMcp'

export function CodexMcpView() {
  return <BaseMcp config={codexMcpConfig} />
}
