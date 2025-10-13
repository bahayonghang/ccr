// 🔌 API 适配器 - 支持 Tauri 和 Web 双模式
// 自动检测运行环境，选择合适的 API 调用方式

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
// 🔍 环境检测
// ============================================================================

/**
 * 检测当前是否在 Tauri 环境中
 */
function isTauriEnvironment(): boolean {
  return typeof window !== 'undefined' && '__TAURI__' in window
}

/**
 * Web 调试模式的后端 API 基础 URL
 */
const WEB_API_BASE = 'http://localhost:3030'

// ============================================================================
// 🔌 API 适配器函数
// ============================================================================

/**
 * 通用 API 调用适配器
 * @param tauriCommand Tauri 命令名
 * @param tauriArgs Tauri 命令参数
 * @param webEndpoint Web API 端点
 * @param webMethod HTTP 方法
 * @param webBody 请求体
 */
async function apiCall<T>(
  tauriCommand: string,
  tauriArgs: any = {},
  webEndpoint: string,
  webMethod: 'GET' | 'POST' | 'PUT' | 'DELETE' = 'GET',
  webBody?: any
): Promise<T> {
  if (isTauriEnvironment()) {
    // Tauri 桌面模式
    console.log(`🖥️ Tauri API: ${tauriCommand}`, tauriArgs)
    return await invoke(tauriCommand, tauriArgs)
  } else {
    // Web 调试模式
    console.log(`🌐 Web API: ${webMethod} ${webEndpoint}`, webBody)
    
    const options: RequestInit = {
      method: webMethod,
      headers: {
        'Content-Type': 'application/json',
      }
    }

    if (webBody) {
      options.body = JSON.stringify(webBody)
    }

    const response = await fetch(`${WEB_API_BASE}${webEndpoint}`, options)
    
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`)
    }

    return await response.json()
  }
}

// ============================================================================
// 🎯 配置管理 API（适配版本）
// ============================================================================

export async function listConfigs(): Promise<ConfigList> {
  return await apiCall(
    'list_configs',
    {},
    '/api/configs',
    'GET'
  )
}

export async function getCurrentConfig(): Promise<ConfigInfo | null> {
  return await apiCall(
    'get_current_config', 
    {},
    '/api/current',
    'GET'
  )
}

export async function switchConfig(name: string): Promise<string> {
  return await apiCall(
    'switch_config',
    { name },
    '/api/switch',
    'POST',
    { name }
  )
}

export async function createConfig(request: CreateConfigRequest): Promise<string> {
  return await apiCall(
    'create_config',
    {
      name: request.name,
      description: request.description,
      baseUrl: request.base_url,
      authToken: request.auth_token,
      model: request.model,
      smallFastModel: request.small_fast_model,
      provider: request.provider,
      providerType: request.provider_type,
      account: request.account,
      tags: request.tags,
    },
    '/api/config',
    'POST',
    request
  )
}

export async function updateConfig(request: UpdateConfigRequest): Promise<string> {
  return await apiCall(
    'update_config',
    {
      oldName: request.old_name,
      newName: request.new_name,
      description: request.description,
      baseUrl: request.base_url,
      authToken: request.auth_token,
      model: request.model,
      smallFastModel: request.small_fast_model,
      provider: request.provider,
      providerType: request.provider_type,
      account: request.account,
      tags: request.tags,
    },
    `/api/config/${encodeURIComponent(request.old_name)}`,
    'PUT',
    request
  )
}

export async function deleteConfig(name: string): Promise<string> {
  return await apiCall(
    'delete_config',
    { name },
    `/api/config/${encodeURIComponent(name)}`,
    'DELETE'
  )
}

export async function importConfig(content: string, merge: boolean = true, backup: boolean = true): Promise<string> {
  return await apiCall(
    'import_config',
    { content, merge, backup },
    '/api/import',
    'POST',
    { content, merge, backup }
  )
}

export async function exportConfig(includeSecrets: boolean = true): Promise<string> {
  return await apiCall(
    'export_config',
    { includeSecrets },
    `/api/export?includeSecrets=${includeSecrets}`,
    'GET'
  )
}

export async function validateAll(): Promise<string> {
  return await apiCall(
    'validate_all',
    {},
    '/api/validate',
    'POST'
  )
}

// ============================================================================
// 📜 历史和备份 API（适配版本）
// ============================================================================

export async function getHistory(limit?: number): Promise<HistoryEntry[]> {
  return await apiCall(
    'get_history',
    { limit },
    `/api/history${limit ? `?limit=${limit}` : ''}`,
    'GET'
  )
}

export async function listBackups(): Promise<BackupInfo[]> {
  return await apiCall(
    'list_backups',
    {},
    '/api/backups',
    'GET'
  )
}

export async function restoreBackup(backupPath: string): Promise<string> {
  return await apiCall(
    'restore_backup',
    { backupPath },
    '/api/restore',
    'POST',
    { backupPath }
  )
}

export async function getSystemInfo(): Promise<SystemInfo> {
  return await apiCall(
    'get_system_info',
    {},
    '/api/system',
    'GET'
  )
}

// ============================================================================
// 🔧 调试和工具函数
// ============================================================================

/**
 * 获取当前运行模式
 */
export function getRunMode(): 'tauri' | 'web' {
  return isTauriEnvironment() ? 'tauri' : 'web'
}

/**
 * 显示当前 API 配置
 */
export function getApiConfig() {
  const mode = getRunMode()
  console.log('🔧 API 配置信息:')
  console.log(`   运行模式: ${mode === 'tauri' ? '🖥️ Tauri 桌面' : '🌐 Web 浏览器'}`)
  
  if (mode === 'web') {
    console.log(`   后端地址: ${WEB_API_BASE}`)
    console.log(`   前端地址: ${window.location.origin}`)
  }
  
  return { mode, webApiBase: WEB_API_BASE }
}

/**
 * 测试 API 连接
 */
export async function testApiConnection(): Promise<boolean> {
  try {
    if (isTauriEnvironment()) {
      // 测试 Tauri 连接
      await invoke('get_system_info')
      console.log('✅ Tauri API 连接正常')
      return true
    } else {
      // 测试 Web API 连接
      const response = await fetch(`${WEB_API_BASE}/api/system`)
      if (response.ok) {
        console.log('✅ Web API 连接正常')
        return true
      } else {
        console.error(`❌ Web API 连接失败: ${response.status}`)
        return false
      }
    }
  } catch (error) {
    console.error(`❌ API 连接测试失败:`, error)
    return false
  }
}
