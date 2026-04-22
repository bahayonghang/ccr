/**
 * Droid Domain —— Droid CLI 配置 / MCP / Agents / Plugins / 模型 / Profiles API
 *
 * 真迁移自 tauri.ts 第 7 分组。对应后端 commands::droid::* 命令。
 *
 * Droid 的 Models 与 Profiles 在后端没有独立命令，全部通过 settings 读写实现：
 * 前端维护 profiles/customModels 的数组/对象结构并最终调用 `droid_update_settings`。
 */

import { invoke } from '@tauri-apps/api/core'
import {
  asRecord,
  isRecord,
  pickArray,
  resolveName,
  resolveNameAndConfig,
  type UnknownRecord,
} from '../_shared'
import type { DroidPlugin } from '../tauri'

// ── Settings ──

/** 获取 Droid 设置 */
export const getDroidSettings = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('droid_get_settings')
}

/** 更新 Droid 设置 */
export const updateDroidSettings = async <T = UnknownRecord>(settings: unknown): Promise<T> => {
  return invoke('droid_update_settings', { settings })
}

// ── MCP ──

/** 列出 Droid MCP 服务器 */
export const listDroidMcpServers = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('droid_list_mcp_servers')
}

/** 添加 Droid MCP 服务器 */
export const addDroidMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown,
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('droid_add_mcp_server', { name, config: resolvedConfig })
}

/** 更新 Droid MCP 服务器 */
export const updateDroidMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown,
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('droid_update_mcp_server', { name, config: resolvedConfig })
}

/** 删除 Droid MCP 服务器 */
export const deleteDroidMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object,
): Promise<T> => {
  const name = resolveName(nameOrRequest)
  return invoke('droid_delete_mcp_server', { name })
}

// ── Agents ──

/** 列出 Droid Agents */
export const listDroidAgents = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('droid_list_agents')
}

/** 获取 Droid Agent 详情（从列表过滤） */
export const getDroidAgent = async <T = UnknownRecord>(name: string): Promise<T> => {
  const agents = asRecord(await invoke<unknown>('droid_list_agents'))
  const target = asRecord(agents[name])
  if (Object.keys(target).length > 0) {
    return { name, ...target } as T
  }
  return null as T
}

/** 添加 Droid Agent */
export const addDroidAgent = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown,
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('droid_add_agent', { name, config: resolvedConfig })
}

/** 更新 Droid Agent */
export const updateDroidAgent = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown,
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('droid_update_agent', { name, config: resolvedConfig })
}

/** 删除 Droid Agent */
export const deleteDroidAgent = async <T = UnknownRecord>(
  nameOrRequest: string | object,
): Promise<T> => {
  const name = resolveName(nameOrRequest)
  return invoke('droid_delete_agent', { name })
}

// ── Plugins ──

/** 列出 Droid 插件（兼容数组 / `{ plugins: [] }` 两种后端返回形态） */
export const listDroidPlugins = async (): Promise<DroidPlugin[]> => {
  const result = await invoke<unknown>('droid_list_plugins')
  if (Array.isArray(result)) {
    return result as DroidPlugin[]
  }
  const plugins = pickArray(result, 'plugins')
  if (plugins.length > 0) {
    return plugins as DroidPlugin[]
  }
  return []
}

/** 添加 Droid 插件 */
export const addDroidPlugin = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown,
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('droid_add_plugin', { name, config: resolvedConfig })
}

/** 更新 Droid 插件 */
export const updateDroidPlugin = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown,
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('droid_update_plugin', { name, config: resolvedConfig })
}

/** 删除 Droid 插件 */
export const deleteDroidPlugin = async <T = UnknownRecord>(
  nameOrRequest: string | object,
): Promise<T> => {
  const name = resolveName(nameOrRequest)
  return invoke('droid_delete_plugin', { name })
}

// ── Slash Commands ──

/** 列出 Droid 斜杠命令 */
export const listDroidSlashCommands = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('droid_list_slash_commands')
}

/** 添加 Droid 斜杠命令 */
export const addDroidSlashCommand = async <T = UnknownRecord>(
  name: string,
  config: unknown,
): Promise<T> => {
  return invoke('droid_add_slash_command', { name, config })
}

/** 更新 Droid 斜杠命令 */
export const updateDroidSlashCommand = async <T = UnknownRecord>(
  name: string,
  config: unknown,
): Promise<T> => {
  return invoke('droid_update_slash_command', { name, config })
}

/** 删除 Droid 斜杠命令 */
export const deleteDroidSlashCommand = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('droid_delete_slash_command', { name })
}

// ── Models（通过 settings.customModels 读写） ──

/** 列出 Droid 模型 */
export const listDroidModels = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('droid_list_models')
}

/** 添加 Droid 模型（追加到 settings.customModels） */
export const addDroidModel = async <T = UnknownRecord>(
  model: Record<string, unknown>,
): Promise<T> => {
  const settings = await getDroidSettings<UnknownRecord>()
  const models = [...pickArray(settings, 'customModels')]

  if (models.some((item) => isRecord(item) && item.model === model.model)) {
    throw new Error(`模型 '${model?.model || ''}' 已存在`)
  }

  models.push(model)
  await updateDroidSettings({ customModels: models })
  return model as T
}

/** 更新 Droid 模型 */
export const updateDroidModel = async <T = UnknownRecord>(
  modelId: string,
  model: Record<string, unknown>,
): Promise<T> => {
  const settings = await getDroidSettings<UnknownRecord>()
  const models = [...pickArray(settings, 'customModels')]
  const index = models.findIndex((item) => isRecord(item) && item.model === modelId)

  if (index === -1) {
    throw new Error(`模型 '${modelId}' 不存在`)
  }

  models[index] = { ...asRecord(models[index]), ...model }
  await updateDroidSettings({ customModels: models })
  return models[index] as T
}

/** 删除 Droid 模型 */
export const deleteDroidModel = async (modelId: string): Promise<string> => {
  const settings = await getDroidSettings<UnknownRecord>()
  const models = [...pickArray(settings, 'customModels')]
  const nextModels = models.filter((item) => !isRecord(item) || item.model !== modelId)

  if (nextModels.length === models.length) {
    throw new Error(`模型 '${modelId}' 不存在`)
  }

  await updateDroidSettings({ customModels: nextModels })
  return modelId
}

// ── Profiles（通过 settings.profiles 读写；支持数组/对象两种存储形态） ──

const normalizeDroidProfiles = (profiles: unknown): Record<string, unknown> => {
  if (Array.isArray(profiles)) {
    return profiles.reduce((acc: Record<string, unknown>, profile) => {
      if (isRecord(profile) && profile.name) {
        const profileName = String(profile.name)
        const profileData = { ...profile }
        delete profileData.name
        acc[profileName] = profileData
      }
      return acc
    }, {})
  }

  if (profiles && typeof profiles === 'object') {
    return { ...profiles }
  }

  return {}
}

/** 列出 Droid Profiles */
export const listDroidProfiles = async (): Promise<unknown[]> => {
  const settings = await getDroidSettings<UnknownRecord>()
  const profiles = normalizeDroidProfiles(settings?.profiles)

  return Object.entries(profiles).map(([name, config]) => ({
    name,
    ...(config as Record<string, unknown>),
  }))
}

/** 添加 Droid Profile */
export const addDroidProfile = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown,
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  const settings = await getDroidSettings<UnknownRecord>()
  const profiles = normalizeDroidProfiles(settings?.profiles)

  if (profiles[name]) {
    throw new Error(`Profile '${name}' 已存在`)
  }

  profiles[name] = resolvedConfig
  await updateDroidSettings({ profiles })
  return { name, ...resolvedConfig } as T
}

/** 更新 Droid Profile */
export const updateDroidProfile = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown,
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  const settings = await getDroidSettings<UnknownRecord>()
  const profiles = normalizeDroidProfiles(settings?.profiles)

  if (!profiles[name]) {
    throw new Error(`Profile '${name}' 不存在`)
  }

  profiles[name] = { ...asRecord(profiles[name]), ...resolvedConfig }
  await updateDroidSettings({ profiles })
  return { name, ...asRecord(profiles[name]) } as T
}

/** 删除 Droid Profile */
export const deleteDroidProfile = async (nameOrRequest: string | object): Promise<string> => {
  const name = resolveName(nameOrRequest)
  const settings = await getDroidSettings<UnknownRecord>()
  const profiles = normalizeDroidProfiles(settings?.profiles)

  if (!profiles[name]) {
    throw new Error(`Profile '${name}' 不存在`)
  }

  delete profiles[name]
  await updateDroidSettings({ profiles })
  return name
}

/** 切换 Droid Profile（把 enabled 标记迁到目标 profile，并同步 currentProfile） */
export const switchDroidProfile = async <T = UnknownRecord>(name: string): Promise<T> => {
  const settings = await getDroidSettings<UnknownRecord>()
  const profiles = normalizeDroidProfiles(settings?.profiles)

  if (!profiles[name]) {
    throw new Error(`Profile '${name}' 不存在`)
  }

  Object.keys(profiles).forEach((profileName) => {
    profiles[profileName] = {
      ...asRecord(profiles[profileName]),
      enabled: profileName === name,
    }
  })

  await updateDroidSettings({ profiles, currentProfile: name })
  return { name, ...asRecord(profiles[name]) } as T
}
