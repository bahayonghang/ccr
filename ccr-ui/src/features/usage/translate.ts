import { useSyncExternalStore } from 'react'
import { bootLocaleMessages } from '@/i18n/bootMessages'
import { readStoredLocale, type SupportedLocale } from '@/i18n'
import type { TranslateFunction } from '@/utils/tf'

type MessageTree = Record<string, unknown>
type TranslateValues = Record<string, string | number | boolean | null | undefined>

const catalogs: Record<SupportedLocale, MessageTree> = {
  'zh-CN': bootLocaleMessages['zh-CN'] as unknown as MessageTree,
  'en-US': bootLocaleMessages['en-US'] as unknown as MessageTree,
}

const hydrated = new Set<SupportedLocale>()
let catalogVersion = 0
const listeners = new Set<() => void>()

const subscribeCatalog = (listener: () => void) => {
  listeners.add(listener)
  return () => {
    listeners.delete(listener)
  }
}

const getCatalogVersion = () => catalogVersion

const bumpCatalog = () => {
  catalogVersion += 1
  listeners.forEach((listener) => listener())
}

const PLACEHOLDER_RE = /\{([a-zA-Z_][a-zA-Z0-9_]*)\}/g

const interpolate = (template: string, values: TranslateValues = {}) =>
  template.replace(PLACEHOLDER_RE, (raw, key: string) => {
    const value = values[key]
    return value == null ? raw : String(value)
  })

const lookup = (tree: MessageTree, key: string): string | undefined => {
  const parts = key.split('.')
  let current: unknown = tree
  for (const part of parts) {
    if (!current || typeof current !== 'object') return undefined
    current = (current as MessageTree)[part]
  }
  return typeof current === 'string' ? current : undefined
}

export const translateUsage = (
  locale: SupportedLocale,
  key: string,
  values?: TranslateValues,
): string => {
  const resolved = lookup(catalogs[locale], key) ?? lookup(catalogs['zh-CN'], key)
  if (!resolved) return key
  return interpolate(resolved, values)
}

export async function hydrateUsageLocale(locale?: SupportedLocale): Promise<SupportedLocale> {
  const next = locale ?? readStoredLocale()
  if (hydrated.has(next)) return next

  const mod =
    next === 'en-US'
      ? await import('@/i18n/locales/en-US')
      : await import('@/i18n/locales/zh-CN')
  catalogs[next] = mod.default as unknown as MessageTree
  hydrated.add(next)
  bumpCatalog()
  return next
}

export function useUsageT(): TranslateFunction {
  const locale = readStoredLocale()
  useSyncExternalStore(subscribeCatalog, getCatalogVersion, getCatalogVersion)
  return (key, values) => translateUsage(locale, key, values)
}
