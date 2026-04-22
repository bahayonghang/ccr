/**
 * Events Domain —— 事件总线与运行时指标 API
 *
 * 对应后端 commands::events::* 命令。
 * 真迁移自 tauri.ts 第 18 分组。
 */

import { invoke } from '@tauri-apps/api/core'
import type { UnknownRecord } from '../_shared'

/** 获取最近事件（环形缓冲 EventLog 快照） */
export const getRecentEvents = async <T = UnknownRecord>(count?: number): Promise<T> => {
  return invoke('get_recent_events', { count })
}

/** 获取运行时性能指标（cache 命中、命令耗时、DB 查询耗时等） */
export const getRuntimeMetrics = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('get_runtime_metrics')
}
