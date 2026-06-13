/**
 * API 层内部共享工具（仅限 @/api/* 与 @/api/domains/* 使用）
 *
 * 从 tauri.ts 提取，供 domain 文件复用，避免每个 domain 重复声明相同辅助。
 * 外部业务代码不应直接 import 此模块。
 */

import type { UnknownRecord } from '@/types/common'

export type { UnknownRecord }

export const isRecord = (value: unknown): value is UnknownRecord => {
  return typeof value === 'object' && value !== null
}

export const asRecord = (value: unknown): UnknownRecord => {
  return isRecord(value) ? value : {}
}

export const asArray = (value: unknown): unknown[] => {
  return Array.isArray(value) ? value : []
}

export const pickArray = (value: unknown, key: string): unknown[] => {
  if (!isRecord(value)) return []
  return asArray(value[key])
}

export const pickRecord = (value: unknown, key: string): UnknownRecord => {
  if (!isRecord(value)) return {}
  return asRecord(value[key])
}

/**
 * 兼容 (name, config) 与 ({ name, ...config }) 两种调用形态，解析为统一的 { name, config }。
 * 主要用于 Claude / Codex 等平台的 add/update 命令。
 */
export function resolveNameAndConfig(
  arg1: string | object,
  arg2?: unknown,
): { name: string; config: UnknownRecord } {
  if (typeof arg1 === 'string') {
    return { name: arg1, config: asRecord(arg2) }
  }

  const request = { ...asRecord(arg1) }
  const name = String(request.name ?? request.id ?? '')
  delete request.name
  delete request.id

  return { name, config: request }
}

export function resolveName(arg1: string | object): string {
  if (typeof arg1 === 'string') return arg1
  const request = asRecord(arg1)
  return String(request.name ?? request.id ?? '')
}
