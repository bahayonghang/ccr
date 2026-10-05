import { fileURLToPath } from 'node:url'
import react from '../../../../ccr-ui/node_modules/@vitejs/plugin-react/dist/index.js'

const root = fileURLToPath(new URL('../../../../ccr-ui/', import.meta.url))
const research = fileURLToPath(new URL('./', import.meta.url))

export default {
  root,
  plugins: [react()],
  cacheDir: research + '.frontend-audit-cache',
  resolve: {
    alias: { '@': root + 'src' },
    dedupe: ['react', 'react-dom', 'react-router', '@tauri-apps/api', '@testing-library/react', '@tanstack/react-query', 'vitest'],
  },
  test: {
    name: 'architecture-audit-evidence',
    environment: 'jsdom',
    include: [
      research.replaceAll(String.fromCharCode(92), '/') + 'frontend-audit.evidence.test.tsx',
      'tests/shell/event-bridge-leak.smoke.test.tsx',
      'tests/shell/cache-route.smoke.test.ts',
      'tests/platforms/platform-base-settings.smoke.test.tsx',
      'tests/platforms/platform-surface-unify.smoke.test.ts',
      'tests/platforms/grok-settings-api.smoke.test.ts',
      'tests/platforms/claude-auth-view.smoke.test.tsx',
      'tests/api/api-facade-boundary.smoke.test.ts',
      'tests/api/typed-command-boundary.smoke.test.ts',
      'tests/api/command-runtime-policy.smoke.test.ts',
    ],
    setupFiles: [root + 'tests/setup/localStorage.ts', root + 'tests/setup/react-cleanup.ts'],
    maxWorkers: 1,
    restoreMocks: true,
    clearMocks: true,
    testTimeout: 15000,
  },
}
