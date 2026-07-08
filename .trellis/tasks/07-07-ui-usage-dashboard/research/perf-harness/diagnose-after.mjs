/*
 * diagnose-after.mjs —— 第 9 项复测发现问题后的定位 probe(研究产物)
 * 问题 1:toTokens/toCost 再进入 rebuilt=true(async 包装 tab 疑似未被 KeepAlive 缓存)
 * 问题 2:窗口切换 37ms 内出现新 canvas(refetch 前),U2 未消除
 * 本脚本回答:① tokens 根 DOM/组件实例是否跨往返存活;② 窗口切换时具体哪个图表 remount。
 */
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const REPO = resolve(HERE, "../../../../..");
const requireFromUi = createRequire(resolve(REPO, "ccr-ui/package.json"));
const { chromium } = requireFromUi("playwright");

const SHIM = readFileSync(resolve(HERE, "tauri-shim.js"), "utf8");
const browser = await chromium.launch();
const context = await browser.newContext({
  viewport: { width: 1920, height: 1080 },
});
const page = await context.newPage();
await page.addInitScript(() => {
  localStorage.setItem("ccr-theme", "light");
  localStorage.setItem("ccr-flavor", "clay");
  localStorage.setItem("ccr-accent", "clay");
  localStorage.setItem("ccr-usage-fixture-variant", "stale");
});
await page.addInitScript(SHIM);
await page.goto("http://127.0.0.1:15173/usage");
await page.waitForSelector(".apexcharts-canvas", { timeout: 15000 });
await page.waitForTimeout(800);

await page.evaluate(() => {
  window.__probe = {
    n: 0,
    tag() {
      document.querySelectorAll(".apexcharts-canvas").forEach((el) => {
        if (!el.dataset.pid) el.dataset.pid = String(++this.n);
      });
    },
    // 每个 canvas 的身份 + 可定位的父级上下文(向上找带 class 的宿主组件)
    inventory() {
      return [...document.querySelectorAll(".apexcharts-canvas")].map((el) => {
        let p = el.parentElement;
        const chain = [];
        while (p && chain.length < 4) {
          if (p.className && typeof p.className === "string") {
            const c = p.className
              .split(" ")
              .filter((x) => x && !x.startsWith("apexcharts"))
              .slice(0, 2)
              .join(".");
            if (c) chain.push(c);
          }
          p = p.parentElement;
        }
        return { pid: el.dataset.pid ?? null, host: chain.join(" < ") };
      });
    },
  };
  window.__probe.tag();
});

const clickTab = (idx) =>
  page.evaluate((i) => {
    document.querySelectorAll(".usage-tabs .usage-tab")[i].click();
  }, idx);
const clickSeg = (idx) =>
  page.evaluate((i) => {
    document.querySelectorAll(".usage-dashboard-toolbar__segment")[i].click();
  }, idx);

// ── ① tokens 实例身份:进入 tokens,给 tab 根节点与 canvas 打标,往返后看是否同一 DOM ──
await clickTab(1);
await page.waitForTimeout(900);
const tokensBefore = await page.evaluate(() => {
  window.__probe.tag();
  // usage-content 下当前渲染的 tab 根元素
  const root = document.querySelector(
    ".usage-content > *:not(.glass-panel), .usage-content > *",
  );
  if (root) root.dataset.probeRoot = "tokens-v1";
  return {
    rootClass: root?.className ?? null,
    canvases: window.__probe.inventory(),
  };
});
await clickTab(0);
await page.waitForTimeout(500);
await clickTab(1);
await page.waitForTimeout(900);
const tokensAfter = await page.evaluate(() => {
  const root = document.querySelector(".usage-content > *");
  const inv = window.__probe.inventory();
  window.__probe.tag();
  return {
    rootMarkerSurvived: root?.dataset.probeRoot === "tokens-v1",
    canvases: inv,
  };
});

// ── ② 窗口切换:回 overview,打标后切"本月",1.5s 后盘点新旧 canvas ──
await clickTab(0);
await page.waitForTimeout(700);
const overviewBefore = await page.evaluate(() => {
  window.__probe.tag();
  return window.__probe.inventory();
});
await clickSeg(2); // 本月
await page.waitForTimeout(1500);
const afterThisMonth = await page.evaluate(() => {
  const inv = window.__probe.inventory();
  window.__probe.tag();
  return inv;
});

const result = {
  tokensProbe: {
    before: tokensBefore,
    after: tokensAfter,
    verdict: tokensAfter.rootMarkerSurvived
      ? "tokens 根 DOM 存活(KeepAlive 命中)"
      : "tokens 根 DOM 未存活(KeepAlive 未缓存 → 整树重建)",
  },
  windowProbe: {
    overviewBefore,
    afterThisMonth,
    gone: overviewBefore
      .filter((b) => !afterThisMonth.some((a) => a.pid === b.pid))
      .map((b) => `${b.pid}:${b.host}`),
    fresh: afterThisMonth.filter((a) => a.pid === null).map((a) => a.host),
    freshTagged: afterThisMonth.filter(
      (a) => a.pid && !overviewBefore.some((b) => b.pid === a.pid),
    ),
  },
};
console.log(JSON.stringify(result, null, 2));
await browser.close();
