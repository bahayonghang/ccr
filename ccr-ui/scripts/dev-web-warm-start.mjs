#!/usr/bin/env node

import { spawn } from 'node:child_process'
import { existsSync } from 'node:fs'
import path from 'node:path'

const ANSI_ESCAPE_PATTERN = new RegExp(String.raw`\x1b\[[0-9;]*[a-zA-Z]`, 'g')
const args = process.argv.slice(2)
const cliPortIndex = args.indexOf('--port')
const port =
  cliPortIndex >= 0 && args[cliPortIndex + 1]
    ? args[cliPortIndex + 1]
    : process.env.PORT || '5173'
const cliHostIndex = args.indexOf('--host')
const host = cliHostIndex >= 0 ? args[cliHostIndex + 1] || '127.0.0.1' : '127.0.0.1'
const strictPort = args.includes('--strictPort')

const viteBinScript = path.join(process.cwd(), 'node_modules', 'vite', 'bin', 'vite.js')
const viteBinary = path.join(
  process.cwd(),
  'node_modules',
  '.bin',
  process.platform === 'win32' ? 'vite.exe' : 'vite',
)
const command = existsSync(viteBinScript) ? process.execPath : existsSync(viteBinary) ? viteBinary : 'vite'
const viteArgs = existsSync(viteBinScript)
  ? [viteBinScript, '--port', port, '--host', host]
  : ['--port', port, '--host', host]
if (strictPort) {
  viteArgs.push('--strictPort')
}

const warmUrls = [
  '/',
  '/src/main.ts',
  '/src/App.vue',
  '/src/components/MainLayout.vue',
  '/src/views/DashboardView.vue',
  '/src/views/dashboard/dashboardPresentation.ts',
  '/src/router/index.ts',
  '/src/api/index.ts',
  '/src/api/tauri.ts',
  '/src/stores/usage.ts',
  '/src/stores/usageDashboardPayload.ts',
  '/src/stores/usageImportNormalization.ts',
  '/src/views/CodexAuthView.vue',
  '/src/views/codex/codexAuthAccounts.ts',
  '/src/views/UsageDashboardView.vue',
  '/src/views/usage/useUsageDashboardState.ts',
  '/src/views/usage/usageChartOptions.ts',
  '/src/views/usage/usageDiagnostics.ts',
  '/src/views/usage/usageOverviewInsights.ts',
  '/src/views/usage/usageSummaryCards.ts',
  '/src/components/usage/UsageOverviewTab.vue',
  '/src/components/usage/UsageMetricCard.vue',
]

const server = spawn(command, viteArgs, {
  stdio: ['inherit', 'pipe', 'pipe'],
  windowsHide: true,
})

let warmupStarted = false
let shuttingDown = false

const shutdown = () => {
  if (shuttingDown) return
  shuttingDown = true
  if (!server.killed) {
    server.kill(process.platform === 'win32' ? 'SIGTERM' : 'SIGINT')
  }
}

process.on('SIGINT', shutdown)
process.on('SIGTERM', shutdown)
process.on('beforeExit', shutdown)

server.stdout.on('data', (chunk) => {
  const text = chunk.toString()
  process.stdout.write(chunk)
  const plainText = text.replace(ANSI_ESCAPE_PATTERN, '')
  if (!warmupStarted && /\bLocal:\s+https?:\/\//.test(plainText)) {
    warmupStarted = true
    void warmDevServer()
  }
})

server.stderr.on('data', (chunk) => {
  process.stderr.write(chunk)
})

server.on('exit', (code, signal) => {
  process.exitCode = code ?? (signal ? 1 : 0)
})

async function warmDevServer() {
  const base = `http://${host}:${port}`
  const startedAt = Date.now()

  await Promise.all(
    warmUrls.map(async (url) => {
      try {
        const response = await fetch(`${base}${url}`)
        if (!response.ok) {
          process.stderr.write(`[dev:web] warm request ${url} returned ${response.status}\n`)
        }
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error)
        process.stderr.write(`[dev:web] warm request ${url} failed: ${message}\n`)
      }
    }),
  )

  process.stderr.write(`[dev:web] warmed ${warmUrls.length} startup modules in ${Date.now() - startedAt}ms\n`)
}
