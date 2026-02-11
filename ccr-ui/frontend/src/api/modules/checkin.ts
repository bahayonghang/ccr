/**
 * Checkin Management API Module
 *
 * 签到管理：提供商、账号、签到操作、余额、记录、统计、导入导出
 */

import { api } from '../core'
import type {
    CheckinProvider,
    CreateProviderRequest,
    UpdateProviderRequest,
    ProvidersResponse,
    AccountInfo,
    CreateAccountRequest,
    UpdateAccountRequest,
    AccountsResponse,
    CheckinRequest,
    CheckinResponse,
    CheckinExecutionResult,
    CheckinRecordsResponse,
    CheckinRecordsQuery,
    BalanceSnapshot,
    BalanceHistoryResponse,
    TodayCheckinStats,
    CheckinAccountDashboardResponse,
    ExportOptions as CheckinExportOptions,
    ExportData as CheckinExportData,
    ImportPreviewResponse as CheckinImportPreviewResponse,
    CheckinImportRequest,
    ImportResult as CheckinImportResult,
    TestConnectionResponse,
    BuiltinProvidersResponse,
    AddBuiltinProviderRequest,
} from '@/types/checkin'

// ═══════════════════════════════════════════════════════════
// 📋 本地类型定义 (Local Types)
// ═══════════════════════════════════════════════════════════

/** 账号 Cookies 响应（用于编辑） */
export interface AccountCookiesResponse {
    cookies_json: string
    api_user: string
}

// ═══════════════════════════════════════════════════════════
// 🏢 提供商管理 (Provider Management)
// ═══════════════════════════════════════════════════════════

/** 获取所有签到提供商 */
export const listCheckinProviders = async (): Promise<ProvidersResponse> => {
    const response = await api.get<ProvidersResponse>('/checkin/providers')
    return response.data
}

/** 获取单个签到提供商 */
export const getCheckinProvider = async (id: string): Promise<CheckinProvider> => {
    const response = await api.get<CheckinProvider>(`/checkin/providers/${id}`)
    return response.data
}

/** 创建签到提供商 */
export const createCheckinProvider = async (request: CreateProviderRequest): Promise<CheckinProvider> => {
    const response = await api.post<CheckinProvider>('/checkin/providers', request)
    return response.data
}

/** 更新签到提供商 */
export const updateCheckinProvider = async (id: string, request: UpdateProviderRequest): Promise<CheckinProvider> => {
    const response = await api.put<CheckinProvider>(`/checkin/providers/${id}`, request)
    return response.data
}

/** 删除签到提供商 */
export const deleteCheckinProvider = async (id: string): Promise<void> => {
    await api.delete(`/checkin/providers/${id}`)
}

// ═══════════════════════════════════════════════════════════
// 🏗️ 内置提供商 (Builtin Providers)
// ═══════════════════════════════════════════════════════════

/** 获取所有内置提供商 */
export const listBuiltinProviders = async (): Promise<BuiltinProvidersResponse> => {
    const response = await api.get<BuiltinProvidersResponse>('/checkin/providers/builtin')
    return response.data
}

/** 添加内置提供商到用户配置 */
export const addBuiltinProvider = async (builtinId: string): Promise<CheckinProvider> => {
    const request: AddBuiltinProviderRequest = { builtin_id: builtinId }
    const response = await api.post<CheckinProvider>('/checkin/providers/builtin/add', request)
    return response.data
}

// ═══════════════════════════════════════════════════════════
// 👤 账号管理 (Account Management)
// ═══════════════════════════════════════════════════════════

/** 获取所有签到账号 */
export const listCheckinAccounts = async (providerId?: string): Promise<AccountsResponse> => {
    const params = providerId ? { provider_id: providerId } : {}
    const response = await api.get<AccountsResponse>('/checkin/accounts', { params })
    return response.data
}

/** 获取单个签到账号 */
export const getCheckinAccount = async (id: string): Promise<AccountInfo> => {
    const response = await api.get<AccountInfo>(`/checkin/accounts/${id}`)
    return response.data
}

/** 获取账号 Dashboard 聚合数据 */
export const getCheckinAccountDashboard = async (
    id: string,
    params?: { year?: number; month?: number; days?: number }
): Promise<CheckinAccountDashboardResponse> => {
    const response = await api.get<CheckinAccountDashboardResponse>(`/checkin/accounts/${id}/dashboard`, { params })
    return response.data
}

/** 创建签到账号 */
export const createCheckinAccount = async (request: CreateAccountRequest): Promise<AccountInfo> => {
    const response = await api.post<AccountInfo>('/checkin/accounts', request)
    return response.data
}

/** 更新签到账号 */
export const updateCheckinAccount = async (id: string, request: UpdateAccountRequest): Promise<AccountInfo> => {
    const response = await api.put<AccountInfo>(`/checkin/accounts/${id}`, request)
    return response.data
}

/** 删除签到账号 */
export const deleteCheckinAccount = async (id: string): Promise<void> => {
    await api.delete(`/checkin/accounts/${id}`)
}

/** 获取账号的解密后 Cookies（用于编辑） */
export const getCheckinAccountCookies = async (id: string): Promise<AccountCookiesResponse> => {
    const response = await api.get<AccountCookiesResponse>(`/checkin/accounts/${id}/cookies`)
    return response.data
}

// ═══════════════════════════════════════════════════════════
// ✅ 签到操作 (Checkin Operations)
// ═══════════════════════════════════════════════════════════

/** 执行签到（批量或全部） */
export const executeCheckin = async (request?: CheckinRequest): Promise<CheckinResponse> => {
    const response = await api.post<CheckinResponse>('/checkin/execute', request || {})
    return response.data
}

/** 执行单个账号签到 */
export const checkinAccount = async (id: string): Promise<CheckinExecutionResult> => {
    const response = await api.post<CheckinExecutionResult>(`/checkin/accounts/${id}/checkin`)
    return response.data
}

// ═══════════════════════════════════════════════════════════
// 💰 余额查询 (Balance)
// ═══════════════════════════════════════════════════════════

/** 查询账号余额 */
export const queryCheckinBalance = async (id: string): Promise<BalanceSnapshot> => {
    const response = await api.post<BalanceSnapshot>(`/checkin/accounts/${id}/balance`)
    return response.data
}

/** 获取账号余额历史 */
export const getCheckinBalanceHistory = async (id: string, limit?: number): Promise<BalanceHistoryResponse> => {
    const params = limit ? { limit } : {}
    const response = await api.get<BalanceHistoryResponse>(`/checkin/accounts/${id}/balance/history`, { params })
    return response.data
}

// ═══════════════════════════════════════════════════════════
// 📋 签到记录 (Records)
// ═══════════════════════════════════════════════════════════

/** 获取所有签到记录 */
export const listCheckinRecords = async (params?: CheckinRecordsQuery): Promise<CheckinRecordsResponse> => {
    const response = await api.get<CheckinRecordsResponse>('/checkin/records', { params: params || {} })
    return response.data
}

/** 获取账号签到记录 */
export const getAccountCheckinRecords = async (
    id: string,
    params?: CheckinRecordsQuery
): Promise<CheckinRecordsResponse> => {
    const response = await api.get<CheckinRecordsResponse>(`/checkin/accounts/${id}/records`, {
        params: params || {},
    })
    return response.data
}

/** 导出签到记录 */
export const exportCheckinRecords = async (
    params?: CheckinRecordsQuery
): Promise<{ blob: Blob; filename: string }> => {
    const response = await api.get('/checkin/records/export', {
        params: params || {},
        responseType: 'blob',
    })
    const disposition = (response.headers['content-disposition'] as string | undefined) || ''
    const match = disposition.match(/filename="?(?<filename>[^"]+)"?/i)
    const filename = match?.groups?.filename || 'checkin_records.json'
    return {
        blob: response.data as Blob,
        filename,
    }
}

// ═══════════════════════════════════════════════════════════
// 📊 统计 (Statistics)
// ═══════════════════════════════════════════════════════════

/** 获取今日签到统计 */
export const getTodayCheckinStats = async (): Promise<TodayCheckinStats> => {
    const response = await api.get<TodayCheckinStats>('/checkin/stats/today')
    return response.data
}

// ═══════════════════════════════════════════════════════════
// 📤 导入/导出 (Import/Export)
// ═══════════════════════════════════════════════════════════

/** 导出签到配置 */
export const exportCheckinConfig = async (options?: CheckinExportOptions): Promise<CheckinExportData> => {
    const response = await api.post<CheckinExportData>('/checkin/export', options || {})
    return response.data
}

/** 预览导入 */
export const previewCheckinImport = async (data: CheckinExportData): Promise<CheckinImportPreviewResponse> => {
    const response = await api.post<CheckinImportPreviewResponse>('/checkin/import/preview', data)
    return response.data
}

/** 执行导入 */
export const importCheckinConfig = async (request: CheckinImportRequest): Promise<CheckinImportResult> => {
    const response = await api.post<CheckinImportResult>('/checkin/import', request)
    return response.data
}

// ═══════════════════════════════════════════════════════════
// 🔗 连接测试 (Connection Test)
// ═══════════════════════════════════════════════════════════

/** 测试账号连接 */
export const testCheckinConnection = async (id: string): Promise<TestConnectionResponse> => {
    const response = await api.post<TestConnectionResponse>(`/checkin/accounts/${id}/test`)
    return response.data
}
