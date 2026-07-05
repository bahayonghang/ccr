# usage 命令家族重叠盘点（PRD item 5：只标记，不删除）

盘点时点：2026-07-05，基于 handler_registry 分组与前端全量调用扫描（views/stores/composables/components，排除 api 层自身）。

## 四条数据链路

| 家族 | 命令数 | 数据源 | 前端活跃度 |
|---|---|---|---|
| Usage V2 | 17 | llmusage DB（`~/.llmusage/llmusage.db`，只读投影）+ session_archive 旁路（`~/.ccr/analytics/usage.db`） | **活跃**：stores/usage.ts、stores/homeUsageOverview.ts、composables/usePlatformUsageInsight.ts、views/MonitoringView.vue |
| 统计（stats 组） | 3 | legacy `ccr_store::CostTracker`（JSONL 全量扫描） | **零调用** |
| 统计扩展 | 7 | 同上 CostTracker | 仅 1 条活跃（get_provider_usage ← ConfigsView.vue） |
| Claude Observer | 9 | 独立 SQLite `claude_tool_calls`（scanner 摄取 `~/.claude/projects`，state.db_pool） | **活跃**：stores/claudeObserver.ts + 5 个 claude-observer 组件 |

注：`invoke('get_*')` 直接调用仅存在于 api 层（domains/stats.ts 与 tauri.ts 门面），无视图旁路——调用面结论可信。

## 吸收矩阵（stats/统计扩展 → usage_v2）

| 旧命令 | 前端调用 | usage_v2 对应 | 标记 |
|---|---|---|---|
| get_cost_overview | 无 | get_usage_summary_v2 (+by_model/by_project 组合) | 可吸收 |
| get_heatmap_data | 无 | get_usage_heatmap_v2（llmusage usage_bucket_30m，维度更准） | 可吸收 |
| get_session_stats | 无 | get_home_usage_overview_v2 的 session 维度 | 可吸收 |
| get_cost_trend | 无 | get_usage_trends_v2 | 可吸收 |
| get_cost_by_model | 无 | get_usage_by_model_v2 | 可吸收 |
| get_cost_by_project | 无 | get_usage_by_project_v2 | 可吸收 |
| get_provider_usage | ConfigsView.vue（provider→count 计数） | get_usage_by_provider_v2（注意：v2 的 provider 是 llmusage 归因维度，与 CostTracker 的 provider 字段语义非严格等价，迁移时需核对 ConfigsView 展示语义） | 迁移后可吸收 |
| get_top_sessions | 无 | usage logs / observer top_sessions（主题重叠，无一比一对应） | 可吸收（无消费者，直接废弃候选） |
| get_stats_summary | 无 | get_usage_summary_v2 | 可吸收 |
| get_daily_stats | 无 | get_usage_trends_v2 + home overview series | 可吸收 |

**结论**：10 条 CostTracker 系命令中 9 条前端零调用、1 条（get_provider_usage）迁移 ConfigsView 后可吸收。全部吸收后 `ccr_store::CostTracker` JSONL 链路在 ccr-ui 侧变纯死代码（stats_snapshot.rs 的 create_cost_tracker 亦随之待清理），是后续 deepening 候选。删除/迁移工作另立任务（本任务只标记）。

## Claude Observer：不可吸收

9 条命令全部保留。数据维度是 per-tool/per-session 行为分析（tool heatmap、top tools、cache 命中、订阅额度），llmusage 无此投影；与 usage_v2 仅主题相邻（cost_breakdown / top_sessions），数据源与语义不同。前端已有手写富类型（types/claudeObserver.ts）且 store/组件活跃消费——若未来做第二个 typed-ipc domain，observer 是好候选（类型已稳定，只差生成链路接入）。

## 顺带发现（不在本任务处理）

- `commands/usage.rs`（现 services/usage.rs）本地 `HomeOverviewPlatformStats/Summary/SeriesItem`（u64）与 `ccr-usage` 同名类型（i64）并存：wire 走本地 u64 版（经 non_negative_i64 钳制），ccr-usage 版只在 home_overview 投影内部使用。合并需统一符号语义，标记为后续小型收敛候选。
- stats.rs 头注释自述"真正的 SQL 聚合归档查询留给 Phase 2b"——吸收进 usage_v2 后该 Phase 2b 自然取消。
