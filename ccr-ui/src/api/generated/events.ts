/* Generated from commands/handler_registry.rs; do not edit. */

import { invoke } from '@/api/invokeRuntime'
import type { EventLogEntryDto } from '@/types/generated/events/EventLogEntryDto'
import type { FrontendLogInputDto } from '@/types/generated/events/FrontendLogInputDto'
import type { MonitoringEntryDto } from '@/types/generated/events/MonitoringEntryDto'
import type { MonitoringFeedQueryDto } from '@/types/generated/events/MonitoringFeedQueryDto'
import type { RuntimeMetricsResponse } from '@/types/generated/events/RuntimeMetricsResponse'

export const getRecentEvents = (count?: number): Promise<EventLogEntryDto[]> => invoke('get_recent_events', { count })
export const getMonitoringFeed = (query: MonitoringFeedQueryDto = {}): Promise<MonitoringEntryDto[]> => invoke('get_monitoring_feed', { query })
export const appendFrontendLogs = (entries: FrontendLogInputDto[]): Promise<void> => invoke('append_frontend_logs', { entries })
export const getRuntimeMetrics = (): Promise<RuntimeMetricsResponse> => invoke('get_runtime_metrics')
