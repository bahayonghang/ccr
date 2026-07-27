/**
 * Environment Domain —— 执行环境切换 / SSH 远程 API
 *
 * 真迁移自 tauri.ts 第 19 分组（含 SSH 子节）。
 * 对应后端 commands::environment::* 与 commands::ssh::* 命令。
 *
 * Shell / Desktop 偏好相关接口（shellGetPreferences 等）位于 `../runtime/environment`，
 * 由 `./system` 汇总 re-export 以维持 `systemApi` 命名空间契约，不在此文件重复。
 */

import { invoke } from '@/api/invokeRuntime'
import type { UnknownRecord } from '../_shared'
import {
  getCurrentEnvironment,
  listEnvironments,
  refreshEnvironments,
  switchEnvironment,
} from '../generated/environment'
import {
  sshAddHost as addTypedSshHost,
  sshConfirmHostFingerprint as confirmTypedSshHostFingerprint,
  sshConnect as connectTypedSsh,
  sshDetectCli as detectTypedSshCli,
  sshDisconnect as disconnectTypedSsh,
  sshGetConnectionState as getTypedSshConnectionState,
  sshListHosts as listTypedSshHosts,
  sshListKeys as listTypedSshKeys,
  sshProbeHostFingerprint as probeTypedSshHostFingerprint,
  sshReadConfig as readTypedSshConfig,
  sshReconnect as reconnectTypedSsh,
  sshTestConnection as testTypedSshConnection,
  sshWriteConfig as writeTypedSshConfig,
} from '../generated/ssh'
import type { AddSshHostRequest } from '@/types/generated/ssh/AddSshHostRequest'
import type { SshCliStatusDto } from '@/types/generated/ssh/SshCliStatusDto'
import type { SshConnectionState as GeneratedSshConnectionState } from '@/types/generated/ssh/SshConnectionState'
import type { SshConnectionStateResponse } from '@/types/generated/ssh/SshConnectionStateResponse'
import type { SshConnectResultDto } from '@/types/generated/ssh/SshConnectResultDto'
import type { SshFingerprintProbeResult as GeneratedSshFingerprintProbeResult } from '@/types/generated/ssh/SshFingerprintProbeResult'
import type { SshHostConfigDto } from '@/types/generated/ssh/SshHostConfigDto'
import type { SshKeyInfoDto } from '@/types/generated/ssh/SshKeyInfoDto'

// ── 环境注册表 ──

/** 列出所有执行环境（local / wsl / ssh） */
export { getCurrentEnvironment, listEnvironments, refreshEnvironments, switchEnvironment }

/** 通过当前环境列出受支持的平台 */
export const envListPlatforms = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('env_list_platforms')
}

/** 通过当前环境检测各 CLI 可用性 */
export const envDetectCli = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('env_detect_cli')
}

// ── SSH 数据结构 ──

export type SshHostConfig = Pick<SshHostConfigDto, 'host'> & Partial<SshHostConfigDto>
export type SshConnectionState = GeneratedSshConnectionState
export type SshFingerprintProbeResult = GeneratedSshFingerprintProbeResult
export type SshConnectResult = SshConnectResultDto
export type SshKeyInfo = SshKeyInfoDto
export type SshCliStatus = SshCliStatusDto

// ── SSH 命令 ──

export const sshListHosts = async (): Promise<SshHostConfig[]> => {
  return listTypedSshHosts()
}

export const sshAddHost = async (host: SshHostConfig): Promise<SshHostConfig> => {
  const request: AddSshHostRequest = {
    id: host.id ?? undefined,
    name: host.name ?? undefined,
    host: host.host,
    port: host.port ?? undefined,
    user: host.user ?? undefined,
    identity_file: host.identity_file ?? undefined,
    remote_home: host.remote_home ?? undefined,
  }
  return addTypedSshHost(request)
}

export const sshConnect = async (
  envId: string,
  password?: string,
): Promise<SshConnectionState> => {
  return connectTypedSsh({ envId, password })
}

export const sshReconnect = async (
  envId: string,
  password?: string,
): Promise<SshConnectionState> => {
  return reconnectTypedSsh({ envId, password })
}

export const sshDisconnect = async (): Promise<SshConnectionState> => {
  return disconnectTypedSsh()
}

export const sshGetConnectionState = async (
  envId?: string,
): Promise<SshConnectionStateResponse> => {
  return getTypedSshConnectionState(envId)
}

export const sshProbeHostFingerprint = async (
  envId?: string,
  host?: string,
  port?: number,
): Promise<SshFingerprintProbeResult> => {
  return probeTypedSshHostFingerprint({ env_id: envId, host, port })
}

export const sshConfirmHostFingerprint = async (challengeId: string): Promise<void> => {
  return confirmTypedSshHostFingerprint(challengeId)
}

export const sshReadConfig = async (
  envId: string,
  platform: string,
  path: string,
): Promise<string> => {
  return readTypedSshConfig({ envId, platform, path })
}

export const sshWriteConfig = async (
  envId: string,
  platform: string,
  path: string,
  content: string,
  enableBackup = true,
): Promise<void> => {
  return writeTypedSshConfig({ envId, platform, path, content, enableBackup })
}

export const sshDetectCli = async (envId: string): Promise<SshCliStatus[]> => {
  return detectTypedSshCli(envId)
}

export const sshTestConnection = async (envId: string): Promise<SshConnectResult> => {
  return testTypedSshConnection(envId)
}

export const sshListKeys = async (): Promise<SshKeyInfo[]> => {
  return listTypedSshKeys()
}
