/*
 * tauri-shim.js —— ccr-ui Usage Dashboard 浏览器测试桩（研究产物，非产品代码）
 *
 * 用途：通过 Playwright `page.addInitScript()` 在应用 bundle 之前注入，伪造 Tauri v2 IPC 层，
 * 让 Usage Dashboard 页面能在纯浏览器（vite dev，无 Tauri）里完整跑起来。
 *
 * 契约来源（@tauri-apps/api v2.11.0）：
 *   - core.js:      invoke → window.__TAURI_INTERNALS__.invoke(cmd, args, options)
 *                   transformCallback → window.__TAURI_INTERNALS__.transformCallback(cb, once)
 *                   Channel.cleanupCallback → window.__TAURI_INTERNALS__.unregisterCallback(id)
 *   - event.js:     listen → invoke('plugin:event|listen', { event, target, handler:<cbId> })
 *                   _unlisten → window.__TAURI_EVENT_PLUGIN_INTERNALS__.unregisterListener(event, id)
 *                               + invoke('plugin:event|unlisten', { event, eventId })
 *   - window.js:    getCurrentWindow → new Window(window.__TAURI_INTERNALS__.metadata.currentWindow.label)
 *
 * isTauriRuntime()（src/utils/tauriRuntime.ts）检测 `'__TAURI__' in window || '__TAURI_INTERNALS__' in window`，
 * 两者都置上；runtimeUnavailable = !isTauriRuntime() 因此为 false，仪表盘正常渲染而非“运行时不可用”面板。
 *
 * 变体：localStorage['ccr-usage-fixture-variant'] = 'healthy'（默认）| 'stale'
 */
(function () {
  "use strict";

  // ══════════════════════════════════════════════════════════════════
  // 调试日志：每次 invoke 都 push {cmd, args}，方便排查缺失的桩
  // ══════════════════════════════════════════════════════════════════
  if (!window.__usageShimLog) {
    window.__usageShimLog = [];
  }

  var VARIANT = "healthy";
  try {
    var stored = window.localStorage.getItem("ccr-usage-fixture-variant");
    if (stored === "stale" || stored === "healthy") {
      VARIANT = stored;
    }
  } catch (e) {
    // localStorage 不可用时退回 healthy
  }
  window.__usageShimVariant = VARIANT;

  // ══════════════════════════════════════════════════════════════════
  // 确定性日期工具（全部以 UTC 计算，避免时区漂移）
  // ══════════════════════════════════════════════════════════════════
  var END_YEAR = 2026;
  var END_MONTH = 6; // 0-indexed：6 = 七月
  var END_DAY = 7; // 基准结束日 2026-07-07
  var END_UTC = Date.UTC(END_YEAR, END_MONTH, END_DAY);
  var DAY_MS = 86400000;

  // offsetFromEnd=0 → 2026-07-07，正数表示更早的日期，返回 'YYYY-MM-DD'
  function dayString(offsetFromEnd) {
    var d = new Date(END_UTC - offsetFromEnd * DAY_MS);
    var y = d.getUTCFullYear();
    var m = String(d.getUTCMonth() + 1).padStart(2, "0");
    var day = String(d.getUTCDate()).padStart(2, "0");
    return y + "-" + m + "-" + day;
  }

  // 诊断/快照用的完成与生成时间戳（healthy=新鲜，stale=约 4 天前）
  var FRESH_COMPLETED = "2026-07-07T09:30:00Z";
  var STALE_COMPLETED = "2026-07-03T09:30:00Z";
  var FRESH_GENERATED = "2026-07-07T10:00:00Z";
  var STALE_GENERATED = "2026-07-03T10:00:00Z";
  var RANGE_START = dayString(29); // 30 天窗口起点
  var RANGE_END = dayString(0);

  // ══════════════════════════════════════════════════════════════════
  // Fixture：Summary（UsageSummaryDto）
  // 头条 total_tokens=12_527_400_000（后续格式化工作可验证 "12.53B"），
  // total_cost_usd=26114.04，total_requests=48000（与 model_stats 合计一致）
  // ══════════════════════════════════════════════════════════════════
  var SUMMARY = {
    total_requests: 48000,
    total_tokens: 12527400000,
    total_input_tokens: 1879110000, // ≈ 15% 输入
    total_output_tokens: 751644000, // ≈ 6% 输出
    total_cache_read_tokens: 9395550000, // ≈ 75% 缓存读取
    total_cost_usd: 26114.04,
    cache_efficiency: 0.86,
  };

  // ══════════════════════════════════════════════════════════════════
  // Fixture：Trends（DailyTrendDto[]）—— 30 个连续日，止于 2026-07-07
  // 全部由 index 取模生成，确定性；趋势/tokens/cost 三张图都能出线
  // ══════════════════════════════════════════════════════════════════
  function buildTrends() {
    var out = [];
    for (var i = 0; i < 30; i++) {
      var input = 42000000 + ((i * 7000000) % 55000000);
      var output = 15000000 + ((i * 3000000) % 22000000);
      var reasoning = Math.round(output * 0.35);
      var cacheRead = 220000000 + ((i * 11000000) % 130000000);
      var cacheCreate = 12000000 + ((i * 2000000) % 18000000);
      var total = input + output + cacheRead + cacheCreate;
      var cost = 600 + ((i * 37) % 420);
      out.push({
        date: dayString(29 - i),
        request_count: 1200 + ((i * 53) % 700),
        total_tokens: total,
        input_tokens: input,
        output_tokens: output,
        reasoning_output_tokens: reasoning,
        cache_read_tokens: cacheRead,
        cache_creation_tokens: cacheCreate,
        cost_usd: Number(cost.toFixed(2)),
      });
    }
    return out;
  }
  var TRENDS = buildTrends();

  // ══════════════════════════════════════════════════════════════════
  // Fixture：model_stats（ModelStatDto[]）—— 5 个模型
  // 合计 cost=26114.04 / tokens=12_527_400_000 / requests=48000，与 SUMMARY 完全对齐
  // ══════════════════════════════════════════════════════════════════
  function modelStat(model, req, totalTokens, totalCost, pricingRate) {
    var input = Math.round(totalTokens * 0.15);
    var output = Math.round(totalTokens * 0.06);
    var cacheCreate = Math.round(totalTokens * 0.04);
    var cacheRead = totalTokens - input - output - cacheCreate;
    var costWithout = Number((totalCost * 1.9).toFixed(2));
    return {
      model: model,
      request_count: req,
      total_tokens: totalTokens,
      total_cost: totalCost,
      input_tokens: input,
      output_tokens: output,
      cache_read_tokens: cacheRead,
      cache_creation_tokens: cacheCreate,
      cost_with_cache: totalCost,
      cost_without_cache: costWithout,
      cache_savings: Number((costWithout - totalCost).toFixed(2)),
      pricing_status: "resolved",
      pricing_source: "builtin",
      pricing_rate: pricingRate,
    };
  }
  var MODEL_STATS = [
    modelStat(
      "claude-sonnet-4-5",
      19200,
      4900000000,
      9800.5,
      "$3.00 / $15.00 per M",
    ),
    modelStat(
      "claude-opus-4-8",
      7300,
      2600000000,
      8600.2,
      "$15.00 / $75.00 per M",
    ),
    modelStat("gpt-5", 9100, 2300000000, 4200.1, "$1.25 / $10.00 per M"),
    modelStat(
      "gemini-2.5-pro",
      8000,
      1800000000,
      2100.44,
      "$1.25 / $10.00 per M",
    ),
    modelStat(
      "claude-haiku-4-5",
      4400,
      927400000,
      1412.8,
      "$1.00 / $5.00 per M",
    ),
  ];

  // ══════════════════════════════════════════════════════════════════
  // Fixture：project_stats（ProjectStatDto[]）—— 6 个项目
  // ══════════════════════════════════════════════════════════════════
  var PROJECT_STATS = [
    {
      project_path: "D:\\Documents\\Code\\Github\\ccr",
      request_count: 15200,
      total_tokens: 4200000000,
      total_cost: 9120.44,
    },
    {
      project_path: "D:\\Documents\\Code\\Github\\ccr-ui",
      request_count: 9800,
      total_tokens: 2600000000,
      total_cost: 5340.12,
    },
    {
      project_path: "D:\\Documents\\Code\\Github\\llmusage",
      request_count: 7300,
      total_tokens: 1900000000,
      total_cost: 4210.9,
    },
    {
      project_path: "D:\\Documents\\Code\\Work\\api-gateway",
      request_count: 6100,
      total_tokens: 1600000000,
      total_cost: 3980.6,
    },
    {
      project_path: "/home/lyh/projects/agent-runtime",
      request_count: 5400,
      total_tokens: 1400000000,
      total_cost: 2260.5,
    },
    {
      project_path: "(no project)",
      request_count: 4200,
      total_tokens: 827400000,
      total_cost: 1200.48,
    },
  ];

  // ══════════════════════════════════════════════════════════════════
  // Fixture：provider_stats（ProviderBreakdownDto[]）—— 3 个 provider
  // ══════════════════════════════════════════════════════════════════
  function providerStat(name, req, totalTokens, costWith) {
    var input = Math.round(totalTokens * 0.15);
    var output = Math.round(totalTokens * 0.06);
    var reasoning = Math.round(output * 0.35);
    var cacheCreate = Math.round(totalTokens * 0.04);
    var cacheRead = totalTokens - input - output - cacheCreate;
    return {
      provider: name,
      request_count: req,
      input_tokens: input,
      cache_read_tokens: cacheRead,
      cache_creation_tokens: cacheCreate,
      output_tokens: output,
      reasoning_output_tokens: reasoning,
      total_tokens: totalTokens,
      cost_with_cache_usd: costWith,
      cost_without_cache_usd: Number((costWith * 1.9).toFixed(2)),
    };
  }
  var PROVIDER_STATS = [
    providerStat("anthropic", 30900, 8427400000, 19813.5),
    providerStat("openai", 9100, 2300000000, 4200.1),
    providerStat("google", 8000, 1800000000, 2100.44),
  ];

  // ══════════════════════════════════════════════════════════════════
  // Fixture：source_stats（SourceBreakdownDto[]）—— 3 个来源 claude/codex/gemini
  // ══════════════════════════════════════════════════════════════════
  var SOURCE_STATS = [
    {
      source: "claude",
      event_count: 30900,
      total_tokens: 8427400000,
      total_cost: 19813.5,
      active_days: 178,
      share_tokens: 0.6727,
      share_cost: 0.7587,
    },
    {
      source: "codex",
      event_count: 9100,
      total_tokens: 2300000000,
      total_cost: 4200.1,
      active_days: 140,
      share_tokens: 0.1836,
      share_cost: 0.1608,
    },
    {
      source: "gemini",
      event_count: 8000,
      total_tokens: 1800000000,
      total_cost: 2100.44,
      active_days: 96,
      share_tokens: 0.1437,
      share_cost: 0.0804,
    },
  ];

  // ══════════════════════════════════════════════════════════════════
  // Fixture：诊断投影（archive + snapshot）—— staleness/degraded 呈现的唯一驱动源
  //
  // opsCockpit（views/usage/usageOpsCockpit.ts）读取优先级：
  //   readiness = snapshot.readiness ?? archive.readiness
  //   freshness = snapshot.freshness ?? archive.freshness
  //   source_health = snapshot.source_health ?? archive.source_health
  //   state = (importing || loading) ? 'syncing' : readiness.state ?? 'empty'
  // 因此 snapshot.* 优先生效；两侧保持同一变体一致。
  //
  //   - 顶部座舱横幅 = readiness.state（stale 变体设 'stale' → 触发“Usage data is stale”警告）
  //   - Freshness 健康项 = freshness.state / age_seconds / latest_completed_at
  //   - Source health 健康项 L/M/D = archive.live_sources / missing_sources / deleted_sources
  //   - Source 徽章色 = 每个 source_health[].state（'degraded'→warning，'missing'→danger）
  //   - Snapshot cache 健康项 = snapshot.generated_at
  //
  // 前端不会用 Date.now() 重新计算 staleness（formatDateTime 只做 toLocaleString，
  // formatAge 只格式化给定 age_seconds），故时间戳可硬编码、完全确定性。
  // ══════════════════════════════════════════════════════════════════
  function freshnessFresh() {
    return {
      state: "fresh",
      latest_completed_at: FRESH_COMPLETED,
      age_seconds: 1800,
      stale_after_seconds: 86400,
    };
  }
  function freshnessStale() {
    return {
      state: "stale",
      latest_completed_at: STALE_COMPLETED,
      age_seconds: 345600,
      stale_after_seconds: 86400,
    };
  }
  function freshnessMissing() {
    return {
      state: "missing",
      latest_completed_at: null,
      age_seconds: null,
      stale_after_seconds: 86400,
    };
  }

  function sourceHealth(
    source,
    state,
    live,
    missing,
    deleted,
    completed,
    fresh,
  ) {
    return {
      source: source,
      state: state,
      live_sources: live,
      missing_sources: missing,
      deleted_sources: deleted,
      recent_completed_at: completed,
      history_completed_at: completed,
      freshness: fresh,
    };
  }

  var DRILLDOWN = {
    dimensions: ["platform", "model", "project", "source"],
    supports_logs: true,
    supports_projects: true,
    supports_sessions: true,
  };

  function buildDiagnostics(variant) {
    var isStale = variant === "stale";
    var freshness = isStale ? freshnessStale() : freshnessFresh();
    var readiness = isStale
      ? {
          state: "stale",
          next_action: "refresh_usage",
          detail: "Local usage archive is older than the freshness window.",
          has_live_sources: true,
          has_missing_sources: true,
          has_deleted_sources: false,
          active_usage_import: false,
          active_session_index: false,
          recent_completed_at: STALE_COMPLETED,
        }
      : {
          state: "ready",
          next_action: null,
          detail: "The local read model is fresh enough for cost decisions.",
          has_live_sources: true,
          has_missing_sources: true,
          has_deleted_sources: false,
          active_usage_import: false,
          active_session_index: false,
          recent_completed_at: FRESH_COMPLETED,
        };

    // 每来源健康：healthy 全部 live；stale 变体让 codex=degraded、gemini=missing 以呈现降级徽章
    var sources = isStale
      ? [
          sourceHealth(
            "claude",
            "live",
            1200,
            900,
            0,
            STALE_COMPLETED,
            freshnessStale(),
          ),
          sourceHealth(
            "codex",
            "degraded",
            640,
            1500,
            0,
            STALE_COMPLETED,
            freshnessStale(),
          ),
          sourceHealth(
            "gemini",
            "missing",
            0,
            429,
            0,
            null,
            freshnessMissing(),
          ),
        ]
      : [
          sourceHealth(
            "claude",
            "live",
            1200,
            900,
            0,
            FRESH_COMPLETED,
            freshnessFresh(),
          ),
          sourceHealth(
            "codex",
            "live",
            640,
            1500,
            0,
            FRESH_COMPLETED,
            freshnessFresh(),
          ),
          sourceHealth(
            "gemini",
            "live",
            213,
            429,
            0,
            FRESH_COMPLETED,
            freshnessFresh(),
          ),
        ];

    // 归档级 L/M/D 计数两变体一致：在档 2053 / 缺失 2829 / 已删 0
    var archive = {
      archive_root: "D:\\Users\\lyh\\.local\\share\\llmusage\\archive",
      live_sources: 2053,
      missing_sources: 2829,
      deleted_sources: 0,
      archived_sessions: 2053,
      recent_completed_at: isStale ? STALE_COMPLETED : FRESH_COMPLETED,
      history_completed_at: isStale ? STALE_COMPLETED : FRESH_COMPLETED,
      source_health: sources,
      freshness: freshness,
      readiness: readiness,
    };

    var snapshot = {
      generated_at: isStale ? STALE_GENERATED : FRESH_GENERATED,
      platform_scope: "all",
      start_date: RANGE_START,
      end_date: RANGE_END,
      cache_ttl_seconds: 30,
      freshness: freshness,
      readiness: readiness,
      source_health: sources,
      drilldown: DRILLDOWN,
    };

    return { archive: archive, snapshot: snapshot };
  }

  // ══════════════════════════════════════════════════════════════════
  // Fixture：热力图（HeatmapResponseDto）—— 365 天，止于 2026-07-07
  // dashboard V2 以 include_heatmap=false 调用（LAZY_HEATMAP_LOAD 默认开），
  // 因此仪表盘响应里 heatmap=null，热力图单独走 get_usage_heatmap_v2(platform, 365)。
  // ══════════════════════════════════════════════════════════════════
  function buildHeatmap() {
    var data = {};
    for (var i = 0; i < 365; i++) {
      data[dayString(i)] = 40 + ((i * 17) % 260);
    }
    return { data: data };
  }
  var HEATMAP = buildHeatmap();

  // ══════════════════════════════════════════════════════════════════
  // Fixture：日志记录（UsageRecordDto[]）—— 60 条，3 页 × 20，cursor 分页
  // ══════════════════════════════════════════════════════════════════
  var LOG_MODELS = [
    "claude-sonnet-4-5",
    "claude-opus-4-8",
    "gpt-5",
    "gemini-2.5-pro",
    "claude-haiku-4-5",
  ];
  var LOG_PLATFORMS = ["claude", "codex", "gemini"];
  var LOG_PROJECTS = [
    "D:\\Documents\\Code\\Github\\ccr",
    "D:\\Documents\\Code\\Github\\ccr-ui",
    "D:\\Documents\\Code\\Github\\llmusage",
  ];
  var LOG_BASE_MS = Date.UTC(2026, 6, 7, 12, 0, 0);

  function buildLogRecords() {
    var records = [];
    for (var i = 0; i < 60; i++) {
      var model = LOG_MODELS[i % LOG_MODELS.length];
      var platform = LOG_PLATFORMS[i % LOG_PLATFORMS.length];
      var input = 12000 + ((i * 130) % 40000);
      var output = 3000 + ((i * 70) % 12000);
      var cacheRead = 60000 + ((i * 900) % 180000);
      var cacheCreate = 4000 + ((i * 40) % 9000);
      var costWith = Number((0.12 + ((i * 7) % 90) / 100).toFixed(4));
      records.push({
        id: "rec-" + (1000 + i),
        platform: platform,
        project_path: LOG_PROJECTS[i % LOG_PROJECTS.length],
        record_json: "{}",
        recorded_at: new Date(LOG_BASE_MS - i * 3600000).toISOString(),
        source_id: "src-" + platform,
        model: model,
        input_tokens: input,
        output_tokens: output,
        cache_read_tokens: cacheRead,
        cache_creation_tokens: cacheCreate,
        cost_with_cache_usd: costWith,
        cost_without_cache_usd: Number((costWith * 1.9).toFixed(4)),
        pricing_status: "resolved",
        pricing_source: "builtin",
      });
    }
    return records;
  }
  var LOG_RECORDS = buildLogRecords();

  // ══════════════════════════════════════════════════════════════════
  // Fixture：能力矩阵（CapabilityReport）—— 全部 supported，schema_version=14
  // 消费方读取 features.overview / sync_json_events / provider_breakdown / home_overview
  // ══════════════════════════════════════════════════════════════════
  function cap() {
    return { supported: true, reason: null, detail: null };
  }
  var CAPABILITIES = {
    cli_available: true,
    cli_version: "0.9.0",
    root_dir: "D:\\Users\\lyh\\.local\\share\\llmusage",
    db_path: "D:\\Users\\lyh\\.local\\share\\llmusage\\usage.db",
    db_exists: true,
    db_readable: true,
    schema_version: 14,
    features: {
      overview: cap(),
      sync_json_events: cap(),
      provider_breakdown: cap(),
      home_overview: cap(),
      model_breakdown: cap(),
      project_breakdown: cap(),
      trends: cap(),
      logs: cap(),
      heatmap: cap(),
    },
  };

  // ══════════════════════════════════════════════════════════════════
  // 命令处理器：usage 域
  // ══════════════════════════════════════════════════════════════════
  function buildDashboard(args) {
    var diag = buildDiagnostics(VARIANT);
    var includeHeatmap = args && args.includeHeatmap === true;
    return {
      summary: SUMMARY,
      trends: TRENDS,
      model_stats: MODEL_STATS,
      project_stats: PROJECT_STATS,
      source_stats: SOURCE_STATS,
      provider_stats: PROVIDER_STATS,
      archive: diag.archive,
      snapshot: diag.snapshot,
      heatmap: includeHeatmap ? HEATMAP : null,
      generated_at: VARIANT === "stale" ? STALE_GENERATED : FRESH_GENERATED,
    };
  }

  // get_usage_logs_v2：cursor 分页，每页固定 20；cursor 编码 offset；尊重 model 过滤
  function handleLogs(args) {
    var query = (args && args.query) || {};
    var pageSize = 20;
    var all = LOG_RECORDS;
    if (query.model) {
      all = all.filter(function (r) {
        return r.model === query.model;
      });
    }
    var offset = query.cursor ? parseInt(query.cursor, 10) || 0 : 0;
    var slice = all.slice(offset, offset + pageSize);
    var nextOffset = offset + pageSize;
    var hasMore = nextOffset < all.length;
    return {
      records: slice,
      total: query.include_total ? all.length : null,
      page: query.page || 1,
      page_size: pageSize,
      next_cursor: hasMore ? String(nextOffset) : null,
      mode: "cursor",
    };
  }

  // start_usage_import_job_v2 / get_usage_import_job_status_v2：手动点
  // “刷新/导入 usage”触发。之前两条命令均未打桩，惰性兜底 resolve(null)：
  //   - startImportJob 读 response.snapshot 直接抛 TypeError（stores/usage.ts:800）
  //   - 即便 ①修好，ensureImportJobListeners 紧接着调用 getUsageImportJobStatusV2
  //     再读 latest.status，同样对 null 抛错（stores/usage.ts:432-434）
  // 两处都是终态判定用的只读查询，这里统一用 buildImportJobSnapshot 生成一份
  // status='finished' 的快照：syncImportFeedbackFromJob 读到终态会立即把
  // importing 置回 false，UI 不会卡在“导入中”。live/missing/deleted 复用当前
  // VARIANT 的 archive 计数，保持与页面其余诊断信息一致。事件监听器仍然只是
  // no-op 注册（本桩事件永不 emit，属既有行为，无需额外处理）。
  function buildImportJobSnapshot(jobId, args) {
    var diag = buildDiagnostics(VARIANT);
    var now = new Date().toISOString();
    return {
      job_id: jobId,
      status: "finished",
      stage: "finished",
      platform_scope: (args && args.platform) || "all",
      recent_window_days: (args && args.recentDays) || 30,
      files_total: 1,
      files_scanned: 1,
      files_imported: 1,
      records_imported: 0,
      records_skipped: 0,
      history_cursor_hit: true,
      live_sources: diag.archive.live_sources,
      missing_sources: diag.archive.missing_sources,
      deleted_sources: diag.archive.deleted_sources,
      started_at: now,
      updated_at: now,
      recent_ready_at: now,
      finished_at: now,
      warnings: [],
      results: [],
    };
  }

  function handleStartImportJob(args) {
    var jobId = "job-" + Date.now();
    return {
      job_id: jobId,
      snapshot: buildImportJobSnapshot(jobId, args),
    };
  }

  function handleImportJobStatus(args) {
    var jobId = (args && args.jobId) || "job-unknown";
    return buildImportJobSnapshot(jobId, args);
  }

  // ══════════════════════════════════════════════════════════════════
  // 命令处理器：应用启动期非 usage 命令（消费方会解构字段，必须给非 null 形状）
  // ══════════════════════════════════════════════════════════════════
  var LOCAL_ENV = {
    id: "local",
    name: "Local",
    env_type: "local",
    is_active: true,
    description: "Local execution environment",
  };

  // 显式桩表：cmd → (args) => 返回值（或 Promise）
  var HANDLERS = {
    // ── usage 域 ──
    get_usage_capabilities_v2: function () {
      return CAPABILITIES;
    },
    get_usage_dashboard_v2: function (args) {
      return buildDashboard(args);
    },
    get_usage_summary_v2: function () {
      return SUMMARY;
    },
    get_usage_trends_v2: function () {
      return TRENDS;
    },
    get_usage_by_model_v2: function () {
      return MODEL_STATS;
    },
    get_usage_by_project_v2: function () {
      return PROJECT_STATS;
    },
    get_usage_by_provider_v2: function () {
      return PROVIDER_STATS;
    },
    get_usage_heatmap_v2: function () {
      return HEATMAP;
    },
    get_usage_logs_v2: function (args) {
      return handleLogs(args);
    },
    start_usage_import_job_v2: function (args) {
      return handleStartImportJob(args);
    },
    get_usage_import_job_status_v2: function (args) {
      return handleImportJobStatus(args);
    },

    // ── 应用外壳启动命令（解构字段，需非 null）──
    // EnvironmentSwitcher.vue：environments.value = await list_environments()，随后 .find()——必须是数组
    list_environments: function () {
      return [LOCAL_ENV];
    },
    refresh_environments: function () {
      return [LOCAL_ENV];
    },
    get_current_environment: function () {
      return LOCAL_ENV;
    },
    // BackendStatusBanner → useBackendHealth：读 result.status/database，healthy 使红色降级横幅不显示
    health_check: function () {
      return { status: "healthy", database: true };
    },
    // MainLayout → shellPreferencesStore.hydrateRuntimePreferences：读 confirm_before_exit 等
    shell_get_preferences: function () {
      return {
        confirm_before_exit: true,
        close_to_tray: false,
        open_panel_on_tray_click: true,
        tray_panel: { placement_mode: "anchored", manual_position: null },
      };
    },
  };

  // ══════════════════════════════════════════════════════════════════
  // 回调注册（transformCallback / unregisterCallback）—— 镜像 v2 语义
  // 事件永不真正触发，但 listen() 需要一个稳定数字 id
  // ══════════════════════════════════════════════════════════════════
  var callbacks = new Map();
  var nextCallbackId = 1;

  function transformCallback(callback, once) {
    var id = nextCallbackId++;
    callbacks.set(id, { callback: callback, once: once === true });
    // 兼容部分代码路径通过 window['_'+id] 直接调用回调
    try {
      Object.defineProperty(window, "_" + id, {
        value: function (result) {
          var entry = callbacks.get(id);
          if (entry && entry.once) {
            callbacks.delete(id);
            try {
              Reflect.deleteProperty(window, "_" + id);
            } catch (e) {}
          }
          return entry && entry.callback ? entry.callback(result) : undefined;
        },
        writable: false,
        configurable: true,
      });
    } catch (e) {
      // defineProperty 失败不影响 id 返回
    }
    return id;
  }

  function unregisterCallback(id) {
    callbacks.delete(id);
    try {
      Reflect.deleteProperty(window, "_" + id);
    } catch (e) {}
  }

  // ══════════════════════════════════════════════════════════════════
  // 事件监听注册（plugin:event|listen / unlisten）
  // ══════════════════════════════════════════════════════════════════
  var eventListeners = new Map();
  var nextEventId = 1;

  // ══════════════════════════════════════════════════════════════════
  // invoke 路由：日志 → 事件插件 → usage/启动桩 → 惰性兜底（resolve null）
  // ══════════════════════════════════════════════════════════════════
  function invoke(cmd, args, options) {
    args = args || {};
    try {
      window.__usageShimLog.push({ cmd: cmd, args: args, ts: Date.now() });
    } catch (e) {}

    // 事件系统：注册/注销监听器，返回数字 id（事件不真正 emit，但调用不能 reject）
    if (cmd === "plugin:event|listen") {
      var eventId = nextEventId++;
      eventListeners.set(eventId, { event: args.event, handler: args.handler });
      return Promise.resolve(eventId);
    }
    if (cmd === "plugin:event|unlisten") {
      eventListeners.delete(args.eventId);
      return Promise.resolve(null);
    }

    // 显式桩
    if (Object.prototype.hasOwnProperty.call(HANDLERS, cmd)) {
      try {
        return Promise.resolve(HANDLERS[cmd](args, options));
      } catch (err) {
        return Promise.reject(err);
      }
    }

    // 其它 plugin:*（window|show / window|create 等）与所有未知命令 → 惰性兜底
    return Promise.resolve(null);
  }

  // ══════════════════════════════════════════════════════════════════
  // 装配全局：__TAURI_INTERNALS__ / __TAURI__ / 事件插件内部对象
  // ══════════════════════════════════════════════════════════════════
  window.__TAURI_INTERNALS__ = {
    invoke: invoke,
    transformCallback: transformCallback,
    unregisterCallback: unregisterCallback,
    convertFileSrc: function (filePath) {
      return filePath;
    },
    // getCurrentWindow()/getCurrentWebview() 读取 metadata.*.label
    metadata: {
      currentWindow: { label: "main" },
      currentWebview: { label: "main", windowLabel: "main" },
      windows: [{ label: "main" }],
      webviews: [{ label: "main", windowLabel: "main" }],
    },
  };

  // event.js 的 _unlisten 会先调用 __TAURI_EVENT_PLUGIN_INTERNALS__.unregisterListener
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
    unregisterListener: function () {
      // no-op：真正的注销在 invoke('plugin:event|unlisten') 里完成
    },
  };

  // isTauriRuntime() 只需 '__TAURI__' in window 即为真；置空对象即可
  window.__TAURI__ = {};
  // @tauri-apps/api core.js 的 isTauri() 读 globalThis.isTauri
  window.isTauri = true;
})();
