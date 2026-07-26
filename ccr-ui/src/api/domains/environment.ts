/**
 * Environment Domain —— 执行环境切换 / SSH 远程 API
 *
 * 真迁移自 tauri.ts 第 19 分组（含 SSH 子节）。
 * 对应后端 commands::environment::* 与 commands::ssh::* 命令。
 *
 * Shell / Desktop 偏好相关接口（shellGetPreferences 等）位于 `../runtime/environment`，
 * 由 `./system` 汇总 re-export 以维持 `systemApi` 命名空间契约，不在此文件重复。
 */

import { invoke } from '@tauri-apps/api/core'
import type { UnknownRecord } from '../_shared'

// ── 环境注册表 ──

/** 列出所有执行环境（local / wsl / ssh） */
export const listEnvironments = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('list_environments')
}

/** 获取当前活跃环境 */
export const getCurrentEnvironment = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('get_current_environment')
}

/** 切换活跃环境 */
export const switchEnvironment = async <T = UnknownRecord>(envId: string): Promise<T> => {
  return invoke('switch_environment', { envId })
}

/** 刷新环境列表（WSL 重新探测 / SSH 重新查 DB） */
export const refreshEnvironments = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('refresh_environments')
}

/** 通过当前环境列出受支持的平台 */
export const envListPlatforms = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('env_list_platforms')
}

/** 通过当前环境检测各 CLI 可用性 */
export const envDetectCli = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('env_detect_cli')
}

// ── SSH 数据结构 ──

export interface SshHostConfig {
  id?: string
  name?: string
  host: string
  port?: number
  user?: string
  identity_file?: string
  remote_home?: string
}

export interface SshConnectionState {
  env_id: string
  connected: boolean
  has_password: boolean
  last_checked_at?: string | null
  last_error?: string | null
}

export interface SshFingerprintProbeResult {
  challenge_id: string
  host: string
  port: number
  key_type: string
  public_key: string
  fingerprint: string
  status: 'new' | 'matched' | 'mismatch'
  stored_fingerprint?: string | null
}

export interface SshConnectResult {
  success: boolean
  latency_ms: number
  error_code?: string | null
  error?: string | null
}

export interface SshKeyInfo {
  path: string
  key_type: string
  has_passphrase: boolean
  fingerprint?: string | null
}

// ── SSH 命令 ──

export const sshListHosts = async (): Promise<SshHostConfig[]> => {
  return invoke('ssh_list_hosts')
}

export const sshAddHost = async (host: SshHostConfig): Promise<SshHostConfig> => {
  return invoke('ssh_add_host', { host })
}

export const sshConnect = async (
  envId: string,
  password?: string,
): Promise<SshConnectionState> => {
  return invoke('ssh_connect', { envId, password })
}

export const sshReconnect = async (
  envId: string,
  password?: string,
): Promise<SshConnectionState> => {
  return invoke('ssh_reconnect', { envId, password })
}

export const sshDisconnect = async (): Promise<SshConnectionState> => {
  return invoke('ssh_disconnect')
}

export const sshGetConnectionState = async (
  envId?: string,
): Promise<SshConnectionState | SshConnectionState[]> => {
  return invoke('ssh_get_connection_state', { envId })
}

export const sshProbeHostFingerprint = async (
  envId?: string,
  host?: string,
  port?: number,
): Promise<SshFingerprintProbeResult> => {
  return invoke('ssh_probe_host_fingerprint', { request: { env_id: envId, host, port } })
}

export const sshConfirmHostFingerprint = async (challengeId: string): Promise<void> => {
  return invoke('ssh_confirm_host_fingerprint', {
    request: { challenge_id: challengeId },
  })
}

export const sshReadConfig = async (
  envId: string,
  platform: string,
  path: string,
): Promise<string> => {
  return invoke('ssh_read_config', { envId, platform, path })
}

export const sshWriteConfig = async (
  envId: string,
  platform: string,
  path: string,
  content: string,
  enableBackup = true,
): Promise<void> => {
  return invoke('ssh_write_config', { envId, platform, path, content, enableBackup })
}

export const sshDetectCli = async <T = UnknownRecord>(envId: string): Promise<T> => {
  return invoke('ssh_detect_cli', { envId })
}

export const sshTestConnection = async (envId: string): Promise<SshConnectResult> => {
  return invoke('ssh_test_connection', { envId })
}

export const sshListKeys = async (): Promise<SshKeyInfo[]> => {
  return invoke('ssh_list_keys')
}
