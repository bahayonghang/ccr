// V2 Usage Analytics Types - 匹配后端 v2 聚合 API 响应

/** 使用量汇总 */
export interface UsageSummary {
  total_requests: number
  total_input_tokens: number
  total_output_tokens: number
  total_cache_read_tokens: number
  total_cost_usd: number
  cache_efficiency: number
}

/** 每日趋势 */
export interface DailyTrend {
  date: string
  request_count: number
  input_tokens: number
  output_tokens: number
  cache_read_tokens: number
  cost_usd: number
}

/** 模型统计 */
export interface ModelStat {
  model: string
  request_count: number
  total_tokens: number
  total_cost: number
}

/** 项目统计 */
export interface ProjectStat {
  project_path: string
  request_count: number
  total_tokens: number
  total_cost: number
}

/** 使用记录（v2，含提取列） */
export interface UsageRecordV2 {
  id: string
  platform: string
  project_path: string
  record_json: string
  recorded_at: string
  source_id: string
  model: string | null
  input_tokens: number
  output_tokens: number
  cache_read_tokens: number
  cost_usd: number
}

/** 分页日志 */
export interface PaginatedLogs {
  records: UsageRecordV2[]
  total?: number | null
  page: number
  page_size: number
  next_cursor?: string | null
  mode?: 'cursor' | 'offset'
}

/** 日志查询参数 */
export interface UsageLogsQuery {
  platform?: string
  model?: string
  start_date?: string
  end_date?: string
  page?: number
  page_size?: number
  cursor?: string
  include_total?: boolean
  mode?: 'cursor' | 'offset'
}

/** 热力图响应 */
export interface HeatmapResponse {
  data: Record<string, number>
}

/** 仪表盘聚合响应 */
export interface UsageDashboardResponse {
  summary: UsageSummary
  trends: DailyTrend[]
  model_stats: ModelStat[]
  project_stats: ProjectStat[]
  heatmap: HeatmapResponse
  generated_at: string
}

/** 首页概览视图模式 */
export type HomeOverviewViewMode = 'sessions' | 'requests' | 'tokens'

/** 首页平台统计 */
export interface HomeOverviewPlatformStats {
  sessions: number
  requests: number
  tokens: number
}

/** 首页趋势项 */
export interface HomeOverviewSeriesItem {
  date: string
  claude: HomeOverviewPlatformStats
  codex: HomeOverviewPlatformStats
  gemini: HomeOverviewPlatformStats
}

/** 首页概览汇总 */
export interface HomeOverviewSummary {
  total_sessions: number
  total_requests: number
  total_tokens: number
  active_days: number
  platforms: number
}

/** 首页自举信息 */
export interface HomeOverviewBootstrap {
  usage_import_attempted: boolean
  usage_imported_records: number
  session_reindex_attempted: boolean
  indexed_sessions: number
}

/** 首页概览响应 */
export interface HomeUsageOverviewResponse {
  summary: HomeOverviewSummary
  by_platform: Record<string, HomeOverviewPlatformStats>
  series: HomeOverviewSeriesItem[]
  bootstrap: HomeOverviewBootstrap
  empty_reason?: 'no_usage_logs' | 'no_session_index' | 'no_usage_and_sessions'
  last_updated: string
}

/** 导入结果 */
export interface ImportResult {
  platform: string
  files_processed: number
  records_imported: number
  records_skipped: number
  duration_ms: number
  completed: boolean
  error?: string | null
}

/** 导入摘要 */
export interface UsageImportSummary {
  success_count: number
  failure_count: number
  imported_records: number
  processed_files: number
  has_partial: boolean
}

/** 全量导入响应 */
export interface ImportAllUsageResponse {
  results: ImportResult[]
  summary: UsageImportSummary
}

export type UsageImportJobStatus = 'pending' | 'running' | 'recent_ready' | 'finished' | 'failed'

export type UsageImportJobStage = 'queued' | 'importing_recent' | 'importing_history' | 'finished' | 'failed'

export interface UsageImportJobSnapshot {
  job_id: string
  status: UsageImportJobStatus
  stage: UsageImportJobStage
  platform_scope: string
  recent_window_days: number
  files_total: number
  files_scanned: number
  files_imported: number
  records_imported: number
  records_skipped: number
  started_at: string
  updated_at: string
  recent_ready_at?: string | null
  finished_at?: string | null
  current_file?: string | null
  warnings: string[]
  error?: string | null
  results: ImportResult[]
  summary?: UsageImportSummary | null
}

export interface StartUsageImportJobResponse {
  job_id: string
  snapshot: UsageImportJobSnapshot
}

/** 平台类型 */
export type Platform = 'claude' | 'codex' | 'gemini'
