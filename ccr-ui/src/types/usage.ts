// V2 Usage Analytics Types - 匹配后端 v2 聚合 API 响应

/** 使用量汇总 */
export interface UsageSummary {
  total_requests: number
  total_tokens: number
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
  total_tokens: number
  input_tokens: number
  /**
   * Compatibility field from the backend: assistant output plus reasoning output.
   * Use `Math.max(0, output_tokens - reasoning_output_tokens)` when a view needs
   * assistant-only output.
   */
  output_tokens: number
  reasoning_output_tokens: number
  cache_read_tokens: number
  cache_creation_tokens: number
  cost_usd: number
}

/** 模型统计 */
export interface ModelStat {
  model: string
  request_count: number
  total_tokens: number
  total_cost: number
  input_tokens?: number
  output_tokens?: number
  cache_read_tokens?: number
  cache_creation_tokens?: number
  cost_with_cache?: number
  cost_without_cache?: number
  cache_savings?: number
  pricing_status?: string
  pricing_source?: string | null
  pricing_rate?: string | null
}

/** 项目统计 */
export interface ProjectStat {
  project_path: string
  request_count: number
  total_tokens: number
  total_cost: number
}

/** Source/platform 聚合统计 */
export interface SourceBreakdown {
  source: UsagePlatform
  event_count: number
  total_tokens: number
  total_cost: number
  active_days: number
  share_tokens: number
  share_cost: number
}

/** Provider 聚合统计 */
export interface ProviderBreakdown {
  provider: string | null
  request_count: number
  input_tokens: number
  cache_read_tokens: number
  cache_creation_tokens: number
  output_tokens: number
  reasoning_output_tokens: number
  total_tokens: number
  cost_with_cache_usd: number
  cost_without_cache_usd: number
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
  // 后端 UsageRecordDto 全是 required（pricing_source 为 Option<String> -> string | null）。
  // 这些字段对齐后端契约，不再 optional；前端无需 ?? 0 / ?? '' 防御代码。
  cache_creation_tokens: number
  cost_with_cache_usd: number
  cost_without_cache_usd: number
  pricing_status: string
  pricing_source: string | null
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

export interface UsageArchiveDiagnostics {
  archive_root: string
  live_sources: number
  missing_sources: number
  deleted_sources: number
  archived_sessions: number
  recent_completed_at?: string | null
  history_completed_at?: string | null
  source_health?: UsageSourceHealth[]
  freshness?: UsageFreshnessProjection
  readiness?: UsageReadinessProjection
}

export type UsageFreshnessState = 'fresh' | 'stale' | 'missing'

export interface UsageFreshnessProjection {
  state: UsageFreshnessState
  latest_completed_at?: string | null
  age_seconds?: number | null
  stale_after_seconds: number
}

export type UsageSourceHealthState = 'live' | 'degraded' | 'missing'

export interface UsageSourceHealth {
  source: string
  state: UsageSourceHealthState
  live_sources: number
  missing_sources: number
  deleted_sources: number
  recent_completed_at?: string | null
  history_completed_at?: string | null
  freshness: UsageFreshnessProjection
}

export type UsageReadinessState =
  | 'ready'
  | 'syncing'
  | 'stale'
  | 'degraded'
  | 'needs_import'
  | 'needs_session_index'
  | 'empty'

export interface UsageReadinessProjection {
  state: UsageReadinessState
  next_action?: string | null
  detail: string
  has_live_sources: boolean
  has_missing_sources: boolean
  has_deleted_sources: boolean
  active_usage_import: boolean
  active_session_index: boolean
  recent_completed_at?: string | null
}

export interface UsageDrilldownProjection {
  dimensions: string[]
  supports_logs: boolean
  supports_projects: boolean
  supports_sessions: boolean
}

export interface UsageSnapshotProjection {
  generated_at: string
  platform_scope: string
  start_date?: string | null
  end_date?: string | null
  cache_ttl_seconds: number
  freshness: UsageFreshnessProjection
  readiness: UsageReadinessProjection
  source_health: UsageSourceHealth[]
  drilldown: UsageDrilldownProjection
}

/** 仪表盘聚合响应 */
export interface UsageDashboardResponse {
  summary: UsageSummary
  trends: DailyTrend[]
  model_stats: ModelStat[]
  provider_stats: ProviderBreakdown[]
  project_stats: ProjectStat[]
  source_stats: SourceBreakdown[]
  archive: UsageArchiveDiagnostics
  snapshot: UsageSnapshotProjection
  heatmap: HeatmapResponse
  generated_at: string
}

export type UsageUnsupportedReason =
  | 'cli_missing'
  | 'db_missing'
  | 'db_unreadable'
  | 'schema_unsupported'
  | 'missing_table'
  | 'missing_column'
  | 'waiting_for_llmusage'

export interface UsageFeatureCapability {
  supported: boolean
  reason?: UsageUnsupportedReason | null
  detail?: string | null
}

export interface UsageCapabilityReport {
  cli_available: boolean
  cli_version?: string | null
  root_dir: string
  db_path: string
  db_exists: boolean
  db_readable: boolean
  schema_version?: number | null
  features: Record<string, UsageFeatureCapability>
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
  opencode: HomeOverviewPlatformStats
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
  usage_job_id?: string | null
  session_job_id?: string | null
  needs_usage_import: boolean
  needs_session_index: boolean
  is_warm: boolean
}

/** 首页概览响应 */
export interface HomeUsageOverviewResponse {
  summary: HomeOverviewSummary
  by_platform: Record<string, HomeOverviewPlatformStats>
  series: HomeOverviewSeriesItem[]
  bootstrap: HomeOverviewBootstrap
  archive: UsageArchiveDiagnostics
  snapshot: UsageSnapshotProjection
  empty_reason?: 'no_usage_logs' | 'no_session_index' | 'no_usage_and_sessions'
  last_updated: string
}

export interface UsageSnapshotUpdatedPayload {
  reason: string
  platform_scope: string
  job_id?: string | null
  imported_records: number
  generated_at: string
}

/** 导入结果 */
export interface UsageImportResult {
  platform: string
  files_processed: number
  records_imported: number
  records_skipped: number
  duration_ms: number
  completed: boolean
  error?: string | null
  /**
   * optional source（如 OpenCode）缺失安装时为 true。
   * 用此 typed 标志判断"导入失败"是不是用户没装这个 optional source 的正常情况，
   * 不要嗅探 `error` 字符串。后端契约见 `ccr-ui/src-tauri/src/commands/usage.rs`
   * 的 `UsageImportResultV2`。
   */
  is_optional_absent?: boolean
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
  results: UsageImportResult[]
  summary: UsageImportSummary
}

export type UsageImportJobStatus = 'pending' | 'running' | 'recent_ready' | 'finished' | 'failed' | 'cancelled'

export type UsageImportJobStage = 'queued' | 'importing_recent' | 'importing_history' | 'finished' | 'failed' | 'cancelled'

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
  results: UsageImportResult[]
  summary?: UsageImportSummary | null
  history_cursor_hit?: boolean
  live_sources?: number
  missing_sources?: number
  deleted_sources?: number
}

export interface StartUsageImportJobResponse {
  job_id: string
  snapshot: UsageImportJobSnapshot
}

export type SessionIndexJobStatus = 'pending' | 'running' | 'finished' | 'failed'

export type SessionIndexJobStage = 'queued' | 'indexing' | 'finished' | 'failed'

export interface SessionIndexJobSnapshot {
  job_id: string
  status: SessionIndexJobStatus
  stage: SessionIndexJobStage
  platforms_total: number
  platforms_completed: number
  files_total: number
  files_scanned: number
  sessions_added: number
  sessions_updated: number
  errors: number
  started_at: string
  updated_at: string
  finished_at?: string | null
  current_platform?: string | null
  warnings: string[]
  error?: string | null
}

export interface StartSessionIndexJobResponse {
  job_id: string
  snapshot: SessionIndexJobSnapshot
}

/** 平台类型 */
export type UsagePlatform = 'claude' | 'codex' | 'gemini' | 'opencode'
