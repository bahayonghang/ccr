# ccr-ui 首页重构：运维首屏 + Wake 式 Insights

## Goal

参考 Wake 的 Insights 页（`ref/repo/Wake`，只读参考），重构 ccr-ui 首页（路由 `/`，`DashboardView`）：

1. 首屏保留 `ccr-ui/DESIGN.md` 规定的签名构图（平台行情带、就绪状态、有界用量图、事件流、底部命令状态栏），并修复已观察到的缺陷。
2. 首屏之下新增 Insights 区块：总量行、最近 7 天对比、53 周活动热力图与连续活跃天数、按周分 agent 趋势、按小时/星期/月分布、Agents/Projects/Models 榜单。
3. 后端在 `crates/ccr-usage` 新增 Insights 投影，在 `ccr-ui/src-tauri` 新增一个 typed 命令返回单次快照。

本文件是父任务。父任务持有需求集合、任务图和跨子任务验收；实现在子任务中进行。

## 用户已确认的决策

- D1 首页结构：运维首屏 + Insights 下半屏。不新增 `/insights` 路由，不把首页整体改成统计页。
- D2 度量口径：请求数（llmusage `event_count`）、会话数（`usage_session_archive`）、token（llmusage `total_tokens`）。不新增 prompts（用户消息数）口径，不做数据库迁移。
- D3 后端范围：只做 Insights 投影与 typed 命令（含现有快照缓存复用）。不做新的后台快照刷新 job；不做平台键全局统一。

## 已核实的现状

- 首页组件：`ccr-ui/src/features/usage/dashboard/DashboardView.tsx`（322 行），展示层 `ccr-ui/src/views/dashboard/dashboardPresentation.ts`（752 行），样式 `ccr-ui/src/features/usage/styles/dashboard-*.css`。路由登记 `ccr-ui/src/shell/routeCatalog.ts:15`。
- 首页数据命令：`get_home_usage_overview_v2`（`ccr-ui/src-tauri/src/commands/usage.rs:1240-1287`）→ `services::usage::compute_home_overview`（`ccr-ui/src-tauri/src/services/usage.rs:1129-1244`）→ `ccr_usage::Dashboard::home_overview`（`crates/ccr-usage/src/db.rs:789-869`）。
- llmusage 来源共 9 个（`crates/ccr-usage/src/source.rs:34-44`：claude、codex、opencode、antigravity、kimi_code、pi、grok、zcode、deepseek_harness）。`home_overview` 的 series 只保留 4 个平台（`crates/ccr-usage/src/db.rs:841-848`，`HomeOverviewSeriesItem` 定义于 `crates/ccr-usage/src/queries.rs:226-232`），其余 5 个来源在逐日序列中丢失。
- `usage_bucket_30m` 表按 30 分钟分桶，列含 `source`、`model`、`hour_start`、`project_hash`、`project_label`、`total_tokens`、`event_count`、成本列（`crates/ccr-usage/src/fixtures.rs:31-51`）。该表可以支撑日、小时、星期、周、模型、项目维度统计。
- 现有本地日期换算只有 SQL 表达式 `local_date_expr`（`crates/ccr-usage/src/timezone.rs:58-71`），没有本地小时换算。
- 会话数来源 `usage_session_archive`（迁移 v11，`crates/ccr-db/src/database/migrations.rs:1204-1219`）只有 `message_count`，没有用户消息数列。
- 会话归档的平台名（agent-sessions 注册表：grok、claude、codex、opencode、pi、omp、antigravity、kimi，见 `.trellis/spec/ccr-ui/frontend/agent-session-observability-contracts.md`）与 llmusage 来源键不同（`omp`↔`pi`，`kimi`↔`kimi_code`）。
- 用量快照缓存：前缀 `usage:snapshot:`、TTL 30 秒（`ccr-ui/src-tauri/src/services/usage.rs:31-32`）；导入后按前缀整体失效（`ccr-ui/src-tauri/src/commands/usage.rs:128-133`）；dashboard 命令已有 single-flight 填充（`commands/usage.rs:1180-1215`）。
- 用户截图（2026-09-24，桌面端 v7.3.0）观察到：
  - 首屏用量图渲染高度约 465px。`dashboard-usage-movement.css:145` 已写 `height: clamp(10rem, 26vh, 16rem)`，上限 256px。越界原因未查明。
  - 用量区"会话"显示 0，同时 Claude Code 请求 205、Codex 请求 5.6K。原因未查明。
  - 平台行情带只有 Claude Code、Codex、Antigravity、OpenCode 4 张卡。
- Wake Insights 的实现：`ref/repo/Wake/crates/wake-core/src/db.rs:2210-2440`（`Store::insights`，一次私有连接两遍扫描后在 Rust 侧派生全部分桶）与 `ref/repo/Wake/crates/wake-core/src/models.rs:466-630`（`InsightsData`、`TREND_WEEKS = 53`、周一为周起点、streak 规则）。

## 任务图

| 子任务 | 交付物 | 依赖 |
| --- | --- | --- |
| `09-24-home-insights-backend` | `ccr_usage::Dashboard::insights` 投影、`get_home_insights` typed 命令、TS 绑定、命令清单、Rust 测试 | 无 |
| `09-24-home-insights-design` | Insights 区块设计稿（design 画布）、`DESIGN.md` 新签名组件条目、首屏图表越界的复现结论 | 无 |
| `09-24-home-insights-frontend` | 首屏修复、Insights 区块组件、查询 hook、i18n、smoke 测试 | backend 的 DTO 已生成；design 已定稿 |

backend 与 design 可以并行。frontend 必须等两者完成后再 `task.py start`。

## 跨子任务验收标准

- AC-P1 首页 `/` 首屏构图符合 `ccr-ui/DESIGN.md` 的 Layout 与签名组件条目；桌面端 1920×1080 下首屏用量图高度不超过 16rem。
- AC-P2 Insights 区块全部数据来自 `get_home_insights` 单次调用；前端不对 llmusage 或会话归档发起第二条统计请求。
- AC-P3 Insights 覆盖 llmusage 全部 9 个来源；来源显示名与颜色由前端平台描述符提供，未登记来源有可读回退名。
- AC-P4 所有度量只用 D2 的三种口径；界面文案不出现 "prompts"。
- AC-P5 空库、仅有会话无用量、仅有用量无会话、Web 预览（非 Tauri 运行时）四种状态各有明确的空态或说明，不渲染伪造数据。
- AC-P6 `just ci` 通过。

## 不在范围内

- prompts（用户消息数）口径与相关迁移。
- 新的后台快照刷新 job 与刷新进度事件。
- agent-sessions 与 llmusage 平台键的全局统一；首页"会话 0"的根因修复（Insights 内部的平台键映射除外，见 backend 设计）。如 design 子任务复现后确认根因，另开任务。
- 首屏平台行情带扩展到 9 个来源。
- Wake 的 "Agents asking Wake"、远程主机合并、会话去重逻辑。
