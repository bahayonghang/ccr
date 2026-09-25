// 复现首页用量图越界：注入模拟 __TAURI_INTERNALS__，按截图数据返回首页概览，测量图表尺寸。
import { chromium } from "playwright";

const url = process.env.TARGET ?? "http://127.0.0.1:5173/";
const width = Number(process.env.W ?? 1920);
const height = Number(process.env.H ?? 1080);
const dpr = Number(process.env.DPR ?? 1);
const shot = process.env.SHOT;

const mockScript = () => {
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
      total_sessions: 0,
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
    get_cli_versions: {
      versions: {},
      entries: [
        {
          platform: "claude",
          installed: true,
          version: "2.1.281",
          status: "ok",
          elapsed_ms: 10,
        },
        {
          platform: "codex",
          installed: true,
          version: "0.156.1",
          status: "ok",
          elapsed_ms: 10,
        },
      ],
      mode: "fast",
      timeout_ms: 3500,
      parallelism: 4,
    },
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
};

const browser = await chromium.launch();
const page = await browser.newPage({
  viewport: { width, height },
  deviceScaleFactor: dpr,
});
const errors = [];
page.on("pageerror", (e) => errors.push(String(e)));
await page.addInitScript(mockScript);
await page.goto(url, { waitUntil: "networkidle" });
await page.waitForSelector(".dashboard-usage", { timeout: 30_000 });
await page.waitForTimeout(3000);
const report = await page.evaluate(() => {
  const box = (sel) => {
    const el = document.querySelector(sel);
    if (!el) return null;
    const r = el.getBoundingClientRect();
    const cs = getComputedStyle(el);
    return {
      top: Math.round(r.top),
      height: Math.round(r.height),
      width: Math.round(r.width),
      cssHeight: cs.height,
      overflow: cs.overflow,
      display: cs.display,
    };
  };
  const stacks = [...document.querySelectorAll(".dashboard-usage-stack")].map(
    (el) => {
      const r = el.getBoundingClientRect();
      return {
        h: Math.round(r.height),
        var: el.style.getPropertyValue("--stack-height"),
        anim: getComputedStyle(el).transform,
      };
    },
  );
  const sheets = [...document.styleSheets].some((s) => {
    try {
      return [...s.cssRules].some(
        (r) => r.selectorText === ".dashboard-usage__chart",
      );
    } catch {
      return false;
    }
  });
  return {
    rootFont: getComputedStyle(document.documentElement).fontSize,
    usage: box(".dashboard-usage"),
    chart: box(".dashboard-usage__chart"),
    lower: box(".dashboard-lower"),
    rail: box(".dashboard-rail"),
    state: document.querySelector(".dashboard-usage")?.dataset.state,
    count: document.querySelector(".dashboard-usage")?.dataset.count,
    stacks,
    chartRuleLoaded: sheets,
    calls: window.__MOCK_CALLS__
      .filter((c) => !c.startsWith("plugin:"))
      .slice(0, 30),
  };
});
console.log(
  JSON.stringify(
    { viewport: { width, height, dpr }, report, errors: errors.slice(0, 5) },
    null,
    2,
  ),
);
if (shot) await page.screenshot({ path: shot, fullPage: false });
await browser.close();
