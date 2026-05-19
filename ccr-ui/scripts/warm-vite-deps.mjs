#!/usr/bin/env node

import { resolveConfig } from 'vite'

const startedAt = Date.now()

try {
  await resolveConfig({}, 'serve')
  process.stderr.write(`[prebundle] Vite optimizeDeps will run automatically at dev-server startup (${Date.now() - startedAt}ms)
`)
} catch (error) {
  const message = error instanceof Error ? error.stack || error.message : String(error)
  process.stderr.write(`[prebundle] failed to validate Vite config: ${message}
`)
  process.exitCode = 1
}
