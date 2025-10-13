// 🎨 CCR Desktop API 接口封装
// 支持 Tauri 桌面模式和 Web 调试模式双模式
// 本小姐用 TypeScript 打造的类型安全 API！(￣▽￣)ゞ

// ============================================================================
// 🔌 统一 API 接口导出
// ============================================================================
// 
// 本文件作为统一的 API 入口，自动适配运行环境：
// - 🖥️ Tauri 桌面模式: 使用 invoke() 调用 Rust Commands
// - 🌐 Web 调试模式: 使用 fetch() 调用 HTTP API
//
// 使用方式完全一致，无需修改业务代码
// ============================================================================

// 导入适配器（自动选择 Tauri 或 Web API）
export {
  // 配置管理
  listConfigs,
  getCurrentConfig,
  switchConfig,
  createConfig,
  updateConfig,
  deleteConfig,
  
  // 导入导出
  importConfig,
  exportConfig,
  
  // 验证
  validateAll,
  
  // 历史记录
  getHistory,
  
  // 备份管理
  listBackups,
  restoreBackup,
  
  // 系统信息
  getSystemInfo,
  
  // 调试工具
  getRunMode,
  getApiConfig,
  testApiConnection
} from './adapter'

// ============================================================================
// 📝 类型定义导出
// ============================================================================

export type {
  ConfigInfo,
  ConfigList,
  HistoryEntry,
  BackupInfo,
  SystemInfo,
  CreateConfigRequest,
  UpdateConfigRequest,
} from '../types'

// ============================================================================
// 🔧 使用说明
// ============================================================================
//
// import { listConfigs, switchConfig } from './api'
//
// // 在 Tauri 环境中：自动使用 invoke()
// // 在浏览器环境中：自动使用 fetch()
//
// const configs = await listConfigs()  // 适配器自动选择调用方式
// await switchConfig('anthropic')      // 业务代码保持不变
//
// ============================================================================