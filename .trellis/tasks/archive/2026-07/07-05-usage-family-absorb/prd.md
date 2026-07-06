# usage-family-absorb: stats 零调用命令下线 + provider usage 迁移 + CostTracker 链路清理

## 背景

07-03-arch-typed-ipc 的重叠盘点（research/usage-family-overlap.md）确认：CostTracker 系 10 条 stats 命令中 9 条前端零调用，1 条（get_provider_usage）仅 ConfigsView.vue 使用且 usage_v2 有语义对应（get_usage_by_provider_v2 的 request_count）。usage_v2（llmusage 投影）已是活跃主链路，legacy JSONL 扫描链路应下线。

## 需求

1. **下线 9 条零调用 stats 命令**：get_cost_overview / get_heatmap_data / get_session_stats / get_cost_trend / get_cost_by_model / get_cost_by_project / get_top_sessions / get_stats_summary / get_daily_stats（Rust 命令 + 注册 + 前端 wrapper 全链路删除）。
2. **迁移 get_provider_usage 的唯一消费者**：ConfigsView.vue 改用 get_usage_by_provider_v2，随后同样下线 get_provider_usage。
3. **清理 ccr-ui 侧 CostTracker stats 链路死代码**：stats.rs、stats_snapshot.rs（create_cost_tracker / normalize_usage_platform / run_cached_stats_command）。

## 边界（不做）

- claude 预算链路（claude_settings.rs claude_get_budgets 经 BudgetManager 使用 CostTracker）保留，不在本任务范围。
- main.rs `verify_usage_storage_dir` 启动校验保留（预算链路仍读同一目录）。
- AppState 缓存基建（cache_get/begin_cache_fill）为 codex/usage/system 共用，不动。
- pricing 命令组（set_pricing 等）与 stats 同域但独立实现（commands/pricing.rs），保留。
- ProviderStatsModal.vue 组件契约（Record<string, number>）不变，映射在 ConfigsView 完成。

## 验收标准

- [ ] 全仓（排除 ref/ 与任务归档）不再出现 10 条命令名及其前端 wrapper 名。
- [ ] ConfigsView 的 provider 统计弹窗数据源为 get_usage_by_provider_v2，展示语义保持"provider → 调用次数"，时间窗保持近 30 天。
- [ ] handler_registry 形状测试更新并通过（模块 30→28，命令 320→310 / 312→302）。
- [ ] `cd ccr-ui/src-tauri && cargo clippy` + `cargo test` 通过；`just frontend-check-quick` 通过。
