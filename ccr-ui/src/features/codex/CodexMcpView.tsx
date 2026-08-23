import { codexMcpConfig } from '@/configs/mcp'
import { BaseMcp } from '@/features/platform'

export function CodexMcpView() {
  return <BaseMcp config={codexMcpConfig} />
}
