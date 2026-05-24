#!/usr/bin/env node

import { spawn } from 'node:child_process'
import { existsSync, readFileSync } from 'node:fs'
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

const warmTargetsPath = path.join(process.cwd(), 'scripts', 'dev-warm-targets.json')
const warmTargets = JSON.parse(readFileSync(warmTargetsPath, 'utf8'))
const probeUrls = Array.isArray(warmTargets.probeUrls) ? warmTargets.probeUrls : ['/']
const requestUrls = Array.isArray(warmTargets.requestUrls) ? warmTargets.requestUrls : []
const warmUrls = [
  ...probeUrls,
  ...requestUrls,
  ...warmTargets.clientFiles.map((file) => file.replace(/^\./, '')),
]
const requestedWarmConcurrency = Number(process.env.CCR_DEV_WARM_CONCURRENCY || 4)
const warmConcurrency = Number.isFinite(requestedWarmConcurrency)
  ? Math.max(1, Math.min(16, Math.round(requestedWarmConcurrency)))
  : 4
const browserNavigationHeaders = {
  accept: 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8',
  'sec-fetch-dest': 'document',
  'sec-fetch-mode': 'navigate',
  'sec-fetch-site': 'none',
  'upgrade-insecure-requests': '1',
}

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

  const warmUrl = async (url, init = undefined) => {
    try {
      const response = await fetch(`${base}${url}`, init)
      await response.arrayBuffer()
      if (!response.ok) {
        process.stderr.write(`[dev:web] warm request ${url} returned ${response.status}\n`)
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      process.stderr.write(`[dev:web] warm request ${url} failed: ${message}\n`)
    }
  }

  for (let index = 0; index < warmUrls.length; index += warmConcurrency) {
    await Promise.all(
      warmUrls.slice(index, index + warmConcurrency).map(async (url) => {
        await warmUrl(url, probeUrls.includes(url) ? { headers: browserNavigationHeaders } : undefined)
      }),
    )
  }

  // A final route probe keeps the "warmed" signal honest: Vite may still be
  // draining transform work after individual module responses complete,
  // especially when Tailwind CSS and Vue SFC style blocks are cold on Windows.
  await Promise.all(
    probeUrls.map(async (url) => {
      try {
        const probeStartedAt = Date.now()
        const response = await fetch(`${base}${url}`, { headers: browserNavigationHeaders })
        await response.arrayBuffer()
        if (!response.ok) {
          process.stderr.write(`[dev:web] probe request ${url} returned ${response.status}\n`)
        } else {
          const elapsed = Date.now() - probeStartedAt
          process.stderr.write(`[dev:web] probe ${url} ${elapsed}ms\n`)
        }
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error)
        process.stderr.write(`[dev:web] probe request ${url} failed: ${message}\n`)
      }
    }),
  )

  process.stderr.write(`[dev:web] warmed ${warmUrls.length} startup modules in ${Date.now() - startedAt}ms\n`)
}