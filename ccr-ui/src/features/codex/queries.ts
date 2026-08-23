import {
  getCodexAgentSourceCatalog,
  getCodexDashboardOverview,
  getCodexDashboardUsageSummary,
  listCodexAgentSources,
  listCodexAgents,
  listCodexModels,
} from '@/api'
import { getCliVersion } from '@/api/runtime/system'

// codex 域 Query 层（08-22-state-logic-port 批次 5）。
// 原 useCodexDashboard / useCodexAgents / useCodexAgentSources 的模块级 TTL 缓存
// 与 in-flight 去重由 Query 缓存承担。staleTime 取值记录：
// - dashboard overview / usage summary：30s（原 DASHBOARD_TTL_MS）；
// - cli version：60s（原 VERSION_TTL_MS）；
// - agents 列表 / models / agent sources / catalog：0（原实现每次显式刷新，
//   无 TTL；挂载即拉取保持等价节奏）。
// runtime summary（sessions_total）复用 dashboard overview 缓存（同 key）。

import type { CodexAgentContextRequest } from '@/types'

export const codexKeys = {
  all: ['codex'] as const,
  dashboard: {
    all: ['codex', 'dashboard'] as const,
    overview: () => [...codexKeys.dashboard.all, 'overview'] as const,
    usageSummary: () => [...codexKeys.dashboard.all, 'usage-summary'] as const,
    version: () => [...codexKeys.dashboard.all, 'version'] as const,
  },
  agents: {
    all: ['codex', 'agents'] as const,
    list: (mode: string, projectRoot?: string | null) =>
      [...codexKeys.agents.all, 'list', mode, projectRoot ?? null] as const,
    models: () => [...codexKeys.agents.all, 'models'] as const,
  },
  agentSources: {
    all: ['codex', 'agent-sources'] as const,
    list: () => [...codexKeys.agentSources.all, 'list'] as const,
    catalog: (sourceId: string | null) =>
      [...codexKeys.agentSources.all, 'catalog', sourceId] as const,
  },
  providers: {
    all: ['codex', 'providers'] as const,
    list: () => [...codexKeys.providers.all, 'list'] as const,
  },
  tray: {
    all: ['codex', 'tray'] as const,
    snapshot: () => [...codexKeys.tray.all, 'snapshot'] as const,
  },
}

export const CODEX_DASHBOARD_STALE_TIME = 30_000
export const CODEX_VERSION_STALE_TIME = 60_000

export function fetchCodexDashboardOverview() {
  return getCodexDashboardOverview()
}

export function fetchCodexDashboardUsageSummary() {
  return getCodexDashboardUsageSummary()
}

export function fetchCodexCliVersion() {
  return getCliVersion({ tool: 'codex', timeoutMs: 1_500 })
}

export function fetchCodexAgentSources() {
  return listCodexAgentSources()
}

export function fetchCodexAgentSourceCatalog(sourceId: string) {
  return getCodexAgentSourceCatalog(sourceId)
}

export function fetchCodexAgents(context?: CodexAgentContextRequest) {
  return listCodexAgents(context)
}

export function fetchCodexModels() {
  return listCodexModels()
}
