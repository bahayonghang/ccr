import { invoke } from '@tauri-apps/api/core'
import { getSystemInfo as getSystemInfoTyped } from '../generated/systemInfo'
import {
  getCliVersion as getCliVersionTyped,
  getCliVersions as getCliVersionsTyped,
} from '../generated/systemExtended'
import type { CliVersionOptions } from '@/types/generated/system/CliVersionOptions'
import type { CliVersionsOptions } from '@/types/generated/system/CliVersionsOptions'

import type { UnknownRecord } from '@/types/common'

export type CliVersionsCommandOptions = CliVersionsOptions & {
  timeout?: number
}

export type CliVersionCommandOptions = CliVersionOptions

export const getSystemInfo = getSystemInfoTyped

export const healthCheck = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('health_check')
}

export const getCliVersions = async (
  options?: CliVersionsCommandOptions
) => {
  const normalizedOptions = options
    ? {
        mode: options.mode,
        timeoutMs: options.timeoutMs ?? options.timeout,
        parallelism: options.parallelism,
      }
    : undefined

  return getCliVersionsTyped(normalizedOptions)
}

export const getCliVersion = async (
  options: CliVersionCommandOptions
) => getCliVersionTyped(options)
