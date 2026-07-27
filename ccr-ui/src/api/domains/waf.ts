/**
 * WAF Domain —— WebView bypass 登录与 Cookie 状态 API
 *
 * 对应后端 commands::waf::* 命令。
 * 真迁移自 tauri.ts 第 16 分组。
 */

import { invoke } from '@/api/invokeRuntime'
import type {
  WafCookieRecoveryResult,
  WafCookieStatus,
  WafCookieValidationResult,
} from '@/types/checkin'

/** 打开 WAF 登录窗口 */
export const openWafLogin = async <T = WafCookieRecoveryResult>(
  loginUrl: string,
  providerId: string,
): Promise<T> => {
  return invoke('open_waf_login', { loginUrl, providerId })
}

/** 获取 WAF Cookie 状态 */
export const getWafCookieStatus = async <T = WafCookieStatus>(providerId: string): Promise<T> => {
  return invoke('get_waf_cookie_status', { providerId })
}

/** 验证缓存的 WAF Cookie 是否能通过账号用户信息接口 */
export const validateWafCookieForAccount = async <T = WafCookieValidationResult>(
  accountId: string,
): Promise<T> => {
  return invoke('validate_waf_cookie_for_account', { accountId })
}
