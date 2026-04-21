/**
 * Config Domain —— 配置管理 API
 *
 * 对应后端 commands::config::* 命令。
 * 真迁移自 tauri.ts 第 2 分组（配置管理）与 HTTP-only 桩函数中的 updateConfig / cleanBackups。
 *
 * 业务代码可从以下三种路径任选其一：
 *   - `import { listConfigs } from '@/api/tauri'` （向后兼容，tauri.ts 会 re-export）
 *   - `import { listConfigs } from '@/api/domains/config'`（域直接）
 *   - `import { configApi } from '@/api'` 然后 `configApi.listConfigs()`（命名空间）
 */

import { invoke } from '@tauri-apps/api/core'
import { asRecord, type UnknownRecord } from '../_shared'

/** 列出所有配置（包装为 { configs: [...] } 格式供前端消费） */
export const listConfigs = async <T = UnknownRecord>(): Promise<T> => {
  const configs = await invoke('list_configs')
  return { configs } as T
}

/** 切换到指定配置 */
export const switchConfig = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('switch_config', { name })
}

/** 添加新配置（兼容 addConfig(name, config) 与 addConfig({name,...})） */
export const addConfig = async <T = UnknownRecord>(
  nameOrData: string | object,
  config?: unknown,
): Promise<T> => {
  if (typeof nameOrData === 'string') {
    return invoke('add_config', { name: nameOrData, config })
  }
  const data = asRecord(nameOrData)
  const { name, ...rest } = data
  return invoke('add_config', { name, config: rest })
}

/** 更新配置 */
export const updateConfig = async <T = UnknownRecord>(
  name: string,
  config: unknown,
): Promise<T> => {
  return invoke('update_config', { name, data: config })
}

/** 删除指定配置 */
export const deleteConfig = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('delete_config', { name })
}

/** 重命名配置 */
export const renameConfig = async <T = UnknownRecord>(
  oldName: string,
  newName: string,
): Promise<T> => {
  return invoke('rename_config', { oldName, newName })
}

/** 复制配置 */
export const duplicateConfig = async <T = UnknownRecord>(
  name: string,
  newName: string,
): Promise<T> => {
  return invoke('duplicate_config', { name, newName })
}

/** 验证所有配置 */
export const validateConfigs = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('validate_configs')
}

/** 导入配置 */
export const importConfig = async <T = UnknownRecord>(data: unknown): Promise<T> => {
  return invoke('import_config', { data })
}

/** 导出配置 */
export const exportConfig = async <T = UnknownRecord>(name?: string): Promise<T> => {
  return invoke('export_config', { name })
}

/** 获取历史记录（包装为 { entries: [...] } 格式供前端消费） */
export const getHistory = async <T = UnknownRecord>(limit?: number): Promise<T> => {
  const entries = await invoke('get_history', { limit: limit ?? 100 })
  return { entries } as T
}

/** 清理历史记录 */
export const clearHistory = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('clear_history')
}

/** 清理备份（参数 _days 保留作向后兼容签名，后端未使用） */
export const cleanBackups = async <T = UnknownRecord>(_days?: number): Promise<T> => {
  return invoke('clean_backups')
}
