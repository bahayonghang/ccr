// 一次性桌面端探针：通过 CDP 接入 Tauri 的 WebView2，量真实 DOM（首屏图表高度、
// Insights 区块位置、周趋势柱高）。只读测量 + 截图，不改代码。
import { chromium } from "playwright";

const browser = await chromium.connectOverCDP("http://127.0.0.1:9222");
const contexts = browser.contexts();
let page = null;
for (const context of contexts) {
  for (const candidate of context.pages()) {
    const url = candidate.url();
    if (url.startsWith("http://127.0.0.1:15173") && !url.includes("/tray/")) {
      page = candidate;
      break;
    }
  }
  if (page) break;
}
if (!page) {
  for (const context of contexts) {
    for (const candidate of context.pages()) process.stderr.write(`target: ${candidate.url()}\n`);
  }
  throw new Error("main window target not found");
}
process.stderr.write(`using ${page.url()}\n`);
await page.waitForSelector(".dashboard-usage", { timeout: 60_000 });
await page.waitForTimeout(4000);

const first = await page.evaluate(() => {
  const box = (sel) => {
    const el = document.querySelector(sel);
    if (!el) return null;
    const r = el.getBoundingClientRect();
    return { top: Math.round(r.top), bottom: Math.round(r.bottom), height: Math.round(r.height) };
  };
  const chart = document.querySelector(".dashboard-usage__chart");
  return {
    innerWidth: window.innerWidth,
    innerHeight: window.innerHeight,
    dpr: window.devicePixelRatio,
    chartBox: box(".dashboard-usage__chart"),
    chartCssHeight: chart ? getComputedStyle(chart).height : null,
    chartRule: chart ? getComputedStyle(chart).getPropertyValue("height") : null,
    lower: box(".dashboard-lower"),
    statusBar: box(".dashboard-statusbar, .dashboard-status-bar"),
    insights: box("[data-dashboard-insights]"),
    insightsHeader: box(".dashboard-insights__header"),
  };
});
await page.screenshot({ path: "C:/Users/lyh/AppData/Local/Temp/cdp-first-screen.png" });

// 滚到 Insights，量柱高与面板高。
await page.evaluate(() => {
  document.querySelectorAll("*").forEach((el) => {
    if (el.scrollHeight > el.clientHeight + 4) el.scrollTop = 4000;
  });
});
await page.waitForTimeout(2500);
const insights = await page.evaluate(() => {
  const box = (sel) => {
    const el = document.querySelector(sel);
    if (!el) return null;
    const r = el.getBoundingClientRect();
    return { top: Math.round(r.top), height: Math.round(r.height), width: Math.round(r.width) };
  };
  const bars = [...document.querySelectorAll(".insights-trend__bar")].map((el) => ({
    h: Math.round(el.getBoundingClientRect().height),
    segs: el.children.length,
  }));
  const segments = [...document.querySelectorAll(".insights-trend__segment")].slice(0, 8).map((el) => ({
    h: Math.round(el.getBoundingClientRect().height),
    style: el.getAttribute("style"),
  }));
  const track = document.querySelector(".insights-trend__chart");
  return {
    totals: box(".insights-totals-wrap"),
    heatmap: box(".insights-heatmap"),
    compare: box(".insights-compare"),
    trend: box(".insights-trend"),
    distribution: box(".insights-distribution"),
    leaderboard: box(".insights-leaderboard"),
    chartBox: box(".insights-trend__chart"),
    chartStyleHeight: track ? getComputedStyle(track).height : null,
    barCount: bars.length,
    barHeights: bars.map((b) => b.h),
    segmentSample: segments,
    text: document.querySelector("[data-dashboard-insights]")?.innerText.replace(/\n{2,}/g, "\n").slice(0, 900),
  };
});
await page.screenshot({ path: "C:/Users/lyh/AppData/Local/Temp/cdp-insights.png" });
await page.locator(".insights-trend").first().screenshot({ path: "C:/Users/lyh/AppData/Local/Temp/cdp-trend.png" }).catch(() => {});

process.stdout.write(`${JSON.stringify({ first, insights }, null, 2)}\n`);
await browser.close();
