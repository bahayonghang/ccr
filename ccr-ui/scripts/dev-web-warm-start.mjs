#!/usr/bin/env node

import { spawn } from 'node:child_process'
import { existsSync, readFileSync } from 'node:fs'
import path from 'node:path'
import { terminateProcessTree } from './process-tree.mjs'

const ANSI_ESCAPE_PATTERN = new RegExp(String.raw`\x1b\[[0-9;]*[a-zA-Z]`, 'g')
const args = process.argv.slice(2)
const cliPortIndex = args.indexOf('--port')
const port = cliPortIndex >= 0 && args[cliPortIndex + 1]
  ? args[cliPortIndex + 1]
  : process.env.PORT || '5173'
const cliHostIndex = args.indexOf('--host')
const host = cliHostIndex >= 0 ? args[cliHostIndex + 1] || '127.0.0.1' : '127.0.0.1'
const strictPort = args.includes('--strictPort')
const requestedHealthTimeoutMs = Number(process.env.CCR_DEV_HEALTH_TIMEOUT_MS || 60_000)
const healthTimeoutMs = Number.isFinite(requestedHealthTimeoutMs) && requestedHealthTimeoutMs > 0
  ? Math.min(Math.round(requestedHealthTimeoutMs), 300_000)
  : 60_000

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
if (strictPort) viteArgs.push('--strictPort')

const warmTargetsPath = path.join(process.cwd(), 'scripts', 'dev-warm-targets.json')
const { healthPath = '/' } = JSON.parse(readFileSync(warmTargetsPath, 'utf8'))
const server = spawn(command, viteArgs, {
  stdio: ['inherit', 'pipe', 'pipe'],
  windowsHide: true,
})

let healthCheckStarted = false
let expectedShutdown = false
let fatalExitCode = 0

const shutdown = async (exitCode = 0) => {
  expectedShutdown = true
  fatalExitCode = Math.max(fatalExitCode, exitCode)
  try {
    await terminateProcessTree(server)
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    process.stderr.write(`[dev:web] failed to stop Vite process tree: ${message}\n`)
    fatalExitCode = 1
  }
  process.exitCode = fatalExitCode
}

const fail = (label, error) => {
  const message = error instanceof Error ? error.stack || error.message : String(error)
  process.stderr.write(`[dev:web] ${label}: ${message}\n`)
  void shutdown(1)
}

process.on('SIGINT', () => void shutdown(0))
process.on('SIGTERM', () => void shutdown(0))
process.on('uncaughtException', (error) => fail('uncaught exception', error))
process.on('unhandledRejection', (error) => fail('unhandled rejection', error))

server.stdout.on('data', (chunk) => {
  process.stdout.write(chunk)
  const plainText = chunk.toString().replace(ANSI_ESCAPE_PATTERN, '')
  if (!healthCheckStarted && /\bLocal:\s+https?:\/\//.test(plainText)) {
    healthCheckStarted = true
    void checkHealth().catch((error) => fail('health check failed', error))
  }
})

server.stderr.on('data', (chunk) => process.stderr.write(chunk))
server.on('error', (error) => fail('failed to start Vite', error))
server.on('exit', (code, signal) => {
  if (expectedShutdown) {
    process.exitCode = fatalExitCode
    return
  }
  void shutdown(code ?? (signal ? 1 : 0))
})

async function checkHealth() {
  const controller = new AbortController()
  const timer = setTimeout(() => controller.abort(), healthTimeoutMs)
  const startedAt = Date.now()

  try {
    const response = await fetch(`http://${host}:${port}${healthPath}`, {
      signal: controller.signal,
      headers: {
        accept: 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8',
      },
    })
    await response.arrayBuffer()
    if (!response.ok) {
      throw new Error(`${healthPath} returned ${response.status}`)
    }
    process.stderr.write(`[dev:web] ready ${healthPath} ${Date.now() - startedAt}ms\n`)
  } finally {
    clearTimeout(timer)
  }
}
