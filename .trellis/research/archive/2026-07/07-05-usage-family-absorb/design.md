# design: usage-family-absorb

## 数据链路现状（盘点已验证，2026-07-05 复核）

- `commands/stats.rs`：10 条 `#[tauri::command]`，全部经 `stats_snapshot::run_cached_stats_command` + `create_cost_tracker()`（`ccr_store::CostTracker`，JSONL 全量扫描）。
- `stats_snapshot.rs` 三个符号（create_cost_tracker / normalize_usage_platform / run_cached_stats_command）**仅** stats.rs 引用；AppState 缓存方法另有 codex.rs / usage.rs / system.rs 直接使用，不受影响。
- 前端调用面复核（rg 全量，排除 api 层）：仅 `ConfigsView.vue` 调 `getProviderUsage`，其余 9 条零调用。测试目录零引用。
- ccr-ui 内 CostTracker 残余消费者：`claude_settings.rs`（claude_get_budgets，经 BudgetManager）、`main.rs`（verify_usage_storage_dir 目录校验）——均不在本任务范围。

## ConfigsView 迁移契约

旧：`get_provider_usage` → CostTracker.generate_stats(近 30 天).by_provider → `Record<string, number>`（provider → 记录条数）。

新：`getUsageByProviderV2(undefined, startDate)` → `ProviderBreakdownDto[]`，其中：
- `provider: string | null` → key，null 归入 `'unknown'`（模态框对空串/unknown 已有 `$t('configs.provider.unknown')` 兜底展示）；
- `request_count: number` → value，与旧"调用次数"语义一致；
- `startDate` 传 30 天前的 `YYYY-MM-DD`（build_filter 用 `NaiveDate %Y-%m-%d` 解析），保持旧时间窗语义；endDate 不传（到当前）。

已知语义差异（接受）：provider 归因维度从 CostTracker 记录字段变为 llmusage 归因，数据源从 `~/.ccr/costs/` JSONL 变为 llmusage DB 投影。盘点已判定该差异可接受，v2 维度更准。

映射在 ConfigsView 的 `loadProviderUsage` 内完成，`ProviderStatsModal` props 契约不变。

## 删除面

| 层 | 文件 | 动作 |
|---|---|---|
| Rust 命令 | `src-tauri/src/commands/stats.rs` | 整文件删除 |
| Rust 模块 | `src-tauri/src/commands/mod.rs` | 移除 `pub mod stats;` |
| Rust 注册 | `src-tauri/src/commands/handler_registry.rs` | 移除 `stats:`（3 条）与 `stats_extended:`（7 条）两组；形状测试 30→28 模块、320→310（win）/312→302（非 win） |
| Rust 基建 | `src-tauri/src/stats_snapshot.rs` | 整文件删除；main.rs 移除 `mod stats_snapshot;` |
| TS wrapper | `src/api/domains/stats.ts` | 删 10 个 wrapper；保留 V2 系与 pricing 系；头注释同步 |
| TS 门面 | `src/api/tauri.ts` | 第 12 分组 re-export 删 10 项（冻结门面允许删除，只禁新增 invoke） |
| TS 命名空间 | `src/api/domains/usage.ts` | usageApi re-export 删 10 项 |
| Vue | `src/views/ConfigsView.vue` | import 与 loadProviderUsage 改 V2 |

## 回滚

单 commit 实施，git revert 即整体回滚。无持久化/schema 变更，无兼容窗口需求（Tauri IPC 前后端同版本发布）。
