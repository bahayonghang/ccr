/*
 * measure-after.mjs —— 第 9 项“改造后”性能复测(与 baseline-perf.json 同口径)
 *
 * 运行方式(dev server 需已在 15173 起好):
 *   node .trellis/tasks/07-07-ui-usage-dashboard/research/perf-harness/measure-after.mjs
 *
 * 口径对齐 baseline-perf.json:
 *   - vite dev http://127.0.0.1:15173/usage + tauri-shim.js 注入(fixture 30 天数据)
 *   - localStorage 预置 ccr-theme=light / ccr-flavor=clay / ccr-accent=clay
 *     + ccr-usage-fixture-variant=stale;viewport 1920x1080
 *   - 计时:纯 setTimeout(10ms) 轮询;rebuilt = 旧 canvas DOM 节点未存活(data-perf-id 标记法)
 *   - 序列:warmup(overview→tokens→cost→overview) + 3 轮 cycle(toTokens/backOverview/
 *     toCost/backOverview)+ 窗口切换 ×2(本月→近 30 天)+ 20 次快速往返测内存
 *
 * 与基线唯一的口径差异(记入 method 字段):
 *   基线计时终点是“首个『无标记』canvas 出现”(彼时每次切换都重建,新 canvas 必然出现);
 *   KeepAlive 后再进入不再产生新 canvas,故终点放宽为“活动 tab 出现 canvas(无论新旧)”,
 *   rebuilt 判定本身(旧节点是否存活)与基线完全一致。
 */
import { readFileSync, mkdirSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const REPO = resolve(HERE, "../../../../..");
const OUT_DIR = resolve(HERE, "../after");
mkdirSync(OUT_DIR, { recursive: true });

// playwright 从 ccr-ui 的 node_modules 解析(脚本位于 .trellis 下,裸导入解析不到)
const requireFromUi = createRequire(resolve(REPO, "ccr-ui/package.json"));
const { chromium } = requireFromUi("playwright");

const SHIM = readFileSync(resolve(HERE, "tauri-shim.js"), "utf8");
const BASE_URL = "http://127.0.0.1:15173/usage";
// tabKeys 顺序(src/views/usage/state/useUsageFilters.ts):overview=0, tokens=1, cost=2
const TAB = { overview: 0, tokens: 1, cost: 2 };
// 窗口 segment 顺序(UsageDashboardToolbar.vue rangeOptions):
// today=0, this_week=1, this_month=2, last_30d=3(默认), all_time=4
const SEG = { this_month: 2, last_30d: 3 };

const browser = await chromium.launch();
const context = await browser.newContext({
  viewport: { width: 1920, height: 1080 },
});
const page = await context.newPage();

const consoleErrors = [];
page.on("console", (m) => {
  if (m.type() === "error") consoleErrors.push(m.text());
});
page.on("pageerror", (e) => consoleErrors.push(String(e)));

// localStorage 预置必须先于 shim 注入:shim 在自身初始化时即读取 fixture-variant
await page.addInitScript(() => {
  localStorage.setItem("ccr-theme", "light");
  localStorage.setItem("ccr-flavor", "clay");
  localStorage.setItem("ccr-accent", "clay");
  localStorage.setItem("ccr-usage-fixture-variant", "stale");
});
await page.addInitScript(SHIM);

await page.goto(BASE_URL);
await page.waitForSelector(".usage-tabs .usage-tab", { timeout: 15000 });
await page.waitForSelector(".apexcharts-canvas", { timeout: 15000 });
await page.waitForTimeout(600); // 首屏落定(懒加载 heatmap 等)

// dataset 断言(基线:light/clay/clay)
const datasetSnapshot = await page.evaluate(() => ({
  ...document.documentElement.dataset,
}));
const datasetValues = Object.values(datasetSnapshot);
const datasetOk =
  datasetValues.includes("light") &&
  datasetValues.filter((v) => v === "clay").length >= 2;
const variant = await page.evaluate(() => window.__usageShimVariant);
if (variant !== "stale")
  throw new Error(`fixture variant 应为 stale,实际 ${variant}`);

// 页内测量工具:data-perf-id 节点标记(与基线同法)
await page.evaluate(() => {
  window.__perf = {
    idCounter: 0,
    tag() {
      document.querySelectorAll(".apexcharts-canvas").forEach((el) => {
        if (!el.dataset.perfId) el.dataset.perfId = String(++this.idCounter);
      });
    },
    connectedIds() {
      return [...document.querySelectorAll(".apexcharts-canvas")].map(
        (el) => el.dataset.perfId ?? null,
      );
    },
  };
});

/**
 * tab 切换测量:click → 10ms 轮询,等活动 tab 出现 canvas。
 * rebuilt = 出现的 canvas 全部没有旧标记(旧节点未存活)——与基线判定一致。
 */
const switchTab = (tabIndex) =>
  page.evaluate(
    (idx) =>
      new Promise((resolveP, rejectP) => {
        const perf = window.__perf;
        perf.tag();
        const beforeSet = new Set(perf.connectedIds());
        const btn = document.querySelectorAll(".usage-tabs .usage-tab")[idx];
        if (!btn) {
          rejectP(new Error("tab button not found: " + idx));
          return;
        }
        const t0 = performance.now();
        btn.click();
        const poll = () => {
          const canvases = [...document.querySelectorAll(".apexcharts-canvas")];
          // “出现”= 不属于切换前可见集合的 canvas(KeepAlive 回填的旧节点带 id 但不在 beforeSet;全新节点无 id)
          const appeared = canvases.filter(
            (el) => !el.dataset.perfId || !beforeSet.has(el.dataset.perfId),
          );
          if (appeared.length > 0) {
            const ms = Math.round(performance.now() - t0);
            const kept = appeared.filter((el) => el.dataset.perfId).length;
            const fresh = appeared.length - kept;
            perf.tag();
            resolveP({
              ms,
              rebuilt: kept === 0,
              keptCanvases: kept,
              freshCanvases: fresh,
            });
            return;
          }
          if (performance.now() - t0 > 5000) {
            rejectP(new Error("timeout: no canvas appeared for tab " + idx));
            return;
          }
          setTimeout(poll, 10);
        };
        setTimeout(poll, 10);
      }),
    tabIndex,
  );

/**
 * 窗口切换测量:click → 轮询。若出现无标记 canvas → remount(基线行为);
 * 若 refetch(get_usage_dashboard_v2 日志计数 +1)后再等 600ms 仍无新 canvas
 * → 判定不重挂,并核对切换前 canvas 节点仍全部存活。
 */
const switchWindow = (segIndex, label) =>
  page.evaluate(
    ({ idx, label: lbl }) =>
      new Promise((resolveP, rejectP) => {
        const perf = window.__perf;
        perf.tag();
        const beforeIds = perf.connectedIds().filter(Boolean);
        const dashCalls = () =>
          (window.__usageShimLog || []).filter(
            (e) => e.cmd === "get_usage_dashboard_v2",
          ).length;
        const callsBefore = dashCalls();
        const seg = document.querySelectorAll(
          ".usage-dashboard-toolbar__segment",
        )[idx];
        if (!seg) {
          rejectP(new Error("segment not found: " + idx));
          return;
        }
        const t0 = performance.now();
        seg.click();
        let fetchMs = null;
        const poll = () => {
          const nowT = performance.now();
          const fresh = [
            ...document.querySelectorAll(".apexcharts-canvas"),
          ].filter((el) => !el.dataset.perfId);
          if (fresh.length > 0) {
            perf.tag();
            resolveP({
              label: lbl,
              ms: Math.round(nowT - t0),
              chartRemounted: true,
              fetchMs,
            });
            return;
          }
          if (fetchMs === null && dashCalls() > callsBefore)
            fetchMs = Math.round(nowT - t0);
          // 结算条件二选一:refetch 后再静默 600ms,或 store 命中 30s 快照缓存
          // (DASHBOARD_CACHE_TTL_MS,无 IPC)时以 1500ms 静默为准。
          const settled =
            (fetchMs !== null && nowT - t0 > fetchMs + 600) ||
            (fetchMs === null && nowT - t0 > 1500);
          if (settled) {
            const survived = beforeIds.filter((id) => {
              const el = document.querySelector(
                `.apexcharts-canvas[data-perf-id="${id}"]`,
              );
              return !!el && el.isConnected;
            }).length;
            resolveP({
              label: lbl,
              ms: fetchMs ?? Math.round(nowT - t0),
              chartRemounted: false,
              fetchMs,
              servedFromCache: fetchMs === null,
              survivedCanvases: survived,
              canvasesBefore: beforeIds.length,
            });
            return;
          }
          if (nowT - t0 > 4000) {
            const segs = [
              ...document.querySelectorAll(".usage-dashboard-toolbar__segment"),
            ];
            const diag = {
              activeSegIndex: segs.findIndex((s) =>
                s.className.includes("--active"),
              ),
              clickedIndex: idx,
              dashCallsBefore: callsBefore,
              dashCallsNow: dashCalls(),
              canvases: document.querySelectorAll(".apexcharts-canvas").length,
              shimLogTail: (window.__usageShimLog || [])
                .slice(-8)
                .map((e) => e.cmd),
            };
            rejectP(
              new Error(
                "window switch: no refetch observed for " +
                  lbl +
                  " diag=" +
                  JSON.stringify(diag),
              ),
            );
            return;
          }
          setTimeout(poll, 10);
        };
        setTimeout(poll, 10);
      }),
    { idx: segIndex, label },
  );

const settle = () => page.waitForTimeout(300);

// ── warmup:overview → tokens → cost → overview(基线同序;tokens/cost 懒 chunk 在此加载)──
const warmupToTokens = await switchTab(TAB.tokens);
await settle();
const warmupToCost = await switchTab(TAB.cost);
await settle();
const warmupToOverview = await switchTab(TAB.overview);
await settle();

// ── 3 轮 cycle:toTokens / backOverview1 / toCost / backOverview2 ──
const cycles = [];
const cyclesRebuilt = [];
for (let i = 0; i < 3; i++) {
  const toTokens = await switchTab(TAB.tokens);
  await settle();
  const backOverview1 = await switchTab(TAB.overview);
  await settle();
  const toCost = await switchTab(TAB.cost);
  await settle();
  const backOverview2 = await switchTab(TAB.overview);
  await settle();
  cycles.push({
    toTokens: toTokens.ms,
    backOverview1: backOverview1.ms,
    toCost: toCost.ms,
    backOverview2: backOverview2.ms,
  });
  cyclesRebuilt.push({
    toTokens: toTokens.rebuilt,
    backOverview1: backOverview1.rebuilt,
    toCost: toCost.rebuilt,
    backOverview2: backOverview2.rebuilt,
  });
}
const rebuiltCount = cyclesRebuilt.reduce(
  (n, c) => n + Object.values(c).filter(Boolean).length,
  0,
);

// tokens tab 再进入留档截图(KeepAlive 命中态)
await switchTab(TAB.tokens);
await page.waitForTimeout(500);
await page.screenshot({
  path: resolve(OUT_DIR, "after-tokens-reentry-1920.png"),
});
await switchTab(TAB.overview);
await settle();

// ── 窗口切换 ×2(基线同序:本月 → 近 30 天)──
const windowSwitches = [];
const w1 = await switchWindow(SEG.this_month, "本月");
console.error("[window] 本月:", JSON.stringify(w1));
windowSwitches.push(w1);
await page.waitForTimeout(700);
const w2 = await switchWindow(SEG.last_30d, "近 30 天");
console.error("[window] 近 30 天:", JSON.stringify(w2));
windowSwitches.push(w2);
await page.waitForTimeout(700);
await page.screenshot({
  path: resolve(OUT_DIR, "after-window-last30-1920.png"),
});

// ── 内存:GC → before,20 次 overview↔tokens 快速往返,GC → after ──
const cdp = await context.newCDPSession(page);
await cdp.send("HeapProfiler.enable");
const gcAndMeasure = async () => {
  await cdp.send("HeapProfiler.collectGarbage");
  await page.waitForTimeout(200);
  return page.evaluate(() =>
    performance.memory
      ? +(performance.memory.usedJSHeapSize / 1048576).toFixed(1)
      : null,
  );
};
const beforeMB = await gcAndMeasure();
for (let i = 0; i < 20; i++) {
  await switchTab(TAB.tokens);
  await page.waitForTimeout(60);
  await switchTab(TAB.overview);
  await page.waitForTimeout(60);
}
const afterMB = await gcAndMeasure();

await page.screenshot({
  path: resolve(OUT_DIR, "after-overview-final-1920.png"),
});

// ── 结果落盘 ──
const result = {
  capturedAt: "2026-07-08",
  method:
    "vite dev http://127.0.0.1:15173/usage + Playwright + research/perf-harness/tauri-shim.js 注入(fixture 30 天数据);localStorage 预置 ccr-theme=light/ccr-flavor=clay/ccr-accent=clay + ccr-usage-fixture-variant=stale;viewport 1920x1080;计时:纯 setTimeout(10ms) 轮询;rebuilt=旧 canvas DOM 节点未存活(data-perf-id 节点标记法,与基线一致)。口径差异:基线计时终点为“首个无标记 canvas 出现”,KeepAlive 后再进入不再产生新 canvas,终点放宽为“活动 tab 出现 canvas(无论新旧)”。窗口切换结算:出现无标记 canvas → remount(基线行为);否则在 refetch(get_usage_dashboard_v2)后静默 600ms,或命中 store 30s 快照缓存(DASHBOARD_CACHE_TTL_MS,无 IPC,servedFromCache=true)时点击后静默 1500ms 结算,两种路径均核对切换前 canvas 节点是否全部存活。",
  baselineRef: "../baseline/baseline-perf.json",
  datasetAssertion: { ok: datasetOk, dataset: datasetSnapshot },
  warmup: {
    toTokens: warmupToTokens.ms,
    toCost: warmupToCost.ms,
    toOverview: warmupToOverview.ms,
    note: `tokens/cost 首次进入为冷挂载(懒 chunk),rebuilt=${warmupToTokens.rebuilt}/${warmupToCost.rebuilt};overview 首次返回 rebuilt=${warmupToOverview.rebuilt}(KeepAlive 命中)`,
  },
  tabSwitchCyclesMs: cycles,
  tabSwitchCyclesRebuilt: cyclesRebuilt,
  chartRebuiltOnEveryReentry: rebuiltCount === 12,
  chartRebuiltCount: `${rebuiltCount}/12`,
  windowSwitches,
  memoryAfter20Toggles: {
    beforeMB,
    afterMB,
    leak: beforeMB != null && afterMB != null ? afterMB - beforeMB > 8 : null,
  },
  consoleErrors,
};
writeFileSync(
  resolve(OUT_DIR, "after-perf.json"),
  JSON.stringify(result, null, 2) + "\n",
);
console.log(JSON.stringify(result, null, 2));
await browser.close();
