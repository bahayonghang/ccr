#!/usr/bin/env node
/* eslint-disable no-console -- 测量脚本，与 check-bundle-budget.mjs 同一豁免模式 */
// 场景1 大表单输入延迟（08-22-arch-quality-perf 批次7）
//
// 方法：在 AppSettingsView / ClaudeCodeSettingsView / CodexSettingsView 三页各选一个
// 文本字段，连续键入 200 字符，in-page 记录每次 input 事件到「下一帧 rAF」的时间差
// （rAF 回调先于该帧绘制，等价于 input → 下一帧 paint 的时间代理）。报告每页 P50/P95。
//
// 框架无关：只依赖 DOM + performance API + playwright 驱动，无 vue/react import。
// 运行：bun ./scripts/perf/perf-form-input.mjs --base-url http://127.0.0.1:4180 --runs 3
import { parseArgs, percentiles, round, printJson, rsd, mean, launchPage, connectDesktopPage } from './_lib.mjs'

const CHARS = 200
const TYPE_DELAY_MS = 8 // 每键间隔，保证大部分按键落在独立帧，样本量与帧率解耦
const TEXT = 'a'.repeat(CHARS)

const PAGE_CONFIGS = [
  {
    name: 'AppSettingsView',
    route: '/settings',
    // 页面就绪信号（首屏骨架/设置页渲染），setup 之后 inputSelector 才出现
    readySelector: '.app-settings-view, #app > *',
    // 字体自定义输入在 select 选 __custom__ 后才渲染
    setup: async (page) => {
      await page.selectOption('[data-testid="settings-font-ui"]', '__custom__')
      await page.waitForSelector('[data-testid="settings-font-ui-input"]', { timeout: 8000 })
    },
    inputSelector: '[data-testid="settings-font-ui-input"]',
  },
  {
    name: 'ClaudeCodeSettingsView',
    route: '/claude-code/settings',
    // React BaseSettings：默认 model 页，maxOutputTokens 对应 Vue 占位 31999 的输出上限字段
    readySelector: '#platform-settings-form, #app > *',
    inputSelector: 'input[name="maxOutputTokens"]',
  },
  {
    name: 'CodexSettingsView',
    route: '/codex/settings',
    // React BaseSettings：默认 model 页，model 文本框对应 Vue placeholder^="gpt-5"
    readySelector: '#platform-settings-form, #app > *',
    inputSelector: 'input[name="model"]',
  },
]

const measurePage = async (page, config) => {
  await page.goto(config.baseUrl + config.route, { waitUntil: 'domcontentloaded', timeout: 30000 })
  await page.waitForSelector(config.readySelector, { timeout: 15000 })
  if (config.setup) await config.setup(page)
  await page.waitForSelector(config.inputSelector, { timeout: 15000 })
  // 等待首屏 / 懒加载 / 主题样式稳定后再开始测量
  await page.waitForTimeout(800)

  // in-page 记录器：input 事件打点，下一次 rAF 回调（先于该帧 paint）取时间差
  await page.evaluate(({ selector }) => {
    window.__perfInputDeltas = []
    window.__perfInputPendingAt = null
    const el = document.querySelector(selector)
    if (!el) return
    const loop = () => {
      if (window.__perfInputPendingAt !== null) {
        window.__perfInputDeltas.push(performance.now() - window.__perfInputPendingAt)
        window.__perfInputPendingAt = null
      }
      requestAnimationFrame(loop)
    }
    requestAnimationFrame(loop)
    el.addEventListener('input', () => {
      window.__perfInputPendingAt = performance.now()
    })
  }, { selector: config.inputSelector })

  await page.focus(config.inputSelector)
  await page.keyboard.type(TEXT, { delay: TYPE_DELAY_MS })
  await page.waitForTimeout(400)

  const deltas = await page.evaluate(() => (window.__perfInputDeltas || []).slice())
  return {
    page: config.name,
    route: config.route,
    samples: deltas.length,
    stats: percentiles(deltas),
  }
}

const main = async () => {
  const args = parseArgs(process.argv.slice(2))
  const warmup = args.runs > 1
  const useCdp = process.argv.includes('--cdp')
  const { browser, page } = useCdp
    ? await connectDesktopPage(args.cdpUrl)
    : await launchPage()
  const results = []

  try {
    if (warmup) {
      // 预热一轮（不计入统计）：首访路由的 Vite 转译与首屏资源加载排除在测量外
      await measurePage(page, { ...PAGE_CONFIGS[0], baseUrl: args.baseUrl })
      console.log('[perf-form-input] 预热一轮完成（不计入统计）')
    }

    for (let run = 1; run <= args.runs; run++) {
      const runResult = []
      for (const config of PAGE_CONFIGS) {
        try {
          const row = await measurePage(page, { ...config, baseUrl: args.baseUrl })
          runResult.push(row)
          console.log(
            `[perf-form-input] run${run} ${row.page} ${row.route} samples=${row.samples} ` +
            `P50=${round(row.stats.P50, 2)}ms P95=${round(row.stats.P95, 2)}ms mean=${round(row.stats.mean, 2)}ms`,
          )
        } catch (error) {
          const message = error instanceof Error ? error.message : String(error)
          console.log(`[perf-form-input] run${run} ${config.name} ${config.route} FAILED ${message}`)
          runResult.push({
            page: config.name,
            route: config.route,
            samples: 0,
            stats: {},
            error: message,
          })
        }
      }
      results.push(runResult)
    }
  } finally {
    await browser.close()
  }

  const summary = PAGE_CONFIGS.map((config, i) => {
    const perRun = results.map((r) => r[i])
    const ok = perRun.filter((r) => !r.error && Number.isFinite(r.stats?.P50))
    const p50s = ok.map((r) => r.stats.P50)
    const p95s = ok.map((r) => r.stats.P95)
    return {
      page: config.name,
      route: config.route,
      runs: perRun.map((r) => (
        r.error
          ? { samples: 0, error: r.error }
          : { samples: r.samples, P50: round(r.stats.P50, 2), P95: round(r.stats.P95, 2) }
      )),
      aggregate: ok.length === 0
        ? { error: 'no successful runs' }
        : {
            P50: round(mean(p50s), 2),
            P95: round(mean(p95s), 2),
            P50_RSD: round(rsd(p50s) * 100, 1),
            P95_RSD: round(rsd(p95s) * 100, 1),
            n: ok.length,
          },
    }
  })

  printJson({
    scenario: 1,
    method: '连续键入200字符，记录 input→下一帧 rAF 间隔，按页取 P50/P95',
    chars: CHARS,
    viewport: '1800x1125',
    warmupExcluded: warmup,
    pages: summary,
  })

  // 人类可读行
  for (const s of summary) {
    console.log(
      `| ${s.page} | ${s.route} | ${s.runs.map((r) => `P50 ${r.P50}ms / P95 ${r.P95}ms`).join(' | ')} | ` +
      `均值 P50 ${s.aggregate.P50}ms / P95 ${s.aggregate.P95}ms | RSD ${s.aggregate.P50_RSD}% / ${s.aggregate.P95_RSD}% |`,
    )
  }
}

await main()
