import { fileURLToPath } from 'node:url'
import { defineConfig } from 'vitest/config'

export default defineConfig({
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  },
  test: {
    name: 'smoke',
    environment: 'jsdom',
    include: ['tests/**/*.smoke.test.ts'],
    restoreMocks: true,
    clearMocks: true
  }
})
