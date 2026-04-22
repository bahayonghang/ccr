/**
 * OpenCode Domain —— OpenCode CLI 配置 / TUI / 键位 / 主题 / Providers / MCP / Agents / Commands / Plugins API
 *
 * 真迁移自 tauri.ts 第 10 分组。对应后端 commands::opencode::* 命令。
 *
 * Providers / MCP / Plugins 在后端没有独立命令，全部通过 settings 读写：
 * 前端解析 `provider` / `mcp` / `plugin` 字段（兼容 legacy `providers` / `mcpServers` / `plugins`），
 * 修改后调用 `opencode_update_settings` 回写。
 */

import { invoke } from '@tauri-apps/api/core'
import {
  asArray,
  asRecord,
  pickRecord,
  resolveNameAndConfig,
  type UnknownRecord,
} from '../_shared'

// ── Settings ──

/** 获取 OpenCode 配置 */
export const getOpenCodeConfig = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('opencode_get_settings')
}

/** 更新 OpenCode 配置 */
export const updateOpenCodeConfig = async <T = UnknownRecord>(settings: unknown): Promise<T> => {
  return invoke('opencode_update_settings', { settings })
}

/** 获取 OpenCode TUI 配置 */
export const getOpenCodeTuiSettings = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('opencode_get_tui_settings')
}

/** 更新 OpenCode TUI 配置 */
export const updateOpenCodeTuiSettings = async <T = UnknownRecord>(
  settings: unknown,
): Promise<T> => {
  return invoke('opencode_update_tui_settings', { settings })
}

/** 获取 OpenCode 快捷键 */
export const getOpenCodeKeybindings = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('opencode_get_keybindings')
}

/** 更新 OpenCode 快捷键 */
export const updateOpenCodeKeybindings = async <T = UnknownRecord>(
  keybindings: unknown,
): Promise<T> => {
  return invoke('opencode_update_keybindings', { keybindings })
}

/** 列出 OpenCode 主题 */
export const listOpenCodeThemes = async <T = UnknownRecord[]>(): Promise<T> => {
  return invoke('opencode_list_themes')
}

// ── Internal normalizers：兼容新旧字段 ──

function normalizedOpenCodeProviders(settings: UnknownRecord) {
  const providers = pickRecord(settings, 'provider')
  if (Object.keys(providers).length > 0) {
    return Object.entries(providers).map(([id, config]) => ({ id, ...asRecord(config) }))
  }
  const legacyProviders = pickRecord(settings, 'providers')
  return Object.entries(legacyProviders).map(([id, config]) => ({ id, ...asRecord(config) }))
}

function normalizedOpenCodeMcpServers(settings: UnknownRecord) {
  const servers = pickRecord(settings, 'mcp')
  if (Object.keys(servers).length > 0) {
    return Object.entries(servers).map(([id, config]) => ({ id, ...asRecord(config) }))
  }
  const legacyServers = asRecord(settings.mcpServers ?? settings.mcp_servers)
  return Object.entries(legacyServers).map(([id, config]) => ({ id, ...asRecord(config) }))
}

function normalizedOpenCodePlugins(settings: UnknownRecord): string[] {
  const pluginList = asArray(settings.plugin).filter(
    (item): item is string => typeof item === 'string',
  )
  if (pluginList.length > 0) return [...new Set(pluginList)]

  const legacyPlugins = pickRecord(settings, 'plugins')
  if (Object.keys(legacyPlugins).length > 0) {
    return Object.keys(legacyPlugins)
  }

  return []
}

// ── Providers（通过 settings.provider 读写） ──

/** 列出 OpenCode Providers */
export const listOpenCodeProviders = async <T = UnknownRecord[]>(): Promise<T> => {
  const settings = await getOpenCodeConfig<UnknownRecord>()
  return normalizedOpenCodeProviders(settings) as T
}

/** 添加 OpenCode Provider */
export const addOpenCodeProvider = async <T = UnknownRecord>(
  providerOrName: string | object,
  config?: unknown,
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(providerOrName, config)
  const settings = await getOpenCodeConfig<UnknownRecord>()
  const providers = pickRecord(settings, 'provider')
  providers[name] = resolvedConfig
  await updateOpenCodeConfig({ provider: providers })
  return { id: name, ...resolvedConfig } as T
}

/** 更新 OpenCode Provider */
export const updateOpenCodeProvider = async <T = UnknownRecord>(
  providerOrName: string | object,
  config?: unknown,
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(providerOrName, config)
  const settings = await getOpenCodeConfig<UnknownRecord>()
  const providers = pickRecord(settings, 'provider')
  providers[name] = { ...asRecord(providers[name]), ...resolvedConfig }
  await updateOpenCodeConfig({ provider: providers })
  return { id: name, ...asRecord(providers[name]) } as T
}

/** 删除 OpenCode Provider */
export const deleteOpenCodeProvider = async <T = UnknownRecord>(name: string): Promise<T> => {
  const settings = await getOpenCodeConfig<UnknownRecord>()
  const providers = pickRecord(settings, 'provider')
  delete providers[name]
  await updateOpenCodeConfig({ provider: providers })
  return name as T
}

// ── MCP（通过 settings.mcp 读写） ──

/** 列出 OpenCode MCP 服务器 */
export const listOpenCodeMcpServers = async <T = UnknownRecord[]>(): Promise<T> => {
  const settings = await getOpenCodeConfig<UnknownRecord>()
  return normalizedOpenCodeMcpServers(settings) as T
}

/** 添加 OpenCode MCP 服务器 */
export const addOpenCodeMcpServer = async <T = UnknownRecord>(
  serverOrName: string | object,
  config?: unknown,
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(serverOrName, config)
  const settings = await getOpenCodeConfig<UnknownRecord>()
  const servers = pickRecord(settings, 'mcp')
  servers[name] = resolvedConfig
  await updateOpenCodeConfig({ mcp: servers })
  return { id: name, ...resolvedConfig } as T
}

/** 更新 OpenCode MCP 服务器 */
export const updateOpenCodeMcpServer = async <T = UnknownRecord>(
  serverOrName: string | object,
  config?: unknown,
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(serverOrName, config)
  const settings = await getOpenCodeConfig<UnknownRecord>()
  const servers = pickRecord(settings, 'mcp')
  servers[name] = { ...asRecord(servers[name]), ...resolvedConfig }
  await updateOpenCodeConfig({ mcp: servers })
  return { id: name, ...asRecord(servers[name]) } as T
}

/** 删除 OpenCode MCP 服务器 */
export const deleteOpenCodeMcpServer = async <T = UnknownRecord>(name: string): Promise<T> => {
  const settings = await getOpenCodeConfig<UnknownRecord>()
  const servers = pickRecord(settings, 'mcp')
  delete servers[name]
  await updateOpenCodeConfig({ mcp: servers })
  return name as T
}

// ── Agents ──

export const listOpenCodeAgents = async <T = UnknownRecord[]>(): Promise<T> => {
  return invoke('opencode_list_agents')
}

export const addOpenCodeAgent = async <T = UnknownRecord>(config: unknown): Promise<T> => {
  return invoke('opencode_add_agent', { config })
}

export const updateOpenCodeAgent = async <T = UnknownRecord>(config: unknown): Promise<T> => {
  return invoke('opencode_update_agent', { config })
}

export const deleteOpenCodeAgent = async <T = UnknownRecord>(
  name: string,
  context?: unknown,
): Promise<T> => {
  return invoke('opencode_delete_agent', { name, context })
}

// ── Commands ──

export const listOpenCodeCommands = async <T = UnknownRecord[]>(): Promise<T> => {
  return invoke('opencode_list_commands')
}

export const addOpenCodeCommand = async <T = UnknownRecord>(config: unknown): Promise<T> => {
  return invoke('opencode_add_command', { config })
}

export const updateOpenCodeCommand = async <T = UnknownRecord>(config: unknown): Promise<T> => {
  return invoke('opencode_update_command', { config })
}

export const deleteOpenCodeCommand = async <T = UnknownRecord>(
  name: string,
  context?: unknown,
): Promise<T> => {
  return invoke('opencode_delete_command', { name, context })
}

// ── Plugins（通过 settings.plugin 读写，兼容 legacy `plugins` 对象） ──

export const listOpenCodePlugins = async <T = string[]>(): Promise<T> => {
  const settings = await getOpenCodeConfig<UnknownRecord>()
  return normalizedOpenCodePlugins(settings) as T
}

export const addOpenCodePlugin = async <T = string[]>(
  pluginOrName: string | object,
): Promise<T> => {
  const name =
    typeof pluginOrName === 'string'
      ? pluginOrName
      : String(asRecord(pluginOrName).name ?? asRecord(pluginOrName).npm ?? '')

  const settings = await getOpenCodeConfig<UnknownRecord>()
  const nextPlugins = [...normalizedOpenCodePlugins(settings)]
  if (!nextPlugins.includes(name)) nextPlugins.push(name)
  await updateOpenCodeConfig({ plugin: nextPlugins })
  return nextPlugins as T
}

export const deleteOpenCodePlugin = async <T = string[]>(name: string): Promise<T> => {
  const settings = await getOpenCodeConfig<UnknownRecord>()
  const nextPlugins = normalizedOpenCodePlugins(settings).filter((item) => item !== name)
  await updateOpenCodeConfig({ plugin: nextPlugins })
  return nextPlugins as T
}

export const listOpenCodeLocalPlugins = async <T = UnknownRecord[]>(): Promise<T> => {
  return invoke('opencode_list_local_plugins')
}

export const listOpenCodeSkillLocations = async <T = UnknownRecord[]>(): Promise<T> => {
  return invoke('opencode_list_skill_locations')
}
