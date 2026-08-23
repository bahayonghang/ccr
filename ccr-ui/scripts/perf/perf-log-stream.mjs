#!/usr/bin/env node
/* eslint-disable no-console -- 测量脚本，与 check-bundle-budget.mjs 同一豁免模式 */
// 场景3 日志流（08-22-arch-quality-perf 批次7）
//
// 方法：桌面运行时（tauri dev，WebView2 真实 IPC）MonitoringView 持续注入日志 5 分钟。
// 注入走应用自身 logger 模块（useMonitoringFeed 通过 logger.subscribe 消费，与真实
// 前端日志同一管道）。每 10 秒采样 performance.memory.usedJSHeapSize 与 rAF 帧率
// （1 秒窗口内帧数）。报告内存增长斜率（线性回归）与帧率分布。
//
// 说明：日志流条目受 DEFAULT_MAX_ENTRIES=500 上限约束，DOM 行数稳定在 500 附近，
// 内存增长斜率反映「稳定流下的稳态开销」，为真实（可能接近零）而非编造。
//
// 框架无关：只依赖 DOM / performance API + playwright CDP 驱动。
// 运行：bun ./scripts/perf/perf-log-stream.mjs --cdp-url http://127.0.0.1:9222 --runs 3
import { parseArgs, round, printJson, rsd, mean, linearSlope, percentiles, connectDesktopPage } from './_lib.mjs'

// 桌面运行时经 CDP 连接，baseUrl 默认指向 tauri dev 的 devUrl（与 connectDesktopPage 配套）
const DEFAULT_DESKTOP_BASE = 'http://127.0.0.1:15173'

const DURATION_MS = 5 * 60 * 1000 // 5 分钟
const SAMPLE_INTERVAL_MS = 10_000 // 每 10 秒采样
const FPS_WINDOW_MS = 1000 // 帧率采样窗口
const INJECT_INTERVAL_MS = 300 // 注入速率 ≈ 3.3 条/秒，持续且不触发瞬间满 500 上限的尖峰
const ROW_SELECTOR = '[data-testid="monitoring-log-row"]'

const startInjection = async (page) => {
  await page.evaluate((intervalMs) => {
    window.__perfLogSeq = 0
    window.__perfLogRunning = true
    window.__perfLogTimer = setInterval(() => {
      if (!window.__perfLogRunning) return
      const seq = window.__perfLogSeq++
      // 应用自身 logger 模块；import 幂等（模块缓存），首次成功后为同步调用
      void import('/src/utils/logger.ts').then((mod) => {
        mod.logger.info(`perf-scenario3-stream-${seq}`, {
          seq,
          payload: 'log-stream-synthetic-entry',
        })
      })
    }, intervalMs)
  }, INJECT_INTERVAL_MS)
}

const stopInjection = async (page) => {
  await page.evaluate(() => {
    window.__perfLogRunning = false
    if (window.__perfLogTimer) clearInterval(window.__perfLogTimer)
    window.__perfLogTimer = null
  })
}

const sampleOnce = async (page, session) => {
  // 采样前先强制一次 GC（CDP HeapProfiler.collectGarbage）：堆读数反映「保留内存」，
  // 而非任意 GC 时刻的瞬时堆大小。WebView2 的 GC 时机不确定，不强制时相邻两次
  // 采样可能分别落在 GC 前后，使堆增量 RSD 远超 15%（R7 迭代记录见 perf-baseline.md §3）。
  if (session) {
    try {
      await session.send('HeapProfiler.collectGarbage')
    } catch {
      // 收集失败不阻断采样（旧内核/会话已关），退化回瞬时读数
    }
  }
  return page.evaluate(async ({ fpsWindow, rowSelector }) => {
    // 1 秒窗口内的 rAF 帧数 → FPS
    let frames = 0
    const s0 = performance.now()
    await new Promise((resolve) => {
      const tick = () => {
        frames += 1
        if (performance.now() - s0 < fpsWindow) {
          requestAnimationFrame(tick)
        } else {
          resolve()
        }
      }
      requestAnimationFrame(tick)
    })
    const memory = typeof performance.memory !== 'undefined' ? performance.memory : null
    return {
      usedJSHeapSize: memory ? memory.usedJSHeapSize : null,
      totalJSHeapSize: memory ? memory.totalJSHeapSize : null,
      fps: frames,
      rows: document.querySelectorAll(rowSelector).length,
    }
  }, { fpsWindow: FPS_WINDOW_MS, rowSelector: ROW_SELECTOR })
}

const main = async () => {
  const args = parseArgs(process.argv.slice(2), { baseUrl: DEFAULT_DESKTOP_BASE })
  const warmup = args.runs > 1
  const { browser, page } = await connectDesktopPage(args.cdpUrl)
  const results = []

  // CDP 会话用于采样前强制 GC（见 sampleOnce 注释）
  let cdpSession = null
  try {
    cdpSession = await page.context().newCDPSession(page)
    await cdpSession.send('HeapProfiler.enable')
  } catch {
    cdpSession = null
  }

  try {
    await page.goto(args.baseUrl + '/monitoring', { waitUntil: 'domcontentloaded', timeout: 30000 })
    await page.waitForTimeout(3000)

    if (warmup) {
      // R7 迭代 1（2026-08-23，完整记录见 perf-baseline.md §3）：原预热仅 30 秒注入
      // （约 100 行），首轮测量 run 仍承担完整冷启动开销（500 行渲染路径的 V8 编译
      // 与模块图加载），使 run1 保留堆高出后续 run 约 3.5MB（堆 Δ 5.12M vs 1.41/2.20M，
      // 首轮 RSD 67%）。按 design §7「固定更多变量」，预热升级为完整弃用轮：注入满
      // DURATION_MS（5 分钟，等价于正式 run）后弃用其数据，每个测量 run 均从等价
      // 饱和态出发。
      await startInjection(page)
      await new Promise((resolve) => setTimeout(resolve, DURATION_MS))
      await stopInjection(page)
      await page.reload({ waitUntil: 'domcontentloaded' })
      await page.waitForTimeout(3000)
      console.log('[perf-log-stream] 预热弃用轮完成（完整 5 分钟注入，数据弃用不计入统计）')
    }

    for (let run = 1; run <= args.runs; run++) {
      await startInjection(page)
      const samples = []
      const t0 = Date.now()
      while (Date.now() - t0 < DURATION_MS) {
        await new Promise((resolve) => setTimeout(resolve, SAMPLE_INTERVAL_MS))
        const sample = await sampleOnce(page, cdpSession)
        samples.push(sample)
        const elapsedSec = round((Date.now() - t0) / 1000, 0)
        console.log(
          `[perf-log-stream] run${run} t=${elapsedSec}s heap=${sample.usedJSHeapSize} ` +
          `fps=${sample.fps} rows=${sample.rows}`,
        )
      }
      await stopInjection(page)

      const valid = samples.filter((s) => s.usedJSHeapSize !== null)
      // 用采样序号换算相对时间（samples 已按 10s 间隔推进）
      const elapsed = valid.map((s, i) => i * SAMPLE_INTERVAL_MS)
      const heaps = valid.map((s) => s.usedJSHeapSize)
      const fpsList = samples.map((s) => s.fps)
      const slopePerSec = linearSlope(elapsed, heaps)
      const memDelta = heaps.length > 0 ? heaps[heaps.length - 1] - heaps[0] : 0

      results.push({
        run,
        samples: samples.length,
        fps: percentiles(fpsList, [50, 95]),
        fpsMean: round(mean(fpsList), 2),
        memory: {
          startBytes: heaps[0],
          endBytes: heaps[heaps.length - 1],
          deltaBytes: memDelta,
          slopeBytesPerSec: slopePerSec === null ? null : round(slopePerSec * 1000, 1),
        },
        rowsFinal: samples[samples.length - 1]?.rows ?? null,
      })

      // run 间隔离：重载清空注入与内存态
      if (run < args.runs) {
        await page.reload({ waitUntil: 'domcontentloaded' })
        await page.waitForTimeout(3000)
      }
    }
  } finally {
    try {
      await stopInjection(page)
    } catch {
      // 页面已关时忽略
    }
    await browser.close()
  }

  const slopes = results.map((r) => r.memory.slopeBytesPerSec).filter((v) => v !== null)
  const fpsMeans = results.map((r) => r.fpsMean)
  const deltas = results.map((r) => r.memory.deltaBytes)

  printJson({
    scenario: 3,
    method: '桌面运行时（tauri dev）MonitoringView 持续注入日志 5 分钟，每 10 秒采样 usedJSHeapSize + rAF 帧率',
    durationMs: DURATION_MS,
    sampleIntervalMs: SAMPLE_INTERVAL_MS,
    injectIntervalMs: INJECT_INTERVAL_MS,
    viewport: 'tauri WebView2 默认窗口（1800x1125 CDP 视口）',
    runs: results.map((r) => ({
      fpsMean: r.fpsMean,
      fpsP50: round(r.fps.P50, 2),
      fpsP95: round(r.fps.P95, 2),
      heapStartBytes: r.memory.startBytes,
      heapEndBytes: r.memory.endBytes,
      heapDeltaBytes: r.memory.deltaBytes,
      slopeBytesPerSec: r.memory.slopeBytesPerSec,
      rowsFinal: r.rowsFinal,
    })),
    aggregate: {
      fpsMean: round(mean(fpsMeans), 2),
      fpsMeanRSD: round(rsd(fpsMeans) * 100, 1),
      heapDeltaMeanBytes: round(mean(deltas), 1),
      heapDeltaRSD: round(rsd(deltas) * 100, 1),
      slopeBytesPerSec: slopes.length > 0 ? round(mean(slopes), 1) : null,
      slopeRSD: slopes.length > 0 ? round(rsd(slopes) * 100, 1) : null,
    },
    note: '日志条目受 DEFAULT_MAX_ENTRIES=500 上限约束，DOM 行数稳定，内存增长反映稳态流开销',
  })

  for (const r of results) {
    console.log(
      `| run${r.run} | 帧率均值 ${r.fpsMean} FPS | P50 ${round(r.fps.P50, 2)} / P95 ${round(r.fps.P95, 2)} | ` +
      `堆 ${r.memory.startBytes} → ${r.memory.endBytes} B (Δ${r.memory.deltaBytes} B) | ` +
      `斜率 ${r.memory.slopeBytesPerSec} B/s | 最终行 ${r.rowsFinal} |`,
    )
  }
  console.log(
    `| 聚合 | 帧率均值 ${round(mean(fpsMeans), 2)} FPS (RSD ${round(rsd(fpsMeans) * 100, 1)}%) | ` +
    `堆 Δ均值 ${round(mean(deltas), 1)} B (RSD ${round(rsd(deltas) * 100, 1)}%) | ` +
    `斜率均值 ${slopes.length > 0 ? round(mean(slopes), 1) : 'n/a'} B/s |`,
  )
}

await main()
