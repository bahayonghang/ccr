// 插件冷启动/HMR 测量脚本（批次 2，测量数据落盘 plugin-selection.md）
// 用法：bun .trellis/tasks/<task>/measure-plugin.mjs <variant-label> <runs>
// 前置：vite.config.ts 已切换到被测插件；每次运行前删除 node_modules/.vite。
import { readFileSync, rmSync, writeFileSync } from 'node:fs'
import { spawn } from 'node:child_process'
import path from 'node:path'

const cwd = path.resolve(process.argv[2] ?? '.')
const label = process.argv[3] ?? 'unknown'
const runs = Number(process.argv[4] ?? 3)
const port = 15173
const baseUrl = `http://127.0.0.1:${port}/`
const appFile = path.join(cwd, 'src', 'shell', 'App.tsx')
const viteJs = path.join(cwd, 'node_modules', 'vite', 'bin', 'vite.js')

const sleep = (ms) => new Promise((r) => setTimeout(r, ms))

function killTree(pid) {
  if (process.platform === 'win32') {
    spawn('taskkill', ['/pid', String(pid), '/T', '/F'], { stdio: 'ignore' })
  } else {
    process.kill(-pid, 'SIGKILL')
  }
}

async function startVite() {
  rmSync(path.join(cwd, 'node_modules', '.vite'), { recursive: true, force: true })
  const t0 = performance.now()
  const proc = spawn(process.execPath, [viteJs, '--strictPort'], {
    cwd,
    stdio: ['ignore', 'pipe', 'pipe'],
  })
  let out = ''
  proc.stdout.on('data', (d) => { out += d.toString() })
  proc.stderr.on('data', (d) => { out += d.toString() })
  await new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error(`ready timeout\n${out}`)), 120_000)
    const onData = () => {
      if (/Local:\s+http/.test(out)) {
        clearTimeout(timer)
        proc.stdout.off('data', onData)
        resolve()
      }
    }
    proc.stdout.on('data', onData)
    proc.on('exit', (code) => { clearTimeout(timer); reject(new Error(`vite exited early code=${code}\n${out}`)) })
  })
  return { proc, readyMs: performance.now() - t0, output: out }
}

async function firstPageLoad() {
  // 首页加载 = GET / (index.html) + GET 入口模块（走完整转换管线）
  const t = performance.now()
  const indexRes = await fetch(baseUrl)
  if (!indexRes.ok) throw new Error(`GET / -> ${indexRes.status}`)
  await indexRes.text()
  const entryRes = await fetch(`${baseUrl}src/main.tsx`)
  if (!entryRes.ok) throw new Error(`GET /src/main.tsx -> ${entryRes.status}`)
  await entryRes.text()
  return performance.now() - t
}

async function hmrSample() {
  // 经 vite HMR websocket 观测：改源文件到收到 update 的延迟
  const ws = new WebSocket(`ws://127.0.0.1:${port}/`, 'vite-hmr')
  try {
    await new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error('ws open timeout')), 15_000)
      ws.addEventListener('open', () => {}, { once: true })
      ws.addEventListener('message', (e) => {
        const msg = JSON.parse(e.data)
        if (msg.type === 'connected') { clearTimeout(timer); resolve() }
      }, { once: false })
      ws.addEventListener('error', () => { clearTimeout(timer); reject(new Error('ws error')) })
    })
    await sleep(300)

    const original = readFileSync(appFile, 'utf8')
    writeFileSync(appFile, `${original}\n/* hmr-probe */`)
    const t = performance.now()
    await new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error('hmr update timeout')), 30_000)
      ws.addEventListener('message', (e) => {
        const msg = JSON.parse(e.data)
        if (msg.type === 'update' || msg.type === 'full-reload') { clearTimeout(timer); resolve() }
      })
      ws.addEventListener('close', () => { clearTimeout(timer); reject(new Error('ws closed before update')) })
    })
    writeFileSync(appFile, original)
    return performance.now() - t
  } finally {
    try { ws.close() } catch {}
  }
}

const results = []
for (let i = 1; i <= runs; i++) {
  const { proc, readyMs, output } = await startVite()
  try {
    const pageMs = await firstPageLoad()
    const hmrMs = await hmrSample()
    results.push({ variant: label, run: i, readyMs: Math.round(readyMs * 10) / 10, pageMs: Math.round(pageMs * 10) / 10, hmrMs: Math.round(hmrMs * 10) / 10 })
    console.log(JSON.stringify(results[results.length - 1]))
  } catch (err) {
    console.error(`[${label} run ${i}] FAILED: ${err.message}`)
    results.push({ variant: label, run: i, error: err.message })
  } finally {
    proc.kill()
    await sleep(500)
    killTree(proc.pid)
    await sleep(1000)
  }
}
console.log('SUMMARY ' + JSON.stringify(results))
if (results.some((r) => r.error)) process.exitCode = 1
