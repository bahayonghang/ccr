// 🎨 Tauri API 接口封装
// 本小姐用 TypeScript 打造的类型安全 API！(￣▽￣)ゞ

import { invoke } from '@tauri-apps/api/core'
import type {
  ConfigInfo,
  ConfigList,
  HistoryEntry,
  BackupInfo,
  SystemInfo,
  CreateConfigRequest,
  UpdateConfigRequest,
} from '../types'

// ============================================================================
// 🎯 配置管理 API
// ============================================================================

/**
 * 列出所有配置
 * @returns 配置列表（包含当前配置、默认配置、所有配置）
 */
export async function listConfigs(): Promise<ConfigList> {
  return await invoke('list_configs')
}

/**
 * 获取当前配置
 * @returns 当前配置信息
 */
export async function getCurrentConfig(): Promise<ConfigInfo | null> {
  return await invoke('get_current_config')
}

/**
 * 切换到指定配置
 * @param name 配置名称
 * @returns 成功消息
 */
export async function switchConfig(name: string): Promise<string> {
  return await invoke('switch_config', { name })
}

/**
 * 创建新配置
 * @param request 配置信息
 * @returns 成功消息
 */
export async function createConfig(request: CreateConfigRequest): Promise<string> {
  return await invoke('create_config', {
    name: request.name,
    description: request.description ?? null,
    baseUrl: request.base_url ?? null,
    authToken: request.auth_token ?? null,
    model: request.model ?? null,
    smallFastModel: request.small_fast_model ?? null,
    provider: request.provider ?? null,
    providerType: request.provider_type ?? null,
    account: request.account ?? null,
    tags: request.tags ?? null,
  })
}

/**
 * 更新配置
 * @param request 更新请求参数
 * @returns 成功消息
 */
export async function updateConfig(request: UpdateConfigRequest): Promise<string> {
  return await invoke('update_config', {
    oldName: request.old_name,
    newName: request.new_name,
    description: request.description ?? null,
    baseUrl: request.base_url ?? null,
    authToken: request.auth_token ?? null,
    model: request.model ?? null,
    smallFastModel: request.small_fast_model ?? null,
    provider: request.provider ?? null,
    providerType: request.provider_type ?? null,
    account: request.account ?? null,
    tags: request.tags ?? null,
  })
}

/**
 * 删除配置
 * @param name 配置名称
 * @returns 成功消息
 */
export async function deleteConfig(name: string): Promise<string> {
  return await invoke('delete_config', { name })
}

/**
 * 导入配置
 * @param content 配置文件内容
 * @param merge 是否合并模式（true: 合并, false: 替换）
 * @param backup 是否备份
 * @returns 成功消息
 */
export async function importConfig(content: string, merge: boolean = true, backup: boolean = true): Promise<string> {
  return await invoke('import_config', { content, merge, backup })
}

/**
 * 导出配置
 * @param includeSecrets 是否包含敏感信息
 * @returns 配置文件内容
 */
export async function exportConfig(includeSecrets: boolean = true): Promise<string> {
  return await invoke('export_config', { includeSecrets })
}

/**
 * 验证所有配置
 * @returns 验证报告
 */
export async function validateAll(): Promise<string> {
  return await invoke('validate_all')
}

// ============================================================================
// 📜 历史记录 API
// ============================================================================

/**
 * 获取操作历史记录
 * @param limit 限制返回条数（默认 50）
 * @returns 历史记录列表
 */
export async function getHistory(limit?: number): Promise<HistoryEntry[]> {
  return await invoke('get_history', { limit: limit ?? 50 })
}

// ============================================================================
// 💾 备份管理 API
// ============================================================================

/**
 * 列出所有备份
 * @returns 备份列表
 */
export async function listBackups(): Promise<BackupInfo[]> {
  return await invoke('list_backups')
}

/**
 * 从备份恢复
 * @param backupPath 备份文件路径
 * @returns 成功消息
 */
export async function restoreBackup(backupPath: string): Promise<string> {
  return await invoke('restore_backup', { backupPath })
}

// ============================================================================
// 🖥️ 系统信息 API
// ============================================================================

/**
 * 获取系统信息
 * @returns 系统信息
 */
export async function getSystemInfo(): Promise<SystemInfo> {
  return await invoke('get_system_info')
}
