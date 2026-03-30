import { invoke } from '@tauri-apps/api/core'

type UnknownRecord = Record<string, unknown>

const isRecord = (value: unknown): value is UnknownRecord => {
  return typeof value === 'object' && value !== null
}

const pickRecord = (value: unknown, key: string): UnknownRecord => {
  if (!isRecord(value)) {
    return {}
  }

  const candidate = value[key]
  return isRecord(candidate) ? candidate : {}
}

export interface CliVersionsCommandOptions {
  mode?: 'fast' | 'full'
  timeoutMs?: number
  parallelism?: number
  timeout?: number
}

export interface CliVersionCommandOptions {
  tool: string
  timeoutMs?: number
  force?: boolean
}

export const getSystemInfo = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('get_system_info')
}

export const healthCheck = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('health_check')
}

export const getCliVersions = async <T = UnknownRecord>(
  options?: CliVersionsCommandOptions,
): Promise<T> => {
  const normalizedOptions = options
    ? {
        mode: options.mode,
        timeoutMs: options.timeoutMs ?? options.timeout,
        parallelism: options.parallelism,
      }
    : undefined

  const raw = await invoke<unknown>('get_cli_versions', { options: normalizedOptions })
  if (!isRecord(raw)) {
    return raw as T
  }

  const entries = Array.isArray(raw.entries)
    ? raw.entries
    : Array.isArray(raw.versions)
      ? raw.versions
      : Object.entries(pickRecord(raw, 'versions')).map(([platform, value]) => {
          const text = String(value ?? '')
          if (!text || text === 'not found') {
            return {
              platform,
              installed: false,
              status: 'not_installed',
            }
          }

          return {
            platform,
            installed: true,
            version: text,
            status: 'ok',
          }
        })

  return {
    ...raw,
    versions: entries,
  } as T
}

export const getCliVersion = async <T = UnknownRecord>(
  options: CliVersionCommandOptions,
): Promise<T> => {
  const normalizedOptions = {
    tool: options.tool,
    timeoutMs: options.timeoutMs,
    force: options.force,
  }

  return invoke('get_cli_version', { options: normalizedOptions })
}
