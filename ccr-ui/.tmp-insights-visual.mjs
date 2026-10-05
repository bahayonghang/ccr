// 一次性视觉验收脚本（frontend 子任务 implement.md 步骤 1.2 与步骤 6）。
// 注入模拟 __TAURI_INTERNALS__：首页概览沿用 design 子任务的脚本，Insights 用真实
// llmusage/usage.db 派生的快照（由 Python 预先生成到 INSIGHTS_JSON）。
import { chromium } from "playwright";
import { readFileSync, mkdirSync } from "node:fs";

const url = process.env.TARGET ?? "http://127.0.0.1:5173/";
const width = Number(process.env.W ?? 1920);
const height = Number(process.env.H ?? 1080);
const dpr = Number(process.env.DPR ?? 1);
const shotDir = process.env.SHOT_DIR;
const insightsJson = process.env.INSIGHTS_JSON;
mkdirSync(shotDir, { recursive: true });

const variant = process.env.VARIANT ?? "real";
const base = JSON.parse(readFileSync(insightsJson, "utf8"));
const insights = structuredClone(base);
if (variant === "empty") {
  insights.totals = { requests: 0, tokens: 0, sessions: 0, agents: 0, projects: 0, active_days: 0 };
  insights.first_date = null;
  insights.daily = [];
  insights.trend = [];
  insights.agents = [];
  insights.projects = [];
  insights.models = [];
  insights.hourly = insights.hourly.map(() => 0);
  insights.weekday = insights.weekday.map(() => 0);
  insights.monthly = insights.monthly.map(() => 0);
  insights.last_7_days = { requests: 0, sessions: 0, active_days: 0 };
  insights.previous_7_days = { requests: 0, sessions: 0, active_days: 0 };
  insights.current_streak = 0;
  insights.longest_streak = 0;
  insights.busiest_day = null;
}
if (variant === "sessions") {
  insights.totals = { requests: 0, tokens: 0, sessions: 3362, agents: 0, projects: 0, active_days: 0 };
  insights.first_date = null;
  insights.daily = [];
  insights.trend = [];
  insights.projects = [];
  insights.models = [];
  insights.hourly = insights.hourly.map(() => 0);
  insights.weekday = insights.weekday.map(() => 0);
  insights.monthly = insights.monthly.map(() => 0);
  insights.last_7_days = { requests: 0, sessions: 0, active_days: 0 };
  insights.previous_7_days = { requests: 0, sessions: 0, active_days: 0 };
  insights.agents = [
    { source: "claude", requests: 0, tokens: 0, sessions: 2908, unmapped: false },
    { source: "codex", requests: 0, tokens: 0, sessions: 359, unmapped: false },
    { source: "antigravity", requests: 0, tokens: 0, sessions: 95, unmapped: false },
  ];
  insights.current_streak = 0;
  insights.longest_streak = 0;
  insights.busiest_day = null;
}
if (variant === "unindexed") insights.sessions_indexed = false;
if (variant === "unmapped") {
  insights.agents = [
    ...base.agents,
    { source: "trae", requests: 120, tokens: 900000, sessions: 7, unmapped: true },
  ];
}

const mockBody = `(() => {
  const insights = ${JSON.stringify(insights)};
  const day = (offset) => {
    const d = new Date();
    d.setDate(d.getDate() - offset);
    return d.toISOString().slice(0, 10);
  };
  const zero = { sessions: 0, requests: 0, tokens: 0 };
  const series = [6, 5, 4, 3, 2, 1, 0].map((o) => ({
    date: day(o),
    claude: { ...zero },
    codex: { ...zero },
    antigravity: { ...zero },
    opencode: { ...zero },
  }));
  series[0].claude = { sessions: 0, requests: 205, tokens: 43_600_000 };
  series[0].codex = { sessions: 0, requests: 3900, tokens: 400_000_000 };
  series[1].codex = { sessions: 0, requests: 5400, tokens: 305_300_000 };
  const readiness = {
    state: "ready",
    next_action: null,
    detail: "",
    has_live_sources: true,
    has_missing_sources: false,
    has_deleted_sources: false,
    active_usage_import: false,
    active_session_index: false,
    recent_completed_at: new Date().toISOString(),
  };
  const freshness = {
    state: "fresh",
    latest_completed_at: new Date().toISOString(),
    age_seconds: 60,
    stale_after_seconds: 86400,
  };
  const overview = {
    summary: {
      total_sessions: 3362,
      total_requests: 9505,
      total_tokens: 1_000_000_000,
      active_days: 2,
      platforms: 2,
    },
    by_platform: {
      claude: { sessions: 0, requests: 205, tokens: 43_600_000 },
      codex: { sessions: 0, requests: 9300, tokens: 705_300_000 },
    },
    series,
    bootstrap: {
      usage_import_attempted: false,
      usage_imported_records: 0,
      session_reindex_attempted: false,
      indexed_sessions: 0,
      usage_job_id: null,
      session_job_id: null,
      needs_usage_import: false,
      needs_session_index: false,
      is_warm: true,
    },
    archive: {
      archive_root: "",
      live_sources: 2,
      missing_sources: 0,
      deleted_sources: 0,
      archived_sessions: 10,
      recent_completed_at: new Date().toISOString(),
      history_completed_at: null,
      source_health: [],
      freshness,
      readiness,
    },
    snapshot: {
      generated_at: new Date().toISOString(),
      platform_scope: "all",
      start_date: day(6),
      end_date: day(0),
      cache_ttl_seconds: 30,
      freshness,
      readiness,
      source_health: [],
      drilldown: {
        dimensions: [],
        supports_logs: true,
        supports_projects: true,
        supports_sessions: true,
      },
    },
    empty_reason: null,
    last_updated: new Date().toISOString(),
  };
  const responses = {
    get_home_usage_overview_v2: overview,
    get_home_insights: insights,
    get_system_info: {
      hostname: "mock",
      os: "windows",
      os_name: "Windows",
      os_version: "11",
      kernel_version: "",
      arch: "x86_64",
      cpu_brand: "",
      cpu_cores: 8,
      cpu_count: 16,
      cpu_usage: 1,
      total_memory_gb: 32,
      used_memory_gb: 8,
      memory_usage_percent: 25,
      total_memory_mb: 32768,
      total_swap_gb: 0,
      used_swap_gb: 0,
      uptime_seconds: 100,
      ccr_version: "7.3.0",
    },
    get_cli_versions: { versions: {}, entries: [], mode: "fast", timeout_ms: 3500, parallelism: 4 },
  };
  let cb = 1;
  window.__MOCK_CALLS__ = [];
  window.__TAURI_INTERNALS__ = {
    metadata: {
      currentWindow: { label: "main" },
      currentWebview: { label: "main", windowLabel: "main" },
    },
    transformCallback: () => cb++,
    unregisterCallback: () => {},
    convertFileSrc: (p) => p,
    invoke: async (cmd) => {
      window.__MOCK_CALLS__.push(cmd);
      if (cmd in responses) return structuredClone(responses[cmd]);
      if (cmd.startsWith("plugin:event")) return cb++;
      return null;
    },
  };
})();`;

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width, height }, deviceScaleFactor: dpr });
const errors = [];
page.on("pageerror", (e) => errors.push(String(e)));
if (variant !== "web-preview") await page.addInitScript({ content: mockBody });
await page.goto(url, { waitUntil: "networkidle" });
await page.waitForSelector(".dashboard-usage", { timeout: 30_000 });
await page.waitForTimeout(3000);

// 步骤 1.2：挂载 Insights 后重测首屏用量图高度。
const chart = await page.evaluate(() => {
  const el = document.querySelector(".dashboard-usage__chart");
  if (!el) return null;
  const r = el.getBoundingClientRect();
  const cs = getComputedStyle(el);
  return { height: Math.round(r.height), cssHeight: cs.height, overflow: cs.overflow };
});

try {
  await page.waitForSelector(
    variant === "web-preview" ? '[data-dashboard-insights] [data-state="web-preview"]' : ".insights-totals__value",
    { timeout: 12_000 },
  );
} catch (error) {
  const debug = await page.evaluate(() => ({
    hasTauri: typeof window.__TAURI_INTERNALS__,
    hasCalls: Array.isArray(window.__MOCK_CALLS__),
    calls: Array.isArray(window.__MOCK_CALLS__)
      ? window.__MOCK_CALLS__.filter((c) => !c.startsWith("plugin:"))
      : null,
    state: document.querySelector("[data-dashboard-insights]")?.getAttribute("data-state"),
    inner: document.querySelector("[data-dashboard-insights]")?.innerText?.slice(0, 800),
    panels: [...document.querySelectorAll("[data-dashboard-insights] .insights-panel")].map((el) => ({
      cls: el.className,
      state: el.dataset.state,
      text: el.innerText.slice(0, 200),
    })),
  }));
  process.stdout.write(`${JSON.stringify({ debug }, null, 2)}\n`);
  await page.screenshot({ path: `${shotDir}/debug.png`, fullPage: true });
  throw error;
}
await page.waitForTimeout(1200);

const firstViewport = await page.evaluate(() => {
  const insights = document.querySelector("[data-dashboard-insights]");
  const status = document.querySelector(".dashboard-statusbar, .dashboard-status-bar");
  const r = insights?.getBoundingClientRect();
  return {
    insightsTop: r ? Math.round(r.top + window.scrollY) : null,
    insightsHeight: r ? Math.round(r.height) : null,
    viewportHeight: window.innerHeight,
    insightsBelowFirstScreen: r ? r.top >= window.innerHeight : null,
    statusBarHeight: status ? Math.round(status.getBoundingClientRect().height) : null,
    docHeight: document.documentElement.scrollHeight,
  };
});

const blocks = [
  [".insights-totals-wrap", "totals"],
  [".insights-heatmap", "heatmap"],
  [".insights-compare", "week-compare"],
  [".insights-trend", "agent-trend"],
  [".insights-distribution", "distribution"],
  [".insights-leaderboard", "leaderboard"],
];
const measured = [];
for (const [selector, name] of blocks) {
  const el = page.locator(selector).first();
  if ((await el.count()) === 0) {
    measured.push({ name, missing: true });
    continue;
  }
  const box = await el.boundingBox();
  await el.scrollIntoViewIfNeeded();
  await page.waitForTimeout(250);
  await el.screenshot({ path: `${shotDir}/dark-${name}.png` });
  measured.push({ name, width: Math.round(box.width), height: Math.round(box.height) });
}

// 亮色主题下再截一次整块，用于对照 DESIGN.md 的两套主题 token。
await page.evaluate(() => {
  document.documentElement.setAttribute("data-theme", "light");
  document.documentElement.classList.remove("dark");
});
await page.waitForTimeout(600);
for (const [selector, name] of blocks) {
  const el = page.locator(selector).first();
  if ((await el.count()) === 0) continue;
  await el.scrollIntoViewIfNeeded();
  await page.waitForTimeout(200);
  await el.screenshot({ path: `${shotDir}/light-${name}.png` });
}
await page.evaluate(() => {
  document.documentElement.setAttribute("data-theme", "dark");
  document.documentElement.classList.add("dark");
});
await page.locator("[data-dashboard-insights]").first().scrollIntoViewIfNeeded();
await page.waitForTimeout(400);
await page.screenshot({ path: `${shotDir}/insights-dark-viewport.png`, fullPage: false });
await page.screenshot({ path: `${shotDir}/insights-dark-full.png`, fullPage: true });
let checks = null;

// 分布面板：网格线相对柱轨的四分位 + 视图切换后说明行。
const distribution = page.locator(".insights-distribution").first();
if ((await distribution.count()) > 0) {
  await distribution.scrollIntoViewIfNeeded();
  const grid = await page.evaluate(() => {
    const plot = document.querySelector(".insights-distribution__plot");
    if (!plot) return null;
    const base = plot.getBoundingClientRect();
    return {
      plotHeight: Math.round(base.height),
      lines: [...plot.querySelectorAll(".insights-gridline")].map((el) => {
        const r = el.getBoundingClientRect();
        return { fromBottomPct: Math.round(((base.bottom - r.top) / base.height) * 1000) / 10 };
      }),
    };
  });
  const weekday = distribution.locator('[data-value="weekday"]');
  await weekday.click();
  await page.waitForTimeout(250);
  const captionAfterSwitch = await distribution
    .locator('[data-testid="insights-distribution-caption"]')
    .count();
  await distribution.locator('[data-value="hour"]').click();
  await page.waitForTimeout(250);
  const captionBack = await distribution
    .locator('[data-testid="insights-distribution-caption"]')
    .count();
  checks = { grid, captionAfterSwitch, captionBack };
}

// 首屏构图：滚回顶部，量首屏最后一个签名区块与 Insights 头部的位置关系。
await page.evaluate(() => {
  document.querySelectorAll("*").forEach((el) => {
    if (el.scrollHeight > el.clientHeight + 4) el.scrollTop = 0;
  });
  window.scrollTo(0, 0);
});
await page.waitForTimeout(500);
const firstScreen = await page.evaluate(() => {
  const box = (sel) => {
    const el = document.querySelector(sel);
    if (!el) return null;
    const r = el.getBoundingClientRect();
    return { top: Math.round(r.top), bottom: Math.round(r.bottom), height: Math.round(r.height) };
  };
  return {
    lower: box(".dashboard-lower"),
    statusBar: box(".dashboard-statusbar, .dashboard-status-bar"),
    insights: box("[data-dashboard-insights]"),
    insightsHeader: box(".dashboard-insights__header"),
    totals: box(".insights-totals-wrap"),
  };
});
await page.screenshot({ path: `${shotDir}/first-screen-dark.png`, fullPage: false });
await page.evaluate(() => {
  document.documentElement.setAttribute("data-theme", "light");
  document.documentElement.classList.remove("dark");
});
await page.waitForTimeout(400);
await page.screenshot({ path: `${shotDir}/first-screen-light.png`, fullPage: false });

// 整块截图（含所有空态/异常态变体）。
await page.evaluate(() => {
  document.documentElement.setAttribute("data-theme", "dark");
  document.documentElement.classList.add("dark");
  document.querySelectorAll("*").forEach((el) => {
    if (el.scrollHeight > el.clientHeight + 4) el.scrollTop = 0;
  });
});
await page.waitForTimeout(400);
const section = page.locator("[data-dashboard-insights]").first();
await section.scrollIntoViewIfNeeded();
await page.waitForTimeout(300);
await section.screenshot({ path: `${shotDir}/variant-${variant}-section.png` });

const text = await page.evaluate(() => {
  const el = document.querySelector("[data-dashboard-insights]");
  return el ? el.innerText.replace(/\n{2,}/g, "\n").slice(0, 2500) : null;
});

process.stdout.write(
  JSON.stringify(
    {
      variant,
      chart,
      firstViewport,
      firstScreen,
      checks,
      errors: errors.slice(0, 5),
      insightsText: text,
    },
    null,
    2,
  ) + "\n",
);
await browser.close();
