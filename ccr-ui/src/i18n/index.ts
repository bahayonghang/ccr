import { createI18n } from 'vue-i18n'
import { logger } from '@/utils/logger'
import { bootLocaleMessages } from './bootMessages'

type SupportedLocale = 'zh-CN' | 'en-US'
type LocaleMessages = Record<string, unknown>

const DEFAULT_LOCALE: SupportedLocale = 'zh-CN'

const localeLoaders: Record<SupportedLocale, () => Promise<{ default: LocaleMessages }>> = {
  'zh-CN': () => import('./locales/zh-CN'),
  'en-US': () => import('./locales/en-US'),
}

const hydratedLocales = new Set<SupportedLocale>()

const normalizeLocale = (locale: string): SupportedLocale => {
  return locale === 'en-US' ? 'en-US' : DEFAULT_LOCALE
}

const getSavedLocale = (): SupportedLocale => {
  try {
    return normalizeLocale(localStorage.getItem('ccr-ui-locale') || DEFAULT_LOCALE)
  } catch (error) {
    logger.warn('[i18n] localStorage unavailable, falling back to default locale', error)
    return DEFAULT_LOCALE
  }
}

const loadLocaleMessages = async (locale: SupportedLocale): Promise<LocaleMessages> => {
  const loader = localeLoaders[locale]
  const module = await loader()
  hydratedLocales.add(locale)
  return module.default
}

const preferredLocale = getSavedLocale()

const i18n = createI18n({
  legacy: false,
  locale: preferredLocale,
  fallbackLocale: DEFAULT_LOCALE,
  messages: bootLocaleMessages,
  globalInjection: true,
  missingWarn: import.meta.env.DEV,
  fallbackWarn: false,
} as never)

export const ensureLocaleLoaded = async (locale: string): Promise<SupportedLocale> => {
  const normalized = normalizeLocale(locale)
  if (hydratedLocales.has(normalized)) {
    return normalized
  }

  const messages = await loadLocaleMessages(normalized)
  ;(i18n.global as never as { setLocaleMessage: (locale: SupportedLocale, message: LocaleMessages) => void })
    .setLocaleMessage(normalized, messages)
  return normalized
}

export const setLocale = async (locale: string): Promise<void> => {
  const normalized = await ensureLocaleLoaded(locale)
  ;(i18n.global.locale as unknown as { value: SupportedLocale }).value = normalized

  try {
    localStorage.setItem('ccr-ui-locale', normalized)
  } catch (error) {
    logger.warn('[i18n] failed to persist locale preference', error)
  }
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
