/**
 * WAF Domain —— WebView bypass 登录与 Cookie 状态 API
 *
 * 对应后端 commands::waf::* 命令。
 * 真迁移自 tauri.ts 第 16 分组。
 */

import { invoke } from '@tauri-apps/api/core'
import type { UnknownRecord } from '../_shared'

/** 打开 WAF 登录窗口 */
export const openWafLogin = async <T = UnknownRecord>(
  loginUrl: string,
  providerId: string,
): Promise<T> => {
  return invoke('open_waf_login', { loginUrl, providerId })
}

/** 获取 WAF Cookie 状态 */
export const getWafCookieStatus = async <T = UnknownRecord>(providerId: string): Promise<T> => {
  return invoke('get_waf_cookie_status', { providerId })
}
