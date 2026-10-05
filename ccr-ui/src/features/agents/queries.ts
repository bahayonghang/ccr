// agents 域 Query 层（08-22-state-logic-port 批次 5）。
// 原 `composables/useAgents.ts`：agents/gemini 两模块列表走
// listAgents / listGeminiAgents IPC，系统信息辅助切片走 listConfigs/getHistory。

export const agentsKeys = {
  all: ['agents'] as const,
  /** module 取值与 useAgents 的 ModuleType 一致（'agents' | 'gemini'）。 */
  list: (module: string) => [...agentsKeys.all, 'list', module] as const,
}
