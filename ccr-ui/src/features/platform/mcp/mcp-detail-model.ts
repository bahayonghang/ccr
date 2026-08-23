import type { McpGroup } from '@/types/mcpManager'
import type { UnifiedMcpServer } from '@/types/unifiedMcp'

const SCOPE_ORDER: Record<string, number> = { local: 0, project: 1, user: 2 }

export function pickPrimaryServer(group: McpGroup | null): UnifiedMcpServer | null {
  if (!group) return null
  return (
    group.items.find((item) => item.effective !== false && !item.hidden_by) ??
    group.items[0] ??
    null
  )
}

export function sortPrecedence(items: UnifiedMcpServer[]): UnifiedMcpServer[] {
  return [...items].sort((left, right) => {
    const leftOrder = SCOPE_ORDER[String(left.scope)] ?? 9
    const rightOrder = SCOPE_ORDER[String(right.scope)] ?? 9
    if (leftOrder !== rightOrder) return leftOrder - rightOrder
    return String(left.platform).localeCompare(String(right.platform))
  })
}

export function buildRawConfigPreview(server: UnifiedMcpServer | null): string {
  const raw = server?.raw_config
  if (raw && typeof raw === 'object') {
    return JSON.stringify(raw, null, 2)
  }

  return JSON.stringify(
    {
      command: server?.command ?? undefined,
      url: server?.url ?? undefined,
      args: server?.args,
      env: server?.env,
      headers: server?.headers,
      disabled: server?.disabled,
    },
    null,
    2,
  )
}
