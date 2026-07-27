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

import { invoke } from '@/api/invokeRuntime'
import { asRecord, type UnknownRecord } from '../_shared'
import {
  addConfigTyped,
  clearHistoryTyped,
  deleteConfigTyped,
  duplicateConfigTyped,
  exportConfigTyped,
  getHistoryTyped,
  importConfigTyped,
  listConfigsTyped,
  renameConfigTyped,
  restoreConfigTyped,
  switchConfigTyped,
  validateConfigsTyped,
  type AddConfigInput,
} from '../generated/config'
import type { ConfigListResponse, HistoryResponse } from '@/types/config'
import type { ExportResult } from '@/types/generated/config/ExportResult'
import type { ImportResult } from '@/types/generated/config/ImportResult'

interface ImportConfigPayload {
  content?: string
  mode?: 'merge' | 'replace' | string
  backup?: boolean
}

/** 列出所有配置（包装为 { configs: [...] } 格式供前端消费） */
export const listConfigs = async (): Promise<ConfigListResponse> => {
  const configs = await listConfigsTyped()
  return {
    configs,
    current_config: configs.find((config) => config.is_current)?.name ?? '',
    default_config: configs.find((config) => config.is_default)?.name ?? '',
  }
}

/** 切换到指定配置 */
export const switchConfig = switchConfigTyped

/** 添加新配置（兼容 addConfig(name, config) 与 addConfig({name,...})） */
export const addConfig = async (
  nameOrData: string | object,
  config?: unknown,
): Promise<string> => {
  const data = typeof nameOrData === 'string'
    ? { ...asRecord(config), name: nameOrData }
    : asRecord(nameOrData)
  const input: AddConfigInput = {
    name: String(data.name ?? ''),
    description: typeof data.description === 'string' ? data.description : null,
    baseUrl: String(data.base_url ?? data.baseUrl ?? ''),
    authToken: String(data.auth_token ?? data.authToken ?? ''),
    model: typeof data.model === 'string' ? data.model : null,
    smallFastModel: typeof data.small_fast_model === 'string'
      ? data.small_fast_model
      : typeof data.smallFastModel === 'string' ? data.smallFastModel : null,
    provider: typeof data.provider === 'string' ? data.provider : null,
    providerType: typeof data.provider_type === 'string'
      ? data.provider_type
      : typeof data.providerType === 'string' ? data.providerType : null,
    account: typeof data.account === 'string' ? data.account : null,
    tags: Array.isArray(data.tags) ? data.tags.filter((tag): tag is string => typeof tag === 'string') : null,
  }
  return addConfigTyped(input)
}

/** 更新配置 */
export const updateConfig = async <T = UnknownRecord>(
  name: string,
  config: unknown,
): Promise<T> => {
  return invoke('update_config', { name, data: config })
}

/** 删除指定配置 */
export const deleteConfig = deleteConfigTyped

/** 重命名配置 */
export const renameConfig = renameConfigTyped

/** 复制配置 */
export const duplicateConfig = duplicateConfigTyped

/** 验证所有配置 */
export const validateConfigs = validateConfigsTyped

/** 导入配置 */
export const importConfig = async (data: unknown): Promise<ImportResult> => {
  const payload = asRecord(data) as ImportConfigPayload
  return importConfigTyped({
    content: payload.content ?? '',
    mode: payload.mode ?? 'merge',
    backup: payload.backup ?? true,
  })
}

/** 从备份文件恢复配置 */
export const restoreConfig = restoreConfigTyped

/** 导出配置 */
export const exportConfig = async (_name?: string): Promise<ExportResult> => exportConfigTyped(false)

/** 获取历史记录（包装为 { entries: [...] } 格式供前端消费） */
export const getHistory = async (limit?: number): Promise<HistoryResponse> => {
  const entries = await getHistoryTyped(limit)
  return { entries, total: entries.length }
}

/** 清理历史记录 */
export const clearHistory = clearHistoryTyped

/** 清理备份（参数 _days 保留作向后兼容签名，后端未使用） */
export const cleanBackups = async <T = UnknownRecord>(_days?: number): Promise<T> => {
  return invoke('clean_backups')
}
