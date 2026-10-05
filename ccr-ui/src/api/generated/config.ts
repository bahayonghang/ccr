/* Generated from commands/handler_registry.rs; do not edit. */

import { invoke } from '@/api/invokeRuntime'
import type { ConfigInfo } from '@/types/generated/config/ConfigInfo'
import type { ExportResult } from '@/types/generated/config/ExportResult'
import type { HistoryEntry } from '@/types/generated/config/HistoryEntry'
import type { ImportResult } from '@/types/generated/config/ImportResult'

import type { ConfigPlatform } from '@/types/generated/config/ConfigPlatform'
import type { ConfigPatchInput } from '@/types/generated/config/ConfigPatchInput'
import type { ConfigMutationResult } from '@/types/generated/config/ConfigMutationResult'
export type AddConfigInput = { platform: ConfigPlatform; name: string; data: ConfigPatchInput }
export type UpdateConfigInput = AddConfigInput & { expectedVersion?: string }
export type ImportConfigInput = { content: string; mode?: string; backup?: boolean }

const confirmationTokenFor = (action: 'delete_config' | 'import_config' | 'restore_config') => `desktop-confirm:${action}`

export const listConfigsTyped = (platform: ConfigPlatform): Promise<ConfigInfo[]> => invoke('list_configs', { platform })
export const switchConfigTyped = (platform: ConfigPlatform, name: string, enable = false): Promise<ConfigMutationResult> => invoke('switch_config', { platform, name, enable })
export const addConfigTyped = (input: AddConfigInput): Promise<ConfigMutationResult> => invoke('add_config', input)
export const deleteConfigTyped = (platform: ConfigPlatform, name: string): Promise<ConfigMutationResult> => invoke('delete_config', { platform, name, confirmationToken: confirmationTokenFor('delete_config') })
export const renameConfigTyped = (platform: ConfigPlatform, oldName: string, newName: string): Promise<ConfigMutationResult> => invoke('rename_config', { platform, oldName, newName })
export const duplicateConfigTyped = (platform: ConfigPlatform, source: string, target: string): Promise<ConfigMutationResult> => invoke('duplicate_config', { platform, source, target })
export const updateConfigTyped = (input: UpdateConfigInput): Promise<ConfigMutationResult> => invoke('update_config', input)
export const validateConfigsTyped = (): Promise<string> => invoke('validate_configs')
export const importConfigTyped = (input: ImportConfigInput): Promise<ImportResult> => invoke('import_config', { content: input.content, mode: input.mode ?? 'merge', backup: input.backup ?? true, confirmationToken: confirmationTokenFor('import_config') })
export const restoreConfigTyped = (backupPath: string): Promise<string> => invoke('restore_config', { backupPath, confirmationToken: confirmationTokenFor('restore_config') })
export const exportConfigTyped = (includeSecrets = false): Promise<ExportResult> => invoke('export_config', { includeSecrets })
export const getHistoryTyped = (limit = 100): Promise<HistoryEntry[]> => invoke('get_history', { limit })
export const clearHistoryTyped = (): Promise<string> => invoke('clear_history')
