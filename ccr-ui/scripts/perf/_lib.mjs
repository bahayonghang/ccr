#!/usr/bin/env node
// 批次7 性能测量共享工具（08-22-arch-quality-perf）
// 框架无关：只依赖 DOM / performance API + playwright 驱动，不 import 任何 vue/react。
// 五个场景脚本共用的参数解析、统计与打印工具。
import { chromium } from 'playwright'

// Phase 0 采集用同一视口（1800x1125），固定视口保证可重复性（R7）。
export const DEFAULT_VIEWPORT = { width: 1800, height: 1125 }

export const parseArgs = (argv, defaults = {}) => {
  const out = {
    baseUrl: 'http://127.0.0.1:4180',
    cdpUrl: 'http://127.0.0.1:9222',
    runs: 1,
    dry: false,
    ...defaults,
  }
  const take = (flag) => {
    const idx = argv.indexOf(flag)
    return idx >= 0 && idx + 1 < argv.length ? argv[idx + 1] : undefined
  }
  for (const flag of ['--base-url', '--cdp-url', '--runs']) {
    const inline = argv.find((a) => a.startsWith(`${flag}=`))
    if (inline) {
      const value = inline.slice(flag.length + 1)
      if (flag === '--runs') out.runs = Number(value) || 1
      else if (flag === '--base-url') out.baseUrl = value
      else out.cdpUrl = value
    } else if (take(flag) !== undefined) {
      const value = take(flag)
      if (flag === '--runs') out.runs = Number(value) || 1
      else if (flag === '--base-url') out.baseUrl = value
      else out.cdpUrl = value
    }
  }
  out.dry = argv.includes('--dry')
  return out
}

export const sum = (arr) => arr.reduce((a, b) => a + b, 0)
export const mean = (arr) => (arr.length === 0 ? NaN : sum(arr) / arr.length)

export const stdev = (arr) => {
  if (arr.length < 2) return 0
  const m = mean(arr)
  const variance = sum(arr.map((v) => (v - m) ** 2)) / (arr.length - 1)
  return Math.sqrt(variance)
}

// 相对标准差 = stdev / mean，R7/AC7 要求连续 3 次运行 RSD ≤ 15%。
export const rsd = (arr) => {
  const m = mean(arr)
  return Math.abs(m) < 1e-9 ? 0 : stdev(arr) / Math.abs(m)
}

export const percentiles = (arr, qs = [50, 95]) => {
  if (arr.length === 0) return {}
  const sorted = [...arr].sort((a, b) => a - b)
  const out = {}
  for (const q of qs) {
    const idx = Math.min(sorted.length - 1, Math.floor((q / 100) * sorted.length))
    out[`P${q}`] = sorted[idx]
  }
  out.mean = mean(sorted)
  out.max = sorted[sorted.length - 1]
  out.count = sorted.length
  return out
}

export const round = (value, digits = 1) => {
  if (!Number.isFinite(value)) return value
  return Number(value.toFixed(digits))
}

export const printJson = (obj) => {
  process.stdout.write(`${JSON.stringify(obj)}\n`)
}

// 线性回归斜率（y 对 t 的 slope），用于日志流内存增长速率（bytes/ms → bytes/s）。
export const linearSlope = (times, values) => {
  if (times.length < 2) return null
  const n = times.length
  const mT = mean(times)
  const mV = mean(values)
  let num = 0
  let den = 0
  for (let i = 0; i < n; i++) {
    num += (times[i] - mT) * (values[i] - mV)
    den += (times[i] - mT) ** 2
  }
  if (Math.abs(den) < 1e-9) return null
  return num / den
}

// 打开无头浏览器并固定视口（Phase 0 同口径）。
export const launchPage = async (viewport = DEFAULT_VIEWPORT) => {
  const browser = await chromium.launch({ headless: true })
  const page = await browser.newPage({ viewport })
  return { browser, page }
}

// 连接桌面运行时（tauri dev 的 WebView2 CDP），返回主应用页。
// 主窗口为应用页面，托盘窗 URL 以 /tray/ 开头；二者都算 http 页，
// 必须排除 /tray 才能稳定选中主窗口（页面可能已导航到任意路由）。
export const connectDesktopPage = async (cdpUrl, viewport = DEFAULT_VIEWPORT) => {
  const browser = await chromium.connectOverCDP(cdpUrl)
  const context = browser.contexts()[0]
  if (!context) {
    await browser.close()
    throw new Error(`No CDP context at ${cdpUrl}`)
  }
  const pages = context.pages().filter((p) => p.url().startsWith('http'))
  const nonTray = pages.filter((p) => {
    try {
      return !new URL(p.url()).pathname.startsWith('/tray')
    } catch {
      return false
    }
  })
  const main = nonTray[0] || pages[pages.length - 1]
  if (!main) {
    await browser.close()
    throw new Error(`No app page at ${cdpUrl}`)
  }
  // Phase 0 同口径：桌面窗口实测 1200x800，CDP 视口模拟统一为 1800x1125
  // （与 web 模式 launchPage 一致，保证前后对比口径相同）。
  try {
    await main.setViewportSize(viewport)
  } catch {
    // WebView2 主窗口不支持 setViewportSize 时保持原生窗口尺寸
  }
  return { browser, page: main }
}
