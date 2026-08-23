import { useMemo } from 'react'
import { bootLocaleMessages } from '@/i18n/bootMessages'
import { readStoredLocale, type SupportedLocale } from '@/i18n'
import zhCN from '@/i18n/locales/zh-CN'
import enUS from '@/i18n/locales/en-US'
import type { TranslateFunction } from '@/utils/tf'

type MessageTree = Record<string, unknown>

const catalogs: Record<SupportedLocale, MessageTree> = {
  'zh-CN': zhCN as MessageTree,
  'en-US': enUS as MessageTree,
}

const lookup = (tree: unknown, key: string): string | undefined => {
  let current: unknown = tree
  for (const part of key.split('.')) {
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

export function useEditorT(): TranslateFunction {
  const locale = readStoredLocale()
  return useMemo(() => {
    return (key, values) => {
      const hit =
        lookup(catalogs[locale], key)
        ?? lookup(catalogs['zh-CN'], key)
        ?? lookup(bootLocaleMessages[locale], key)
      if (!hit) return key
      return interpolate(hit, values)
    }
  }, [locale])
}
