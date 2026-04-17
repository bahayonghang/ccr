/**
 * Shared i18n stub for smoke tests.
 * 装载 en-US 作为默认 locale，避免 useI18n() 在无插件时抛错。
 */
import { createI18n } from 'vue-i18n'
import enUS from '@/i18n/locales/en-US'
import zhCN from '@/i18n/locales/zh-CN'

export function createI18nStub() {
  return createI18n({
    legacy: false,
    locale: 'en-US',
    fallbackLocale: 'en-US',
    messages: {
      'en-US': enUS,
      'zh-CN': zhCN,
    },
  })
}
