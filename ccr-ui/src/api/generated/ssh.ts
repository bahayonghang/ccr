/* Generated from commands/handler_registry.rs; do not edit. */

import { invoke } from '@tauri-apps/api/core'
import type { AddSshHostRequest } from '@/types/generated/ssh/AddSshHostRequest'
import type { SshCliStatusDto } from '@/types/generated/ssh/SshCliStatusDto'
import type { SshConnectionState } from '@/types/generated/ssh/SshConnectionState'
import type { SshConnectionStateResponse } from '@/types/generated/ssh/SshConnectionStateResponse'
import type { SshConnectResultDto } from '@/types/generated/ssh/SshConnectResultDto'
import type { SshFingerprintProbeResult } from '@/types/generated/ssh/SshFingerprintProbeResult'
import type { SshHostConfigDto } from '@/types/generated/ssh/SshHostConfigDto'
import type { SshKeyInfoDto } from '@/types/generated/ssh/SshKeyInfoDto'
import type { SshProbeFingerprintRequest } from '@/types/generated/ssh/SshProbeFingerprintRequest'

export type SshConnectInput = { envId: string; password?: string }
export type SshReadConfigInput = { envId: string; platform: string; path: string }
export type SshWriteConfigInput = SshReadConfigInput & { content: string; enableBackup?: boolean }

export const sshListHosts = (): Promise<SshHostConfigDto[]> => invoke('ssh_list_hosts')
export const sshAddHost = (host: AddSshHostRequest): Promise<SshHostConfigDto> => invoke('ssh_add_host', { host })
export const sshConnect = (input: SshConnectInput): Promise<SshConnectionState> => invoke('ssh_connect', input)
export const sshReconnect = (input: SshConnectInput): Promise<SshConnectionState> => invoke('ssh_reconnect', input)
export const sshDisconnect = (): Promise<SshConnectionState> => invoke('ssh_disconnect')
export const sshGetConnectionState = (envId?: string): Promise<SshConnectionStateResponse> => invoke('ssh_get_connection_state', { envId })
export const sshProbeHostFingerprint = (request: SshProbeFingerprintRequest): Promise<SshFingerprintProbeResult> => invoke('ssh_probe_host_fingerprint', { request })
export const sshConfirmHostFingerprint = (challengeId: string): Promise<void> => invoke('ssh_confirm_host_fingerprint', { request: { challenge_id: challengeId } })
export const sshReadConfig = (input: SshReadConfigInput): Promise<string> => invoke('ssh_read_config', input)
export const sshWriteConfig = (input: SshWriteConfigInput): Promise<void> => invoke('ssh_write_config', input)
export const sshDetectCli = (envId: string): Promise<SshCliStatusDto[]> => invoke('ssh_detect_cli', { envId })
export const sshTestConnection = (envId: string): Promise<SshConnectResultDto> => invoke('ssh_test_connection', { envId })
export const sshListKeys = (): Promise<SshKeyInfoDto[]> => invoke('ssh_list_keys')
