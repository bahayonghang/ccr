/* Generated from commands/handler_registry.rs; do not edit. */

import { invoke } from '@/api/invokeRuntime'
import type { SystemInfo } from '@/types/generated/system/SystemInfo'
import type { VersionInfo } from '@/types/generated/system/VersionInfo'

export const getSystemInfo = (): Promise<SystemInfo> => invoke('get_system_info')
export const checkVersion = (): Promise<VersionInfo> => invoke('check_version')
