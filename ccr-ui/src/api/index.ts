/**
 * API 统一入口
 *
 * Tauri 桌面应用模式：所有 API 调用均通过 Tauri invoke() 发送到 Rust 后端。
 * 不再包含 HTTP/Axios 双模式分支，直接从 tauri.ts 导出全部函数。
 */

export * from './tauri'
export type { McpPreset, McpServerInfo, McpSyncResult as SyncResult } from '@/types/api'

// Domain-first modular API (new), while keeping legacy named exports above for compatibility.
export * as configApi from './domains/config'
export * as syncApi from './domains/sync'
export * as platformApi from './domains/platforms'
export * as skillsApi from './domains/skills'
export * as usageApi from './domains/usage'
export * as systemApi from './domains/system'
