# Claude 工作台可用数据清单

## 摘要

- GPUI 工作台已持有两份读数：30 天 Claude `UsageSnapshot`（overview 含 `last_sync_at` 与 `cache_efficiency`、日趋势、按模型全量拆分、按项目、来源占比、日热力）和 observer `Report`（`insight` 有 `roi` 与 `total_sessions`，没有 `pricing_version`）。目前只渲染价值 KPI、订阅摘要、30 天合计和费用趋势。
- 不需要新 SQL 的有：订阅价值比、日费用、按模型费用、Token 与缓存明细、新鲜度、请求分布、Claude 费用排行。
- 拿不到的是 Claude 按项目费用。上游 llmusage 从日志文件所在目录向上找 `.git` 来解析 Claude 项目，Claude 行的 project 字段因此全空，`total_projects` 结构性为 0。
- 工具与会话的费用是假 0，因为 `claude_tool_calls.cost_usd` 的唯一写入方恒写 NULL。工具表只在显式导入的尾部写入，热力图按 UTC 分桶。会话效率表和工具聚合要在 `crates/ccr-usage` 新增查询：`usage_event` 按会话聚合，`usage_tool_call` 聚合（schema ≥ 11）。

研究票 #141（地图 #138）。只读核对了代码和本机库结构，未运行同步，文中不含本机数值。代码基线为 `origin/dev` @ `bb0bd86f`，上游 llmusage 为 `adf657d`。

## 对照表

路径缩写：`obs` = `crates/ccr-app/src/usage/observer`，`cu` = `crates/ccr-usage/src`，`tcr` = `crates/ccr-db/src/database/repositories/claude_tool_calls_repo.rs`，`desk` = `ccr-desktop/src`，`tauri` = `ccr-ui/src/features/claude/observer`，`up:` = 上游 llmusage 仓库。

| 模块 | 数据来源（文件与函数） | 可得性 | 缺口与所需查询 |
|---|---|---|---|
| 价值三卡：今日、本月、全部历史（`tauri/ObserverChrome.tsx:102-144`） | `insight_with_clock` → `overview_in_window`（`obs/mod.rs:107-123, 151-220`），渲染于 `desk/features/workbench/claude.rs:219-313` | 已有，已渲染 | `total_projects` 结构性为 0（见按项目行）。`total_sessions` 是 `claude_tool_calls` 全历史去重会话数（`tcr:14-21`），只含导入过工具调用的会话，与运行概览的归档会话口径不同；归档按平台计数是现有查询（`crates/ccr-db/src/database/repositories/usage_repo.rs:1436-1478`）。“全部历史”没有起点。GPUI 导入总带 `--recent-days`（`desk/runtime/insights/import.rs:85-90`，可选 7/30/90，见 `desk/features/insights/import.rs:260-273`）。有界同步只导入窗口内的事件，更早的历史要靠一次完整同步（`up:.trellis/spec/llmusage/backend/source-sync-contracts.md:90-95`）。上游注释说 `history_completed_at` 尚未跟踪（`up:src/query/mod.rs:776-777`），代码搜索也没找到写入方。起点可以取 `Dashboard::insights`（source = claude）的 `first_date`（`cu/db.rs:917-955`，`cu/insights.rs:214`），无需新 SQL。“今日”为 0 时要配上新鲜度。 |
| 订阅价值比横幅（`tauri/UsageInsightPanel.tsx:115-120`，`tauri/useObserverPanel.ts:116-124`） | `insight.roi`（`obs/mod.rs:198-202`）+ `Report.subscription`（`obs/subscription.rs:41-51`） | 已有。工作台未渲染，Insights 的 Observer 页已显示（`desk/features/insights/observer/presentation.rs:148-150`） | 只有 `mode == "subscription"` 且 `monthly_usd > 0` 时 `roi` 才有值，分子是本月至今的费用（`month_window`，`obs/mod.rs:131-140`）。`read()` 把缺失的键回落成代码默认值（`obs/subscription.rs:19-21, 41-51`），工作台拿到 `Some` 就直接显示（`desk/features/workbench/claude.rs:150-166`），分不清“未设置”和“设成了默认值”。需要在 `obs/subscription.rs` 增加一个保留键存在性的读取（读 `user_settings` 表），这不属于 ccr-usage。 |
| 日费用趋势（`tauri/CostAttributionTab.tsx:21-92`） | `UsageSnapshot.trends` ← `Dashboard::trends_daily`（`cu/db.rs:383-422`）；`FeeTrend::from_rows`（`desk/features/workbench/claude_trend.rs:20-46`） | 已有，已渲染 | 无缺口。`Report.usage.daily`（`obs/mod.rs:229-287`）与它重复。缺天的地方断线，不补零（`claude_trend.rs:44-46`）。 |
| 按模型费用归因（同上） | `UsageSnapshot.models` ← `Dashboard::model_breakdown`（`cu/db.rs:424-475`）；`Report.usage.models` 与它重复（`obs/mod.rs:355-371`） | 已有，工作台未渲染 | 无需新查询。字段比 Tauri 全，含 token 拆分、`cache_savings_usd`、`pricing_status`、`pricing_source`、`pricing_rate`（`cu/queries.rs:76-92`）。`pricing_version` 不必补：Tauri 的内嵌价表不参与计费（`ccr-ui/src-tauri/src/claude_observer/mod.rs:4-9`，`pricing.rs:17-19`），原生端不复制价表是既有结论（`.trellis/research/archive/2026-11/10-17-gpui-desktop/research/resume-observer-20261106.md:6`）。Usage 页已经渲染定价状态（`desk/features/insights/rows.rs:207-233`）。 |
| 按项目费用归因（同上） | `UsageSnapshot.projects`、`Report.usage.projects` ← `Dashboard::project_breakdown`（`cu/db.rs:547-588`，过滤条件为 `project_hash <> ''`，见 554） | 拿不到（Claude） | 上游从日志文件所在目录向上找 `.git` 来解析项目（`up:src/parsers/claude.rs:369-370`，`up:src/domain/project.rs:19-33, 62-98`），不用记录里的 `cwd`。本机只读探查显示 Claude 的桶与事件 project 字段全空，其他来源有值。有两条路，都未验证：① 上游改用记录里的 `cwd`（这是 llmusage 侧的缺陷）；② 在 ccr-usage 新增按 `usage_event.session_id` 的费用聚合，ccr-app 再用 `claude_tool_calls.project_path`（取自记录 `cwd`，`obs/jsonl.rs:82-86`）或归档的 `cwd` 映射到项目。②要跨库合并，覆盖率取决于映射表。 |
| Token 与缓存明细（`tauri/TokenDetailTab.tsx:19-95`，`tauri/chartOptions.ts:63-68`） | 30 天口径：`UsageSnapshot.overview`（`total.*`、`cache_efficiency`，`cu/db.rs:264-381`）、`trends[]` 四类 token、`models[]` 拆分；全历史口径：`Report.usage.cache`（`obs/mod.rs:390-426`） | 已有，无需新 SQL | `usage.cache` 已经读到，但 GPUI 没有任何页面渲染它。`TokenSummary` 没有缓存写入字段（`cu/queries.rs:9-15`），30 天的缓存写入总量要对 `trends[].cache_creation_tokens` 求和（与 `obs/mod.rs:405-411` 做法相同），或对 `models[].cache_creation_tokens` 求和。`cache_efficiency` = cache_read /（input + cache_read）（`cu/queries.rs:22-29`）。Tauri 的卡片用全历史，图表用 30 天，GPUI 宜统一用 30 天。 |
| 工具热力图（`tauri/BehaviorAnalysisTab.tsx`，`tauri/chartOptions.ts:116-127`） | `Report.tools.heatmap` ← `tool_heatmap_with_clock`（`obs/mod.rs:499-527`）← `claude_tool_calls`（`tcr:223-247`） | 形状有，数据依赖显式导入 | `claude_tool_calls` 只在包含 Claude 的导入尾部写入（`desk/runtime/insights/import.rs:327-360`，`obs/scanner.rs:209-253`），本机这张表为空。查询用 `strftime('%w'/'%H', ts)` 且不带 localtime，`ts` 又按 UTC 存储（`obs/scanner.rs:233`），所以桶是 UTC 时间。替代方案是在 ccr-usage 新增 `usage_tool_call.occurred_at` 聚合：这张表同步时就写入，本机有 Claude 行。本地小时与星期沿用 `Dashboard::insights` 在 Rust 侧的换算（`cu/db.rs:926-955`），能力门为 schema ≥ 11（`cu/capabilities.rs:281-403` 目前未覆盖该表）。 |
| 工具排行（`tauri/BehaviorAnalysisTab.tsx:36-46`，只显示调用次数） | `Report.tools.tools` ← `top_tools_with_clock`（`obs/mod.rs:537-567`，`tcr:266-290`） | 调用次数已有（同样依赖导入），费用是假 0 | 查询为 `COALESCE(SUM(cost_usd), 0)`，而 `cost_usd` 恒为 NULL（`obs/scanner.rs:236-237`），GPUI 却显示了 USD 列（`desk/features/insights/observer/presentation.rs:223`）。替代方案是对 `usage_tool_call` 按 `tool_name`、`tool_kind`、`mcp_server`、`mcp_tool` 计数（`up:src/store/migrations.rs:740-757`），不得投影 `safe_preview`。该表没有成功/失败列，成功率只能从 `claude_tool_calls` 取。 |
| 会话效率表（`tauri/BehaviorAnalysisTab.tsx:21-24, 87-127`） | `Report.tools.sessions` ← `top_sessions_with_clock`（`obs/mod.rs:436-490`）← `efficiency_sessions`（`tcr:312-349`） | 部分可得：只有调用次数与 `project_path`。费用、$/工具、模型、tokens、开始时间都拿不到 | 窗口固定 30 天，不随 `days` 变（`obs/mod.rs:449-454`）。`model: None`、`tokens: 0`、`started_at: None` 是写死的（`obs/mod.rs:470-478`）。`cost_usd` 恒为 0，按费用排序没有意义，但 GPUI 仍显示 USD 列（`presentation.rs:239`）。需要在 ccr-usage 新增按会话聚合 `usage_event` 的查询：SUM `cost_with_cache_usd` 与 tokens，取 MIN/MAX `event_at` 和模型。模板是 `up:src/query/top_sessions.rs:96-117`，它的 canonical id 带 `source:` 前缀（243-260）。`usage_event.session_id` 在上游基线里就有（`up:src/store/migrations.rs:274-293`），但 `cu/capabilities.rs` 尚未登记，要新增能力键。工具数可以取 `usage_tool_call`（schema ≥ 11）、`claude_tool_calls`，或归档的 `tool_use_count`（`crates/ccr-db/src/database/migrations.rs:1621-1634`），$/工具在 ccr-app 合并后计算。两侧的 `session_id` 都取自 JSONL 的 `sessionId`（`obs/jsonl.rs:50-51, 72-75`；`up:src/parsers/claude.rs:436-442`），代码层面可以对齐，但没有用数据验证。 |
| 新鲜度（运行概览，`desk/features/overview/header.rs:57-105`） | `overview.last_sync_at`（`cu/db.rs:349-363`）+ `SyncFreshness::at`（`desk/runtime/insights.rs:177-204`） | 已读到（在 `UsageSnapshot.overview` 中），工作台未渲染，可复用 | `last_sync_at` 是全局值，取自 `run_log` 中任意来源的 sync/hook-run，按来源导入也会推进它。按来源的新鲜度可用 `Dashboard::diagnostics()` 的 `recent_completed_at`（`cu/db.rs:799-829, 1037-1045`），上游含义是“有界窗口扫描完成”（`up:.trellis/spec/llmusage/backend/source-sync-contracts.md:216-218`）。这是现有查询，GPUI 还没调用。工具数据没有持久化的时间戳，导入结果只保存在内存快照里（`desk/runtime/insights/import.rs:327-360`）。 |
| 请求分布（运行概览，`desk/features/overview/insights.rs:61-102`） | `activity_window` → `Dashboard::insights`（默认 filter 覆盖全部来源；`desk/runtime/insights.rs:658-698`） | 可复用：换成 `source = claude` 再调用一次即可，无需新 SQL | 口径是请求数。`hourly`、`weekday`、`monthly` 覆盖截至 `as_of` 的全历史（`cu/insights.rs:181-232`），不是 30 天。 |
| 排行（运行概览，`desk/features/overview/insights.rs:104-157, 347-384`） | `InsightsPayload.models/projects`（`InsightsTally`，含 requests 与 tokens，无费用，`cu/insights.rs:88-121`）+ 归档 agents（`crates/ccr-app/src/usage/home.rs:198-236`） | 覆盖全部来源，按请求数排。Claude 的费用排行改用 `UsageSnapshot.models` | `insights` 的项目排行同样跳过空 `project_hash`（`cu/db.rs:957-1015`），Claude 为空。 |

## 附：GPUI 已读字段

- 工作台的 Claude 页发两次读：`InsightsQuery::Usage`（source = claude，`ReportClock::window(30)`）和 `observer_read(30, false)`（`desk/features/workbench.rs:167-185`），渲染 `claude_status`、`claude_insights`、`claude_trend`（同文件 599-604）。
- `Report{usage, tools, insight, subscription}`（`desk/runtime/insights/observer.rs:23-29`，组装见 145-202）：
  - `insight`（`obs/mod.rs:15-27`）：`today/month/total_value_usd`、`today/month_tokens`、`total_sessions`、`total_projects`、`subscription`、`roi`。`pricing_version` 只存在于 Tauri 的 wire DTO 中（`ccr-ui/src-tauri/src/services/claude_observer.rs:11-30`）。
  - `usage`（observer.rs:132-144）：`daily`；`projects` 与 `models` 取按费用排序的前 20；`cache` 为 Claude 全历史，不跟随报告时钟（`obs/mod.rs:390-426`）。
  - `tools`（observer.rs:180-188）：`sessions` 只投影 `session_id`、`project_path`、`cost_usd`、`tool_call_count`；另有 `heatmap` 与 `tools`。
  - `subscription`：只要 CCR 库存在就是 `Ok(Some)`。CCR 库缺失时，`tools` 与 `insight` 为 `Err(Missing)`，`subscription` 为 `Ok(None)`（observer.rs:158-165）。
- `UsageSnapshot{overview, trends, models, providers, providers_supported, projects, sources, heatmap}`（`crates/ccr-app/src/usage/read.rs:9-19`，`read_dashboard_at` 见 30-76，调用见 `desk/runtime/insights.rs:553-568`）：
  - `sources` 忽略 source 过滤，给出全部来源的占比（`cu/db.rs:590-648`）。
  - `providers` 需要 schema ≥ 14（`cu/capabilities.rs:13-14`）。上游 Claude 解析器写入空的 `provider_label`（`up:src/parsers/claude.rs:523`），归属依赖 `--provider-map`（`.trellis/spec/ccr/backend/llmusage-provider-adapter.md:117, 125`）。
  - `heatmap` 是按日的 `event_count`，用 366 天滚动窗覆盖 since/until，无数据的日期补 0（`cu/db.rs:657-707`），因此分不清“无用量”和“未同步”。

## 缺陷与疑点（均未修复）

1. `claude_tool_calls.cost_usd` 与 `duration_ms` 恒为 NULL（`obs/scanner.rs:236-237`，这是唯一写入方），所以工具费用、会话费用、$/工具和按费用排序都是假 0。
2. 上游 llmusage 不解析 Claude 项目（它看的是日志目录加 `.git`），导致 `total_projects` 恒为 0、按项目归因为空。这是上游问题。
3. 工具热力图按 UTC 分桶，不是本地时间（`tcr:230-231`）。
4. `report_window` 的 live 过滤 `ts >= datetime('now', ?)`（`tcr:48`）拿 RFC3339 文本（带 `T`）与 SQLite 的空格分隔时间做文本比较，截止当天会整天算进去。影响轻微。
5. `subscription::read` 把缺失的键回落成默认值，默认值冒充了用户设置（`obs/subscription.rs:41-51`）。
6. `cache_stats` 不跟随报告时钟与窗口，总是全历史加 live 时钟（`obs/mod.rs:390-426`）。
7. JSONL 行的时间戳缺失或无法解析时，会被记成扫描时刻（`obs/jsonl.rs:76-81`）。
8. 全局 `last_sync_at` 在按来源同步之后会误导（`cu/db.rs:349-363`）。

疑点：

- 归档 `usage_session_archive.session_id` 与 JSONL `sessionId` 是否属于同一空间，未核对。
- `pricing_status` 的取值，除了上游约定的 `unpriced`（`up:.trellis/spec/llmusage/backend/pricing-catalog-contracts.md:40`），其余未解读。
- Claude 的 `provider_label` 经 `--provider-map` 后是否有值，未在本机核对。
- `usage_tool_call.project_hash` 对 Claude 是否同样为空，未在本机核对；它走同一个解析器，推断为空。

## 来源

- CCR（`origin/dev` @ `bb0bd86f`）：
  - GPUI：`ccr-desktop/src/features/workbench.rs`、`features/workbench/{claude,claude_trend}.rs`、`features/overview/{header,insights}.rs`、`features/insights/{rows,mod,import}.rs`、`features/insights/observer/presentation.rs`、`runtime/insights.rs`、`runtime/insights/{observer,import}.rs`。
  - 共享 owner：`crates/ccr-app/src/usage/{read,home}.rs`、`crates/ccr-app/src/usage/observer/{mod,subscription,scanner,jsonl}.rs`、`crates/ccr-usage/src/{db,queries,insights,capabilities}.rs`、`crates/ccr-db/src/database/migrations.rs`、`crates/ccr-db/src/database/repositories/{claude_tool_calls_repo,usage_repo}.rs`。
  - Tauri：`ccr-ui/src/features/claude/observer/`、`ccr-ui/src-tauri/src/services/claude_observer.rs`、`ccr-ui/src-tauri/src/claude_observer/{mod,pricing}.rs`、`ccr-ui/src/i18n/locales/zh-CN.ts:1028, 1047, 1079`。
  - 规格与既有研究：`.trellis/spec/ccr/backend/llmusage-provider-adapter.md:108-125`、`.trellis/research/archive/2026-11/10-17-gpui-desktop/research/resume-observer-20261106.md`。
- 上游 llmusage（github.com/bahayonghang/llmuasage @ `adf657d03495c7d454b1f354321bc83069eb3cd4`）：`src/store/migrations.rs`（基线 274-293，v11 `add_behavior_facts` 716-775）、`src/parsers/claude.rs`、`src/domain/project.rs`、`src/query/{mod,top_sessions}.rs`、`src/store/sync_status.rs:154-187`、`.trellis/spec/llmusage/backend/{source-sync-contracts,pricing-catalog-contracts}.md`。
- 本机只读探查（`sqlite3` 以 `mode=ro` 打开，`python -I` 运行，只输出结构与布尔值）：llmusage 库满足 schema ≥ 11 与 ≥ 14；Claude 有桶，事件带 `session_id`；`usage_tool_call` 与 `usage_turn` 有 Claude 行；Claude 的 project 字段全空；`run_log` 中有成功的同步记录。CCR 主库的 `claude_tool_calls` 表存在但无行。归档中有 Claude 会话及扩展列。未读取 `user_settings`。
