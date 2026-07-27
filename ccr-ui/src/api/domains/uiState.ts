/**
 * UiState Domain —— 收藏与最近项 API
 *
 * 对应后端 commands::ui_state::* 命令。
 * 真迁移自 tauri.ts 第 15 分组（Favorites / Recent Items）。
 */

export {
  addFavorite,
  addRecentItem,
  clearRecentItems,
  getFavorites,
  getRecentItems,
  removeFavorite,
} from '../generated/uiState'
