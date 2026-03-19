import { defineStore } from 'pinia'
import { applyThemeToDocument, persistTheme, readStoredTheme, type ThemeMode } from '@/utils/themeBootstrap'

export const useThemeStore = defineStore('theme', {
  state: () => ({
    currentTheme: readStoredTheme() as ThemeMode,
  }),

  actions: {
    setTheme(theme: ThemeMode) {
      this.currentTheme = theme
      applyThemeToDocument(theme)
      persistTheme(theme)
    },

    toggleTheme() {
      const newTheme = this.currentTheme === 'light' ? 'dark' : 'light'
      this.setTheme(newTheme)
    },

    initializeTheme() {
      this.currentTheme = readStoredTheme()
      applyThemeToDocument(this.currentTheme)
    },
  },
})
