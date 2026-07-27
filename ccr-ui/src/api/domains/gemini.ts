/**
 * Gemini Domain —— Antigravity CLI 配置 / MCP / Slash 命令 / Extensions API
 *
 * 真迁移自 tauri.ts 第 6 分组。对应后端 commands::gemini::* 命令。
 *
 * Antigravity/Gemini 平台后端不支持 Agents 与 Plugins，保留相应的桩函数返回"不支持"信号，
 * 业务方按统一接口调用不会报错。
 */

import {
  resolveName,
  resolveNameAndConfig,
  toOpenJsonValue,
  type UnknownRecord,
} from '../_shared'
import {
  addGeminiMcpServer as addGeminiMcpServerGenerated,
  addGeminiSlashCommand as addGeminiSlashCommandGenerated,
  deleteGeminiMcpServer as deleteGeminiMcpServerGenerated,
  deleteGeminiSlashCommand as deleteGeminiSlashCommandGenerated,
  getGeminiSettings,
  listGeminiExtensions as listGeminiExtensionsGenerated,
  listGeminiMcpServers as listGeminiMcpServersGenerated,
  listGeminiSlashCommands as listGeminiSlashCommandsGenerated,
  updateGeminiMcpServer as updateGeminiMcpServerGenerated,
  updateGeminiSettings,
  updateGeminiSlashCommand as updateGeminiSlashCommandGenerated,
} from '../generated/gemini'
import type { OpenJsonValueDto } from '@/types/generated/common/OpenJsonValueDto'
import type { GeminiMcpServer } from '@/types/gemini'

const mutationMessage = (value: OpenJsonValueDto): string => {
  if (value === null || Array.isArray(value) || typeof value !== 'object') return ''
  return typeof value.message === 'string' ? value.message : ''
}

const geminiMcpServerFrom = (value: OpenJsonValueDto): GeminiMcpServer | null => {
  if (value === null || Array.isArray(value) || typeof value !== 'object') return null
  if (typeof value.name !== 'string') return null

  const stringArray = (candidate: OpenJsonValueDto | undefined): string[] | undefined =>
    Array.isArray(candidate)
      ? candidate.filter((item): item is string => typeof item === 'string')
      : undefined
  const stringMap = (candidate: OpenJsonValueDto | undefined): Record<string, string> | undefined => {
    if (candidate === null || Array.isArray(candidate) || typeof candidate !== 'object') return undefined
    return Object.fromEntries(
      Object.entries(candidate).filter((entry): entry is [string, string] => typeof entry[1] === 'string'),
    )
  }

  return {
    name: value.name,
    command: typeof value.command === 'string' ? value.command : undefined,
    args: stringArray(value.args),
    env: stringMap(value.env),
    cwd: typeof value.cwd === 'string' ? value.cwd : undefined,
    timeout: typeof value.timeout === 'number' ? value.timeout : undefined,
    trust: typeof value.trust === 'boolean' ? value.trust : undefined,
    includeTools: stringArray(value.includeTools),
    url: typeof value.url === 'string' ? value.url : undefined,
  }
}

// ── Settings ──

/** 获取 Antigravity CLI 配置（内部 invoke 仍为 gemini） */
export const getGeminiConfig = async (): Promise<OpenJsonValueDto> => {
  return getGeminiSettings()
}

/** 更新 Antigravity CLI 配置（内部 invoke 仍为 gemini） */
export const updateGeminiConfig = async (settings: OpenJsonValueDto): Promise<OpenJsonValueDto> => {
  return updateGeminiSettings(settings)
}

// ── MCP ──

/** 列出 Antigravity MCP 服务器（内部 invoke 仍为 gemini） */
export const listGeminiMcpServers = async (): Promise<GeminiMcpServer[]> => {
  const value = await listGeminiMcpServersGenerated()
  if (!Array.isArray(value)) return []
  return value.map(geminiMcpServerFrom).filter((server): server is GeminiMcpServer => server !== null)
}

/** 添加 Antigravity MCP 服务器（内部 invoke 仍为 gemini） */
export const addGeminiMcpServer = async (
  nameOrRequest: string | object,
  config?: unknown,
): Promise<string> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return mutationMessage(await addGeminiMcpServerGenerated(
    name,
    toOpenJsonValue(resolvedConfig, 'Gemini MCP server payload'),
  ))
}

/** 更新 Antigravity MCP 服务器（内部 invoke 仍为 gemini） */
export const updateGeminiMcpServer = async (
  nameOrRequest: string | object,
  config?: unknown,
): Promise<string> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return mutationMessage(await updateGeminiMcpServerGenerated(
    name,
    toOpenJsonValue(resolvedConfig, 'Gemini MCP server payload'),
  ))
}

/** 删除 Antigravity MCP 服务器（内部 invoke 仍为 gemini） */
export const deleteGeminiMcpServer = async (
  nameOrRequest: string | object,
): Promise<string> => {
  const name = resolveName(nameOrRequest)
  return deleteGeminiMcpServerGenerated(name)
}

// ── Slash Commands ──

/** 列出 Antigravity/Gemini legacy 斜杠命令 */
export const listGeminiSlashCommands = async (): Promise<OpenJsonValueDto> => {
  return listGeminiSlashCommandsGenerated()
}

/** 添加 Antigravity/Gemini legacy 斜杠命令 */
export const addGeminiSlashCommand = async (
  name: string,
  config: unknown,
): Promise<string> => {
  return mutationMessage(await addGeminiSlashCommandGenerated(
    name,
    toOpenJsonValue(config, 'Gemini slash command payload'),
  ))
}

/** 更新 Antigravity/Gemini legacy 斜杠命令 */
export const updateGeminiSlashCommand = async (
  name: string,
  config: unknown,
): Promise<string> => {
  return mutationMessage(await updateGeminiSlashCommandGenerated(
    name,
    toOpenJsonValue(config, 'Gemini slash command payload'),
  ))
}

/** 删除 Antigravity/Gemini legacy 斜杠命令 */
export const deleteGeminiSlashCommand = async (name: string): Promise<string> => {
  return deleteGeminiSlashCommandGenerated(name)
}

/** 切换 Antigravity/Gemini legacy 斜杠命令启用/禁用 */
export const toggleGeminiSlashCommand = async (
  name: string,
  enabled: boolean,
): Promise<string> => {
  return mutationMessage(await updateGeminiSlashCommandGenerated(name, { enabled }))
}

// ── Extensions ──

/** 列出 Antigravity Extensions */
export const listGeminiExtensions = async (): Promise<OpenJsonValueDto> => {
  return listGeminiExtensionsGenerated()
}

// ── 平台限制 —— 未实现能力的安全桩 ──

/** 列出 Antigravity Agents（后端暂不支持） */
export const listGeminiAgents = async <T = UnknownRecord>(): Promise<T> => {
  return { agents: [] } as T
}

export const addGeminiAgent = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
  _config?: unknown,
): Promise<T> => {
  return { success: false, message: 'Antigravity 平台暂不支持 Agents' } as T
}

export const updateGeminiAgent = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
  _config?: unknown,
): Promise<T> => {
  return { success: false, message: 'Antigravity 平台暂不支持 Agents' } as T
}

export const deleteGeminiAgent = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
): Promise<T> => {
  return { success: false, message: 'Antigravity 平台暂不支持 Agents' } as T
}

export const toggleGeminiAgent = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
  _enabled?: boolean,
): Promise<T> => {
  return { success: false, message: 'Antigravity 平台暂不支持 Agents' } as T
}

export const listGeminiPlugins = async <T = UnknownRecord>(): Promise<T> => {
  return { plugins: [] } as T
}

export const addGeminiPlugin = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
  _config?: unknown,
): Promise<T> => {
  return { success: false, message: 'Antigravity 平台暂不支持 Plugins' } as T
}

export const updateGeminiPlugin = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
  _config?: unknown,
): Promise<T> => {
  return { success: false, message: 'Antigravity 平台暂不支持 Plugins' } as T
}

export const deleteGeminiPlugin = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
): Promise<T> => {
  return { success: false, message: 'Antigravity 平台暂不支持 Plugins' } as T
}

export const toggleGeminiPlugin = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
  _enabled?: boolean,
): Promise<T> => {
  return { success: false, message: 'Antigravity 平台暂不支持 Plugins' } as T
}
