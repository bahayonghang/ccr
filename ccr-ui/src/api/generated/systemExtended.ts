/* Generated from commands/handler_registry.rs; do not edit. */

import { invoke } from '@tauri-apps/api/core'
import type { CliVersionEntry } from '@/types/generated/system/CliVersionEntry'
import type { CliVersionOptions } from '@/types/generated/system/CliVersionOptions'
import type { CliVersionsOptions } from '@/types/generated/system/CliVersionsOptions'
import type { CliVersionsResponse } from '@/types/generated/system/CliVersionsResponse'

export const getCliVersions = (options?: CliVersionsOptions): Promise<CliVersionsResponse> => invoke('get_cli_versions', { options })
export const getCliVersion = (options: CliVersionOptions): Promise<CliVersionEntry> => invoke('get_cli_version', { options })
