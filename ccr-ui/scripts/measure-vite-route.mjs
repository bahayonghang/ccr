#!/usr/bin/env node

import { execFile, spawn } from 'node:child_process'
import { promisify } from 'node:util'

const execFileAsync = promisify(execFile)
const ANSI_ESCAPE_PATTERN = new RegExp(String.raw`\x1b\[[0-9;?]*[ -/]*[@-~]`, 'g')
const args = new Set(process.argv.slice(2))
const routeArg = process.argv.find((arg) => arg.startsWith('--route='))
const route = routeArg?.slice('--route='.length) || process.env.ROUTE || '/settings'
const timeoutArg = process.argv.find((arg) => arg.startsWith('--timeout='))
const requestTimeoutMs = Number(timeoutArg?.slice('--timeout='.length) || process.env.TIMEOUT_MS || 60000)
const withBrowser = args.has('--browser')
const includeFetchProbes = !withBrowser || args.has('--fetch-probes')
const host = process.env.HOST || '127.0.0.1'
const port = process.env.PORT || '5173'
const base = `http://${host}:${port}`

const server = spawn(
  process.execPath,
  ['./scripts/dev-web-warm-start.mjs', '--host', host, '--strictPort'],
  {
    stdio: ['ignore', 'pipe', 'pipe'],
    windowsHide: true,
  },
)

let output = ''
let ready = false
let warmed = false
const startedAt = Date.now()

const appendOutput = (chunk) => {
  const text = chunk.toString()
  output += text
  process.stderr.write(text)
  const plainText = text.replace(ANSI_ESCAPE_PATTERN, '')
  if (/\bLocal:\s+https?:\/\//.test(plainText) || /https?:\/\/(?:localhost|127\.0\.0\.1|\[::1\])(?::\d+)?\//.test(plainText)) {
    ready = true
  }
  if (/\[dev:web\] warmed/.test(plainText)) {
    warmed = true
  }
}

server.stdout.on('data', appendOutput)
server.stderr.on('data', appendOutput)
server.on('exit', (code, signal) => {
  if (!warmed) {
    warmed = true
  }
  if (!ready && code !== 0) {
    ready = true
  }
  output += `\n[measure:vite-route] dev server exited code=${code ?? 'null'} signal=${signal ?? 'null'}\n`
})

const waitFor = async (predicate, timeoutMs, label) => {
  const deadline = Date.now() + timeoutMs
  while (Date.now() < deadline) {
    if (predicate()) return
    await new Promise((resolve) => setTimeout(resolve, 100))
  }
  throw new Error(`Timed out waiting for ${label}`)
}

const timedFetch = async (path) => {
  const controller = new AbortController()
  const timer = setTimeout(() => controller.abort(), requestTimeoutMs)
  const start = Date.now()

  try {
    const response = await fetch(`${base}${path}`, { signal: controller.signal })
    await response.text()
    return { path, status: response.status, ms: Date.now() - start }
  } finally {
    clearTimeout(timer)
  }
}

const getViteProcessSnapshot = async () => {
  if (process.platform !== 'win32') {
    return []
  }

  const escapedPort = port.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const command = [
    '$rows = Get-CimInstance Win32_Process |',
    `Where-Object { $_.CommandLine -match 'node_modules\\\\vite\\\\bin\\\\vite\\.js' -and $_.CommandLine -match '--port ${escapedPort}' } |`,
    'ForEach-Object {',
    '$p = Get-Process -Id $_.ProcessId -ErrorAction SilentlyContinue;',
    'if ($p) { [pscustomobject]@{ pid=$_.ProcessId; cpu=$p.CPU; workingSetMB=[math]::Round($p.WorkingSet64/1MB,1); privateMB=[math]::Round($p.PrivateMemorySize64/1MB,1); commandLine=$_.CommandLine } }',
    '};',
    '$rows | ConvertTo-Json -Compress',
  ].join(' ')

  try {
    const { stdout } = await execFileAsync('powershell.exe', ['-NoProfile', '-Command', command], {
      windowsHide: true,
      timeout: 10000,
    })
    const trimmed = stdout.trim()
    if (!trimmed) return []
    const parsed = JSON.parse(trimmed)
    return Array.isArray(parsed) ? parsed : [parsed]
  } catch (error) {
    return [{ error: error instanceof Error ? error.message : String(error) }]
  }
}

const measureBrowserNavigation = async () => {
  const { chromium } = await import('playwright')
  const browser = await chromium.launch({ headless: true })
  const page = await browser.newPage()
  const requests = []
  const consoleMessages = []
  const pageErrors = []
  const started = Date.now()
  let domContentLoadedAt = 0
  let loadAt = 0

  await page.addInitScript(() => {
    window.__ccrLongTasks = []
    if ('PerformanceObserver' in window) {
      try {
        const observer = new PerformanceObserver((list) => {
          window.__ccrLongTasks.push(
            ...list.getEntries().map((entry) => ({
              name: entry.name,
              startTime: Math.round(entry.startTime),
              duration: Math.round(entry.duration),
            })),
          )
        })
        observer.observe({ type: 'longtask', buffered: true })
      } catch {
        // Long task timing is best-effort and not available in every browser mode.
      }
    }
  })
  page.on('request', (request) => {
    requests.push({
      url: request.url(),
      type: request.resourceType(),
      start: Date.now(),
      end: 0,
      status: 0,
    })
  })
  page.on('response', (response) => {
    const request = requests.find((item) => item.url === response.url() && !item.end)
    if (!request) return
    request.end = Date.now()
    request.status = response.status()
  })
  page.on('console', (message) => {
    if (consoleMessages.length >= 20) return
    if (!['error', 'warning'].includes(message.type())) return
    consoleMessages.push({
      type: message.type(),
      text: message.text().slice(0, 300),
    })
  })
  page.on('pageerror', (error) => {
    pageErrors.push(error.message.slice(0, 300))
  })
  page.on('domcontentloaded', () => {
    domContentLoadedAt = Date.now() - started
  })
  page.on('load', () => {
    loadAt = Date.now() - started
  })

  try {
    await page.goto(`${base}${route}`, {
      waitUntil: 'domcontentloaded',
      timeout: requestTimeoutMs,
    })
    const selectorStarted = Date.now()
    await page.waitForSelector('.app-settings-view, #app > :not(#app-loader)', {
      timeout: Math.min(requestTimeoutMs, 15000),
    }).catch(() => {})
    const appReadyMs = Date.now() - started
    const selectorWaitMs = Date.now() - selectorStarted
    await page.waitForTimeout(1000)
    const perf = await page.evaluate(() => {
      const nav = performance.getEntriesByType('navigation')[0]
      const toRelativeName = (name) => {
        try {
          const url = new URL(name)
          return `${url.pathname}${url.search}`
        } catch {
          return name
        }
      }
      const resourceTimings = performance.getEntriesByType('resource')
        .map((entry) => ({
          name: toRelativeName(entry.name),
          initiatorType: entry.initiatorType,
          startTime: Math.round(entry.startTime),
          duration: Math.round(entry.duration),
          responseEnd: Math.round(entry.responseEnd),
        }))
        .sort((a, b) => b.duration - a.duration)
        .slice(0, 30)
      const marks = performance.getEntriesByType('mark')
        .filter((entry) => entry.name.startsWith('app:'))
        .map((entry) => ({
          name: entry.name,
          startTime: Math.round(entry.startTime),
        }))
      return {
        readyState: document.readyState,
        domContentLoadedMs: nav ? Math.round(nav.domContentLoadedEventEnd) : null,
        loadEventMs: nav ? Math.round(nav.loadEventEnd) : null,
        bodySample: document.body.innerText.slice(0, 160),
        marks,
        resourceTimings,
        longTasks: (window.__ccrLongTasks || [])
          .sort((a, b) => b.duration - a.duration)
          .slice(0, 20),
      }
    })
    return {
      ok: true,
      ms: Date.now() - started,
      appReadyMs,
      selectorWaitMs,
      domContentLoadedObservedMs: domContentLoadedAt || null,
      loadObservedMs: loadAt || null,
      requestCount: requests.length,
      slowRequests: summarizeRequests(requests),
      consoleMessages,
      pageErrors,
      perf,
    }
  } catch (error) {
    return {
      ok: false,
      ms: Date.now() - started,
      error: error instanceof Error ? error.message : String(error),
      domContentLoadedObservedMs: domContentLoadedAt || null,
      loadObservedMs: loadAt || null,
      requestCount: requests.length,
      slowRequests: summarizeRequests(requests),
      consoleMessages,
      pageErrors,
    }
  } finally {
    await browser.close().catch(() => {})
  }
}

const summarizeRequests = (requests) => {
  const now = Date.now()
  return requests
    .map((request) => ({
      url: request.url.replace(base, ''),
      type: request.type,
      status: request.status,
      ms: (request.end || now) - request.start,
      done: Boolean(request.end),
    }))
    .sort((a, b) => b.ms - a.ms)
    .slice(0, 20)
}

const stopServer = async () => {
  if (server.killed) return

  if (process.platform === 'win32') {
    await execFileAsync('taskkill.exe', ['/PID', String(server.pid), '/F', '/T'], {
      windowsHide: true,
    }).catch(() => {})
  } else {
    server.kill('SIGTERM')
  }
}

try {
  await waitFor(() => ready, requestTimeoutMs, 'Vite server readiness')
  await waitFor(() => warmed, requestTimeoutMs, 'dev warmup')
  if (server.exitCode !== null) {
    throw new Error(`Vite dev server exited before measurement. Recent output:\n${output.slice(-4000)}`)
  }
  const readyAndWarmMs = Date.now() - startedAt

  // When measuring real browser cold-route behavior, avoid the diagnostic
  // fetch probes by default so they do not occupy Vite's transform worker or
  // force GC around Chromium's first document request. Add --fetch-probes when
  // you explicitly want both sets in one slower diagnostic run.
  const browser = withBrowser ? await measureBrowserNavigation() : null

  const fetchResults = []
  if (includeFetchProbes) {
    fetchResults.push(await timedFetch('/'))
    fetchResults.push(await timedFetch(route))
    fetchResults.push(await timedFetch('/src/views/AppSettingsView.vue'))
  }

  const snapshot = await getViteProcessSnapshot()
  const result = {
    route,
    serverReadyAndWarmMs: readyAndWarmMs,
    fetchResults,
    browser,
    viteProcess: snapshot,
  }
  process.stdout.write(`${JSON.stringify(result, null, 2)}\n`)
} finally {
  await stopServer()
}
