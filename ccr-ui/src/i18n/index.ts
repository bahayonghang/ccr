import { useCallback } from 'react'
import i18n from 'i18next'
import { initReactI18next, useTranslation } from 'react-i18next'
import { logger } from '@/utils/logger'
import type { TranslateFunction } from '@/utils/tf'
import { bootLocaleMessages } from './bootMessages'
import { cloneUnescapedMessages } from './formatMessage'

export type SupportedLocale = 'zh-CN' | 'en-US'
type LocaleMessages = Record<string, unknown>

export const DEFAULT_LOCALE: SupportedLocale = 'zh-CN'
export const LOCALE_STORAGE_KEY = 'ccr-ui-locale'
const DEFAULT_NS = 'translation'
const UNUSED_SEPARATOR = '\u0001'

const localeLoaders: Record<SupportedLocale, () => Promise<{ default: LocaleMessages }>> = {
  'zh-CN': () => import('./locales/zh-CN'),
  'en-US': () => import('./locales/en-US'),
}

const hydratedLocales = new Set<SupportedLocale>()

export const normalizeLocale = (locale: string): SupportedLocale => {
  return locale === 'en-US' ? 'en-US' : DEFAULT_LOCALE
}

export const readStoredLocale = (): SupportedLocale => {
  try {
    return normalizeLocale(localStorage.getItem(LOCALE_STORAGE_KEY) || DEFAULT_LOCALE)
  } catch (error) {
    logger.warn('[i18n] localStorage unavailable, falling back to default locale', error)
    return DEFAULT_LOCALE
  }
}

const persistLocale = (locale: SupportedLocale): void => {
  try {
    localStorage.setItem(LOCALE_STORAGE_KEY, locale)
  } catch (error) {
    logger.warn('[i18n] failed to persist locale preference', error)
  }
}

const applyDocumentLang = (locale: SupportedLocale): void => {
  if (typeof document === 'undefined') return
  document.documentElement.lang = locale
}

const asTranslateString = (value: unknown): string => (typeof value === 'string' ? value : String(value))

const markMissingKeys = import.meta.env.DEV && import.meta.env.MODE !== 'test'
const preferredLocale = readStoredLocale()

void i18n.use(initReactI18next).init({
  lng: preferredLocale,
  fallbackLng: DEFAULT_LOCALE,
  supportedLngs: ['zh-CN', 'en-US'],
  defaultNS: DEFAULT_NS,
  ns: [DEFAULT_NS],
  resources: {
    'zh-CN': { [DEFAULT_NS]: cloneUnescapedMessages(bootLocaleMessages['zh-CN']) },
    'en-US': { [DEFAULT_NS]: cloneUnescapedMessages(bootLocaleMessages['en-US']) },
  },
  interpolation: {
    escapeValue: false,
    prefix: '{',
    suffix: '}',
  },
  keySeparator: '.',
  nsSeparator: false,
  pluralSeparator: UNUSED_SEPARATOR,
  contextSeparator: UNUSED_SEPARATOR,
  returnNull: false,
  returnEmptyString: true,
  initAsync: false,
  parseMissingKeyHandler: (key) => {
    if (markMissingKeys) {
      logger.warn(`[i18n] missing key "${key}"`)
      return `⟦${key}⟧`
    }
    return key
  },
  react: {
    useSuspense: false,
    bindI18n: 'languageChanged loaded',
    bindI18nStore: 'added removed',
  },
})

applyDocumentLang(preferredLocale)

export const translate: TranslateFunction = (key, values) =>
  asTranslateString(values ? i18n.t(key, values as never) : i18n.t(key))

export const tt = (zh: string, en: string): string =>
  normalizeLocale(i18n.language).startsWith('zh') ? zh : en

export function useAppT(): TranslateFunction {
  const { t } = useTranslation()
  return useCallback(
    (key, values) => asTranslateString(values ? t(key, values as never) : t(key)),
    [t],
  )
}

export function useAppLocale(): SupportedLocale {
  const { i18n: instance } = useTranslation()
  return normalizeLocale(instance.resolvedLanguage || instance.language || DEFAULT_LOCALE)
}

export function useAppTt(): (zh: string, en: string) => string {
  const locale = useAppLocale()
  return useCallback((zhText, enText) => (locale.startsWith('zh') ? zhText : enText), [locale])
}

export function useResolvedT(override?: TranslateFunction): TranslateFunction {
  const hooked = useAppT()
  return override ?? hooked
}

export const ensureLocaleLoaded = async (locale: string): Promise<SupportedLocale> => {
  const normalized = normalizeLocale(locale)
  if (hydratedLocales.has(normalized)) return normalized

  const module = await localeLoaders[normalized]()
  i18n.addResourceBundle(
    normalized,
    DEFAULT_NS,
    cloneUnescapedMessages(module.default),
    true,
    true,
  )
  hydratedLocales.add(normalized)
  return normalized
}

export const setLocale = async (locale: string): Promise<SupportedLocale> => {
  const normalized = await ensureLocaleLoaded(locale)
  if (i18n.language !== normalized) {
    await i18n.changeLanguage(normalized)
  }
  persistLocale(normalized)
  applyDocumentLang(normalized)
  return normalized
}

export const hydratePreferredLocale = async (): Promise<void> => {
  try {
    await ensureLocaleLoaded(preferredLocale)
  } catch (error) {
    logger.warn('[i18n] failed to hydrate preferred locale', error)
  }
}

export default i18n

export type MessageSchema = typeof import('./locales/zh-CN').default
