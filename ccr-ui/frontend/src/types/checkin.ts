// 签到功能类型定义

// ═══════════════════════════════════════════════════════════
// 提供商类型
// ═══════════════════════════════════════════════════════════

/** 签到提供商（中转站） */
export interface CheckinProvider {
  id: string
  name: string
  base_url: string
  checkin_path: string
  balance_path: string
  user_info_path: string
  auth_header: string
  auth_prefix: string
  enabled: boolean
  created_at: string
  updated_at?: string
}

/** 内置提供商定义 */
export interface BuiltinProvider {
  id: string
  name: string
  description: string
  domain: string
  base_url: string
  checkin_path?: string
  balance_path: string
  user_info_path: string
  auth_header: string
  auth_prefix: string
  supports_checkin: boolean
  requires_waf_bypass: boolean
  requires_cf_clearance: boolean
  checkin_bugged: boolean
  icon: string
  category: string
  /** CDK 配置（仅 CDK 站点有） */
  cdk_config?: CdkProviderConfig
  /** OAuth 登录配置（支持 GitHub/LinuxDo OAuth 的站点） */
  oauth_config?: OAuthProviderConfig
}

/** CDK 提供商配置 */
export interface CdkProviderConfig {
  /** CDK 类型: "runawaytime" | "b4u" | "x666" */
  cdk_type: string
  /** CDK 来源站点 URL */
  cdk_source_url: string
  /** 充值路径 (如 "/api/user/topup")，x666 为 null */
  topup_path?: string
  /** 是否需要额外的 CDK cookies */
  requires_cdk_cookies: boolean
  /** 是否需要 access_token (x666) */
  requires_access_token: boolean
}

/** 内置提供商列表响应 */
export interface BuiltinProvidersResponse {
  providers: BuiltinProvider[]
  total: number
}

/** 添加内置提供商请求 */
export interface AddBuiltinProviderRequest {
  builtin_id: string
}

/** 创建提供商请求 */
export interface CreateProviderRequest {
  name: string
  base_url: string
  checkin_path?: string
  balance_path?: string
  user_info_path?: string
  auth_header?: string
  auth_prefix?: string
}

/** 更新提供商请求 */
export interface UpdateProviderRequest {
  name?: string
  base_url?: string
  checkin_path?: string
  balance_path?: string
  user_info_path?: string
  auth_header?: string
  auth_prefix?: string
  enabled?: boolean
}

/** 提供商列表响应 */
export interface ProvidersResponse {
  providers: CheckinProvider[]
  total: number
}


// ═══════════════════════════════════════════════════════════
// 账号类型
// ═══════════════════════════════════════════════════════════

/** 账号信息（包含遮罩的 Cookies） */
export interface AccountInfo {
  id: string
  provider_id: string
  provider_name?: string
  name: string
  cookies_masked: string
  api_user: string
  enabled: boolean
  created_at: string
  last_checkin_at?: string
  last_balance_check_at?: string
  latest_balance?: number
  balance_currency?: string
  total_quota?: number
  total_consumed?: number
  /** 扩展配置 (CDK 凭证等) */
  extra_config?: string
}

/** 创建账号请求 */
export interface CreateAccountRequest {
  provider_id: string
  name: string
  cookies_json: string
  api_user?: string
  /** 扩展配置 JSON (CDK 凭证等) */
  extra_config?: string
}

/** 更新账号请求 */
export interface UpdateAccountRequest {
  name?: string
  cookies_json?: string
  api_user?: string
  enabled?: boolean
  /** 扩展配置 JSON (CDK 凭证等) */
  extra_config?: string
}

/** 账号列表响应 */
export interface AccountsResponse {
  accounts: AccountInfo[]
  total: number
}

// ═══════════════════════════════════════════════════════════
// 签到操作类型
// ═══════════════════════════════════════════════════════════

/** 签到状态 */
export type CheckinStatus = 'success' | 'already_checked_in' | 'failed'

/** 签到执行结果 */
export interface CheckinExecutionResult {
  account_id: string
  account_name: string
  provider_name: string
  status: CheckinStatus
  message?: string
  reward?: string
  balance?: number
}

/** 签到请求 */
export interface CheckinRequest {
  account_ids?: string[]
}

/** 签到汇总 */
export interface CheckinSummary {
  total: number
  success: number
  already_checked_in: number
  failed: number
}

/** 签到响应 */
export interface CheckinResponse {
  results: CheckinExecutionResult[]
  summary: CheckinSummary
}

// ═══════════════════════════════════════════════════════════
// 签到记录类型
// ═══════════════════════════════════════════════════════════

/** 签到记录信息 */
export interface CheckinRecordInfo {
  id: string
  account_id: string
  account_name?: string
  provider_name?: string
  status: CheckinStatus
  message?: string
  reward?: string
  balance_before?: number
  balance_after?: number
  balance_change?: number
  checked_in_at: string
}

/** 签到记录响应 */
export interface CheckinRecordsResponse {
  records: CheckinRecordInfo[]
  total: number
}

/** 签到记录查询参数 */
export interface CheckinRecordsQuery {
  limit?: number
  page?: number
  page_size?: number
  status?: string
  account_id?: string
  provider_id?: string
  keyword?: string
}

// ═══════════════════════════════════════════════════════════
// 余额类型
// ═══════════════════════════════════════════════════════════

/** 余额快照 */
export interface BalanceSnapshot {
  account_id: string
  total_quota: number
  used_quota: number
  remaining_quota: number
  usage_percentage: number
  currency: string
  recorded_at: string
}

/** 余额历史项 */
export interface BalanceHistoryItem {
  total_quota: number
  used_quota: number
  remaining_quota: number
  usage_percentage?: number
  change?: number
  currency: string
  recorded_at: string
}

/** 余额历史响应 */
export interface BalanceHistoryResponse {
  account_id: string
  history: BalanceHistoryItem[]
  total: number
}

// ═══════════════════════════════════════════════════════════
// 统计类型
// ═══════════════════════════════════════════════════════════

/** 今日签到统计 */
export interface TodayCheckinStats {
  total_accounts: number
  checked_in: number
  not_checked_in: number
  failed: number
}

// ═══════════════════════════════════════════════════════════
// 导入/导出类型
// ═══════════════════════════════════════════════════════════

/** 导出选项 */
export interface ExportOptions {
  include_plaintext_keys?: boolean
  providers_only?: boolean
}

/** 导出账号数据 */
export interface ExportAccount {
  id: string
  provider_id: string
  name: string
  cookies_json: string
  cookies_json_encrypted: boolean
  api_user: string
  enabled: boolean
  created_at: string
}

/** 导出数据 */
export interface ExportData {
  version: string
  exported_at: string
  providers: CheckinProvider[]
  accounts: ExportAccount[]
}

/** 导入选项 */
export interface ImportOptions {
  conflict_strategy?: 'skip' | 'overwrite'
  providers_only?: boolean
  accounts_only?: boolean
}

/** 导入预览项 */
export interface ImportPreviewItem {
  item_type: 'provider' | 'account'
  name: string
  id: string
  has_conflict: boolean
  conflict_with?: string
}

/** 导入预览响应 */
export interface ImportPreviewResponse {
  version_compatible: boolean
  export_version: string
  items: ImportPreviewItem[]
  new_providers: number
  conflicting_providers: number
  new_accounts: number
  conflicting_accounts: number
  keys_encrypted: boolean
  warnings: string[]
}

/** 签到导入请求 */
export interface CheckinImportRequest {
  data: ExportData
  options: ImportOptions
}

/** 导入结果 */
export interface ImportResult {
  success: boolean
  message: string
  providers_imported: number
  providers_skipped: number
  accounts_imported: number
  accounts_skipped: number
  accounts_need_reauth: number
  warnings: string[]
}

// ═══════════════════════════════════════════════════════════
// 账号 Dashboard 类型
// ═══════════════════════════════════════════════════════════

/** 账号概览 */
export interface CheckinDashboardAccount {
  id: string
  name: string
  provider_id: string
  provider_name: string
  enabled: boolean
  last_checkin_at?: string
  last_balance_check_at?: string
  latest_balance?: number
  balance_currency?: string
  total_quota?: number
  used_quota?: number
  remaining_quota?: number
}

/** 连续签到统计 */
export interface CheckinDashboardStreak {
  current_streak: number
  longest_streak: number
  total_check_in_days: number
  last_check_in_date?: string
}

/** 月度统计 */
export interface CheckinDashboardMonthStats {
  total_days: number
  checked_in_days: number
  check_in_rate: number
  total_quota_increment: number
}

/** 日历单日数据 */
export interface CheckinDashboardDay {
  date: string
  is_checked_in: boolean
  income_increment?: number | null
  current_balance: number
  total_consumed: number
  total_quota: number
}

/** 月度日历 */
export interface CheckinDashboardCalendar {
  account_id: string
  year: number
  month: number
  days: CheckinDashboardDay[]
  month_stats: CheckinDashboardMonthStats
}

/** 趋势数据点 */
export interface CheckinDashboardTrendPoint {
  date: string
  total_quota: number
  income_increment: number
  current_balance: number
  is_checked_in: boolean
}

/** 趋势数据 */
export interface CheckinDashboardTrend {
  account_id: string
  start_date: string
  end_date: string
  data_points: CheckinDashboardTrendPoint[]
}

/** 账号 Dashboard 聚合响应 */
export interface CheckinAccountDashboardResponse {
  account: CheckinDashboardAccount
  streak: CheckinDashboardStreak
  calendar: CheckinDashboardCalendar
  trend: CheckinDashboardTrend
}

// ═══════════════════════════════════════════════════════════
// 连接测试类型
// ═══════════════════════════════════════════════════════════

/** 连接测试响应 */
export interface TestConnectionResponse {
  success: boolean
  message: string
}

// ═══════════════════════════════════════════════════════════
// 签到进度类型（用于 UI 显示）
// ═══════════════════════════════════════════════════════════

/** 签到日志条目状态 */
export type CheckinLogStatus = 'pending' | 'processing' | 'success' | 'already_checked_in' | 'failed'

/** 签到日志条目 */
export interface CheckinLogEntry {
  accountId: string
  accountName: string
  providerName: string
  status: CheckinLogStatus
  message?: string
  reward?: string
  balance?: number
  timestamp: Date
}

/** 签到进度状态 */
export interface CheckinProgressState {
  isRunning: boolean
  total: number
  completed: number
  currentAccountName: string
}

// ═══════════════════════════════════════════════════════════
// CDK 充值类型
// ═══════════════════════════════════════════════════════════

/** CDK 充值结果 */
export interface CdkTopupResult {
  /** CDK 类型 */
  cdk_type: string
  /** 是否整体成功 */
  success: boolean
  /** 汇总消息 */
  message: string
  /** 获取到的 CDK 码列表 */
  codes_found: string[]
  /** 成功充值的数量 */
  codes_redeemed: number
  /** 失败的充值详情 */
  failed_codes: CdkRedeemError[]
  /** x666 直接奖励金额 (仅 x666) */
  direct_reward?: string
}

/** CDK 充值失败详情 */
export interface CdkRedeemError {
  code: string
  error: string
}

/** CDK 扩展配置 (存储在 account.extra_config JSON 中) */
export interface CdkExtraConfig {
  /** fuli.hxi.me cookies (runawaytime) */
  fuli_cookies?: Record<string, string>
  /** tw.b4u.qzz.io cookies (b4u) */
  b4u_cdk_cookies?: Record<string, string>
  /** up.x666.me JWT access_token (x666) */
  x666_access_token?: string
}

// ═══════════════════════════════════════════════════════════
// OAuth 登录类型
// ═══════════════════════════════════════════════════════════

/** OAuth 提供商配置 */
export interface OAuthProviderConfig {
  /** GitHub OAuth client_id */
  github_client_id?: string
  /** LinuxDo OAuth client_id */
  linuxdo_client_id?: string
  /** OAuth state 获取路径 */
  oauth_state_path: string
}

/** OAuth 授权 URL 请求 */
export interface OAuthStateRequest {
  provider_id: string
  oauth_type: 'github' | 'linuxdo'
}

/** OAuth 授权 URL 响应 */
export interface OAuthStateResponse {
  success: boolean
  authorize_url?: string
  provider_name: string
  oauth_type: string
  message?: string
  /** 引导用户提取 cookies 的说明 */
  extraction_guide: string[]
}
