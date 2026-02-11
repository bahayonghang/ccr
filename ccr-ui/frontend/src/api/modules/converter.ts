/**
 * Config Converter API Module
 *
 * 平台间配置格式转换 API
 */

import { api } from '../core'
import type {
    ApiResponse,
    ConverterRequest,
    ConverterResponse,
} from '@/types'

export const convertConfig = async (request: ConverterRequest): Promise<ConverterResponse> => {
    const response = await api.post<ApiResponse<ConverterResponse>>('/converter/convert', request)
    return response.data.data!
}
