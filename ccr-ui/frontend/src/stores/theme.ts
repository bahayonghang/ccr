import { defineStore } from 'pinia'

export const useThemeStore = defineStore('theme', {
  state: () => ({
    currentTheme: 'light' as 'light' | 'dark',
  }),

  actions: {
    setTheme(theme: 'light' | 'dark') {
      this.currentTheme = theme
      document.documentElement.classList.toggle('dark', theme === 'dark')
      document.documentElement.setAttribute('data-theme', theme)
      localStorage.setItem('ccr-theme', theme)
    },

    toggleTheme() {
      const newTheme = this.currentTheme === 'light' ? 'dark' : 'light'
      this.setTheme(newTheme)
    },

    initializeTheme() {
      const savedTheme = localStorage.getItem('ccr-theme') as 'light' | 'dark' | null
      const theme = savedTheme || 'light'
      this.setTheme(theme)
    },
  },
})
