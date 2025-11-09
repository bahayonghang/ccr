// Tauri API Client for CCR Desktop
// 使用 Tauri invoke 调用 Rust backend 命令

import { invoke } from '@tauri-apps/api/core'

// ===================================
// Types for Tauri Commands
// ===================================

export interface ProfileInfo {
  name: string
  description: string
  base_url: string
  model: string
  is_current: boolean
  is_default: boolean
  provider: string | null
}

export interface HistoryEntry {
  id: string
  timestamp: string
  operation: string
  actor: string
}

// ===================================
// Configuration Management
// ===================================

/**
 * 列出所有配置项
 * Tauri Command: list_profiles
 */
export const listProfiles = async (): Promise<ProfileInfo[]> => {
  try {
    const profiles = await invoke<ProfileInfo[]>('list_profiles')
    return profiles
  } catch (error) {
    console.error('[Tauri] listProfiles error:', error)
    throw error
  }
}

/**
 * 切换到指定配置
 * Tauri Command: switch_profile
 */
export const switchProfile = async (name: string): Promise<string> => {
  try {
    const result = await invoke<string>('switch_profile', { name })
    return result
  } catch (error) {
    console.error('[Tauri] switchProfile error:', error)
    throw error
  }
}

/**
 * 获取当前配置名称
 * Tauri Command: get_current_profile
 */
export const getCurrentProfile = async (): Promise<string> => {
  try {
    const current = await invoke<string>('get_current_profile')
    return current
  } catch (error) {
    console.error('[Tauri] getCurrentProfile error:', error)
    throw error
  }
}

/**
 * 验证所有配置
 * Tauri Command: validate_configs
 */
export const validateConfigs = async (): Promise<string> => {
  try {
    const result = await invoke<string>('validate_configs')
    return result
  } catch (error) {
    console.error('[Tauri] validateConfigs error:', error)
    throw error
  }
}

// ===================================
// History Management
// ===================================

/**
 * 获取历史记录
 * Tauri Command: get_history
 */
export const getHistory = async (limit?: number): Promise<HistoryEntry[]> => {
  try {
    const history = await invoke<HistoryEntry[]>('get_history', {
      limit: limit || 100
    })
    return history
  } catch (error) {
    console.error('[Tauri] getHistory error:', error)
    throw error
  }
}

/**
 * 清理历史记录
 * Tauri Command: clear_history
 * ⚠️ TODO: Backend implementation needed
 */
export const clearHistory = async (): Promise<string> => {
  try {
    const result = await invoke<string>('clear_history')
    return result
  } catch (error) {
    console.error('[Tauri] clearHistory error:', error)
    throw error
  }
}

// ===================================
// Sync Management
// ===================================

/**
 * 推送配置到云端
 * Tauri Command: sync_push
 * ⚠️ TODO: Backend implementation needed
 */
export const syncPush = async (force?: boolean): Promise<string> => {
  try {
    const result = await invoke<string>('sync_push', { force })
    return result
  } catch (error) {
    console.error('[Tauri] syncPush error:', error)
    throw error
  }
}

/**
 * 从云端拉取配置
 * Tauri Command: sync_pull
 * ⚠️ TODO: Backend implementation needed
 */
export const syncPull = async (force?: boolean): Promise<string> => {
  try {
    const result = await invoke<string>('sync_pull', { force })
    return result
  } catch (error) {
    console.error('[Tauri] syncPull error:', error)
    throw error
  }
}

/**
 * 获取同步状态
 * Tauri Command: sync_status
 * ⚠️ TODO: Backend implementation needed
 */
export const syncStatus = async (): Promise<string> => {
  try {
    const status = await invoke<string>('sync_status')
    return status
  } catch (error) {
    console.error('[Tauri] syncStatus error:', error)
    throw error
  }
}

// ===================================
// Platform Management
// ===================================

/**
 * 列出所有平台
 * Tauri Command: list_platforms
 */
export const listPlatforms = async (): Promise<string[]> => {
  try {
    const platforms = await invoke<string[]>('list_platforms')
    return platforms
  } catch (error) {
    console.error('[Tauri] listPlatforms error:', error)
    throw error
  }
}

/**
 * 切换平台
 * Tauri Command: switch_platform
 */
export const switchPlatform = async (platform: string): Promise<string> => {
  try {
    const result = await invoke<string>('switch_platform', { platform })
    return result
  } catch (error) {
    console.error('[Tauri] switchPlatform error:', error)
    throw error
  }
}

/**
 * 获取当前平台
 * Tauri Command: get_current_platform
 */
export const getCurrentPlatform = async (): Promise<string> => {
  try {
    const platform = await invoke<string>('get_current_platform')
    return platform
  } catch (error) {
    console.error('[Tauri] getCurrentPlatform error:', error)
    throw error
  }
}

// ===================================
// Utility Functions
// ===================================

/**
 * 检查是否在 Tauri 环境中运行
 */
export const isTauriEnvironment = (): boolean => {
  return '__TAURI__' in window
}

/**
 * 获取 Tauri 版本信息
 */
export const getTauriVersion = async (): Promise<string | null> => {
  if (!isTauriEnvironment()) {
    return null
  }

  try {
    const { getVersion } = await import('@tauri-apps/api/app')
    return await getVersion()
  } catch (error) {
    console.error('[Tauri] Failed to get version:', error)
    return null
  }
}

/**
 * Tauri API 导出对象（便于统一调用）
 */
export const TauriAPI = {
  // Config
  listProfiles,
  switchProfile,
  getCurrentProfile,
  validateConfigs,

  // History
  getHistory,
  clearHistory,

  // Sync
  syncPush,
  syncPull,
  syncStatus,

  // Platform
  listPlatforms,
  switchPlatform,
  getCurrentPlatform,

  // Utils
  isTauriEnvironment,
  getTauriVersion,
}

export default TauriAPI
