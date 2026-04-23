import { fileURLToPath } from 'node:url'
import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vitest/config'

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
    setupFiles: ['./tests/setup/localStorage.ts'],
    restoreMocks: true,
    clearMocks: true,
    testTimeout: 15_000
  }
})
