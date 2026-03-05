/**
 * API 统一入口
 *
 * Tauri 桌面应用模式：所有 API 调用均通过 Tauri invoke() 发送到 Rust 后端。
 * 不再包含 HTTP/Axios 双模式分支，直接从 tauri.ts 导出全部函数。
 */

export * from './tauri'
export type { McpPreset, McpServerInfo, McpSyncResult as SyncResult } from '@/types/api'
