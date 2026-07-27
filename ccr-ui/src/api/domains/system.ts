/**
 * System Domain —— 系统信息、版本、CLI 探测 API
 *
 * 真迁移自 tauri.ts 第 13 分组；同时聚合 runtime/environment 的 shell/env 工具
 * 以保持 `systemApi` 命名空间对外契约（api/index.ts 的 systemApi 指向此文件）。
 */

import { invoke } from '@tauri-apps/api/core'
import { type UnknownRecord } from '../_shared'
import {
  checkVersion as checkVersionTyped,
  getSystemInfo as getSystemInfoTyped,
} from '../generated/systemInfo'
import {
  getCliVersion as getCliVersionTyped,
  getCliVersions as getCliVersionsTyped,
} from '../generated/systemExtended'
import type { CliVersionOptions } from '@/types/generated/system/CliVersionOptions'
import type { CliVersionsOptions } from '@/types/generated/system/CliVersionsOptions'

// ── System 信息与版本 ──

/** 获取系统信息（OS / arch / 时区 / 各 CLI 版本） */
export const getSystemInfo = getSystemInfoTyped

/** 检查版本更新 */
export const checkVersion = checkVersionTyped

/** 健康检查 */
export const healthCheck = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('health_check')
}

/** 获取版本号（对应 check_version 命令） */
export const getVersion = checkVersionTyped

/** 检查更新（checkVersion 别名） */
export const checkUpdate = checkVersion

/** 执行 CCR 自更新（branch 保留参数位，后端当前忽略） */
export const updateCCR = async <T = UnknownRecord>(_branch?: string): Promise<T> => {
  return invoke('update_ccr')
}

// ── CLI 版本探测 ──

export type CliVersionsCommandOptions = CliVersionsOptions & {
  /** 兼容历史调用参数（timeout → timeoutMs） */
  timeout?: number
}

export type CliVersionCommandOptions = CliVersionOptions

/**
 * 获取所有 CLI 版本。
 *
 * 响应同时保留结构化 `entries` 和兼容旧调用方的 `versions` 字符串映射。
 */
export const getCliVersions = async (
  options?: CliVersionsCommandOptions,
) => {
  const normalizedOptions = options
    ? {
        mode: options.mode,
        timeoutMs: options.timeoutMs ?? options.timeout,
        parallelism: options.parallelism,
      }
    : undefined

  return getCliVersionsTyped(normalizedOptions)
}

/** 获取单个工具的 CLI 版本 */
export const getCliVersion = async (
  options: CliVersionCommandOptions,
) => getCliVersionTyped(options)

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
