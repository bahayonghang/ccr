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

/** 账号信息（包含遮罩的 API Key） */
export interface AccountInfo {
  id: string
  provider_id: string
  provider_name?: string
  name: string
  api_key_masked: string
  enabled: boolean
  created_at: string
  last_checkin_at?: string
  last_balance_check_at?: string
  latest_balance?: number
  balance_currency?: string
}

/** 创建账号请求 */
export interface CreateAccountRequest {
  provider_id: string
  name: string
  api_key: string
}

/** 更新账号请求 */
export interface UpdateAccountRequest {
  name?: string
  api_key?: string
  enabled?: boolean
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
export type CheckinStatus = 'Success' | 'AlreadyCheckedIn' | 'Failed'

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
  status: CheckinStatus
  message?: string
  reward?: string
  balance_after?: number
  checked_in_at: string
}

/** 签到记录响应 */
export interface CheckinRecordsResponse {
  records: CheckinRecordInfo[]
  total: number
}

// ═══════════════════════════════════════════════════════════
// 余额类型
// ═══════════════════════════════════════════════════════════

/** 余额快照 */
export interface BalanceSnapshot {
  id: string
  account_id: string
  total_quota: number
  used_quota: number
  remaining_quota: number
  usage_percentage: number
  currency: string
  raw_response?: string
  recorded_at: string
}

/** 余额历史项 */
export interface BalanceHistoryItem {
  remaining_quota: number
  usage_percentage: number
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
  api_key: string
  api_key_encrypted: boolean
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

/** 导入请求 */
export interface ImportRequest {
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
// 连接测试类型
// ═══════════════════════════════════════════════════════════

/** 连接测试响应 */
export interface TestConnectionResponse {
  success: boolean
  message: string
}
