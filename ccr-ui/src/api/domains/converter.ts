/**
 * Converter Domain —— 配置格式转换 API
 *
 * 对应后端 commands::converter::* 命令。
 * 真迁移自 tauri.ts 第 14 分组。
 */

import { invoke } from '@tauri-apps/api/core'
import type { UnknownRecord } from '../_shared'

/** 转换配置格式 */
export const convertConfig = async <T = UnknownRecord>(request: unknown): Promise<T> => {
  return invoke('convert_config', { request })
}
