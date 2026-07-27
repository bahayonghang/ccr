/* Generated from commands/handler_registry.rs; do not edit. */

import { invoke } from '@tauri-apps/api/core'
import type { ConfigInfo } from '@/types/generated/config/ConfigInfo'
import type { ExportResult } from '@/types/generated/config/ExportResult'
import type { HistoryEntry } from '@/types/generated/config/HistoryEntry'
import type { ImportResult } from '@/types/generated/config/ImportResult'

export type AddConfigInput = {
  name: string
  description?: string | null
  baseUrl: string
  authToken: string
  model?: string | null
  smallFastModel?: string | null
  provider?: string | null
  providerType?: string | null
  account?: string | null
  tags?: string[] | null
}
export type ImportConfigInput = { content: string; mode?: string; backup?: boolean }

const confirmationTokenFor = (action: 'delete_config' | 'import_config' | 'restore_config') => `desktop-confirm:${action}`

export const listConfigsTyped = (): Promise<ConfigInfo[]> => invoke('list_configs')
export const switchConfigTyped = (name: string): Promise<string> => invoke('switch_config', { name })
export const addConfigTyped = (input: AddConfigInput): Promise<string> => invoke('add_config', input)
export const deleteConfigTyped = (name: string): Promise<string> => invoke('delete_config', { name, confirmationToken: confirmationTokenFor('delete_config') })
export const renameConfigTyped = (oldName: string, newName: string): Promise<string> => invoke('rename_config', { oldName, newName })
export const duplicateConfigTyped = (source: string, target: string): Promise<string> => invoke('duplicate_config', { source, target })
export const validateConfigsTyped = (): Promise<string> => invoke('validate_configs')
export const importConfigTyped = (input: ImportConfigInput): Promise<ImportResult> => invoke('import_config', { content: input.content, mode: input.mode ?? 'merge', backup: input.backup ?? true, confirmationToken: confirmationTokenFor('import_config') })
export const restoreConfigTyped = (backupPath: string): Promise<string> => invoke('restore_config', { backupPath, confirmationToken: confirmationTokenFor('restore_config') })
export const exportConfigTyped = (includeSecrets = false): Promise<ExportResult> => invoke('export_config', { includeSecrets })
export const getHistoryTyped = (limit = 100): Promise<HistoryEntry[]> => invoke('get_history', { limit })
export const clearHistoryTyped = (): Promise<string> => invoke('clear_history')
