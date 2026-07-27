/**
 * OpenCode Domain —— OpenCode CLI 配置 / TUI / 键位 / 主题 / Providers / MCP / Agents / Commands / Plugins API
 *
 * 真迁移自 tauri.ts 第 10 分组。对应后端 commands::opencode::* 命令。
 *
 * Providers / MCP / Plugins 在后端没有独立命令，全部通过 settings 读写：
 * 前端解析 `provider` / `mcp` / `plugin` 字段（兼容 legacy `providers` / `mcpServers` / `plugins`），
 * 修改后调用 `opencode_update_settings` 回写。
 */

import {
  asArray,
  asRecord,
  pickRecord,
  resolveNameAndConfig,
  toOpenJsonValue as asOpenJson,
  type UnknownRecord,
} from '../_shared'
import {
  addOpenCodeAgent as addOpenCodeAgentGenerated,
  addOpenCodeCommand as addOpenCodeCommandGenerated,
  deleteOpenCodeAgent as deleteOpenCodeAgentGenerated,
  deleteOpenCodeCommand as deleteOpenCodeCommandGenerated,
  getOpenCodeKeybindings as getOpenCodeKeybindingsGenerated,
  getOpenCodeSettings,
  getOpenCodeTuiSettings as getOpenCodeTuiSettingsGenerated,
  listOpenCodeAgents as listOpenCodeAgentsGenerated,
  listOpenCodeCommands as listOpenCodeCommandsGenerated,
  listOpenCodeLocalPlugins as listOpenCodeLocalPluginsGenerated,
  listOpenCodeThemes as listOpenCodeThemesGenerated,
  updateOpenCodeAgent as updateOpenCodeAgentGenerated,
  updateOpenCodeCommand as updateOpenCodeCommandGenerated,
  updateOpenCodeKeybindings as updateOpenCodeKeybindingsGenerated,
  updateOpenCodeSettings,
  updateOpenCodeTuiSettings as updateOpenCodeTuiSettingsGenerated,
} from '../generated/openCode'
import type {
  OpenCodeAgent,
  OpenCodeAgentRequest,
  OpenCodeCommand,
  OpenCodeCommandRequest,
  OpenCodeConfig,
  OpenCodeLocalPluginFile,
  OpenCodeMcpServer,
  OpenCodeProviderConfig,
  OpenCodeTheme,
  OpenCodeTuiConfig,
} from '@/types/opencode'

const optionalString = (value: unknown): string | undefined =>
  typeof value === 'string' ? value : undefined
const optionalNumber = (value: unknown): number | undefined =>
  typeof value === 'number' ? value : undefined
const optionalBoolean = (value: unknown): boolean | undefined =>
  typeof value === 'boolean' ? value : undefined
const optionalRecord = (value: unknown): UnknownRecord | undefined =>
  typeof value === 'object' && value !== null && !Array.isArray(value) ? asRecord(value) : undefined

const openCodeAgentFrom = (value: unknown): OpenCodeAgent | null => {
  const source = asRecord(value)
  if (
    typeof source.name !== 'string'
    || typeof source.path !== 'string'
    || (source.scope !== 'global' && source.scope !== 'project')
    || typeof source.body !== 'string'
  ) return null

  const mode = source.mode === 'primary' || source.mode === 'subagent' || source.mode === 'all'
    ? source.mode
    : undefined
  return {
    name: source.name,
    path: source.path,
    scope: source.scope,
    description: optionalString(source.description),
    mode,
    model: optionalString(source.model),
    temperature: optionalNumber(source.temperature),
    topP: optionalNumber(source.topP),
    steps: optionalNumber(source.steps),
    hidden: optionalBoolean(source.hidden),
    disable: optionalBoolean(source.disable),
    color: optionalString(source.color),
    permission: optionalRecord(source.permission),
    tools: optionalRecord(source.tools),
    body: source.body,
    other: optionalRecord(source.other),
    parseError: optionalString(source.parseError),
  }
}

const openCodeCommandFrom = (value: unknown): OpenCodeCommand | null => {
  const source = asRecord(value)
  if (
    typeof source.name !== 'string'
    || typeof source.path !== 'string'
    || (source.scope !== 'global' && source.scope !== 'project')
    || typeof source.template !== 'string'
  ) return null

  return {
    name: source.name,
    path: source.path,
    scope: source.scope,
    description: optionalString(source.description),
    agent: optionalString(source.agent),
    subtask: optionalBoolean(source.subtask),
    model: optionalString(source.model),
    template: source.template,
    other: optionalRecord(source.other),
    parseError: optionalString(source.parseError),
  }
}

// ── Settings ──

/** 获取 OpenCode 配置 */
export const getOpenCodeConfig = async (): Promise<OpenCodeConfig> => {
  return asRecord(await getOpenCodeSettings()) as OpenCodeConfig
}

/** 更新 OpenCode 配置 */
export const updateOpenCodeConfig = async (settings: UnknownRecord): Promise<OpenCodeConfig> => {
  return asRecord(await updateOpenCodeSettings(asOpenJson(settings))) as OpenCodeConfig
}

/** 获取 OpenCode TUI 配置 */
export const getOpenCodeTuiSettings = async (): Promise<OpenCodeTuiConfig> => {
  return asRecord(await getOpenCodeTuiSettingsGenerated()) as OpenCodeTuiConfig
}

/** 更新 OpenCode TUI 配置 */
export const updateOpenCodeTuiSettings = async (
  settings: UnknownRecord,
): Promise<OpenCodeTuiConfig> => {
  return asRecord(await updateOpenCodeTuiSettingsGenerated(asOpenJson(settings))) as OpenCodeTuiConfig
}

/** 获取 OpenCode 快捷键 */
export const getOpenCodeKeybindings = async (): Promise<UnknownRecord> => {
  return asRecord(await getOpenCodeKeybindingsGenerated())
}

/** 更新 OpenCode 快捷键 */
export const updateOpenCodeKeybindings = async (
  keybindings: UnknownRecord,
): Promise<UnknownRecord> => {
  return asRecord(await updateOpenCodeKeybindingsGenerated(asOpenJson(keybindings)))
}

/** 列出 OpenCode 主题 */
export const listOpenCodeThemes = async (): Promise<OpenCodeTheme[]> => {
  return (await listOpenCodeThemesGenerated()).flatMap((theme) => {
    if (theme.themeType !== 'light' && theme.themeType !== 'dark' && theme.themeType !== 'system') {
      return []
    }
    return [{ ...theme, themeType: theme.themeType }]
  })
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
export const listOpenCodeProviders = async (): Promise<OpenCodeProviderConfig[]> => {
  const settings = await getOpenCodeConfig()
  return normalizedOpenCodeProviders(settings) as OpenCodeProviderConfig[]
}

/** 添加 OpenCode Provider */
export const addOpenCodeProvider = async (
  providerOrName: string | object,
  config?: unknown,
): Promise<OpenCodeProviderConfig> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(providerOrName, config)
  const settings = await getOpenCodeConfig()
  const providers = pickRecord(settings, 'provider')
  providers[name] = resolvedConfig
  await updateOpenCodeConfig({ provider: providers })
  return { id: name, ...resolvedConfig }
}

/** 更新 OpenCode Provider */
export const updateOpenCodeProvider = async (
  providerOrName: string | object,
  config?: unknown,
): Promise<OpenCodeProviderConfig> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(providerOrName, config)
  const settings = await getOpenCodeConfig()
  const providers = pickRecord(settings, 'provider')
  providers[name] = { ...asRecord(providers[name]), ...resolvedConfig }
  await updateOpenCodeConfig({ provider: providers })
  return { id: name, ...asRecord(providers[name]) }
}

/** 删除 OpenCode Provider */
export const deleteOpenCodeProvider = async (name: string): Promise<string> => {
  const settings = await getOpenCodeConfig()
  const providers = pickRecord(settings, 'provider')
  delete providers[name]
  await updateOpenCodeConfig({ provider: providers })
  return name
}

// ── MCP（通过 settings.mcp 读写） ──

/** 列出 OpenCode MCP 服务器 */
export const listOpenCodeMcpServers = async (): Promise<OpenCodeMcpServer[]> => {
  const settings = await getOpenCodeConfig()
  return normalizedOpenCodeMcpServers(settings) as OpenCodeMcpServer[]
}

/** 添加 OpenCode MCP 服务器 */
export const addOpenCodeMcpServer = async (
  serverOrName: string | object,
  config?: unknown,
): Promise<OpenCodeMcpServer> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(serverOrName, config)
  const settings = await getOpenCodeConfig()
  const servers = pickRecord(settings, 'mcp')
  servers[name] = resolvedConfig
  await updateOpenCodeConfig({ mcp: servers })
  return { id: name, ...resolvedConfig } as OpenCodeMcpServer
}

/** 更新 OpenCode MCP 服务器 */
export const updateOpenCodeMcpServer = async (
  serverOrName: string | object,
  config?: unknown,
): Promise<OpenCodeMcpServer> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(serverOrName, config)
  const settings = await getOpenCodeConfig()
  const servers = pickRecord(settings, 'mcp')
  servers[name] = { ...asRecord(servers[name]), ...resolvedConfig }
  await updateOpenCodeConfig({ mcp: servers })
  return { id: name, ...asRecord(servers[name]) } as OpenCodeMcpServer
}

/** 删除 OpenCode MCP 服务器 */
export const deleteOpenCodeMcpServer = async (name: string): Promise<string> => {
  const settings = await getOpenCodeConfig()
  const servers = pickRecord(settings, 'mcp')
  delete servers[name]
  await updateOpenCodeConfig({ mcp: servers })
  return name
}

// ── Agents ──

export const listOpenCodeAgents = async (): Promise<OpenCodeAgent[]> => {
  return asArray(await listOpenCodeAgentsGenerated())
    .map(openCodeAgentFrom)
    .filter((value): value is OpenCodeAgent => value !== null)
}

export const addOpenCodeAgent = async (config: OpenCodeAgentRequest): Promise<OpenCodeAgent> => {
  const agent = openCodeAgentFrom(await addOpenCodeAgentGenerated(asOpenJson(config)))
  if (!agent) throw new Error('OpenCode agent response is invalid')
  return agent
}

export const updateOpenCodeAgent = async (config: OpenCodeAgentRequest): Promise<OpenCodeAgent> => {
  const agent = openCodeAgentFrom(await updateOpenCodeAgentGenerated(asOpenJson(config)))
  if (!agent) throw new Error('OpenCode agent response is invalid')
  return agent
}

export const deleteOpenCodeAgent = async (
  name: string,
  context?: UnknownRecord,
): Promise<string> => {
  return deleteOpenCodeAgentGenerated(name, context ? asOpenJson(context) : undefined)
}

// ── Commands ──

export const listOpenCodeCommands = async (): Promise<OpenCodeCommand[]> => {
  return asArray(await listOpenCodeCommandsGenerated())
    .map(openCodeCommandFrom)
    .filter((value): value is OpenCodeCommand => value !== null)
}

export const addOpenCodeCommand = async (
  config: OpenCodeCommandRequest,
): Promise<OpenCodeCommand> => {
  const command = openCodeCommandFrom(await addOpenCodeCommandGenerated(asOpenJson(config)))
  if (!command) throw new Error('OpenCode command response is invalid')
  return command
}

export const updateOpenCodeCommand = async (
  config: OpenCodeCommandRequest,
): Promise<OpenCodeCommand> => {
  const command = openCodeCommandFrom(await updateOpenCodeCommandGenerated(asOpenJson(config)))
  if (!command) throw new Error('OpenCode command response is invalid')
  return command
}

export const deleteOpenCodeCommand = async (
  name: string,
  context?: UnknownRecord,
): Promise<string> => {
  return deleteOpenCodeCommandGenerated(name, context ? asOpenJson(context) : undefined)
}

// ── Plugins（通过 settings.plugin 读写，兼容 legacy `plugins` 对象） ──

export const listOpenCodePlugins = async (): Promise<string[]> => {
  const settings = await getOpenCodeConfig()
  return normalizedOpenCodePlugins(settings)
}

export const addOpenCodePlugin = async (
  pluginOrName: string | object,
): Promise<string[]> => {
  const name =
    typeof pluginOrName === 'string'
      ? pluginOrName
      : String(asRecord(pluginOrName).name ?? asRecord(pluginOrName).npm ?? '')

  const settings = await getOpenCodeConfig()
  const nextPlugins = [...normalizedOpenCodePlugins(settings)]
  if (!nextPlugins.includes(name)) nextPlugins.push(name)
  await updateOpenCodeConfig({ plugin: nextPlugins })
  return nextPlugins
}

export const deleteOpenCodePlugin = async (name: string): Promise<string[]> => {
  const settings = await getOpenCodeConfig()
  const nextPlugins = normalizedOpenCodePlugins(settings).filter((item) => item !== name)
  await updateOpenCodeConfig({ plugin: nextPlugins })
  return nextPlugins
}

export const listOpenCodeLocalPlugins = async (): Promise<OpenCodeLocalPluginFile[]> => {
  return (await listOpenCodeLocalPluginsGenerated()).flatMap((plugin) => {
    if (plugin.scope !== 'global' && plugin.scope !== 'project') return []
    return [{ ...plugin, scope: plugin.scope }]
  })
}
