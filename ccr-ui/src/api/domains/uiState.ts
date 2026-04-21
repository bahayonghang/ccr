/**
 * UiState Domain —— 收藏与最近项 API
 *
 * 对应后端 commands::ui_state::* 命令。
 * 真迁移自 tauri.ts 第 15 分组（Favorites / Recent Items）。
 */

import { invoke } from '@tauri-apps/api/core'
import type { UnknownRecord } from '../_shared'

/** 获取收藏列表 */
export const getFavorites = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('get_favorites')
}

/** 添加收藏 */
export const addFavorite = async <T = UnknownRecord>(
  command: string,
  args: string[],
  displayName: string | undefined,
  module: string,
): Promise<T> => {
  return invoke('add_favorite', { command, args, displayName, module })
}

/** 移除收藏 */
export const removeFavorite = async <T = UnknownRecord>(id: string): Promise<T> => {
  return invoke('remove_favorite', { id })
}

/** 获取最近项目 */
export const getRecentItems = async <T = UnknownRecord>(limit?: number): Promise<T> => {
  return invoke('get_recent_items', { limit })
}

/** 添加最近项目 */
export const addRecentItem = async <T = UnknownRecord>(
  command: string,
  args: string[],
  success: boolean,
  durationMs: number,
): Promise<T> => {
  return invoke('add_recent_item', { command, args, success, durationMs })
}

/** 清空最近项目 */
export const clearRecentItems = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('clear_recent_items')
}
