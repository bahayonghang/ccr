/**
 * API Modules Index
 *
 * 统一导出所有 API 模块，按功能分类组织
 */

// Core - axios 实例和环境工具
export { api, isTauriEnvironment, resolveApiBaseUrl, getBackendHealth } from '../core'

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
    getHeatmapData,
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
    // 类型
    type HeatmapData,
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

// MCP - MCP 服务器管理、预设、同步、内置提示词
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

// Skill Hub - 技能市场
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

// Agents - Claude Code Agent 管理
export {
    listAgents,
    getAgent,
    addAgent,
    updateAgent,
    deleteAgent,
    toggleAgent,
} from './agents'

// Slash Commands - Claude Code 斜杠命令
export {
    listSlashCommands,
    addSlashCommand,
    updateSlashCommand,
    deleteSlashCommand,
    toggleSlashCommand,
} from './slashCommands'

// Plugins - Claude Code 插件
export {
    listPlugins,
    addPlugin,
    updatePlugin,
    deletePlugin,
    togglePlugin,
} from './plugins'

// Hooks - Claude Code Hooks
export {
    listHooks,
    addHook,
    updateHook,
    deleteHook,
    toggleHook,
} from './hooks'

// Skills - 技能管理
export {
    listSkills,
    addSkill,
    deleteSkill,
    listSkillRepositories,
    addSkillRepository,
    removeSkillRepository,
    scanSkillRepository,
    type Skill,
    type SkillRepository,
    type AddRepositoryRequest,
} from './skills'

// Checkin - 签到管理
export {
    listCheckinProviders,
    getCheckinProvider,
    createCheckinProvider,
    updateCheckinProvider,
    deleteCheckinProvider,
    listBuiltinProviders,
    addBuiltinProvider,
    listCheckinAccounts,
    getCheckinAccount,
    getCheckinAccountDashboard,
    createCheckinAccount,
    updateCheckinAccount,
    deleteCheckinAccount,
    getCheckinAccountCookies,
    executeCheckin,
    checkinAccount,
    queryCheckinBalance,
    getCheckinBalanceHistory,
    listCheckinRecords,
    getAccountCheckinRecords,
    exportCheckinRecords,
    getTodayCheckinStats,
    exportCheckinConfig,
    previewCheckinImport,
    importCheckinConfig,
    testCheckinConnection,
    type AccountCookiesResponse,
} from './checkin'

// Output Styles - 输出样式
export {
    listOutputStyles,
    getOutputStyle,
    createOutputStyle,
    updateOutputStyle,
    deleteOutputStyle,
} from './outputStyles'

// Statusline - 状态栏
export {
    getStatusline,
    updateStatusline,
} from './statusline'

// Converter - 配置转换
export { convertConfig } from './converter'

// Sync (WebDAV) - 同步管理
export {
    getSyncStatus,
    getSyncInfo,
    pushSync,
    pullSync,
} from './sync'

// Codex Platform
export {
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
    listCodexAuthAccounts,
    getCodexAuthCurrent,
    saveCodexAuth,
    switchCodexAuth,
    deleteCodexAuth,
    detectCodexProcess,
    getCodexUsage,
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
} from './codex'

// Gemini Platform
export {
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
} from './gemini'

// Qwen Platform
export {
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
} from './qwen'

// iFlow Platform
export {
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
} from './iflow'

// Droid Platform
export {
    listDroidMcpServers,
    addDroidMcpServer,
    updateDroidMcpServer,
    deleteDroidMcpServer,
    listDroidAgents,
    getDroidAgent,
    addDroidAgent,
    updateDroidAgent,
    deleteDroidAgent,
    listDroidSlashCommands,
    addDroidSlashCommand,
    updateDroidSlashCommand,
    deleteDroidSlashCommand,
    listDroidPlugins,
    addDroidPlugin,
    updateDroidPlugin,
    deleteDroidPlugin,
    type DroidPlugin,
} from './droid'
