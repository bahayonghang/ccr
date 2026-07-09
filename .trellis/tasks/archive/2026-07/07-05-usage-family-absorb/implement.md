# implement: usage-family-absorb

单 commit、先前端迁移后整链删除的顺序，保证每步可编译。

## 步骤

1. **ConfigsView 迁移** → verify: `bun run type-check`
   - `src/views/ConfigsView.vue`：`getProviderUsage` → `getUsageByProviderV2`（近 30 天 startDate），DTO[] 映射为 `Record<string, number>`（`provider ?? 'unknown'` → `request_count` 累加）。
2. **前端 wrapper 下线** → verify: `bun run type-check && bun run lint`
   - `src/api/domains/stats.ts` 删 10 个 legacy wrapper（含 getSessionStats，位于 V2 节内）；
   - `src/api/tauri.ts` 第 12 分组 re-export 同步删除；
   - `src/api/domains/usage.ts` usageApi re-export 同步删除。
3. **Rust 命令下线** → verify: `cd ccr-ui/src-tauri && cargo clippy --all-targets`
   - 删 `commands/stats.rs`；`commands/mod.rs` 去 `pub mod stats;`；
   - `handler_registry.rs` 删 `stats:` / `stats_extended:` 两组，形状测试改 28 / 310 / 302；
   - 删 `stats_snapshot.rs`；`main.rs` 去 `mod stats_snapshot;`。
4. **全量验证**
   - `cd ccr-ui/src-tauri && cargo test`（registry 形状 + 全部单测，含 llmusage_no_crate_guard）
   - `just frontend-check-quick`
   - 残留扫描：`rg "get_cost_overview|get_heatmap_data|get_session_stats|get_cost_trend|get_cost_by_model|get_cost_by_project|get_provider_usage|get_top_sessions|get_stats_summary|get_daily_stats|stats_snapshot" ccr-ui/src ccr-ui/src-tauri/src` 应为空
5. **收尾**：spec 回写（usage 域命令面变化）、journal、commit（`refactor(ui)` + `refactor(tauri)` 可合并为一条 `refactor(ui)` 全链 commit）。

## 回滚点

步骤 1-2 与 3 之间可独立回滚；整体单 commit revert。
