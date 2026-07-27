/**
 * Monitoring Domain —— 监控流 API
 *
 * 对应后端 commands::events::get_monitoring_feed 命令。
 */

export { getMonitoringFeed } from '../generated/events'
export type MonitoringFeedQuery = import('@/types/generated/events/MonitoringFeedQueryDto').MonitoringFeedQueryDto
