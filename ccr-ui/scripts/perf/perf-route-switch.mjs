#!/usr/bin/env node
/* eslint-disable no-console -- 测量脚本，与 check-bundle-budget.mjs 同一豁免模式 */
// 场景5 路由切换（08-22-arch-quality-perf 批次7）
//
// 方法：复用 measure-vite-route.mjs 与 perfTelemetry.ts recordRouteTiming 的口径——
// recordRouteTiming 记录 router.beforeEach → afterEach 的切换耗时（路由解析+组件挂载）。
// 本脚本在页面内安装 #app 子树 MutationObserver（新增节点 = 新路由组件挂载），
// SPA 导航（history.pushState + PopStateEvent，与 createWebHistory 同机制）后记录
// dispatch → 首次内容变更（mount 开始）与 → 双 rAF 后内容稳定（settle）。
// 75 条路由按域采样，每域 2–3 条（见 ROUTE_SAMPLES）。
//
// 框架无关：只依赖 DOM + performance API + playwright 驱动。
// 运行：bun ./scripts/perf/perf-route-switch.mjs --base-url http://127.0.0.1:4180 --runs 3
import { parseArgs, round, printJson, rsd, mean, percentiles, launchPage } from './_lib.mjs'

// 按域采样（域 → 路由），每域 2–3 条。redirect 路由保留（真实用户也会命中）。
const ROUTE_SAMPLES = [
  { domain: 'home', route: '/' },
  { domain: 'settings', route: '/settings' },
  { domain: 'claude-code', route: '/claude-code' },
  { domain: 'claude-code', route: '/claude-code/settings' },
  { domain: 'claude-code', route: '/claude-code/profiles' },
  { domain: 'codex', route: '/codex' },
  { domain: 'codex', route: '/codex/settings' },
  { domain: 'codex', route: '/codex/mcp' },
  { domain: 'grok', route: '/grok' },
  { domain: 'grok', route: '/grok/settings' },
  { domain: 'gemini', route: '/antigravity' },
  { domain: 'gemini', route: '/antigravity/mcp' },
  { domain: 'opencode', route: '/opencode' },
  { domain: 'opencode', route: '/opencode/settings' },
  { domain: 'opencode', route: '/opencode/providers' },
  { domain: 'tools', route: '/commands' },
  { domain: 'tools', route: '/converter' },
  { domain: 'tools', route: '/sync' },
  { domain: 'config', route: '/configs' },
  { domain: 'config', route: '/mcp-manager' },
  { domain: 'config', route: '/slash-commands' },
  { domain: 'data', route: '/budget' },
  { domain: 'data', route: '/pricing' },
  { domain: 'data', route: '/usage' },
  { domain: 'data', route: '/monitoring' },
  { domain: 'checkin', route: '/checkin' },
  { domain: 'environment', route: '/wsl' },
  { domain: 'environment', route: '/ssh' },
  { domain: 'skills', route: '/skills' },
]

const SETTLE_QUIET_MS = 120 // 内容稳定判定：连续 120ms 无新增节点突变
const NAV_TIMEOUT_MS = 10000

const installRouteObserver = async (page) => {
  return page.evaluate(({ quietMs }) => {
    window.__routeObs = null
    window.__routeChangedAt = null
    window.__routeSettledAt = null
    const root = document.querySelector('#app')
    if (!root) return false
    let lastMutationAt = 0
    let settleTimer = null
    window.__routeObs = new MutationObserver((mutations) => {
      const hasAdded = mutations.some((m) => m.type === 'childList' && m.addedNodes.length > 0)
      if (!hasAdded) return
      lastMutationAt = performance.now()
      if (window.__routeChangedAt === null) {
        window.__routeChangedAt = lastMutationAt
      }
      if (settleTimer) clearTimeout(settleTimer)
      settleTimer = setTimeout(() => {
        window.__routeSettledAt = performance.now()
      }, quietMs)
    })
    window.__routeObs.observe(root, { subtree: true, childList: true })
    return true
  }, { quietMs: SETTLE_QUIET_MS })
}

const navigateOnce = async (page, toPath) => {
  return page.evaluate(
    async ({ toPath: to, timeoutMs }) => {
      if (!window.__routeObs) return { route: to, ms: null, error: 'no observer' }
      // 重置本次导航信号
      window.__routeChangedAt = null
      window.__routeSettledAt = null
      const t0 = performance.now()
      // 与 createWebHistory 同机制：pushState + popstate 触发 vue-router 导航
      history.pushState(null, '', to)
      window.dispatchEvent(new PopStateEvent('popstate', { state: null }))
      const deadline = Date.now() + timeoutMs
      return new Promise((resolve) => {
        const check = () => {
          if (window.__routeSettledAt !== null && window.__routeChangedAt !== null) {
            const changedMs = window.__routeChangedAt - t0
            const settledMs = window.__routeSettledAt - t0
            resolve({ route: to, changedMs, settledMs, path: location.pathname })
            return
          }
          if (Date.now() >= deadline) {
            resolve({ route: to, changedMs: window.__routeChangedAt !== null ? window.__routeChangedAt - t0 : null, settledMs: null, timeout: true, path: location.pathname })
            return
          }
          setTimeout(check, 25)
        }
        check()
      })
    },
    { toPath: toPath, timeoutMs: NAV_TIMEOUT_MS },
  )
}

const measureRun = async (page) => {
  const records = []
  for (const sample of ROUTE_SAMPLES) {
    const r = await navigateOnce(page, sample.route)
    records.push({
      domain: sample.domain,
      route: sample.route,
      changedMs: r.changedMs,
      settledMs: r.settledMs,
      timeout: r.timeout,
      actualPath: r.path,
    })
    if (r.timeout) {
      console.log(`[perf-route-switch] ${sample.route} 切换超时（>${NAV_TIMEOUT_MS}ms）`)
    }
  }
  const ok = records.filter((r) => !r.timeout && r.changedMs !== null && r.settledMs !== null)
  return {
    records,
    aggregate: {
      changed: percentiles(ok.map((r) => r.changedMs), [50, 95]),
      settled: percentiles(ok.map((r) => r.settledMs), [50, 95]),
      sampleCount: ok.length,
    },
  }
}

const main = async () => {
  const args = parseArgs(process.argv.slice(2))
  const warmup = args.runs > 1
  const { browser, page } = await launchPage()
  const results = []

  try {
    await page.goto(args.baseUrl + '/', { waitUntil: 'domcontentloaded', timeout: 30000 })
    await page.waitForTimeout(2500)
    const installed = await installRouteObserver(page)
    if (!installed) {
      throw new Error('无法在 #app 上安装路由 MutationObserver')
    }

    if (warmup) {
      await measureRun(page)
      await page.reload({ waitUntil: 'domcontentloaded' })
      await page.waitForTimeout(2500)
      await installRouteObserver(page)
      console.log('[perf-route-switch] 预热一轮完成（首访 Vite 转译已缓存，不计入统计）')
    }

    for (let run = 1; run <= args.runs; run++) {
      const row = await measureRun(page)
      results.push(row)
      console.log(
        `[perf-route-switch] run${run} changed P50=${round(row.aggregate.changed.P50, 1)}ms ` +
        `P95=${round(row.aggregate.changed.P95, 1)}ms settled P50=${round(row.aggregate.settled.P50, 1)}ms ` +
        `P95=${round(row.aggregate.settled.P95, 1)}ms (n=${row.aggregate.sampleCount}/${ROUTE_SAMPLES.length})`,
      )
      if (run < args.runs) {
        await page.reload({ waitUntil: 'domcontentloaded' })
        await page.waitForTimeout(2500)
        await installRouteObserver(page)
      }
    }
  } finally {
    await browser.close()
  }

  // 聚合（按 route 键值对 3 次运行求均值 + RSD）
  const routeKeys = ROUTE_SAMPLES.map((s) => s.route)
  const perRoute = routeKeys.map((route) => {
    const changed = results
      .flatMap((r) => r.records.filter((x) => x.route === route))
      .map((x) => x.changedMs)
      .filter((v) => v !== null)
    const settled = results
      .flatMap((r) => r.records.filter((x) => x.route === route))
      .map((x) => x.settledMs)
      .filter((v) => v !== null)
    return {
      route,
      domain: ROUTE_SAMPLES.find((s) => s.route === route)?.domain ?? '?',
      changed: {
        mean: round(mean(changed), 1),
        P50: round(percentiles(changed).P50, 1),
        P95: round(percentiles(changed).P95, 1),
        rsd: round(rsd(changed) * 100, 1),
        n: changed.length,
      },
      settled: {
        mean: round(mean(settled), 1),
        P50: round(percentiles(settled).P50, 1),
        P95: round(percentiles(settled).P95, 1),
        rsd: round(rsd(settled) * 100, 1),
        n: settled.length,
      },
    }
  })

  const allChanged = results.flatMap((r) => r.records.map((x) => x.changedMs)).filter((v) => v !== null)
  const allSettled = results.flatMap((r) => r.records.map((x) => x.settledMs)).filter((v) => v !== null)
  const changedP50s = results.map((r) => r.aggregate.changed.P50)
  const changedP95s = results.map((r) => r.aggregate.changed.P95)

  printJson({
    scenario: 5,
    method: 'SPA 导航（pushState+popstate），记录 dispatch→#app 首内容变更（mount）与→稳定（120ms 静默），75 条路由按域采样 2–3 条',
    routesSampled: routeKeys.length,
    viewport: '1800x1125',
    perRoute,
    aggregate: {
      changed: { P50: round(mean(changedP50s), 1), P50_RSD: round(rsd(changedP50s) * 100, 1), P95: round(mean(changedP95s), 1), P95_RSD: round(rsd(changedP95s) * 100, 1) },
      allSamplesChanged: percentiles(allChanged, [50, 95]),
      allSamplesSettled: percentiles(allSettled, [50, 95]),
    },
  })

  for (const r of perRoute) {
    console.log(
      `| ${r.domain} | ${r.route} | mount ${r.changed.mean}ms (P50 ${r.changed.P50} / P95 ${r.changed.P95}, RSD ${r.changed.rsd}%) | ` +
      `settle ${r.settled.mean}ms (P50 ${r.settled.P50} / P95 ${r.settled.P95}, RSD ${r.settled.rsd}%) |`,
    )
  }
  console.log(
    `| 聚合 | ${routeKeys.length} 条 | mount P50 ${round(mean(changedP50s), 1)}ms (RSD ${round(rsd(changedP50s) * 100, 1)}%) / ` +
    `P95 ${round(mean(changedP95s), 1)}ms | settle P50 ${round(percentiles(allSettled).P50, 1)}ms / P95 ${round(percentiles(allSettled).P95, 1)}ms |`,
  )
}

await main()
