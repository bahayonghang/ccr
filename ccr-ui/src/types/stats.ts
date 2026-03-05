// Statistics, Budget & Pricing type definitions

// ============ Statistics Types ============

export interface TokenStats {
  total_input_tokens: number;
  total_output_tokens: number;
  total_cache_tokens: number;
  cache_efficiency: number;
}

export interface DailyCost {
  date: string;
  cost: number;
  count: number;
}

export interface CostStats {
  total_cost: number;
  record_count: number;
  token_stats: TokenStats;
  by_provider: Record<string, number>; // provider -> count
  by_model: Record<string, number>;
  by_project: Record<string, number>;
  trend?: DailyCost[];
}

export interface TopSession {
  session_id: string;
  cost: number;
}

export interface StatsSummary {
  today_cost: number;
  week_cost: number;
  month_cost: number;
  total_sessions: number;
}

// ============ Usage Analytics Types ============

export interface UsageData {
  input_tokens?: number;
  output_tokens?: number;
  cache_read_input_tokens?: number;
}

export interface UsageRecord {
  uuid: string;
  timestamp: string;
  model?: string;
  usage?: UsageData;
}

export interface UsageRecordsResponse {
  records: UsageRecord[];
  total_records: number;
  truncated: boolean;
}

export type TimeRange = '5h' | 'today' | '7d' | 'week' | 'month' | 'all';

// ============ Usage Stats Dashboard Types ============

/** 使用统计视图模式 */
export type StatsViewMode = 'sessions' | 'duration' | 'tokens';

/** 平台每日统计 */
export interface PlatformDailyStats {
  sessions: number;
  messages: number;
  tokens: number;
  duration_seconds: number;
}

/** 每日统计项 */
export interface DailyStatsItem {
  date: string;
  claude: PlatformDailyStats;
  codex: PlatformDailyStats;
  gemini: PlatformDailyStats;
}

/** 使用统计汇总 */
export interface UsageStatsSummary {
  total_sessions: number;
  total_messages: number;
  total_duration_seconds: number;
  by_platform: Record<string, PlatformDailyStats>;
}

/** 每日统计 API 响应 */
export interface DailyStatsResponse {
  daily_stats: DailyStatsItem[];
  summary: UsageStatsSummary;
  last_updated: string;
}

// ============ Budget Management Types ============

export interface BudgetStatus {
  enabled: boolean;
  daily_limit: number | null;
  weekly_limit: number | null;
  monthly_limit: number | null;
  warn_threshold: number;
  current_costs: {
    today: number;
    this_week: number;
    this_month: number;
  };
  warnings: BudgetWarning[];
  last_updated: string;
}

export interface BudgetWarning {
  period: string;
  current_cost: number;
  limit: number;
  usage_percent: number;
}

export interface SetBudgetRequest {
  enabled?: boolean;
  daily_limit?: number | null;
  weekly_limit?: number | null;
  monthly_limit?: number | null;
  warn_threshold?: number;
}

// ============ Pricing Management Types ============

export interface ModelPricing {
  model: string;
  input_price: number;
  output_price: number;
  cache_read_price?: number;
  cache_write_price?: number;
}

export interface PricingListResponse {
  pricings: ModelPricing[];
  default_pricing: ModelPricing;
}

export interface SetPricingRequest {
  model: string;
  input_price: number;
  output_price: number;
  cache_read_price?: number;
  cache_write_price?: number;
}
