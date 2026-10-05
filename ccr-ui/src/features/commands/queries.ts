import { useMutation, useQuery } from '@tanstack/react-query'
import { executeCommand, listCommands } from '@/api'

// commands 域 Query 层（08-22-state-logic-port 批次 2）。
// 原 `stores/commands.ts` 的 `useCachedFetch`（2min TTL）由 Query 的 staleTime
// 等效替代；`executeCommand` 为写操作 → useMutation + 失效列表 key。
// Background job identity, output and history belong to the shell-lived command
// stream store. Query owns only the catalog; pages do not duplicate job state.

export const commandsKeys = {
  all: ['commands'] as const,
  list: (client?: string) => [...commandsKeys.all, 'list', client ?? null] as const,
}

/** staleTime 取值记录（批次 2）：2min，等效原 useCachedFetch 的 TTL。 */
const COMMANDS_STALE_TIME = 120_000

export function useCommands(client?: string) {
  return useQuery({
    queryKey: commandsKeys.list(client),
    queryFn: () => listCommands(client),
    staleTime: COMMANDS_STALE_TIME,
  })
}

export function useExecuteCommand() {
  return useMutation({
    mutationFn: (
      payload: string | { command: string; args?: string[]; confirmationToken?: string | null },
    ) => executeCommand(payload),
    // 命令清单不含运行结果，执行本身不改变 list，故不失效列表 key。
  })
}
