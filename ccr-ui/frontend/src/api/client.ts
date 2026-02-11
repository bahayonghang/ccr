/**
 * @fileoverview CCR UI 后端 API 客户端
 *
 * 此文件现在作为统一重导出层，所有 API 函数已迁移到 modules/ 目录下。
 * 保留此文件是为了向后兼容 — 现有的 import { xxx } from './client' 仍然有效。
 *
 * 架构：
 *   core.ts          → Axios 实例 + 环境检测
 *   modules/*.ts     → 按领域拆分的 API 函数
 *   client.ts (本文件) → 向后兼容重导出层
 *   index.ts         → Tauri/HTTP 统一桥接层
 */

// Core - Axios 实例和环境工具
export { api, isTauriEnvironment, resolveApiBaseUrl, getBackendHealth } from './core'

// 从所有模块重导出全部 API 函数
export * from './modules/stats'
export * from './modules/config'
export * from './modules/mcp'
export * from './modules/skillHub'
export * from './modules/agents'
export * from './modules/slashCommands'
export * from './modules/plugins'
export * from './modules/hooks'
export * from './modules/skills'
export * from './modules/checkin'
export * from './modules/outputStyles'
export * from './modules/statusline'
export * from './modules/converter'
export * from './modules/sync'
export * from './modules/codex'
export * from './modules/gemini'
export * from './modules/qwen'
export * from './modules/iflow'
export * from './modules/droid'
