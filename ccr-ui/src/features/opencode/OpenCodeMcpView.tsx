import { opencodeMcpConfig } from '@/configs/mcp'
import { BaseMcp } from '@/features/platform'

export function OpenCodeMcpView() {
  return <BaseMcp config={opencodeMcpConfig} />
}
