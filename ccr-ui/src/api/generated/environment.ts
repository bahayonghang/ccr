/* Generated from commands/handler_registry.rs; do not edit. */

import { invoke } from '@tauri-apps/api/core'
import type { EnvironmentInfo } from '@/types/generated/environment/EnvironmentInfo'

export const listEnvironments = (): Promise<EnvironmentInfo[]> => invoke('list_environments')
export const getCurrentEnvironment = (): Promise<EnvironmentInfo> => invoke('get_current_environment')
export const switchEnvironment = (envId: string): Promise<EnvironmentInfo> => invoke('switch_environment', { envId })
export const refreshEnvironments = (forceRefresh?: boolean): Promise<EnvironmentInfo[]> => invoke('refresh_environments', { forceRefresh })
