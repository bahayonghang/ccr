/**
 * Sync (WebDAV) API Module
 *
 * WebDAV 同步管理 API
 */

import { api } from '../core'
import type {
    ApiResponse,
    SyncStatusResponse,
    SyncOperationRequest,
    SyncOperationResponse,
    SyncInfoResponse,
} from '@/types'

export const getSyncStatus = async (): Promise<SyncStatusResponse> => {
    const response = await api.get<ApiResponse<SyncStatusResponse>>('/sync/status')
    return response.data.data!
}

export const getSyncInfo = async (): Promise<SyncInfoResponse> => {
    const response = await api.get<ApiResponse<SyncInfoResponse>>('/sync/info')
    return response.data.data!
}

export const pushSync = async (req: SyncOperationRequest): Promise<SyncOperationResponse> => {
    const response = await api.post<ApiResponse<SyncOperationResponse>>('/sync/push', req)
    return response.data.data!
}

export const pullSync = async (req: SyncOperationRequest): Promise<SyncOperationResponse> => {
    const response = await api.post<ApiResponse<SyncOperationResponse>>('/sync/pull', req)
    return response.data.data!
}
