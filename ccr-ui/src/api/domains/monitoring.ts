/**
 * Monitoring Domain —— 监控流 API
 *
 * 对应后端 commands::events::get_monitoring_feed 命令。
 */

import { invoke } from '@tauri-apps/api/core'

export interface MonitoringFeedQuery {
  count?: number
  level?: string
  channel?: string
}

/** 获取监控流快照（统一监控通道，替代 legacy get_recent_events 路径） */
export const getMonitoringFeed = async (query: MonitoringFeedQuery): Promise<unknown[]> => {
  return invoke('get_monitoring_feed', { query })
}
