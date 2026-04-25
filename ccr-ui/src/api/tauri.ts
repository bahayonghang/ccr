/**
 * Tauri API Client for CCR Desktop
 *
 * 通过 Tauri invoke() 调用 Rust 后端命令的完整封装。
 * 所有函数名与原 api/modules/ 导出保持一致，以确保 Store 层无缝切换。
 *
 * 分组顺序：
 *   1. 环境检测 & 工具函数
 *   2. 配置管理 (Config)
 *   3. 同步 (Sync / WebDAV)
 *   4. Claude Code 平台
 *   5. Codex 平台
 *   6. Gemini 平台
 *   7. Droid 平台
 *   8. OpenCode 平台
 *   9. 签到 (CheckIn)
 *  10. 统计 (Stats)
 *  11. 系统 (System)
 *  12. 转换器 (Converter)
 *  13. UI 状态 (Favorites / Recent Items)
 *  14. WAF
 *  15. 统一 MCP (Unified MCP)
 *  16. 事件 (Events)
 *  17. 环境管理 (Environment)
 *  18. HTTP-only 桩函数 (无 Tauri 命令对应)
 */

import { invoke } from '@tauri-apps/api/core'
import { isTauriRuntime } from '@/utils/tauriRuntime'

type UnknownRecord = Record<string, unknown>

const isRecord = (value: unknown): value is UnknownRecord => {
  return typeof value === 'object' && value !== null
}

const asRecord = (value: unknown): UnknownRecord => {
  return isRecord(value) ? value : {}
}

const asArray = (value: unknown): unknown[] => {
  return Array.isArray(value) ? value : []
}

const pickArray = (value: unknown, key: string): unknown[] => {
  if (!isRecord(value)) {
    return []
  }
  return asArray(value[key])
}

// ════════════════════════════════════════════════════════════
// 类型导出（兼容历史 `@/api` 类型导入）
// ════════════════════════════════════════════════════════════

export interface HeatmapData {
  data: Record<string, number>
  max_value: number
  total_tokens: number
  active_days: number
}

export interface BuiltinPrompt {
  id: string
  name: string
  description: string
  category: string
  tags: string[]
  content: string
}

export interface ClaudeSettingsData {
  model?: string
  availableModels?: string[]
  alwaysThinkingEnabled?: boolean
  maxThinkingTokens?: number
  maxOutputTokens?: number
  effortLevel?: string
  skipDangerousModePermissionPrompt?: boolean
  theme?: string
  language?: string
  showTurnDuration?: boolean
  prefersReducedMotion?: boolean
  spinnerTipsEnabled?: boolean
  terminalProgressBarEnabled?: boolean
  showSpinnerTree?: boolean
  includeCoAuthoredBy?: boolean
  autoUpdates?: boolean
  autoUpdatesChannel?: string
  cleanupPeriodDays?: number
  respectGitignore?: boolean
  env?: Record<string, string>
  permissions?: {
    allow?: string[]
    deny?: string[]
    defaultMode?: string
    additionalDirectories?: string[]
  }
  sandbox?: {
    enabled?: boolean
    autoAllowBashIfSandboxed?: boolean
    network?: {
      allowLocalBinding?: boolean
      allowedDomains?: string[]
    }
    excludedCommands?: string[]
  }
  attribution?: {
    commit?: string
    pr?: string
  }
  [key: string]: unknown
}

export interface SyncResult {
  platform: string
  success: boolean
  message?: string
}

export interface OAuthAuthorizeUrlResponse {
  success: boolean
  authorize_url?: string
  extraction_guide?: string[]
  message?: string
}

export interface OAuthAuthorizeUrlRequest {
  provider_id: string
  oauth_type: 'github' | 'linuxdo'
}

export interface DroidPlugin {
  id: string
  data: Record<string, unknown>
}

export interface SyncFolderItem {
  name: string
  enabled: boolean
  localPath: string
  remotePath: string
  description?: string
}

export interface SyncStatusResponse {
  configured?: boolean
  config?: {
    webdav_url?: string
    username?: string
    remote_path?: string
  }
  [key: string]: unknown
}

export interface CommandResultLike {
  success?: boolean
  message?: string
  output?: string
  data?: {
    output?: string
  }
}

export interface VersionInfoResponse {
  current_version?: string
  build_time?: string
  git_commit?: string
  latest_version?: string
  has_update?: boolean
  release_url?: string
  release_notes?: string
  published_at?: string
}

// resolveNameAndConfig / resolveName helper 已外移到 ./_shared，供各 domain 共享使用。

// Codex Agent helpers（resolveCodexAgentContext/Mutation/NameAndContext）已随 Codex 分组
// 迁移至 ./domains/codex 内部，不再在此暴露。

// ════════════════════════════════════════════════════════════
// 1. 环境检测 & 工具函数
// ════════════════════════════════════════════════════════════

/** 检查是否在 Tauri 桌面应用环境中运行 */
export const isTauriEnvironment = (): boolean => {
  return isTauriRuntime()
}

/** 获取当前运行环境名称 */
export const getEnvironmentName = (): 'tauri' | 'web' => {
  return isTauriEnvironment() ? 'tauri' : 'web'
}

/** 获取 Tauri 版本信息 */
export const getTauriVersion = async (): Promise<string | null> => {
  if (!isTauriEnvironment()) {
    return null
  }
  try {
    const { getVersion } = await import('@tauri-apps/api/app')
    return await getVersion()
  } catch {
    return null
  }
}

const SKIP_EXIT_CONFIRM_KEY = 'ccr_skip_exit_confirm'

/**
 * 获取是否跳过退出确认。
 * - Tauri 环境：走 invoke('get_skip_exit_confirm')；失败则直接抛出，避免与后端状态不一致
 * - Web 环境：回退到 localStorage
 */
export const getSkipExitConfirm = async (): Promise<boolean> => {
  if (isTauriEnvironment()) {
    return await invoke('get_skip_exit_confirm')
  }
  return localStorage.getItem(SKIP_EXIT_CONFIRM_KEY) === '1'
}

/**
 * 设置是否跳过退出确认。
 * - Tauri 环境：走 invoke('set_skip_exit_confirm')；失败则直接抛出
 * - Web 环境：写入 localStorage
 */
export const setSkipExitConfirm = async (skip: boolean): Promise<void> => {
  if (isTauriEnvironment()) {
    await invoke('set_skip_exit_confirm', { skip })
    return
  }
  localStorage.setItem(SKIP_EXIT_CONFIRM_KEY, skip ? '1' : '0')
}

/** 兼容旧用法：TauriAPI.getTauriVersion() */
export const TauriAPI = {
  getTauriVersion,
}

// ════════════════════════════════════════════════════════════
// 2. 配置管理 (Config) —— 实现已迁移至 ./domains/config
// ════════════════════════════════════════════════════════════
// 以下 re-export 保持 `from '@/api/tauri'` 的历史导入兼容，
// 推荐新代码使用 `from '@/api/domains/config'` 或 `configApi.*`。
export {
  listConfigs,
  switchConfig,
  addConfig,
  updateConfig,
  deleteConfig,
  renameConfig,
  duplicateConfig,
  validateConfigs,
  importConfig,
  exportConfig,
  getHistory,
  clearHistory,
  cleanBackups,
} from './domains/config'

// ════════════════════════════════════════════════════════════
// 3. 同步 (Sync / WebDAV) —— 实现已迁移至 ./domains/sync
// ════════════════════════════════════════════════════════════
export {
  pushSync,
  pullSync,
  getSyncStatus,
  getSyncInfo,
  listSyncFolders,
  addSyncFolder,
  updateSyncFolder,
  deleteSyncFolder,
} from './domains/sync'

// ════════════════════════════════════════════════════════════
// 4. Claude Code 平台 —— 实现已迁移至 ./domains/claude
// ════════════════════════════════════════════════════════════
export {
  getClaudeSettings,
  updateClaudeSettings,
  listMcpServers,
  addMcpServer,
  updateMcpServer,
  deleteMcpServer,
  toggleMcpServer,
  listAgents,
  getAgent,
  addAgent,
  updateAgent,
  deleteAgent,
  toggleAgent,
  listSlashCommands,
  addSlashCommand,
  updateSlashCommand,
  deleteSlashCommand,
  toggleSlashCommand,
  listPlugins,
  addPlugin,
  updatePlugin,
  deletePlugin,
  togglePlugin,
  listOutputStyles,
  getOutputStyle,
  createOutputStyle,
  updateOutputStyle,
  deleteOutputStyle,
  getStatusline,
  updateStatusline,
  listHooks,
  updateHooks,
  getBudgetStatus,
  setBudget,
  resetBudget,
  listPrompts,
  updatePrompts,
  listClaudeProfiles,
  exportClaudeProfiles,
  getClaudeProfile,
  addClaudeProfile,
  updateClaudeProfile,
  deleteClaudeProfile,
  applyClaudeProfile,
  listClaudeAuthAccounts,
  getClaudeAuthCurrent,
  saveClaudeAuth,
  switchClaudeAuth,
  deleteClaudeAuth,
} from './domains/claude'

// ════════════════════════════════════════════════════════════
// 5. Codex 平台 —— 实现已迁移至 ./domains/codex
// ════════════════════════════════════════════════════════════
export {
  listCodexProfiles,
  exportCodexProfiles,
  getCodexConfig,
  updateCodexConfig,
  listCodexMcpServers,
  addCodexMcpServer,
  updateCodexMcpServer,
  deleteCodexMcpServer,
  listCodexAgents,
  addCodexAgent,
  updateCodexAgent,
  deleteCodexAgent,
  toggleCodexAgent,
  renameCodexAgent,
  copyCodexAgent,
  validateCodexAgentToml,
  listCodexAgentSources,
  addCodexAgentSource,
  removeCodexAgentSource,
  syncCodexAgentSource,
  getCodexAgentSourceCatalog,
  installCodexSourceAgent,
  syncCodexSourceInstall,
  forceSyncCodexSourceInstall,
  acceptLocalCodexSourceInstall,
  untrackCodexSourceInstall,
  listCodexModels,
  addCodexCustomModel,
  addCodexProfile,
  updateCodexProfile,
  deleteCodexProfile,
  getCodexProfile,
  getCodexProfileEnv,
  applyCodexProfile,
  listCodexSessions,
  getCodexSessionDetail,
  exportCodexSession,
  cloneCodexSession,
  deleteCodexSession,
  listCodexAuthAccounts,
  getCodexAuthCurrent,
  saveCodexAuth,
  switchCodexAuth,
  deleteCodexAuth,
  renameCodexAuth,
  getCodexTraySnapshot,
  detectCodexProcess,
  codexOAuthLoginStart,
  codexOAuthLoginCompleted,
  codexOAuthLoginCancel,
  codexOAuthSubmitCallbackUrl,
  codexIsOAuthPortInUse,
  codexReleaseOAuthPort,
  codexOpenExternalUrl,
  codexImportAuthPayload,
  codexImportAuthFromLocal,
  codexAddAuthWithApiKey,
  codexListModelProviders,
  codexSaveModelProvider,
  codexDeleteModelProvider,
  getCodexDashboardOverview,
  getCodexDashboardUsageSummary,
  getCodexUsage,
  getCodexAllQuotas,
  getCodexQuota,
  listCodexSlashCommands,
  addCodexSlashCommand,
  updateCodexSlashCommand,
  deleteCodexSlashCommand,
  toggleCodexSlashCommand,
  listCodexPlugins,
  addCodexPlugin,
  updateCodexPlugin,
  deleteCodexPlugin,
  toggleCodexPlugin,
  type CodexDashboardUsageSection,
  type CodexDashboardUsageSummary,
  type CodexDashboardOverview,
  type CodexCommandOptions,
  type CodexUsageCommandOptions,
} from './domains/codex'

// ════════════════════════════════════════════════════════════
// 6. Gemini 平台 —— 实现已迁移至 ./domains/gemini
// ════════════════════════════════════════════════════════════
export {
  getGeminiConfig,
  updateGeminiConfig,
  listGeminiMcpServers,
  addGeminiMcpServer,
  updateGeminiMcpServer,
  deleteGeminiMcpServer,
  listGeminiSlashCommands,
  addGeminiSlashCommand,
  updateGeminiSlashCommand,
  deleteGeminiSlashCommand,
  toggleGeminiSlashCommand,
  listGeminiExtensions,
  listGeminiAgents,
  addGeminiAgent,
  updateGeminiAgent,
  deleteGeminiAgent,
  toggleGeminiAgent,
  listGeminiPlugins,
  addGeminiPlugin,
  updateGeminiPlugin,
  deleteGeminiPlugin,
  toggleGeminiPlugin,
} from './domains/gemini'

// ════════════════════════════════════════════════════════════
// 7. Droid 平台 —— 实现已迁移至 ./domains/droid
// ════════════════════════════════════════════════════════════
export {
  getDroidSettings,
  updateDroidSettings,
  listDroidMcpServers,
  addDroidMcpServer,
  updateDroidMcpServer,
  deleteDroidMcpServer,
  listDroidAgents,
  getDroidAgent,
  addDroidAgent,
  updateDroidAgent,
  deleteDroidAgent,
  listDroidPlugins,
  addDroidPlugin,
  updateDroidPlugin,
  deleteDroidPlugin,
  listDroidSlashCommands,
  addDroidSlashCommand,
  updateDroidSlashCommand,
  deleteDroidSlashCommand,
  listDroidModels,
  addDroidModel,
  updateDroidModel,
  deleteDroidModel,
  listDroidProfiles,
  addDroidProfile,
  updateDroidProfile,
  deleteDroidProfile,
  switchDroidProfile,
} from './domains/droid'

// ════════════════════════════════════════════════════════════
// ════════════════════════════════════════════════════════════
// 10. OpenCode 平台 —— 实现已迁移至 ./domains/opencode
// ════════════════════════════════════════════════════════════
export {
  getOpenCodeConfig,
  updateOpenCodeConfig,
  getOpenCodeTuiSettings,
  updateOpenCodeTuiSettings,
  getOpenCodeKeybindings,
  updateOpenCodeKeybindings,
  listOpenCodeThemes,
  listOpenCodeProviders,
  addOpenCodeProvider,
  updateOpenCodeProvider,
  deleteOpenCodeProvider,
  listOpenCodeMcpServers,
  addOpenCodeMcpServer,
  updateOpenCodeMcpServer,
  deleteOpenCodeMcpServer,
  listOpenCodeAgents,
  addOpenCodeAgent,
  updateOpenCodeAgent,
  deleteOpenCodeAgent,
  listOpenCodeCommands,
  addOpenCodeCommand,
  updateOpenCodeCommand,
  deleteOpenCodeCommand,
  listOpenCodePlugins,
  addOpenCodePlugin,
  deleteOpenCodePlugin,
  listOpenCodeLocalPlugins,
} from './domains/opencode'

// ════════════════════════════════════════════════════════════
// 11. 签到 (CheckIn) —— 实现已迁移至 ./domains/checkin
// ════════════════════════════════════════════════════════════
// 以下 re-export 保持 `from '@/api/tauri'` 的历史导入兼容，
// 推荐新代码使用 `from '@/api/domains/checkin'` 或 `checkinApi.*`。
export {
  listCheckinProviders,
  getCheckinProvider,
  createCheckinProvider,
  updateCheckinProvider,
  deleteCheckinProvider,
  testCheckinConnection,
  listCheckinAccounts,
  getCheckinAccount,
  getCheckinAccountDashboard,
  createCheckinAccount,
  updateCheckinAccount,
  deleteCheckinAccount,
  batchDeleteAccounts,
  executeCheckin,
  checkinAccount,
  batchCheckin,
  startCheckinJob,
  getCheckinJobStatus,
  queryCheckinBalance,
  getCheckinBalanceHistory,
  getBalanceStats,
  listCheckinRecords,
  getAccountCheckinRecords,
  exportCheckinRecords,
  getTodayCheckinStats,
  executeCdkRecharge,
  getCdkHistory,
  listWafCookies,
  addWafCookie,
  deleteWafCookie,
  getCheckinAccountCookies,
  exportCheckinConfig,
  previewCheckinImport,
  importCheckinConfig,
  listBuiltinProviders,
  addBuiltinProvider,
  getOAuthAuthorizeUrl,
} from './domains/checkin'

// ════════════════════════════════════════════════════════════
// ════════════════════════════════════════════════════════════
// 12. 统计 (Stats) —— 实现已迁移至 ./domains/stats
// ════════════════════════════════════════════════════════════
// 以下 re-export 保持 `from '@/api/tauri'` 的历史导入兼容，
// 推荐新代码使用 `from '@/api/domains/stats'` 或 `statsApi.*` / `usageApi.*`。
export {
  getCostOverview,
  getHeatmapData,
  getUsageSummaryV2,
  getUsageTrendsV2,
  getUsageByModelV2,
  getUsageByProjectV2,
  getUsageHeatmapV2,
  getUsageLogsV2,
  getUsageDashboardV2,
  startUsageImportJobV2,
  ensureSessionIndexV2,
  getSessionIndexJobStatusV2,
  getUsageImportJobStatusV2,
  importUsageV2,
  importAllUsageV2,
  getHomeUsageOverviewV2,
  getSessionStats,
  getCostTrend,
  getCostByModel,
  getCostByProject,
  getProviderUsage,
  getTopSessions,
  getStatsSummary,
  setPricing,
  getPricingList,
  removePricing,
  resetPricing,
  getDailyStats,
  type UsageLogsQuery,
} from './domains/stats'

// ════════════════════════════════════════════════════════════
// 13. 系统 (System) —— 实现已迁移至 ./domains/system
// ════════════════════════════════════════════════════════════
export {
  getSystemInfo,
  checkVersion,
  healthCheck,
  getVersion,
  checkUpdate,
  updateCCR,
  getCliVersions,
  getCliVersion,
  type CliVersionsCommandOptions,
  type CliVersionCommandOptions,
} from './domains/system'

// ════════════════════════════════════════════════════════════
// 14. 转换器 (Converter) —— 实现已迁移至 ./domains/converter
// ════════════════════════════════════════════════════════════
export { convertConfig } from './domains/converter'

// ════════════════════════════════════════════════════════════
// 15. UI 状态 (Favorites / Recent Items) —— 实现已迁移至 ./domains/uiState
// ════════════════════════════════════════════════════════════
export {
  getFavorites,
  addFavorite,
  removeFavorite,
  getRecentItems,
  addRecentItem,
  clearRecentItems,
} from './domains/uiState'

// ════════════════════════════════════════════════════════════
// 16. WAF —— 实现已迁移至 ./domains/waf
// ════════════════════════════════════════════════════════════
export { openWafLogin, getWafCookieStatus } from './domains/waf'

// ════════════════════════════════════════════════════════════
// 17. 统一 MCP (Unified MCP) —— 实现已迁移至 ./domains/unifiedMcp
// ════════════════════════════════════════════════════════════
export {
  listUnifiedMcp,
  addUnifiedMcp,
  updateUnifiedMcp,
  deleteUnifiedMcp,
  toggleUnifiedMcp,
} from './domains/unifiedMcp'

// ════════════════════════════════════════════════════════════
// 18. 事件 (Events) —— 实现已迁移至 ./domains/events
// ════════════════════════════════════════════════════════════
export { getRecentEvents, getRuntimeMetrics } from './domains/events'

// ════════════════════════════════════════════════════════════
// 19. 环境管理 (Environment) —— 实现已迁移至 ./domains/environment
// ════════════════════════════════════════════════════════════
export {
  listEnvironments,
  getCurrentEnvironment,
  switchEnvironment,
  refreshEnvironments,
  envListPlatforms,
  envDetectCli,
  sshListHosts,
  sshAddHost,
  sshConnect,
  sshReconnect,
  sshDisconnect,
  sshGetConnectionState,
  sshProbeHostFingerprint,
  sshConfirmHostFingerprint,
  sshReadConfig,
  sshWriteConfig,
  sshDetectCli,
  sshTestConnection,
  sshListKeys,
  type SshHostConfig,
  type SshConnectionState,
  type SshFingerprintProbeResult,
  type SshConnectResult,
  type SshKeyInfo,
} from './domains/environment'

// ════════════════════════════════════════════════════════════
// 20. HTTP-only 通用桩函数
// ════════════════════════════════════════════════════════════

/** 执行 CCR 命令 */
export const executeCommand = async (
  commandOrPayload: string | { command: string; args?: string[] },
  args?: string[]
): Promise<unknown> => {
  const command = typeof commandOrPayload === 'string' ? commandOrPayload : commandOrPayload.command
  const resolvedArgs = typeof commandOrPayload === 'string' ? args : commandOrPayload.args
  return invoke('execute_ccr_command', { command, args: resolvedArgs })
}

/** 列出可用命令 */
export const listCommands = async <T = UnknownRecord>(_client?: string): Promise<T> => {
  return invoke('list_ccr_commands')
}

/** 获取命令帮助 */
export const getCommandHelp = async <T = UnknownRecord>(command: string): Promise<T> => {
  return invoke('get_ccr_command_help', { command })
}

/** 启用配置（等价于 switchConfig，直接 invoke 避免同文件 re-export 循环） */
export const enableConfig = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('switch_config', { name })
}

/** 禁用配置（通过 update_config 设置 enabled=false） */
export const disableConfig = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('update_config', { name, data: { enabled: false } })
}

/** 获取单个配置详情（通过列表后过滤实现） */
export const getConfig = async <T = UnknownRecord>(name: string): Promise<T> => {
  const result = await invoke<unknown>('list_configs')
  const configs = Array.isArray(result) ? result : pickArray(result, 'configs')
  const found = configs.find((item) => isRecord(item) && String(item.name ?? '') === name)
  return (found ?? null) as T
}

// ── MCP 预设 / 同步 / 内置提示词 ──

/** 列出 MCP 预设 */
export const listMcpPresets = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('list_mcp_presets')
}

/** 获取 MCP 预设详情 */
export const getMcpPreset = async <T = UnknownRecord>(id: string): Promise<T> => {
  return invoke('get_mcp_preset', { id })
}

/** 安装 MCP 预设 */
export const installMcpPreset = async <T = UnknownRecord>(
  presetIdOrRequest: string | object,
  platforms?: string[],
  env?: Record<string, string>
): Promise<T> => {
  const request = asRecord(presetIdOrRequest)
  const presetId =
    typeof presetIdOrRequest === 'string'
      ? presetIdOrRequest
      : String(request.preset_id ?? request.id ?? '')
  const envValue = env ?? request.env
  const envVars = isRecord(envValue)
    ? Object.entries(envValue).reduce<Record<string, string>>((acc, [key, value]) => {
        if (typeof value === 'string') {
          acc[key] = value
        }
        return acc
      }, {})
    : undefined
  return invoke('install_mcp_preset', { presetId, platforms, envVars })
}

/** 安装单个 MCP 预设 */
export const installMcpPresetSingle = async <T = UnknownRecord>(
  presetIdOrRequest: string | object,
  platform?: string,
  env?: Record<string, string>
): Promise<T> => {
  const request = asRecord(presetIdOrRequest)
  const presetId =
    typeof presetIdOrRequest === 'string'
      ? presetIdOrRequest
      : String(request.preset_id ?? request.id ?? '')
  const resolvedPlatform =
    platform ?? (typeof request.platform === 'string' ? request.platform : undefined)
  const envValue = env ?? request.env
  const envVars = isRecord(envValue)
    ? Object.entries(envValue).reduce<Record<string, string>>((acc, [key, value]) => {
        if (typeof value === 'string') {
          acc[key] = value
        }
        return acc
      }, {})
    : undefined
  return invoke('install_mcp_preset_single', { platform: resolvedPlatform, presetId, envVars })
}

/** 列出来源 MCP 服务器 */
export const listSourceMcpServers = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('list_source_mcp_servers')
}

/** 同步 MCP 服务器 */
export const syncMcpServer = async <T = UnknownRecord>(
  name: string,
  platforms?: string[]
): Promise<T> => {
  return invoke('sync_mcp_server', { name, platforms })
}

/** 同步所有 MCP 服务器 */
export const syncAllMcpServers = async <T = UnknownRecord>(platforms?: string[]): Promise<T> => {
  return invoke('sync_all_mcp_servers', { platforms })
}

/** 列出内置提示词 */
export const listBuiltinPrompts = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('list_builtin_prompts')
}

/** 获取内置提示词 */
export const getBuiltinPrompt = async <T = UnknownRecord>(id: string): Promise<T> => {
  return invoke('get_builtin_prompt', { id })
}

/** 按分类获取内置提示词 */
export const getBuiltinPromptsByCategory = async <T = UnknownRecord>(
  category: string
): Promise<T> => {
  return invoke('get_builtin_prompts_by_category', { category })
}

// ── Axios / HTTP 核心（不再需要） ──

/** HTTP API 基础 URL（Tauri 模式返回空字符串） */
export const resolveApiBaseUrl = (): string => {
  return ''
}

/** 后端健康检查（使用 Tauri invoke 实现） */
export const getBackendHealth = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('health_check')
}
