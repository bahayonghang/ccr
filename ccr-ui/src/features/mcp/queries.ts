// mcp 域 Query 层（08-22-state-logic-port 批次 5b-ii）。
// 覆盖三组服务端数据（原 usePlatformMcp / usePlatformPlugins / useUnifiedMcp 的
// 模块内手写加载态）：平台 MCP 服务器、平台插件、统一 MCP 列表。
//
// staleTime 取值：全部为 Infinity —— 原实现均为「挂载拉取一次 + 写操作后显式重载」，
// 无 TTL、无窗口聚焦刷新；实时性由 CRUD 后的显式 refetch 保证。
//
// queryFn 直接调用 src/api 既有 wrapper（platformApiMap 内联于 usePlatformMcp，
// 因其携带归一化逻辑，保持原文件内聚）。

export const mcpKeys = {
  all: ['mcp'] as const,
  /** 平台 MCP 服务器列表（按平台隔离缓存） */
  platformServers: (platform: string) => [...mcpKeys.all, 'platform-servers', platform] as const,
  /** 平台插件列表 */
  plugins: (platform: string) => [...mcpKeys.all, 'plugins', platform] as const,
  /** 统一 MCP 列表（全平台快照：servers + capabilities + diagnostics） */
  unifiedList: () => [...mcpKeys.all, 'unified'] as const,
}
