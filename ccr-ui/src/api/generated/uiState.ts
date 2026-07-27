/* Generated from commands/handler_registry.rs; do not edit. */

import { invoke } from '@tauri-apps/api/core'
import type { CommandHistoryDto } from '@/types/generated/ui_state/CommandHistoryDto'
import type { FavoriteCommandDto } from '@/types/generated/ui_state/FavoriteCommandDto'

export const getFavorites = (): Promise<FavoriteCommandDto[]> => invoke('get_favorites')
export const addFavorite = (command: string, args: string[], displayName: string | null | undefined, module: string): Promise<FavoriteCommandDto> =>
  invoke('add_favorite', { command, args, displayName: displayName ?? null, module })
export const removeFavorite = (id: string): Promise<boolean> => invoke('remove_favorite', { id })
export const getRecentItems = (limit?: number): Promise<CommandHistoryDto[]> => invoke('get_recent_items', { limit })
export const addRecentItem = (command: string, args: string[], success: boolean, durationMs: number): Promise<CommandHistoryDto> =>
  invoke('add_recent_item', { command, args, success, durationMs })
export const clearRecentItems = (): Promise<string> => invoke('clear_recent_items')
