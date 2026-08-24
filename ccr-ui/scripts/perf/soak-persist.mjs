#!/usr/bin/env node
/* eslint-disable no-console -- 测量脚本，与 check-bundle-budget.mjs 同一豁免模式 */
// AC13 2 小时浸泡：单条持久 raw CDP（不经过 Playwright）。
// 每 tick：SPA pushState + PopStateEvent → HeapProfiler.collectGarbage → Performance.getMetrics。
// 不启动、不结束 ccr-desktop。禁止用 Playwright 反复连接同一渲染进程。
//
// 环境变量：
//   SOAK_CDP         默认 http://127.0.0.1:9222
//   SOAK_OUT         必填，JSONL 输出路径
//   SOAK_PID         ccr-desktop PID，用于主机 WorkingSet 与进程树内 msedgewebview2
//   SOAK_MS          默认 7200000
//   SOAK_SAMPLE_MS   默认 60000
//   SOAK_SETTLE_MS   默认 800
import { execFile } from 'node:child_process'
import { promisify } from 'node:util'
import { appendFile, writeFile } from 'node:fs/promises'

const execFileAsync = promisify(execFile)
const DURATION_MS = Number(process.env.SOAK_MS || 2 * 60 * 60 * 1000)
const SAMPLE_MS = Number(process.env.SOAK_SAMPLE_MS || 60 * 1000)
const SETTLE_MS = Number(process.env.SOAK_SETTLE_MS || 800)
const CDP_HTTP = process.env.SOAK_CDP || 'http://127.0.0.1:9222'
const PID = Number(process.env.SOAK_PID || 0)
const OUT = process.env.SOAK_OUT

const ROUTES = [
  '/', '/settings', '/claude-code', '/claude-code/settings', '/claude-code/profiles',
  '/codex', '/codex/settings', '/codex/mcp', '/grok', '/grok/settings',
  '/antigravity', '/antigravity/mcp', '/opencode', '/opencode/settings', '/opencode/providers',
  '/commands', '/converter', '/sync', '/configs', '/mcp-manager',
  '/slash-commands', '/budget', '/pricing', '/usage', '/monitoring',
  '/checkin', '/wsl', '/ssh', '/skills',
]

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))

const mean = (arr) => (arr.length === 0 ? NaN : arr.reduce((a, b) => a + b, 0) / arr.length)

const ratio = (a, b) => (!Number.isFinite(a) || !Number.isFinite(b) || a === 0 ? null : b / a)

const logLine = async (obj) => {
  const line = `${JSON.stringify(obj)}\n`
  process.stdout.write(line)
  if (OUT) await appendFile(OUT, line, 'utf8')
}

const listMainTarget = async () => {
  const res = await fetch(`${CDP_HTTP}/json/list`)
  if (!res.ok) throw new Error(`json/list ${res.status}`)
  const pages = await res.json()
  const main = pages.find((page) => {
    const url = page.url || ''
    return url.includes('tauri.localhost') && !url.includes('/tray')
  })
  if (!main?.webSocketDebuggerUrl) throw new Error('no main tauri page')
  return main
}

const cdpClient = (wsUrl) => {
  let nextId = 0
  const pending = new Map()
  const ws = new WebSocket(wsUrl)
  let closed = false

  const ready = new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error('cdp open timeout')), 15_000)
    ws.addEventListener('open', () => {
      clearTimeout(timer)
      resolve()
    })
    ws.addEventListener('error', (event) => {
      clearTimeout(timer)
      reject(event)
    })
  })

  ws.addEventListener('message', (event) => {
    const msg = JSON.parse(String(event.data))
    if (msg.id == null) return
    const waiter = pending.get(msg.id)
    if (!waiter) return
    pending.delete(msg.id)
    if (msg.error) waiter.reject(new Error(JSON.stringify(msg.error)))
    else waiter.resolve(msg.result)
  })

  ws.addEventListener('close', () => {
    closed = true
    for (const [id, waiter] of pending) {
      pending.delete(id)
      waiter.reject(new Error('cdp closed'))
    }
  })

  const send = async (method, params, timeoutMs = 15_000) => {
    if (closed) throw new Error('cdp closed')
    await ready
    nextId += 1
    const id = nextId
    const result = new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        pending.delete(id)
        reject(new Error(`${method} timeout`))
      }, timeoutMs)
      pending.set(id, {
        resolve: (value) => {
          clearTimeout(timer)
          resolve(value)
        },
        reject: (error) => {
          clearTimeout(timer)
          reject(error)
        },
      })
    })
    ws.send(JSON.stringify({ id, method, params }))
    return result
  }

  const close = () => {
    closed = true
    try {
      ws.close()
    } catch {
      /* ignore */
    }
  }

  return { send, close, ready }
}

const memorySets = async (pid) => {
  if (!pid) return { hostWorkingSet: null, rendererWorkingSet: null }
  const script = `
$ErrorActionPreference = 'SilentlyContinue'
$root = ${pid}
$procs = Get-CimInstance Win32_Process
$children = @{}
foreach ($p in $procs) {
  $pp = [int]$p.ParentProcessId
  if (-not $children.ContainsKey($pp)) { $children[$pp] = New-Object 'System.Collections.Generic.List[int]' }
  [void]$children[$pp].Add([int]$p.ProcessId)
}
$seen = New-Object 'System.Collections.Generic.HashSet[int]'
$queue = New-Object 'System.Collections.Generic.Queue[int]'
[void]$seen.Add($root)
$queue.Enqueue($root)
while ($queue.Count -gt 0) {
  $cur = $queue.Dequeue()
  if ($children.ContainsKey($cur)) {
    foreach ($child in $children[$cur]) {
      if ($seen.Add($child)) { $queue.Enqueue($child) }
    }
  }
}
$hostWs = [int64]0
$hp = Get-Process -Id $root
if ($hp) { $hostWs = $hp.WorkingSet64 }
$renderer = [int64]0
Get-Process -Name msedgewebview2 | Where-Object { $seen.Contains($_.Id) } | ForEach-Object { $renderer += $_.WorkingSet64 }
Write-Output ("{0} {1}" -f $hostWs, $renderer)
`
  try {
    const { stdout } = await execFileAsync(
      'powershell.exe',
      ['-NoProfile', '-Command', script],
      { windowsHide: true, timeout: 20_000 },
    )
    const parts = String(stdout).trim().split(/\s+/)
    const hostWorkingSet = Number(parts[0])
    const rendererWorkingSet = Number(parts[1])
    return {
      hostWorkingSet: Number.isFinite(hostWorkingSet) ? hostWorkingSet : null,
      rendererWorkingSet: Number.isFinite(rendererWorkingSet) ? rendererWorkingSet : null,
    }
  } catch {
    return { hostWorkingSet: null, rendererWorkingSet: null }
  }
}

const navJs = (to) => `
(() => {
  const path = ${JSON.stringify(to)};
  const links = document.querySelectorAll('a[href]');
  for (const link of links) {
    const href = link.getAttribute('href');
    if (!href) continue;
    try {
      if (new URL(href, location.origin).pathname === path) {
        link.click();
        return location.pathname;
      }
    } catch {
      /* ignore malformed href */
    }
  }
  const prev = window.history.state;
  const idx = prev && typeof prev.idx === 'number' ? prev.idx + 1 : 0;
  const state = { usr: null, key: Math.random().toString(36).slice(2, 10), idx };
  window.history.pushState(state, '', path);
  window.dispatchEvent(new PopStateEvent('popstate', { state }));
  return location.pathname;
})()
`

const readMetrics = async (send) => {
  try {
    await send('HeapProfiler.collectGarbage', undefined, 30_000)
  } catch {
    /* 域未启用时忽略 */
  }
  const { metrics } = await send('Performance.getMetrics')
  const map = Object.fromEntries((metrics || []).map((item) => [item.name, item.value]))
  let usedJSHeapSize = null
  try {
    const evaluated = await send('Runtime.evaluate', {
      expression: 'performance.memory ? performance.memory.usedJSHeapSize : null',
      returnByValue: true,
    })
    usedJSHeapSize = evaluated?.result?.value ?? null
  } catch {
    usedJSHeapSize = null
  }
  return {
    jsHeapUsed: map.JSHeapUsedSize ?? null,
    jsHeapTotal: map.JSHeapTotalSize ?? null,
    jsEventListeners: map.JSEventListeners ?? null,
    nodes: map.Nodes ?? null,
    usedJSHeapSize,
  }
}

const metricOf = (rows, key) =>
  mean(rows.map((row) => row[key]).filter((value) => typeof value === 'number' && Number.isFinite(value)))

const main = async () => {
  if (!OUT) throw new Error('SOAK_OUT required')
  await writeFile(OUT, '', 'utf8')
  const target = await listMainTarget()
  const client = cdpClient(target.webSocketDebuggerUrl)
  await client.ready
  await client.send('Runtime.enable')
  await client.send('Performance.enable')
  try {
    await client.send('HeapProfiler.enable')
  } catch {
    /* 可选 */
  }

  const t0 = Date.now()
  const visited = new Set()
  const samples = []
  let i = 0

  await logLine({
    type: 'start',
    t0: new Date(t0).toISOString(),
    durationMs: DURATION_MS,
    sampleMs: SAMPLE_MS,
    settleMs: SETTLE_MS,
    pid: PID || null,
    nav: 'spa-click-or-rr-push',
    harness: 'persist-raw-cdp',
    routeCount: ROUTES.length,
    url: target.url,
  })

  try {
    while (Date.now() - t0 < DURATION_MS) {
      const elapsed = Date.now() - t0
      const route = ROUTES[i % ROUTES.length]
      i += 1
      let navError = null
      const memory = await memorySets(PID)

      try {
        const evaluated = await client.send('Runtime.evaluate', {
          expression: navJs(route),
          returnByValue: true,
        })
        const landed = evaluated?.result?.value
        visited.add(route)
        if (landed && landed !== route) navError = `landed ${landed}`
      } catch (error) {
        navError = error instanceof Error ? error.message : String(error)
      }

      await sleep(SETTLE_MS)

      let cdp = { jsHeapUsed: null, jsHeapTotal: null, jsEventListeners: null, nodes: null, usedJSHeapSize: null }
      try {
        cdp = await readMetrics(client.send)
      } catch (error) {
        cdp = { error: error instanceof Error ? error.message : String(error) }
      }

      let soakStats = null
      try {
        const statsEval = await client.send('Runtime.evaluate', {
          expression:
            'typeof window.__CCR_SOAK_STATS === "function" ? window.__CCR_SOAK_STATS() : null',
          returnByValue: true,
        })
        soakStats = statsEval?.result?.value ?? null
      } catch {
        soakStats = null
      }

      const sample = {
        type: 'sample',
        elapsedMs: elapsed,
        hour: elapsed < 60 * 60 * 1000 ? 1 : 2,
        route,
        navError,
        workingSet: memory.hostWorkingSet,
        rendererWorkingSet: memory.rendererWorkingSet,
        ...cdp,
        soakStats,
        visited: visited.size,
      }
      samples.push(sample)
      await logLine(sample)

      const remain = DURATION_MS - (Date.now() - t0)
      if (remain <= 0) break
      await sleep(Math.min(SAMPLE_MS, remain))
    }
  } finally {
    client.close()
  }

  const h1 = samples.filter((row) => row.hour === 1)
  const h2 = samples.filter((row) => row.hour === 2)
  const ws1 = metricOf(h1, 'workingSet')
  const ws2 = metricOf(h2, 'workingSet')
  const renderer1 = metricOf(h1, 'rendererWorkingSet')
  const renderer2 = metricOf(h2, 'rendererWorkingSet')
  const heap1 = metricOf(h1, 'jsHeapUsed')
  const heap2 = metricOf(h2, 'jsHeapUsed')
  const lis1 = metricOf(h1, 'jsEventListeners')
  const lis2 = metricOf(h2, 'jsEventListeners')
  const rWs = ratio(ws1, ws2)
  const rRenderer = ratio(renderer1, renderer2)
  const rHeap = ratio(heap1, heap2)
  const rLis = ratio(lis1, lis2)
  const grokListeners = samples
    .filter((row) => row.route === '/grok/settings' && typeof row.jsEventListeners === 'number')
    .map((row) => row.jsEventListeners)
  const summary = {
    type: 'summary',
    harness: 'persist-raw-cdp',
    samples: samples.length,
    hour1: h1.length,
    hour2: h2.length,
    uniqueRoutes: visited.size,
    workingSetHour1Mean: ws1,
    workingSetHour2Mean: ws2,
    workingSetRatio: rWs,
    rendererWorkingSetHour1Mean: renderer1,
    rendererWorkingSetHour2Mean: renderer2,
    rendererWorkingSetRatio: rRenderer,
    jsHeapHour1Mean: heap1,
    jsHeapHour2Mean: heap2,
    jsHeapRatio: rHeap,
    listenersHour1Mean: lis1,
    listenersHour2Mean: lis2,
    listenersRatio: rLis,
    grokSettingsListeners: grokListeners,
    passHostMemory: rWs !== null && rWs <= 1.1,
    passRendererMemory: rRenderer === null || rRenderer <= 1.1,
    passHeap: rHeap === null || rHeap <= 1.1,
    passListeners: rLis === null || rLis <= 1.1,
    passRoutes: visited.size >= 20,
  }
  summary.passMemory = Boolean(summary.passHostMemory && summary.passRendererMemory)
  summary.pass = Boolean(
    summary.passMemory &&
      summary.passHeap &&
      summary.passListeners &&
      summary.passRoutes &&
      h1.length > 0 &&
      h2.length > 0,
  )
  await logLine(summary)
  process.stdout.write(`SOAK_PASS=${summary.pass}\n`)
  if (!summary.pass) process.exitCode = 2
}

main().catch(async (error) => {
  try {
    await logLine({ type: 'fatal', error: error instanceof Error ? error.message : String(error) })
  } catch {
    console.error(error)
  }
  process.exit(1)
})
