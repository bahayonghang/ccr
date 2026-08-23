import { grokApi } from '@/api'
import { getCurrentEnvironment } from '@/api/runtime/environment'
import { getCliVersion } from '@/api/runtime/system'
import type {
  GrokDashboardCommandResponse,
  GrokDashboardOverview,
} from '@/types'

// grok 域 Query 层（08-22-state-logic-port 批次 5）。
// 原 useGrokDashboard 的模块级 TTL 缓存、in-flight 去重与「环境切换清空缓存」
// 由 Query 缓存承担：environment id 进入 overview/version 的 queryKey，
// 换环境自然落到新缓存条目（旧条目由 gc 回收）。staleTime 取值记录：
// - overview 30s（原 OVERVIEW_TTL_MS）、version 60s（原 VERSION_TTL_MS）；
// - environment 0（原每次 refresh 都拉取，仅 in-flight 去重）。

export const grokKeys = {
  all: ['grok'] as const,
  environment: () => [...grokKeys.all, 'environment'] as const,
  overview: (environmentId: string | null) =>
    [...grokKeys.all, 'overview', environmentId ?? null] as const,
  version: (environmentId: string | null) =>
    [...grokKeys.all, 'version', environmentId ?? null] as const,
  profiles: () => [...grokKeys.all, 'profiles'] as const,
}

export const GROK_OVERVIEW_STALE_TIME = 30_000
export const GROK_VERSION_STALE_TIME = 60_000

export type GrokOverviewLoadResult =
  | { status: 'ok'; data: GrokDashboardOverview }
  | { status: 'unsupported_environment'; envType: string }

const toOverview = (
  response: Extract<GrokDashboardCommandResponse, { status: 'ok' }>,
): GrokDashboardOverview => ({
  activation: response.activation,
  activation_name: response.activation_name,
  current_profile: response.current_profile,
  auth_mode: response.auth_mode,
  profiles_total: response.profiles_total,
  profiles_enabled: response.profiles_enabled,
  config_exists: response.config_exists,
  config_path_display: response.config_path_display,
})

export function fetchGrokEnvironment() {
  return getCurrentEnvironment()
}

/** 原模块级 loadOverview 语义：unsupported_environment 不抛错，作为结果态返回。 */
export async function fetchGrokOverview(): Promise<GrokOverviewLoadResult> {
  const response = await grokApi.getGrokDashboardOverview()
  if (response.status === 'unsupported_environment') {
    return { status: response.status, envType: response.env_type }
  }
  return { status: 'ok', data: toOverview(response) }
}

export function fetchGrokVersion() {
  return getCliVersion({
    tool: 'grok',
    timeoutMs: 1_500,
  })
}
