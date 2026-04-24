/**
 * System Domain —— 系统信息、版本、CLI 探测 API
 *
 * 真迁移自 tauri.ts 第 13 分组；同时聚合 runtime/environment 的 shell/env 工具
 * 以保持 `systemApi` 命名空间对外契约（api/index.ts 的 systemApi 指向此文件）。
 */

import { invoke } from '@tauri-apps/api/core'
import { isRecord, pickRecord, type UnknownRecord } from '../_shared'
import type { VersionInfoResponse } from '../tauri'

// ── System 信息与版本 ──

/** 获取系统信息（OS / arch / 时区 / 各 CLI 版本） */
export const getSystemInfo = async <T = VersionInfoResponse>(): Promise<T> => {
  return invoke('get_system_info')
}

/** 检查版本更新 */
export const checkVersion = async <T = VersionInfoResponse>(): Promise<T> => {
  return invoke('check_version')
}

/** 健康检查 */
export const healthCheck = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('health_check')
}

/** 获取版本号（对应 check_version 命令） */
export const getVersion = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('check_version')
}

/** 检查更新（checkVersion 别名） */
export const checkUpdate = checkVersion

/** 执行 CCR 自更新（branch 保留参数位，后端当前忽略） */
export const updateCCR = async <T = UnknownRecord>(_branch?: string): Promise<T> => {
  return invoke('update_ccr')
}

// ── CLI 版本探测 ──

export interface CliVersionsCommandOptions {
  mode?: 'fast' | 'full'
  timeoutMs?: number
  parallelism?: number
  /** 兼容历史调用参数（timeout → timeoutMs） */
  timeout?: number
}

export interface CliVersionCommandOptions {
  tool: string
  timeoutMs?: number
  force?: boolean
}

/**
 * 获取所有 CLI 版本。
 *
 * 后端返回格式存在历史差异：
 * 1) 新格式：`{ entries: [...] }` 或 `{ versions: [...] }`（数组）
 * 2) 旧格式：`{ versions: { claude: "...", codex: "..." } }`（键值对）
 *
 * 此函数统一归一化为 `{ ..., versions: CliVersionEntry[] }`。
 */
export const getCliVersions = async <T = UnknownRecord>(
  options?: CliVersionsCommandOptions,
): Promise<T> => {
  const normalizedOptions = options
    ? {
        mode: options.mode,
        timeoutMs: options.timeoutMs ?? options.timeout,
        parallelism: options.parallelism,
      }
    : undefined

  const raw = await invoke<unknown>('get_cli_versions', { options: normalizedOptions })
  if (!isRecord(raw)) {
    return raw as T
  }

  const entries = Array.isArray(raw.entries)
    ? raw.entries
    : Array.isArray(raw.versions)
      ? raw.versions
      : Object.entries(pickRecord(raw, 'versions')).map(([platform, value]) => {
          const text = String(value ?? '')
          if (!text || text === 'not found') {
            return {
              platform,
              installed: false,
              status: 'not_installed',
            }
          }
          return {
            platform,
            installed: true,
            version: text,
            status: 'ok',
          }
        })

  return {
    ...raw,
    versions: entries,
  } as T
}

/** 获取单个工具的 CLI 版本 */
export const getCliVersion = async <T = UnknownRecord>(
  options: CliVersionCommandOptions,
): Promise<T> => {
  const normalizedOptions = {
    tool: options.tool,
    timeoutMs: options.timeoutMs,
    force: options.force,
  }

  return invoke('get_cli_version', { options: normalizedOptions })
}

// ── Events / Runtime 指标（随 system 命名空间暴露，真实现见 domains/events） ──

export { getRecentEvents, getRuntimeMetrics } from './events'

// ── Shell / 环境 辅助（来自 runtime/environment，保持 systemApi 契约） ──

export {
  detectSkillportApp,
  detectSkillsManageApp,
  getEnvironmentName,
  getSkipExitConfirm,
  getTauriVersion,
  isTauriEnvironment,
  openSkillportApp,
  openSkillsManageApp,
  shellGetPreferences,
  shellRequestQuit,
  shellSetPreferences,
  shellShowMainWindow,
  setSkipExitConfirm,
  TauriAPI,
  TauriRuntimeApi,
} from '../runtime/environment'

export type {
  SkillportAppPlatform,
  SkillportAppSource,
  SkillportAppStatus,
  SkillsManageAppPlatform,
  SkillsManageAppSource,
  SkillsManageAppStatus,
} from '../runtime/environment'
