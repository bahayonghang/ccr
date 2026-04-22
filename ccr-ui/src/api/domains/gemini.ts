/**
 * Gemini Domain —— Gemini CLI 配置 / MCP / Slash 命令 / Extensions API
 *
 * 真迁移自 tauri.ts 第 6 分组。对应后端 commands::gemini::* 命令。
 *
 * Gemini 平台后端不支持 Agents 与 Plugins，保留相应的桩函数返回"不支持"信号，
 * 业务方按统一接口调用不会报错。
 */

import { invoke } from '@tauri-apps/api/core'
import { resolveName, resolveNameAndConfig, type UnknownRecord } from '../_shared'

// ── Settings ──

/** 获取 Gemini 配置 */
export const getGeminiConfig = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('gemini_get_settings')
}

/** 更新 Gemini 配置 */
export const updateGeminiConfig = async <T = UnknownRecord>(settings: unknown): Promise<T> => {
  return invoke('gemini_update_settings', { settings })
}

// ── MCP ──

/** 列出 Gemini MCP 服务器 */
export const listGeminiMcpServers = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('gemini_list_mcp_servers')
}

/** 添加 Gemini MCP 服务器 */
export const addGeminiMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown,
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('gemini_add_mcp_server', { name, config: resolvedConfig })
}

/** 更新 Gemini MCP 服务器 */
export const updateGeminiMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown,
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('gemini_update_mcp_server', { name, config: resolvedConfig })
}

/** 删除 Gemini MCP 服务器 */
export const deleteGeminiMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object,
): Promise<T> => {
  const name = resolveName(nameOrRequest)
  return invoke('gemini_delete_mcp_server', { name })
}

// ── Slash Commands ──

/** 列出 Gemini 斜杠命令 */
export const listGeminiSlashCommands = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('gemini_list_slash_commands')
}

/** 添加 Gemini 斜杠命令 */
export const addGeminiSlashCommand = async <T = UnknownRecord>(
  name: string,
  config: unknown,
): Promise<T> => {
  return invoke('gemini_add_slash_command', { name, config })
}

/** 更新 Gemini 斜杠命令 */
export const updateGeminiSlashCommand = async <T = UnknownRecord>(
  name: string,
  config: unknown,
): Promise<T> => {
  return invoke('gemini_update_slash_command', { name, config })
}

/** 删除 Gemini 斜杠命令 */
export const deleteGeminiSlashCommand = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('gemini_delete_slash_command', { name })
}

/** 切换 Gemini 斜杠命令启用/禁用 */
export const toggleGeminiSlashCommand = async <T = UnknownRecord>(
  name: string,
  enabled: boolean,
): Promise<T> => {
  return invoke('gemini_update_slash_command', { name, config: { enabled } })
}

// ── Extensions ──

/** 列出 Gemini Extensions */
export const listGeminiExtensions = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('gemini_list_extensions')
}

// ── 平台限制 —— 未实现能力的安全桩 ──

/** 列出 Gemini Agents（后端暂不支持） */
export const listGeminiAgents = async <T = UnknownRecord>(): Promise<T> => {
  return { agents: [] } as T
}

export const addGeminiAgent = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
  _config?: unknown,
): Promise<T> => {
  return { success: false, message: 'Gemini 平台暂不支持 Agents' } as T
}

export const updateGeminiAgent = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
  _config?: unknown,
): Promise<T> => {
  return { success: false, message: 'Gemini 平台暂不支持 Agents' } as T
}

export const deleteGeminiAgent = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
): Promise<T> => {
  return { success: false, message: 'Gemini 平台暂不支持 Agents' } as T
}

export const toggleGeminiAgent = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
  _enabled?: boolean,
): Promise<T> => {
  return { success: false, message: 'Gemini 平台暂不支持 Agents' } as T
}

export const listGeminiPlugins = async <T = UnknownRecord>(): Promise<T> => {
  return { plugins: [] } as T
}

export const addGeminiPlugin = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
  _config?: unknown,
): Promise<T> => {
  return { success: false, message: 'Gemini 平台暂不支持 Plugins' } as T
}

export const updateGeminiPlugin = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
  _config?: unknown,
): Promise<T> => {
  return { success: false, message: 'Gemini 平台暂不支持 Plugins' } as T
}

export const deleteGeminiPlugin = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
): Promise<T> => {
  return { success: false, message: 'Gemini 平台暂不支持 Plugins' } as T
}

export const toggleGeminiPlugin = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
  _enabled?: boolean,
): Promise<T> => {
  return { success: false, message: 'Gemini 平台暂不支持 Plugins' } as T
}
