import { fileURLToPath } from 'node:url'
import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vitest/config'

const localStorageFile = fileURLToPath(new URL('./.vitest-localstorage.json', import.meta.url))

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  },
  test: {
    name: 'smoke',
    environment: 'jsdom',
    include: ['tests/**/*.smoke.test.ts'],
    execArgv: [`--localstorage-file=${localStorageFile}`],
    restoreMocks: true,
    clearMocks: true
  }
})
