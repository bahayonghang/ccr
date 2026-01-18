// Unified API Client
// 根据运行环境自动选择 Tauri API 或 HTTP API

import * as HttpAPI from './client'
import * as TauriAPI from './tauri'
import type {
  ConfigListResponse,
  HistoryResponse,
} from '@/types'

// ===================================
// Environment Detection
// ===================================

/**
 * 检测是否在 Tauri 桌面应用环境中运行
 */
export const isTauriEnvironment = (): boolean => {
  return '__TAURI__' in window
}

/**
 * 获取当前运行环境名称
 */
export const getEnvironmentName = (): 'tauri' | 'web' => {
  return isTauriEnvironment() ? 'tauri' : 'web'
}

// ===================================
// Unified Configuration API
// ===================================

/**
 * 列出所有配置项
 * - Tauri: 调用 list_profiles 命令
 * - Web: 调用 HTTP /api/configs
 */
export const listConfigs = async (): Promise<ConfigListResponse> => {
  if (isTauriEnvironment()) {
    // Tauri 环境：调用 Rust backend
    const profiles = await TauriAPI.listProfiles()

    // 转换为 ConfigListResponse 格式
    const current = await TauriAPI.getCurrentProfile()
    const configs: ConfigListResponse['configs'] = profiles.map(profile => ({
      name: profile.name,
      description: profile.description || '',
      base_url: profile.base_url,
      auth_token: '', // Token 在列表中不显示
      model: profile.model,
      provider_type: profile.provider || undefined,
      is_current: profile.name === current,
      is_default: profile.is_default || false,
    }))

    return {
      configs,
      current_config: current,
      default_config: profiles.find(p => p.is_default)?.name || current,
    }
  } else {
    // Web 环境：调用 HTTP API
    return HttpAPI.listConfigs()
  }
}

/**
 * 切换配置
 * - Tauri: switch_profile 命令
 * - Web: HTTP POST /api/switch
 */
export const switchConfig = async (configName: string): Promise<string> => {
  if (isTauriEnvironment()) {
    return TauriAPI.switchProfile(configName)
  } else {
    return HttpAPI.switchConfig(configName)
  }
}

/**
 * 验证所有配置
 * - Tauri: validate_configs 命令
 * - Web: HTTP GET /api/validate
 */
export const validateConfigs = async (): Promise<string> => {
  if (isTauriEnvironment()) {
    return TauriAPI.validateConfigs()
  } else {
    return HttpAPI.validateConfigs()
  }
}

// ===================================
// Unified History API
// ===================================

/**
 * 获取历史记录
 * - Tauri: get_history 命令
 * - Web: HTTP GET /api/history
 */
export const getHistory = async (): Promise<HistoryResponse> => {
  if (isTauriEnvironment()) {
    const entries = await TauriAPI.getHistory(100)

    // 转换为 HistoryResponse 格式
    return {
      entries: entries.map(entry => ({
        id: entry.id,
        timestamp: entry.timestamp,
        operation: entry.operation,
        from_config: '',
        to_config: '',
        actor: entry.actor,
        changes: [],
      })),
      total: entries.length,
    }
  } else {
    return HttpAPI.getHistory()
  }
}

// ===================================
// Unified Platform API
// ===================================

/**
 * 列出所有平台
 * - Tauri: list_platforms 命令
 * - Web: 返回静态列表（Web版本不支持多平台）
 */
export const listPlatforms = async (): Promise<string[]> => {
  if (isTauriEnvironment()) {
    return TauriAPI.listPlatforms()
  } else {
    // Web 版本默认只支持 Claude
    return ['claude']
  }
}

/**
 * 切换平台
 * - Tauri: switch_platform 命令
 * - Web: 不支持（抛出错误）
 */
export const switchPlatform = async (platform: string): Promise<string> => {
  if (isTauriEnvironment()) {
    return TauriAPI.switchPlatform(platform)
  } else {
    throw new Error('Platform switching is only supported in desktop app')
  }
}

/**
 * 获取当前平台
 * - Tauri: get_current_platform 命令
 * - Web: 返回 'claude'
 */
export const getCurrentPlatform = async (): Promise<string> => {
  if (isTauriEnvironment()) {
    return TauriAPI.getCurrentPlatform()
  } else {
    return 'claude'
  }
}

// ===================================
// Re-export HTTP API functions
// (Web-only features or features not yet implemented in Tauri)
// ===================================

export {
  // Command execution (web-only)
  executeCommand,
  listCommands,
  getCommandHelp,

  // Advanced config operations (web-only for now)
  cleanBackups,
  exportConfig,
  importConfig,
  deleteConfig,
  getConfig,
  updateConfig,
  addConfig,
  enableConfig,
  disableConfig,

  // System info (web-only)
  getSystemInfo,

  // Version management (web-only)
  getVersion,
  checkUpdate,
  updateCCR,

  // MCP Servers (web-only)
  listMcpServers,
  addMcpServer,
  updateMcpServer,
  deleteMcpServer,
  toggleMcpServer,

  // Agents (web-only)
  listAgents,
  addAgent,
  updateAgent,
  deleteAgent,
  toggleAgent,

  // Slash Commands (web-only)
  listSlashCommands,
  addSlashCommand,
  updateSlashCommand,
  deleteSlashCommand,
  toggleSlashCommand,

  // Plugins (web-only)
  listPlugins,
  addPlugin,
  updatePlugin,
  deletePlugin,
  togglePlugin,

  // Codex (web-only)
  listCodexMcpServers,
  addCodexMcpServer,
  updateCodexMcpServer,
  deleteCodexMcpServer,
  listCodexProfiles,
  addCodexProfile,
  updateCodexProfile,
  deleteCodexProfile,
  getCodexProfile,
  applyCodexProfile,
  getCodexConfig,
  updateCodexConfig,
  listCodexAgents,
  addCodexAgent,
  updateCodexAgent,
  deleteCodexAgent,
  toggleCodexAgent,
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
  // Codex Auth (web-only)
  listCodexAuthAccounts,
  getCodexAuthCurrent,
  saveCodexAuth,
  switchCodexAuth,
  deleteCodexAuth,
  detectCodexProcess,
  // Codex Usage (web-only)
  getCodexUsage,

  // Gemini (web-only)
  listGeminiMcpServers,
  addGeminiMcpServer,
  updateGeminiMcpServer,
  deleteGeminiMcpServer,
  getGeminiConfig,
  updateGeminiConfig,
  listGeminiAgents,
  addGeminiAgent,
  updateGeminiAgent,
  deleteGeminiAgent,
  toggleGeminiAgent,
  listGeminiSlashCommands,
  addGeminiSlashCommand,
  updateGeminiSlashCommand,
  deleteGeminiSlashCommand,
  toggleGeminiSlashCommand,
  listGeminiPlugins,
  addGeminiPlugin,
  updateGeminiPlugin,
  deleteGeminiPlugin,
  toggleGeminiPlugin,

  // Qwen (web-only)
  listQwenMcpServers,
  addQwenMcpServer,
  updateQwenMcpServer,
  deleteQwenMcpServer,
  getQwenConfig,
  updateQwenConfig,
  listQwenAgents,
  addQwenAgent,
  updateQwenAgent,
  deleteQwenAgent,
  toggleQwenAgent,
  listQwenSlashCommands,
  addQwenSlashCommand,
  updateQwenSlashCommand,
  deleteQwenSlashCommand,
  toggleQwenSlashCommand,
  listQwenPlugins,
  addQwenPlugin,
  updateQwenPlugin,
  deleteQwenPlugin,
  toggleQwenPlugin,

  // iFlow (web-only)
  listIflowMcpServers,
  addIflowMcpServer,
  updateIflowMcpServer,
  deleteIflowMcpServer,
  listIflowAgents,
  addIflowAgent,
  updateIflowAgent,
  deleteIflowAgent,
  toggleIflowAgent,
  listIflowSlashCommands,
  addIflowSlashCommand,
  updateIflowSlashCommand,
  deleteIflowSlashCommand,
  toggleIflowSlashCommand,
  listIflowPlugins,
  addIflowPlugin,
  updateIflowPlugin,
  deleteIflowPlugin,
  toggleIflowPlugin,

  // Converter (web-only)
  convertConfig,

  // Sync (web-only for now)
  getSyncStatus,
  getSyncInfo,
  pushSync,
  pullSync,

  // Statistics (web-only)
  getCostOverview,
  getCostToday,
  getCostWeek,
  getCostMonth,
  getCostTrend,
  getCostByModel,
  getCostByProject,
  getTopSessions,
  getStatsSummary,

  // Checkin (web-only)
  listCheckinProviders,
  getCheckinProvider,
  createCheckinProvider,
  updateCheckinProvider,
  deleteCheckinProvider,
  listBuiltinProviders,
  addBuiltinProvider,
  listCheckinAccounts,
  getCheckinAccount,
  createCheckinAccount,
  updateCheckinAccount,
  deleteCheckinAccount,
  executeCheckin,
  checkinAccount,
  queryCheckinBalance,
  getCheckinBalanceHistory,
  listCheckinRecords,
  getAccountCheckinRecords,
  getTodayCheckinStats,
  exportCheckinConfig,
  previewCheckinImport,
  importCheckinConfig,
  testCheckinConnection,
} from './client'

// ===================================
// Utility Exports
// ===================================

// isTauriEnvironment 和 getEnvironmentName 已在文件顶部定义和导出
export { TauriAPI } from './tauri'
