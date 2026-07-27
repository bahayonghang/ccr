import { invoke } from '@/api/invokeRuntime'

export interface WslDistro {
  name: string
  is_default: boolean
  version: string
  state: string
}

export interface WslCacheStatus {
  has_memory_cache: boolean
  has_disk_cache: boolean
  cached_at: string | null
  distro_count: number
  is_expired: boolean
  age_secs: number | null
}

export type WslCliStatus = Record<string, boolean>

export interface ReadWslConfigRequest {
  distro: string
  platform: string
  path?: string
}

export interface SyncWslConfigRequest {
  distro: string
  platform: string
  direction: string
}

export const getWslCacheStatus = async (): Promise<WslCacheStatus> => {
  return invoke('wsl_cache_status')
}

export const listWslDistros = async (forceRefresh = false): Promise<WslDistro[]> => {
  return invoke('wsl_list_distros', { forceRefresh })
}

export const refreshWslDistros = async (): Promise<WslDistro[]> => {
  return invoke('wsl_refresh_distros')
}

export const clearWslCache = async (): Promise<void> => {
  await invoke('wsl_clear_cache')
}

export const detectWslCli = async (distro: string): Promise<WslCliStatus> => {
  return invoke('wsl_detect_cli', { distro })
}

export const readWslConfig = async (request: ReadWslConfigRequest): Promise<string> => {
  return invoke('wsl_read_config', { ...request })
}

export const syncWslConfig = async (request: SyncWslConfigRequest): Promise<string> => {
  return invoke('wsl_sync_config', { ...request })
}
