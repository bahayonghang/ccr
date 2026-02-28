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

/** 导入结果 */
export interface ImportResult {
  platform: string
  files_processed: number
  records_imported: number
  records_skipped: number
  duration_ms: number
  completed: boolean
}

/** 平台类型 */
export type Platform = 'claude' | 'codex' | 'gemini'
