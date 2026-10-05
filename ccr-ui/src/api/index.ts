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
export * as claudeApi from './domains/claude'
export * as codexApi from './domains/codex'
export * as grokApi from './domains/grok'
export * as syncApi from './domains/sync'
export * as platformApi from './domains/platforms'
export * as usageApi from './domains/usage'
export * as agentSessionsApi from './domains/agentSessions'
export * as systemApi from './domains/system'
export * as systemPromptsApi from './domains/systemPrompts'

// claudeObserver 对象已从 tauri.ts 门面迁至 domain 模块；具名 re-export 保住
// `import { claudeObserver } from '@/api'` 既有导入路径。
export { claudeObserver } from './domains/claudeObserver'
