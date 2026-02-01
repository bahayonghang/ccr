/**
 * API Modules Index
 * 
 * 统一导出所有 API 模块，按功能分类组织
 */

// Core - axios 实例
export { api } from '../core'

// Statistics - 成本、预算、定价、使用分析
export {
    // 成本统计
    getCostOverview,
    getCostToday,
    getCostWeek,
    getCostMonth,
    getCostTrend,
    getCostByModel,
    getCostByProject,
    getProviderUsage,
    getTopSessions,
    getStatsSummary,
    // 预算管理
    getBudgetStatus,
    setBudget,
    resetBudget,
    // 定价管理
    getPricingList,
    setPricing,
    removePricing,
    resetPricing,
    // 使用分析
    getUsageRecords,
    getDailyStats,
} from './stats'

// Config - 配置管理、命令执行、系统信息
export {
    // 命令执行
    executeCommand,
    listCommands,
    getCommandHelp,
    // 配置管理
    listConfigs,
    switchConfig,
    validateConfigs,
    deleteConfig,
    enableConfig,
    disableConfig,
    getConfig,
    updateConfig,
    addConfig,
    cleanBackups,
    exportConfig,
    importConfig,
    // 历史和系统
    getHistory,
    getSystemInfo,
    // 版本管理
    getVersion,
    checkUpdate,
    updateCCR,
} from './config'

// MCP - MCP 服务器管理、预设、同步
export {
    // MCP 服务器 CRUD
    listMcpServers,
    addMcpServer,
    updateMcpServer,
    deleteMcpServer,
    toggleMcpServer,
    // 预设管理
    listMcpPresets,
    getMcpPreset,
    installMcpPreset,
    installMcpPresetSingle,
    // 同步
    listSourceMcpServers,
    syncMcpServer,
    syncAllMcpServers,
    // 内置提示词
    listBuiltinPrompts,
    getBuiltinPrompt,
    getBuiltinPromptsByCategory,
    // 类型
    type McpPreset,
    type InstallPresetRequest,
    type InstallPresetResult,
    type InstallPresetResponse,
    type McpServerInfo,
    type SyncResult,
    type SyncResponse,
    type SyncAllResponse,
    type BuiltinPrompt,
} from './mcp'

export {
  getSkillHubTrending,
  searchSkillHubMarketplace,
  getSkillHubAgents,
  getSkillHubAgentSkills,
  installSkillHubSkill,
  removeSkillHubSkill,
  type SkillHubMarketplaceItem,
  type SkillHubMarketplaceResponse,
  type SkillHubAgentSummary,
  type SkillHubInstalledSkill,
  type SkillHubOperationResponse,
  type SkillHubOperationResult,
} from './skillHub'
