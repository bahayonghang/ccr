import { createI18n } from 'vue-i18n'
import zhCN from './locales/zh-CN'
import enUS from './locales/en-US'

// Load saved locale from localStorage or default to zh-CN
const getSavedLocale = (): string => {
  try {
    return localStorage.getItem('ccr-ui-locale') || 'zh-CN'
  } catch (error) {
    console.warn('localStorage not available, using default locale')
    return 'zh-CN'
  }
}

// Create i18n instance
const i18n = createI18n({
  legacy: false, // Use Composition API mode
  locale: getSavedLocale(), // Default language
  fallbackLocale: 'zh-CN', // Fallback if key missing
  messages: {
    'zh-CN': zhCN,
    'en-US': enUS,
  },
  globalInjection: true, // Enable $t in templates
  missingWarn: import.meta.env.DEV, // Warn about missing keys only in dev
  fallbackWarn: false, // No warnings for fallback usage
})

export default i18n

// Export type for TypeScript
export type MessageSchema = typeof zhCN
