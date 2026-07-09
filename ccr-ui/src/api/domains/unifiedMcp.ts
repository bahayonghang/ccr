/**
 * Unified MCP Domain —— 跨平台 MCP 服务器统一管理 API
 *
 * 对应后端 commands::unified_mcp::* 命令。
 * 真迁移自 tauri.ts 第 17 分组。
 *
 * Claude update 走后端 merge 更新，避免 toggle/update 丢失原始 command/args/env。
 * 其它平台暂保留 delete + add 兼容路径。
 */

import { invoke } from '@tauri-apps/api/core'
import { getErrorMessage } from '@/utils/errorHandler'
import { asRecord, type UnknownRecord } from '../_shared'

export interface UnifiedMcpImportResult {
  name: string
  ok: boolean
  message?: string
  error?: string
}

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

/** 批量导入：逐条调用 add API，保留每条成功/失败结果。 */
export const importUnifiedMcpServers = async (
  requests: unknown[],
): Promise<UnifiedMcpImportResult[]> => {
  const results: UnifiedMcpImportResult[] = []
  for (const request of requests) {
    const record = asRecord(request)
    const name = typeof record.name === 'string' ? record.name : '(unnamed)'
    try {
      const response = asRecord(await addUnifiedMcp<UnknownRecord>(request))
      results.push({
        name,
        ok: true,
        message: typeof response.message === 'string' ? response.message : undefined,
      })
    } catch (err) {
      results.push({
        name,
        ok: false,
        error: getErrorMessage(err),
      })
    }
  }
  return results
}

/**
 * 更新统一 MCP 服务器。
 *
 * Claude 使用后端 merge 更新；其它平台保留 delete + add 兼容路径。
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

  if (mergedRequest.platform === 'claude') {
    return invoke('unified_update_mcp_server', {
      platform: mergedRequest.platform,
      name: mergedRequest.name,
      request: mergedRequest,
    })
  }

  try {
    await invoke('unified_delete_mcp_server', {
      platform: mergedRequest.platform,
      name: mergedRequest.name,
      scope: mergedRequest.scope,
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
  scope?: string,
): Promise<T> => {
  return invoke('unified_delete_mcp_server', { platform, name, scope })
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
  scope?: string,
): Promise<T> => {
  if (platform === 'claude') {
    return invoke('claude_update_mcp_server', { name, config: { disabled: disabled ?? true }, scope })
  }
  throw new Error(`[Tauri] toggleUnifiedMcp: 平台 ${platform} 不支持启用/禁用切换`)
}
