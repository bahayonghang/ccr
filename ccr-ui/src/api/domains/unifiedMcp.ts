/**
 * Unified MCP Domain —— 跨平台 MCP 服务器统一管理 API
 *
 * 对应后端 commands::unified_mcp::* 命令。
 * 真迁移自 tauri.ts 第 17 分组。
 *
 * update 走 "delete + add" 组合以复用后端命令；toggle 当前仅支持 Claude 平台。
 */

import { invoke } from '@tauri-apps/api/core'
import { asRecord, type UnknownRecord } from '../_shared'

/** 列出所有平台的 MCP 服务器（统一视图） */
export const listUnifiedMcp = async <T = UnknownRecord>(
  platforms?: string[] | string,
): Promise<T> => {
  const normalized = typeof platforms === 'string' ? [platforms] : platforms
  return invoke('unified_list_mcp_servers', { platforms: normalized })
}

/** 添加统一 MCP 服务器 */
export const addUnifiedMcp = async <T = UnknownRecord>(request: unknown): Promise<T> => {
  return invoke('unified_add_mcp_server', { request })
}

/**
 * 更新统一 MCP 服务器（删除 + 添加两步）。
 *
 * 后端没有独立的 update 命令，删除失败默认视为"原本不存在"并忽略。
 */
export const updateUnifiedMcp = async <T = UnknownRecord>(
  platformOrRequest: string | object,
  name?: string,
  request?: unknown,
): Promise<T> => {
  const requestRecord = asRecord(request)
  const mergedRequest =
    typeof platformOrRequest === 'string'
      ? { ...requestRecord, platform: platformOrRequest, name }
      : asRecord(platformOrRequest)

  try {
    await invoke('unified_delete_mcp_server', {
      platform: mergedRequest.platform,
      name: mergedRequest.name,
    })
  } catch {
    // 删除失败默认视为原本不存在，继续添加
  }

  return invoke('unified_add_mcp_server', { request: mergedRequest })
}

/** 删除统一 MCP 服务器 */
export const deleteUnifiedMcp = async <T = UnknownRecord>(
  platform: string,
  name: string,
): Promise<T> => {
  return invoke('unified_delete_mcp_server', { platform, name })
}

/**
 * 切换统一 MCP 服务器启用/禁用。
 *
 * 当前仅 Claude 平台后端实现了 disabled 字段语义；其它平台抛错提示不支持。
 */
export const toggleUnifiedMcp = async <T = UnknownRecord>(
  platform: string,
  name: string,
  disabled?: boolean,
): Promise<T> => {
  if (platform === 'claude') {
    return invoke('claude_update_mcp_server', { name, config: { disabled: disabled ?? true } })
  }
  throw new Error(`[Tauri] toggleUnifiedMcp: 平台 ${platform} 不支持启用/禁用切换`)
}
