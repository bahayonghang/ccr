import { createI18n } from 'vue-i18n'
import type { MessageCompiler, MessageContext, MessageFunction } from 'vue-i18n'
import { logger } from '@/utils/logger'
import { bootLocaleMessages } from './bootMessages'

export type SupportedLocale = 'zh-CN' | 'en-US'
type LocaleMessages = Record<string, unknown>

export const DEFAULT_LOCALE: SupportedLocale = 'zh-CN'

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

const preferredLocale = readStoredLocale()

// runtime-only 构建剥离了 vue-i18n 内置的 message compiler，
// Tauri CSP 又禁止 'unsafe-eval'，所以必须自带不依赖 new Function 的 compiler；
// 仅支持 `{name}` 命名占位符和 `{0}` 索引占位符，足以覆盖项目所有 locale 模板。
const PLACEHOLDER_RE = /\{([a-zA-Z_][a-zA-Z0-9_]*|\d+)\}/g

const cspSafeMessageCompiler: MessageCompiler<string, string> = (message, context) => {
  if (typeof message === 'function') {
    return message as MessageFunction<string>
  }
  if (typeof message !== 'string') {
    logger.warn(`[i18n] unsupported message source for key "${context.key}"`, message)
    const literal = String(message ?? '')
    return () => literal
  }
  const template = message
  return (ctx: MessageContext<string>) => template.replace(PLACEHOLDER_RE, (raw, key: string) => {
    if (/^\d+$/.test(key)) {
      const value = ctx.list(Number(key))
      return value != null ? String(value) : raw
    }
    const value = ctx.named(key)
    return value != null ? String(value) : raw
  })
}

const i18n = createI18n({
  legacy: false,
  locale: preferredLocale,
  fallbackLocale: DEFAULT_LOCALE,
  messages: bootLocaleMessages,
  globalInjection: true,
  missingWarn: import.meta.env.DEV,
  fallbackWarn: false,
  messageCompiler: cspSafeMessageCompiler,
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
