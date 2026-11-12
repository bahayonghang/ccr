# Claude 工作台费用读数与新鲜度核实

研究票 #142（地图 #138）。代码基线 `origin/dev`@`bb0bd86f`；上游 llmusage 基线 `bahayonghang/llmuasage` main@`adf657d0`。本机两个库都以 `mode=ro` 只读打开核对，下文只写判断，不写本机数值、日期、路径和时区。

## 摘要

1. 三格都来自 `insight_with_clock`：对 llmusage 的 `usage_bucket_30m`（`source = 'claude'`）做三次 `Dashboard::overview`，窗口按系统 IANA 本地时区的日历日切分；金额是 llmusage 导入时按价目快照算出的 `cost_with_cache_usd` 之和，是 API 价估算，不是账单。
2. 今日 $0 / 0 tokens 是**同步滞后**：今日窗口没有已导入的行，最后一次同步早于今日且失败，而今日已有未导入的转录。
3. 全部历史金额等于近 30 天是**口径（数据覆盖）**：整个 llmusage 库最早的行就在 30 天窗口内，磁盘上也没有更早的转录（Claude Code 默认只留 30 天）。
4. 全部历史项目数为 0 是**缺陷**（#152）：上游从不给 Claude 用量写项目归属，CCR 又把「没有归属」算成 0。
5. 新鲜度：工作台手里已有 `last_sync_at`，可以复用运行概览的 `SyncFreshness`；但现行查询漏算 `sync --rebuild`，「上次成功同步」偏早（#151），复用前要先修。

## 1. 今日：显示 $0 与 0 tokens → 同步滞后

**取数链路**：`claude_metrics` 读 `report.insight` 的 `today_value_usd`、`today_tokens`（`ccr-desktop/src/features/workbench/claude.rs:219-269`）← `observer::read` 用只读连接调 `insight_with_clock`（`ccr-desktop/src/runtime/insights/observer.rs:145-202`，时钟 `153-156`，insight `189-195`）← `overview_in_window(today, today)`（`crates/ccr-app/src/usage/observer/mod.rs:107-123`、`163-168`）← `Dashboard::overview`（`crates/ccr-usage/src/db.rs:254-381`）。

**口径**（本月、全部历史同此）：

- 来源：`source = 'claude'`（`db.rs:108-113`）。
- 时区：真实启动用 `ReportClock::live()`，即系统 IANA 本地时区（`crates/ccr-app/src/usage/report_clock.rs:10-15`，`crates/ccr-usage/src/timezone.rs:19-24`）；基准夹具改用固定 UTC（`ccr-desktop/src/runtime/mod.rs:258-263`）。
- 窗口：本地日期换算成 UTC 半开区间 `[起始日 00:00, 结束日次日 00:00)`，与 `hour_start` 比较（`db.rs:120-133`，`timezone.rs:51-64`）。今日窗口取 `clock.date()`（`observer/mod.rs:126-130`）。本机 `hour_start` 全是 RFC3339 秒精度 `Z` 格式，字符串比较成立。
- 金额：`SUM(cost_with_cache_usd)`（`db.rs:299`）。这一列是 llmusage 导入时按当时启用的价目目录算出的估算美元：输入 + 缓存读（按缓存读价）+ 缓存写 + 输出，推理单独计价时另加（上游 `src/query/pricing.rs:36-41`、`101-105`）。`pricing_status = snapshot` 表示费率来自加载的价目快照（`pricing.rs:8-12`）。本机 Claude 行全是 `snapshot`、同一价目版本，无 `unpriced` 行。对订阅用户这是 API 等价价值，不是实付金额。
- tokens：`SUM(total_tokens)`（`db.rs:291`）。Claude 的 total = 输入 + 缓存写 + 缓存读 + 输出（上游 `src/parsers/claude.rs:729-733`）；本机五项之和与 `total_tokens` 相等。

**判定：同步滞后造成的假零。** 空窗口经 `COALESCE(SUM(...), 0)`（`db.rs:287-299`）得 0，`InsightDto` 用非可选的 `f64` / `i64` 承载（`observer/mod.rs:15-27`），页面就画成 $0 和 0 tokens。证据：

- 今日窗口内 Claude 行：无。最晚的 Claude 桶早于今日起点。
- `run_log`：最后一次成功的 `sync` 和 `sync --rebuild` 都早于今日；其后有一次失败的 `sync`（`no such table: message`），之后再没有任何运行。
- 那次失败的运行仍导入了更新的 Claude 行（这些行的 `created_at` 晚于最后一次成功运行），符合 `ccr-desktop/DESIGN.md:155`「失败的同步仍可能导入部分数据」。
- 磁盘上有修改时间落在今日窗口内、且晚于最晚已导入桶的 Claude 转录，所以今日确有用量没有导入。
- 同步失败出在 llmusage 一侧：llmusage 库本来就没有 `message` 表；上游唯一查 `message` 表的是 OpenCode 来源读 `opencode.db`（上游 `src/parsers/opencode.rs:45`、`350`）。推断是该来源的外部库结构不符，不是 CCR 缺陷。CCR 打开页面也不会触发导入（`ccr-desktop/src/runtime/insights.rs:42-44`）。

页面没写新鲜度，读者分不清真零和假零。这是地图的设计范围（可用事实见第 5 节），不另开 bug。

## 2. 本月 → 口径正确，读数受同步滞后影响

`month_value_usd`、`month_tokens` 来自 `overview_in_window(本月 1 日, 今日)`（`observer/mod.rs:131-140`、`170-175`），即本地 `[本月 1 日 00:00, 明日 00:00)`，来源、金额、tokens 口径同第 1 节。本月窗口内有行，定义没有问题；但最后一次导入之后的日子缺数据，读数偏小，原因同第 1 节。

## 3. 全部历史：金额等于近 30 天合计 → 口径（数据覆盖）

`total_value_usd` 来自 `overview_in_window(None, None)`（`observer/mod.rs:161`），不加任何时间子句（`db.rs:120-133` 只在有起止日时追加），即 llmusage 已导入的全部 Claude 行。摘要行「最近 30 天」是另一条请求：`InsightsQuery::Usage`，窗口 `clock.window(30)` = 本地 `[今日-29, 今日]`（`ccr-desktop/src/features/workbench.rs:167-183`，`report_clock.rs:28-36`，经 `crates/ccr-app/src/usage/read.rs:30-76`），读数在 `claude.rs:175-195`。

**判定：不是投影缺陷。** 只要最早的 Claude 行落在 30 天窗口内，两数必然相等。证据：

- 全部历史与 30 天窗口的金额、tokens、行数三者都相等。
- 最早的 Claude 桶不早于 30 天窗口起点；整个 llmusage 库（所有来源）最早的桶也不早于窗口起点。
- 磁盘上没有早于窗口起点的 Claude 转录。Claude Code 默认在 `cleanupPeriodDays`（30 天）后删除 `~/.claude/projects/<project>/<session>.jsonl`（官方文档），llmusage 只能导入还在磁盘上的转录。

这种相等是暂时的：llmusage 会保留已导入的行，时间一长，最早的行滑出 30 天窗口，两数就会分开。页面没写「全部历史」从哪天开始，读者无法判断，属设计范围。起始日有现成投影：`Dashboard::insights` 返回的 `InsightsPayload.first_date`（`crates/ccr-usage/src/insights.rs:92`，过滤条件下最早有用量的本地日），工作台带 `source: Claude` 调一次即可，不需要新 SQL。

## 4. 全部历史：项目数为 0 → 缺陷（#152）

`total_projects = dashboard.project_breakdown(全部历史).len()`（`observer/mod.rs:177-185`），只数非空 `project_hash`（`db.rs:554`）。两层原因：

1. **上游**：llmusage 的 Claude 解析器按转录所在目录解析项目 `resolver.resolve(path.parent())`（上游 `src/parsers/claude.rs:369-370`，传入事件在 `534`）。`find_git_root` 从 `~/.claude/projects/<编码目录>` 往上找 `.git`，找不到就返回 `None`（上游 `src/domain/project.rs:30-33`、`62-98`），全程不读记录里的 `cwd`。
2. **本仓库**：`InsightDto.total_projects: i64`（`observer/mod.rs:23`）表达不了「未知」，工作台照样写「0 个项目」（`claude.rs:253-269`），违反 `ccr-desktop/DESIGN.md:323-324`「unknown counts show a reason, never sample zeroes」。Tauri 版同样受影响（`ccr-ui/src-tauri/src/services/claude_observer.rs:192`）。

证据：本机 Claude 行在 `usage_event`、`usage_bucket_30m`、`usage_turn`、`usage_tool_call` 中都没有非空 `project_hash`，`usage_event` 的 `path_hash`、`project_ref`、`project_path` 也全空；codex、grok 等来源有归属。有行、无归属，所以不是同步滞后。

## 5. 新鲜度事实

| 事实 | 来源 | 工作台现状 |
| --- | --- | --- |
| 上次成功同步时间 | `OverviewPayload.last_sync_at`：`run_log` 成功运行的 `MAX(finished_at)`（`db.rs:349-363`） | 已持有：30 天快照的 `UsageSnapshot.overview`（`read.rs:9-19`、`43-46`），未渲染。三格用的三次 overview 也各算了一次，但 `InsightDto` 没有带出。复用前先修 #151。 |
| 本地时间与是否过期 | `SyncFreshness::at`：转成报告时区 `MM-DD HH:mm`；上次成功同步的本地日期早于报告日或没有记录时 `stale`（`ccr-desktop/src/runtime/insights.rs:177-203`） | 可直接复用。运行概览的文案与过期圆点见 `ccr-desktop/src/features/overview/header.rs:53-105`、`rail.rs:150`，合同见 `DESIGN.md:155` |
| 报告日与时区 | `ReportClock::date()` / `timezone`（`report_clock.rs:10-27`） | 已有。`DESIGN.md:323` 要求写明期间、本地时区与计价基准，三格目前都没写 |
| 全部历史起始日 | `InsightsPayload.first_date`（`ccr-usage/src/insights.rs:92`） | 需新增一次 `Dashboard::insights` 调用（带 Claude 过滤），不需要新 SQL |
| 最近一条记录的日期 | 30 天快照 `trends` 的最后一天 | 数据有，但合同规定新鲜度只陈述成功同步、不据空白日推断导入状态（`DESIGN.md:155`；`HANDOFF-gpui-redesign.md` §6），不宜当新鲜度展示 |
| 最近一次同步失败 | `run_log` 中非 `success` 的行及其 `error` | 没有投影；要显示需在 `crates/ccr-usage` 新增查询（后端工作项） |
| 各来源扫描完成时间 | `source_sync_status.recent_completed_at` / `history_completed_at`，经 `Dashboard::diagnostics`（`db.rs:799-829`） | 不可依赖：本机所有来源都为空；上游只在 RecentReady 时写 `recent_completed_at`（上游 `src/store/sync_status.rs:154-191`），代码搜索未见 `history_completed_at` 的写入 |

**「上次成功同步」漏算 `sync --rebuild`（#151）**：`db.rs:352` 只认 `command IN ('sync', 'hook-run')`，上游的导入命令族是 `["sync", "sync --rebuild", "hook-run"]`（上游 `src/store/run_log.rs:7-9`）。本机有一条成功的 `sync --rebuild` 晚于最后一次成功的 `sync`，运行概览显示的却是后者。`HANDOFF-gpui-redesign.md` §7 记下的「上次成功同步」与现行查询结果相等，也就是那条更早的普通 `sync`，并非实际的最后一次成功导入。

## 6. 来源

本仓库（`origin/dev`@`bb0bd86f`）：

- `ccr-desktop/src/features/workbench/claude.rs:175-195`、`219-313`；`ccr-desktop/src/features/workbench.rs:167-185`
- `ccr-desktop/src/runtime/insights/observer.rs:145-202`；`ccr-desktop/src/runtime/insights.rs:42-44`、`177-203`、`553-568`、`600-652`；`ccr-desktop/src/runtime/mod.rs:258-263`
- `ccr-desktop/src/features/overview/header.rs:53-105`、`rail.rs:150`
- `crates/ccr-app/src/usage/observer/mod.rs:15-27`、`107-220`；`crates/ccr-app/src/usage/report_clock.rs:10-36`；`crates/ccr-app/src/usage/read.rs:9-76`
- `crates/ccr-usage/src/db.rs:99-138`、`254-381`、`547-588`、`799-829`；`crates/ccr-usage/src/timezone.rs:19-64`；`crates/ccr-usage/src/insights.rs:88-100`
- `ccr-desktop/DESIGN.md:155`、`323-324`、`336-338`；`HANDOFF-gpui-redesign.md` §6、§7

上游 llmusage（main@`adf657d0`）：

- [`src/store/run_log.rs#L7-L9`](https://github.com/bahayonghang/llmuasage/blob/adf657d03495c7d454b1f354321bc83069eb3cd4/src/store/run_log.rs#L7-L9)
- [`src/parsers/claude.rs#L369-L370`](https://github.com/bahayonghang/llmuasage/blob/adf657d03495c7d454b1f354321bc83069eb3cd4/src/parsers/claude.rs#L369-L370)、[`#L729-L733`](https://github.com/bahayonghang/llmuasage/blob/adf657d03495c7d454b1f354321bc83069eb3cd4/src/parsers/claude.rs#L729-L733)
- [`src/domain/project.rs#L30-L98`](https://github.com/bahayonghang/llmuasage/blob/adf657d03495c7d454b1f354321bc83069eb3cd4/src/domain/project.rs#L30-L98)
- [`src/query/pricing.rs#L8-L49`](https://github.com/bahayonghang/llmuasage/blob/adf657d03495c7d454b1f354321bc83069eb3cd4/src/query/pricing.rs#L8-L49)、[`#L101-L105`](https://github.com/bahayonghang/llmuasage/blob/adf657d03495c7d454b1f354321bc83069eb3cd4/src/query/pricing.rs#L101-L105)
- [`src/store/sync_status.rs#L154-L191`](https://github.com/bahayonghang/llmuasage/blob/adf657d03495c7d454b1f354321bc83069eb3cd4/src/store/sync_status.rs#L154-L191)
- [`src/parsers/opencode.rs#L350`](https://github.com/bahayonghang/llmuasage/blob/adf657d03495c7d454b1f354321bc83069eb3cd4/src/parsers/opencode.rs#L350)

官方文档：Claude Code「Explore the .claude directory」→ [Cleaned up automatically](https://code.claude.com/docs/en/claude-directory#cleaned-up-automatically)（`cleanupPeriodDays` 默认 30 天，覆盖 `projects/<project>/<session>.jsonl`）。

本机只读核对：`~/.llmusage/llmusage.db` 与 `~/.ccr-ui/ccr-ui.db` 以 `mode=ro` 打开；`~/.claude/projects` 只读取文件修改时间。没有运行 `llmusage sync`、应用或迁移。
