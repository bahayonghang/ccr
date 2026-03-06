import { createI18n } from 'vue-i18n'
import { logger } from '@/utils/logger'

type SupportedLocale = 'zh-CN' | 'en-US'
type LocaleMessages = Record<string, unknown>

const DEFAULT_LOCALE: SupportedLocale = 'zh-CN'

const localeLoaders: Record<SupportedLocale, () => Promise<{ default: LocaleMessages }>> = {
  'zh-CN': () => import('./locales/zh-CN'),
  'en-US': () => import('./locales/en-US'),
}

const loadedLocales = new Set<SupportedLocale>()

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
  loadedLocales.add(locale)
  return module.default
}

const initialLocale = getSavedLocale()
const initialMessages = await loadLocaleMessages(initialLocale)

const i18n = createI18n({
  legacy: false,
  locale: initialLocale,
  fallbackLocale: DEFAULT_LOCALE,
  messages: {
    [initialLocale]: initialMessages,
  },
  globalInjection: true,
  missingWarn: import.meta.env.DEV,
  fallbackWarn: false,
} as never)

export const ensureLocaleLoaded = async (locale: string): Promise<SupportedLocale> => {
  const normalized = normalizeLocale(locale)
  if (loadedLocales.has(normalized)) {
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

export default i18n

export type MessageSchema = typeof import('./locales/zh-CN').default
