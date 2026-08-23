#!/usr/bin/env node
/* eslint-disable no-console -- 测量脚本，与 check-bundle-budget.mjs 同一豁免模式 */
// 场景2 列表滚动帧率（08-22-arch-quality-perf 批次7）
//
// 现状：应用内唯一虚拟化列表 HistoryList（@tanstack/vue-virtual）由 IPC get_history
// 供数——web 模式 0 行、桌面运行时实测 17 行，均无法达到设计目标的 10,000 行
// （详见 perf-baseline.md 场景2 说明与阶段7 恢复计划）。
//
// 本脚本测量「可渲染的最大数据集」：监控页日志流经应用自身 logger 模块注入 500 条
// （useMonitoringFeed DEFAULT_MAX_ENTRIES 上限），程序化滚动固定距离，用 rAF 时间戳
// 序列计算帧间隔分布与掉帧数，记录实际行数（500）。10k 目标在阶段7（React 侧可
// mock 数据）恢复，方法不变。
//
// 框架无关：logger 注入走应用自身的日志模块（与真实前端日志同一管道），不 import vue/react。
// 运行：bun ./scripts/perf/perf-list-scroll.mjs --base-url http://127.0.0.1:4180 --runs 3
import { parseArgs, percentiles, round, printJson, rsd, mean, stdev, launchPage } from './_lib.mjs'

const ROW_TARGET = 500 // 日志流条目上限（src/composables/useMonitoringFeed.ts DEFAULT_MAX_ENTRIES）
const SCROLL_STEP_PX = 600 // 每次滚动固定距离
const SCROLL_STEPS = 8
const SCROLL_SETTLE_MS = 350
const ROW_SELECTOR = '[data-testid="monitoring-log-row"]'

const injectLogs = async (page, count) => {
  return page.evaluate(async (n) => {
    // 应用自身 logger 模块：监控页 useMonitoringFeed 通过 logger.subscribe 接收，
    // 与真实前端日志（claudeObserver 告警、Perf 路由计时等）同一注入管道。
    const mod = await import('/src/utils/logger.ts')
    for (let i = 0; i < n; i++) {
      mod.logger.info(`perf-scenario2-synthetic-row-${i}`, { i, pad: 'x'.repeat(40) })
      if (i % 100 === 0) {
        // 让出主线程，避免单帧同步 500 次通知掩盖真实渲染成本
        await new Promise((resolve) => setTimeout(resolve, 0))
      }
    }
    return n
  }, count)
}

const measureScroll = async (page) => {
  const { stamps, steps, scrollTop, scrollHeight, clientHeight, rows } = await page.evaluate(
    async ({ step, steps, settle, rowSelector }) => {
      const rows = document.querySelectorAll(rowSelector)
      const container = rows[0] ? rows[0].parentElement : null
      if (!container || rows.length === 0) {
        return { error: 'no log container / zero rows' }
      }
      const stamps = []
      const collect = () => {
        stamps.push(performance.now())
        requestAnimationFrame(collect)
      }
      requestAnimationFrame(collect)
      // 从顶部开始，向下程序化滚动固定距离
      container.scrollTop = 0
      await new Promise((resolve) => setTimeout(resolve, settle))
      const scrollStart = performance.now()
      for (let i = 0; i < steps; i++) {
        container.scrollTop += step
        await new Promise((resolve) => setTimeout(resolve, settle))
      }
      const scrollEnd = performance.now()
      // 收尾帧
      await new Promise((resolve) => requestAnimationFrame(() => resolve()))
      const measured = stamps.filter((t) => t >= scrollStart && t <= scrollEnd)
      return {
        stamps: measured,
        steps,
        scrollTop: container.scrollTop,
        scrollHeight: container.scrollHeight,
        clientHeight: container.clientHeight,
        rows: rows.length,
      }
    },
    { step: SCROLL_STEP_PX, steps: SCROLL_STEPS, settle: SCROLL_SETTLE_MS, rowSelector: ROW_SELECTOR },
  )

  if (stamps.error) return stamps
  const intervals = []
  for (let i = 1; i < stamps.length; i++) {
    intervals.push(stamps[i] - stamps[i - 1])
  }
  const fpsValues = intervals.map((d) => (d > 0 ? 1000 / d : 0)).filter((v) => v > 0)
  const droppedFrames33 = intervals.filter((d) => d > 33.3).length
  const droppedFrames50 = intervals.filter((d) => d > 50).length
  return {
    steps,
    rows: rows,
    scrollTop,
    scrollHeight,
    clientHeight,
    frameSamples: intervals.length,
    fps: percentiles(fpsValues, [50, 95]),
    fpsMean: round(mean(fpsValues), 2),
    fpsStdev: round(stdev(fpsValues), 2),
    droppedFramesGt33ms: droppedFrames33,
    droppedFramesGt50ms: droppedFrames50,
  }
}

const main = async () => {
  const args = parseArgs(process.argv.slice(2))
  const warmup = args.runs > 1
  const { browser, page } = await launchPage()
  const results = []

  try {
    await page.goto(args.baseUrl + '/monitoring', { waitUntil: 'domcontentloaded', timeout: 30000 })
    await page.waitForTimeout(2500)

    if (warmup) {
      await injectLogs(page, ROW_TARGET)
      await page.waitForTimeout(2000)
      const first = await measureScroll(page)
      if (first.rows !== ROW_TARGET) {
        console.log(`[perf-list-scroll] 警告：注入后实际渲染 ${first.rows} 行（目标 ${ROW_TARGET}）`)
      }
      await page.reload({ waitUntil: 'domcontentloaded' })
      await page.waitForTimeout(2500)
      console.log('[perf-list-scroll] 预热一轮完成（不计入统计）')
    }

    for (let run = 1; run <= args.runs; run++) {
      await injectLogs(page, ROW_TARGET)
      await page.waitForTimeout(2000)
      const row = await measureScroll(page)
      results.push(row)
      console.log(
        `[perf-list-scroll] run${run} rows=${row.rows} fpsMean=${row.fpsMean} ` +
        `fpsP50=${round(row.fps?.P50 ?? 0, 2)} fpsP95=${round(row.fps?.P95 ?? 0, 2)} ` +
        `dropped>33ms=${row.droppedFramesGt33ms} dropped>50ms=${row.droppedFramesGt50ms} frames=${row.frameSamples}`,
      )
      // run 间隔离：重载页面清空注入数据
      if (run < args.runs) {
        await page.reload({ waitUntil: 'domcontentloaded' })
        await page.waitForTimeout(2500)
      }
    }
  } finally {
    await browser.close()
  }

  const fpsMeans = results.map((r) => r.fpsMean)
  const fpsP95 = results.map((r) => r.fps?.P95 ?? 0)
  const dropped33 = results.map((r) => r.droppedFramesGt33ms)
  const dropped50 = results.map((r) => r.droppedFramesGt50ms)

  printJson({
    scenario: 2,
    method: '注入最大可渲染数据集（监控日志流，DEFAULT_MAX_ENTRIES=500），程序化滚动固定距离，rAF 时间戳算帧率与掉帧',
    rows: results.map((r) => r.rows),
    viewport: '1800x1125',
    runs: results.map((r) => ({
      fpsMean: r.fpsMean,
      fpsP50: round(r.fps?.P50 ?? 0, 2),
      fpsP95: round(r.fps?.P95 ?? 0, 2),
      droppedGt33ms: r.droppedFramesGt33ms,
      droppedGt50ms: r.droppedFramesGt50ms,
    })),
    aggregate: {
      fpsMean: round(mean(fpsMeans), 2),
      fpsMeanRSD: round(rsd(fpsMeans) * 100, 1),
      fpsP95: round(mean(fpsP95), 2),
      fpsP95RSD: round(rsd(fpsP95) * 100, 1),
      droppedGt33ms: round(mean(dropped33), 1),
      droppedGt33msRSD: round(rsd(dropped33) * 100, 1),
      droppedGt50ms: round(mean(dropped50), 1),
      droppedGt50msRSD: round(rsd(dropped50) * 100, 1),
    },
    note: '唯一虚拟化列表 HistoryList 依赖 IPC get_history，web 0 行 / 桌面 17 行，未达 10k；10k 目标在阶段7 React 侧（可 mock 数据）恢复',
  })

  for (const r of results) {
    console.log(
      `| run | ${r.rows} 行 | 帧率均值 ${r.fpsMean} FPS | P50 ${round(r.fps?.P50 ?? 0, 2)} / P95 ${round(r.fps?.P95 ?? 0, 2)} | ` +
      `掉帧>33ms ${r.droppedFramesGt33ms} | 掉帧>50ms ${r.droppedFramesGt50ms} | 样本 ${r.frameSamples} |`,
    )
  }
  console.log(
    `| 聚合 | ${results[0]?.rows ?? '-'} 行 | 帧率均值 ${round(mean(fpsMeans), 2)} FPS (RSD ${round(rsd(fpsMeans) * 100, 1)}%) | ` +
    `P95 ${round(mean(fpsP95), 2)} (RSD ${round(rsd(fpsP95) * 100, 1)}%) | 掉帧>33ms ${round(mean(dropped33), 1)} | ` +
    `掉帧>50ms ${round(mean(dropped50), 1)} | ${results.length} 次 |`,
  )
}

await main()
