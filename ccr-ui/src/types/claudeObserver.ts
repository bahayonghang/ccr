/**
 * Claude Observer 模块的 TypeScript 类型定义
 *
 * 与 ccr-ui/src-tauri/src/commands/claude_observer.rs 中的 DTO 形状保持一致。
 * 如果后端字段改名，需要同步更新此文件。
 */

/** 订阅设置（user_settings 三键的聚合视图） */
export interface SubscriptionDto {
  /** auto | api_key | subscription */
  mode: string
  /** free_pro | max5x | max20x | team | enterprise | custom */
  plan: string
  /** 月费美元；mode === 'subscription' 时参与 ROI 计算 */
  monthly_usd: number
}

/** 首屏 Hero 三卡 + 订阅 banner 一次性聚合数据 */
export interface InsightDto {
  today_value_usd: number
  month_value_usd: number
  total_value_usd: number
  today_tokens: number
  month_tokens: number
  total_sessions: number
  total_projects: number
  subscription: SubscriptionDto
  /** 仅当订阅模式时存在：当月 value / 月费 */
  roi: number | null
  /** 嵌入式价目表版本（来自 resources/claude_pricing.json） */
  pricing_version: string
}

/** 缓存效率统计（Token tab 顶部四卡） */
export interface CacheStatsDto {
  /** 0..1 之间，cache_read_tokens / (input_tokens + cache_read_tokens) */
  hit_rate: number
  total_input_tokens: number
  total_output_tokens: number
  total_cache_read_tokens: number
  total_cache_write_tokens: number
}

/** 单日趋势点（Cost / Token tab 横向时间序列） */
export interface DailyPoint {
  date: string
  cost_usd: number
  input_tokens: number
  output_tokens: number
  cache_read_tokens: number
  cache_write_tokens: number
}

/** 按 project 或 model 维度的费用拆分行 */
export interface BreakdownRow {
  key: string
  cost_usd: number
  tokens: number
  count: number
}

/** Top sessions 行（数据来自 claude_tool_calls 维度） */
export interface SessionRow {
  session_id: string
  project_path: string | null
  model: string | null
  cost_usd: number
  tokens: number
  tool_call_count: number
  started_at: string | null
}

/** 工具调用热力图单元格 */
export interface HeatmapCell {
  /** 0=Sun..6=Sat */
  dow: number
  /** 0..23 */
  hour: number
  count: number
}

/** Top tools 行（Behavior tab 横向条形图） */
export interface TopToolRow {
  tool_name: string
  call_count: number
  cost_usd: number
}
