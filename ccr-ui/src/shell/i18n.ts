import { useSyncExternalStore } from 'react'
import { bootLocaleMessages } from '@/i18n/bootMessages'
import { readStoredLocale, type SupportedLocale } from '@/i18n'
import { useShellPreferencesStore } from '@/shell/stores/shellPreferences'

type MessageTree = Record<string, unknown>
type TranslateValues = Record<string, string | number | boolean | null | undefined>

const catalogs: Record<SupportedLocale, MessageTree> = {
  'zh-CN': bootLocaleMessages['zh-CN'] as unknown as MessageTree,
  'en-US': bootLocaleMessages['en-US'] as unknown as MessageTree,
}

const loaded = new Set<SupportedLocale>(['zh-CN', 'en-US'])
const fullyHydrated = new Set<SupportedLocale>()

let catalogVersion = 0
const listeners = new Set<() => void>()

const subscribeCatalog = (listener: () => void): (() => void) => {
  listeners.add(listener)
  return () => {
    listeners.delete(listener)
  }
}

const getCatalogVersion = (): number => catalogVersion

const bumpCatalog = (): void => {
  catalogVersion += 1
  listeners.forEach((listener) => listener())
}

const PLACEHOLDER_RE = /\{([a-zA-Z_][a-zA-Z0-9_]*)\}/g

const interpolate = (template: string, values: TranslateValues = {}): string =>
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

export type ShellTranslate = (key: string, values?: TranslateValues) => string

export const translateShell = (
  locale: SupportedLocale,
  key: string,
  values?: TranslateValues,
): string => {
  const resolved = lookup(catalogs[locale], key) ?? lookup(catalogs['zh-CN'], key)
  if (!resolved) return key
  return interpolate(resolved, values)
}

/** 加载完整 locale 包。settings 等 defer 路由可在挂载后再调用。 */
export async function hydrateShellLocale(locale?: SupportedLocale): Promise<SupportedLocale> {
  const next = locale ?? readStoredLocale()
  if (fullyHydrated.has(next)) return next

  const mod =
    next === 'en-US'
      ? await import('@/i18n/locales/en-US')
      : await import('@/i18n/locales/zh-CN')
  catalogs[next] = mod.default as unknown as MessageTree
  fullyHydrated.add(next)
  loaded.add(next)
  bumpCatalog()
  return next
}

export function useShellT(): ShellTranslate {
  const locale = useShellPreferencesStore((state) => state.locale)
  useSyncExternalStore(subscribeCatalog, getCatalogVersion, getCatalogVersion)
  return (key, values) => translateShell(locale, key, values)
}
