import { bootLocaleMessages } from '@/i18n/bootMessages'
import { readStoredLocale } from '@/i18n'
import type { TranslateFunction } from '@/utils/tf'

type MessageTree = Record<string, unknown>

const lookup = (tree: unknown, key: string): string | undefined => {
  const parts = key.split('.')
  let current: unknown = tree
  for (const part of parts) {
    if (!current || typeof current !== 'object') return undefined
    current = (current as MessageTree)[part]
  }
  return typeof current === 'string' ? current : undefined
}

const interpolate = (
  template: string,
  values: Record<string, string | number | boolean | null | undefined> = {},
): string => template.replace(/\{([a-zA-Z_][a-zA-Z0-9_]*)\}/g, (raw, name: string) => {
  const value = values[name]
  return value == null ? raw : String(value)
})

/** 功能面默认 t：读 boot catalog，未命中则返回 key。壳层可注入完整 t。 */
export const defaultSurfaceT: TranslateFunction = (key, values) => {
  const locale = readStoredLocale()
  const catalog = bootLocaleMessages[locale] as unknown
  const hit = lookup(catalog, key) ?? lookup(bootLocaleMessages['zh-CN'] as unknown, key)
  if (!hit) return key
  return interpolate(hit, values)
}
