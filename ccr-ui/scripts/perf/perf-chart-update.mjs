#!/usr/bin/env node
/* eslint-disable no-console -- 测量脚本，与 check-bundle-budget.mjs 同一豁免模式 */
// 场景4 图表更新与主题切换（08-22-arch-quality-perf 批次7）
//
// 方法：桌面运行时（tauri dev，真实 llmusage 数据）用量页 /usage：
// - 时间范围切换 20 次（Today/This Week/This Month/Last 30 Days/All Time 循环，规避
//   PillToggleGroup 对同值点击的 no-op，保证每次都是真实切换）；
// - 明暗切换 20 次（切换 documentElement 的 data-theme + .dark，与主题 store 的
//   applyThemeToDocument 同源，触发图表 MutationObserver → syncChartTheme → 图表重渲染）。
// 每次记录点击/切换 → 图表画布（.apexcharts-canvas 子树）首次 DOM 变更的耗时。
//
// 框架无关：只依赖 DOM + performance API + playwright CDP 驱动。
// 运行：bun ./scripts/perf/perf-chart-update.mjs --cdp-url http://127.0.0.1:9222 --runs 3
import { parseArgs, round, printJson, rsd, mean, percentiles, connectDesktopPage } from './_lib.mjs'

// 桌面运行时经 CDP 连接，baseUrl 默认指向 tauri dev 的 devUrl（与 connectDesktopPage 配套）
const DEFAULT_DESKTOP_BASE = 'http://127.0.0.1:15173'

const RANGE_BUTTONS = ['Today', 'This Week', 'This Month', 'Last 30 Days', 'All Time']
const RANGE_CLICKS = 20
const THEME_TOGGLES = 20
const OP_TIMEOUT_MS = 8000

// 范围按钮是 i18n 本地化文案（zh-CN 默认「今天/本周/本月/近 30 天/全部时间」），
// 不能按文案匹配；改为按 PillToggleGroup 的 .pill-toggle-group__item 顺序（0–4 =
// today/this_week/this_month/last_30d/all_time）定位，框架无关且与语言无关。
const RANGE_ITEM_SELECTOR = '.pill-toggle-group__item'

const setupChartObserver = async (page) => {
  return page.evaluate(() => {
    window.__chartUpdatedAt = null
    window.__chartPending = false
    window.__ensureChartObserver = () => {
      const canvas = document.querySelector('.apexcharts-canvas')
      if (!canvas) return false
      if (window.__chartObs) window.__chartObs.disconnect()
      window.__chartObs = new MutationObserver(() => {
        if (window.__chartPending) {
          window.__chartPending = false
          window.__chartUpdatedAt = performance.now()
        }
      })
      window.__chartObs.observe(canvas, { subtree: true, childList: true, attributes: true })
      if (canvas.parentElement) {
        window.__chartObs.observe(canvas.parentElement, { childList: true })
      }
      return true
    }
    return window.__ensureChartObserver()
  })
}

const runOp = async (page, action, label) => {
  return page.evaluate(
    async ({ action: act, label: lbl, timeoutMs, itemSelector, rangeLabels }) => {
      const ensure = () => {
        if (typeof window.__ensureChartObserver === 'function') return window.__ensureChartObserver()
        return false
      }
      if (!ensure()) return { label: lbl, ms: null, error: 'no chart canvas' }

      window.__chartPending = true
      window.__chartUpdatedAt = null
      const t0 = performance.now()

      if (act === 'range') {
        const items = Array.from(document.querySelectorAll(itemSelector))
        const idx = rangeLabels.indexOf(lbl)
        const btn = items[idx]
        if (!btn) return { label: lbl, ms: null, error: `range button "${lbl}" not found (got ${items.length} items)` }
        btn.click()
      } else {
        const cur = document.documentElement.getAttribute('data-theme')
        const next = cur === 'dark' ? 'light' : 'dark'
        document.documentElement.setAttribute('data-theme', next)
        document.documentElement.classList.toggle('dark', next === 'dark')
      }

      return new Promise((resolve) => {
        const deadline = Date.now() + timeoutMs
        const check = () => {
          if (window.__chartUpdatedAt !== null) {
            resolve({ label: lbl, ms: window.__chartUpdatedAt - t0 })
            return
          }
          if (Date.now() >= deadline) {
            resolve({ label: lbl, ms: null, timeout: true })
            return
          }
          setTimeout(check, 25)
        }
        check()
      })
    },
    { action, label, timeoutMs: OP_TIMEOUT_MS, itemSelector: RANGE_ITEM_SELECTOR, rangeLabels: RANGE_BUTTONS },
  )
}

const measureRun = async (page) => {
  const rangeTimes = []
  const themeTimes = []
  for (let i = 0; i < RANGE_CLICKS; i++) {
    const label = RANGE_BUTTONS[i % RANGE_BUTTONS.length]
    const r = await runOp(page, 'range', label)
    if (r.ms !== null) rangeTimes.push(r.ms)
    else console.log(`[perf-chart-update] 范围切换 ${label} 超时/失败: ${r.error ?? 'timeout'}`)
  }
  for (let i = 0; i < THEME_TOGGLES; i++) {
    const r = await runOp(page, 'theme', 'toggle')
    if (r.ms !== null) themeTimes.push(r.ms)
    else console.log(`[perf-chart-update] 主题切换 ${i + 1} 超时/失败: ${r.error ?? 'timeout'}`)
  }
  // 主题状态复位，避免影响下一次 run 的初始主题
  await page.evaluate(() => {
    document.documentElement.setAttribute('data-theme', 'light')
    document.documentElement.classList.remove('dark')
  })
  return {
    range: percentiles(rangeTimes, [50, 95]),
    theme: percentiles(themeTimes, [50, 95]),
    rangeSamples: rangeTimes.length,
    themeSamples: themeTimes.length,
  }
}

const main = async () => {
  const args = parseArgs(process.argv.slice(2), { baseUrl: DEFAULT_DESKTOP_BASE })
  const warmup = args.runs > 1
  const { browser, page } = await connectDesktopPage(args.cdpUrl)
  const results = []

  try {
    await page.goto(args.baseUrl + '/usage', { waitUntil: 'domcontentloaded', timeout: 30000 })
    await page.waitForSelector('.apexcharts-canvas', { timeout: 20000 })
    await setupChartObserver(page)
    await page.waitForTimeout(2500)

    if (warmup) {
      await measureRun(page)
      await page.reload({ waitUntil: 'domcontentloaded' })
      await page.waitForSelector('.apexcharts-canvas', { timeout: 20000 })
      await setupChartObserver(page)
      await page.waitForTimeout(2500)
      console.log('[perf-chart-update] 预热一轮完成（不计入统计）')
    }

    for (let run = 1; run <= args.runs; run++) {
      const row = await measureRun(page)
      results.push(row)
      console.log(
        `[perf-chart-update] run${run} rangeP50=${round(row.range.P50, 1)}ms rangeP95=${round(row.range.P95, 1)}ms ` +
        `themeP50=${round(row.theme.P50, 1)}ms themeP95=${round(row.theme.P95, 1)}ms ` +
        `(range n=${row.rangeSamples}, theme n=${row.themeSamples})`,
      )
      if (run < args.runs) {
        await page.reload({ waitUntil: 'domcontentloaded' })
        await page.waitForSelector('.apexcharts-canvas', { timeout: 20000 })
        await setupChartObserver(page)
        await page.waitForTimeout(2500)
      }
    }
  } finally {
    await browser.close()
  }

  const rangeP50s = results.map((r) => r.range.P50)
  const rangeP95s = results.map((r) => r.range.P95)
  const themeP50s = results.map((r) => r.theme.P50)
  const themeP95s = results.map((r) => r.theme.P95)

  printJson({
    scenario: 4,
    method: '桌面运行时用量页：时间范围切换 20 次 + 明暗切换 20 次，记录点击→图表画布首变更耗时',
    rangeClicks: RANGE_CLICKS,
    themeToggles: THEME_TOGGLES,
    viewport: 'tauri WebView2（1800x1125 CDP 视口）',
    runs: results.map((r) => ({
      range: { P50: round(r.range.P50, 1), P95: round(r.range.P95, 1), n: r.rangeSamples },
      theme: { P50: round(r.theme.P50, 1), P95: round(r.theme.P95, 1), n: r.themeSamples },
    })),
    aggregate: {
      range: {
        P50: round(mean(rangeP50s), 1),
        P50_RSD: round(rsd(rangeP50s) * 100, 1),
        P95: round(mean(rangeP95s), 1),
        P95_RSD: round(rsd(rangeP95s) * 100, 1),
      },
      theme: {
        P50: round(mean(themeP50s), 1),
        P50_RSD: round(rsd(themeP50s) * 100, 1),
        P95: round(mean(themeP95s), 1),
        P95_RSD: round(rsd(themeP95s) * 100, 1),
      },
    },
  })

  for (const r of results) {
    console.log(
      `| run | 范围切换 P50 ${round(r.range.P50, 1)} / P95 ${round(r.range.P95, 1)} ms | ` +
      `主题切换 P50 ${round(r.theme.P50, 1)} / P95 ${round(r.theme.P95, 1)} ms |`,
    )
  }
  console.log(
    `| 聚合 | 范围切换 P50 ${round(mean(rangeP50s), 1)} ms (RSD ${round(rsd(rangeP50s) * 100, 1)}%) / ` +
    `P95 ${round(mean(rangeP95s), 1)} ms | 主题切换 P50 ${round(mean(themeP50s), 1)} ms (RSD ${round(rsd(themeP50s) * 100, 1)}%) / ` +
    `P95 ${round(mean(themeP95s), 1)} ms |`,
  )
}

await main()
