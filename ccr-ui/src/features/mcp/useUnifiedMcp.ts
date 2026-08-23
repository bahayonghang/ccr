import { ALL_PLATFORMS, PLATFORM_META } from './mcp-constants'
import { useUnifiedMcpForm } from './useUnifiedMcpForm'
import { useUnifiedMcpList } from './useUnifiedMcpList'

export { ALL_PLATFORMS, PLATFORM_META }

export function useUnifiedMcp() {
  const list = useUnifiedMcpList()
  const form = useUnifiedMcpForm(list)
  return {
    PLATFORM_META,
    ALL_PLATFORMS,
    ...list,
    ...form,
  }
}
