import { useMemo } from 'react'
import { bootLocaleMessages } from '@/i18n/bootMessages'
import { readStoredLocale, type SupportedLocale } from '@/i18n'
import zhCN from '@/i18n/locales/zh-CN'
import enUS from '@/i18n/locales/en-US'
import type { TranslateFunction } from '@/utils/tf'

type MessageTree = Record<string, unknown>

const catalogs: Record<SupportedLocale, MessageTree> = {
  'zh-CN': (zhCN as MessageTree) ?? (bootLocaleMessages['zh-CN'] as unknown as MessageTree),
  'en-US': (enUS as MessageTree) ?? (bootLocaleMessages['en-US'] as unknown as MessageTree),
}

const lookup = (tree: MessageTree, key: string): string | undefined => {
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
): string =>
  template.replace(/\{([a-zA-Z_][a-zA-Z0-9_]*)\}/g, (raw, name: string) => {
    const value = values[name]
    return value == null ? raw : String(value)
  })

export function translateCheckin(
  locale: SupportedLocale,
  key: string,
  values?: Record<string, string | number | boolean | null | undefined>,
): string {
  const resolved = lookup(catalogs[locale], key) ?? lookup(catalogs['zh-CN'], key)
  if (!resolved) return key
  return interpolate(resolved, values)
}

export function useCheckinT(): TranslateFunction {
  const locale = readStoredLocale()
  return useMemo(() => {
    return (key, values) => translateCheckin(locale, key, values)
  }, [locale])
}

export function useCheckinLocale(): SupportedLocale {
  return readStoredLocale()
}

export function useTt(): (zh: string, en: string) => string {
  const locale = useCheckinLocale()
  return useMemo(
    () => (zh: string, en: string) => (locale.startsWith('zh') ? zh : en),
    [locale],
  )
}
