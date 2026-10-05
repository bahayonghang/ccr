/**
 * CheckIn Domain —— 签到系统 API
 *
 * 对应后端 commands::checkin::* 命令。
 * 真迁移自 tauri.ts 第 11 分组（含扩展子节），
 * 另包含随签到分组归入的 WAF Cookie 读写命令（后端位于 commands::waf 但业务耦合签到）。
 *
 * OAuth 辅助 getOAuthAuthorizeUrl 为 HTTP-only 桩函数，Tauri 运行时直接返回失败响应，
 * 保留在此处以维持历史调用契约。
 */

import { invoke } from '@/api/invokeRuntime'
import { isRecord, pickArray, type UnknownRecord } from '../_shared'
import type { CheckinRecordsQuery } from '@/types/checkin'
import type { OAuthAuthorizeUrlRequest, OAuthAuthorizeUrlResponse } from '../tauri'

// ── Provider ──

/** 列出签到 Provider */
export const listCheckinProviders = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('list_providers')
}

/** 获取签到 Provider 详情（前端侧从列表过滤，后端无单条查询命令） */
export const getCheckinProvider = async <T = UnknownRecord>(id: string): Promise<T> => {
  const result = await invoke<unknown>('list_providers')
  const providers: unknown[] = Array.isArray(result) ? result : pickArray(result, 'providers')
  const found = providers.find((item) => isRecord(item) && String(item.id ?? '') === id)
  return (found ?? null) as T
}

/** 创建签到 Provider */
export const createCheckinProvider = async <T = UnknownRecord>(data: unknown): Promise<T> => {
  return invoke('add_provider', { data })
}

/** 更新签到 Provider */
export const updateCheckinProvider = async <T = UnknownRecord>(
  id: string,
  data: unknown
): Promise<T> => {
  return invoke('update_provider', { id, data })
}

/** 删除签到 Provider */
export const deleteCheckinProvider = async <T = UnknownRecord>(id: string): Promise<T> => {
  return invoke('delete_provider', { id })
}

/** 测试签到连接 */
export const testCheckinConnection = async <T = UnknownRecord>(id: string): Promise<T> => {
  return invoke('test_provider_connection', { id })
}

// ── Account ──

/** 列出签到账号 */
export const listCheckinAccounts = async <T = UnknownRecord>(providerId?: string): Promise<T> => {
  return invoke('list_accounts', { providerId })
}

/** 获取签到账号详情（前端侧从列表过滤） */
export const getCheckinAccount = async <T = UnknownRecord>(id: string): Promise<T> => {
  const result = await invoke<unknown>('list_accounts', { providerId: null })
  const accounts: unknown[] = Array.isArray(result) ? result : pickArray(result, 'accounts')
  const found = accounts.find((item) => isRecord(item) && String(item.id ?? '') === id)
  return (found ?? null) as T
}

/** 获取签到账号仪表盘（完整 dashboard 数据：account + streak + calendar + trend） */
export const getCheckinAccountDashboard = async <T = UnknownRecord>(
  id: string,
  query?: { year?: number; month?: number; days?: number }
): Promise<T> => {
  return invoke('get_account_dashboard', {
    accountId: id,
    year: query?.year ?? null,
    month: query?.month ?? null,
    days: query?.days ?? null,
  })
}

/** 创建签到账号 */
export const createCheckinAccount = async <T = UnknownRecord>(data: unknown): Promise<T> => {
  return invoke('add_account', { data })
}

/** 更新签到账号 */
export const updateCheckinAccount = async <T = UnknownRecord>(
  id: string,
  data: unknown
): Promise<T> => {
  return invoke('update_account', { id, data })
}

/** 删除签到账号 */
export const deleteCheckinAccount = async <T = UnknownRecord>(id: string): Promise<T> => {
  return invoke('delete_account', { id })
}

/** 批量删除签到账号 */
export const batchDeleteAccounts = async <T = UnknownRecord>(ids: string[]): Promise<T> => {
  return invoke('batch_delete_accounts', { ids })
}

// ── 执行 / 批量 / Job ──

/** 执行签到 */
export const executeCheckin = async <T = UnknownRecord>(accountId: string): Promise<T> => {
  return invoke('execute_checkin', { accountId })
}

/** 签到（executeCheckin 的别名，保持历史导入契约） */
export const checkinAccount = executeCheckin

/** 批量签到 */
export const batchCheckin = async <T = UnknownRecord>(accountIds: string[]): Promise<T> => {
  return invoke('batch_checkin', { accountIds })
}

/** 启动签到 Job（后端通过 delta 事件通道 emit 进度） */
export const startCheckinJob = async <T = UnknownRecord>(accountIds: string[]): Promise<T> => {
  return invoke('start_checkin_job', { accountIds })
}

/** 获取签到 Job 快照 */
export const getCheckinJobStatus = async <T = UnknownRecord>(jobId: string): Promise<T> => {
  return invoke('get_checkin_job_status', { jobId })
}

// ── 余额 ──

/** 查询签到余额 */
export const queryCheckinBalance = async <T = UnknownRecord>(accountId: string): Promise<T> => {
  return invoke('get_balance', { accountId })
}

/** 获取余额历史 */
export const getCheckinBalanceHistory = async <T = UnknownRecord>(
  accountId: string,
  days?: number
): Promise<T> => {
  return invoke('get_balance_history', { accountId, days })
}

/** 获取余额统计 */
export const getBalanceStats = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('get_balance_stats')
}

// ── 记录 ──

/** 列出签到记录（对象形式支持 status/provider_id/keyword/page/page_size SQL 级过滤） */
export const listCheckinRecords = async <T = UnknownRecord>(
  params?: number | CheckinRecordsQuery
): Promise<T> => {
  if (typeof params === 'number') {
    return invoke('get_checkin_records', { accountId: null, limit: params })
  }

  return invoke('get_checkin_records', {
    accountId: params?.account_id ?? null,
    limit: params?.limit ?? null,
    status: params?.status ?? null,
    providerId: params?.provider_id ?? null,
    keyword: params?.keyword ?? null,
    page: params?.page ?? 1,
    pageSize: params?.page_size ?? 20,
  })
}

/** 获取指定账号签到记录 */
export const getAccountCheckinRecords = async <T = UnknownRecord>(
  accountId: string,
  limit?: number
): Promise<T> => {
  return invoke('get_checkin_records', { accountId, limit })
}

/** 导出签到记录 */
export const exportCheckinRecords = async <T = UnknownRecord>(options: unknown): Promise<T> => {
  return invoke('export_checkin_data', { options })
}

/** 获取今日签到统计 */
export const getTodayCheckinStats = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('export_checkin_stats')
}

// ── CDK ──

/** 执行 CDK 充值 */
export const executeCdkRecharge = async <T = UnknownRecord>(
  accountId: string,
  cdkCode: string
): Promise<T> => {
  return invoke('execute_cdk_recharge', { accountId, cdkCode })
}

/** 获取 CDK 历史 */
export const getCdkHistory = async <T = UnknownRecord>(accountId?: string): Promise<T> => {
  return invoke('get_cdk_history', { accountId })
}

// ── WAF Cookies（归入 CheckIn 分组：业务耦合签到 Provider） ──

/** 列出 WAF Cookies */
export const listWafCookies = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('list_waf_cookies')
}

/** 添加 WAF Cookie */
export const addWafCookie = async <T = UnknownRecord>(
  providerId: string,
  cookie: string
): Promise<T> => {
  return invoke('add_waf_cookie', { providerId, cookie })
}

/** 删除 WAF Cookie */
export const deleteWafCookie = async <T = UnknownRecord>(id: string): Promise<T> => {
  return invoke('delete_waf_cookie', { id })
}

// ── CheckIn 扩展 ──

/** 获取签到账号 Cookies */
export const getCheckinAccountCookies = async <T = UnknownRecord>(
  accountId: string
): Promise<T> => {
  return invoke('get_checkin_account_cookies', { accountId })
}

/** 导出签到配置 */
export const exportCheckinConfig = async <T = UnknownRecord>(
  options?: Record<string, unknown>
): Promise<T> => {
  return invoke('export_checkin_config', { options: options ?? null })
}

/** 预览签到导入 */
export const previewCheckinImport = async <T = UnknownRecord>(data: unknown): Promise<T> => {
  return invoke('preview_checkin_import', { data })
}

/** 导入签到配置 */
export const importCheckinConfig = async <T = UnknownRecord>(
  data: unknown,
  options?: unknown
): Promise<T> => {
  return invoke('import_checkin_config', { data, options: options ?? null })
}

/** 列出内置 Provider */
export const listBuiltinProviders = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('list_builtin_providers')
}

/** 添加内置 Provider */
export const addBuiltinProvider = async <T = UnknownRecord>(providerId: string): Promise<T> => {
  return invoke('add_builtin_provider', { providerId })
}

// ── OAuth（HTTP-only 桩） ──

/** 获取 OAuth 授权链接（仅 HTTP 后端支持；Tauri 运行时返回失败响应） */
export const getOAuthAuthorizeUrl = async (
  _request: OAuthAuthorizeUrlRequest
): Promise<OAuthAuthorizeUrlResponse> => {
  return {
    success: false,
    message: '[Tauri] getOAuthAuthorizeUrl: 仅 HTTP 后端支持',
  }
}
