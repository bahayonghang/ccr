# 统一 usage 投影

## Goal

让 `crates/ccr-usage` 成为唯一只读 SQLite 投影（source/paths/capabilities/db/查询，source-tagged DTO），`llmusage_adapter` 仅保留 CLI sync/事件/DTO 映射并包裹 ccr-usage；删除 TUI `UsageProviderRow` 影子结构。`llmusage-provider-adapter.md` 契约本已要求此归属，现状属违例。审查候选 3（Strong）。

## Requirements

### 现状（探索报告定位）

- 同一"只读 SQLite 投影"实现两遍：`crates/ccr-usage/src/{source,paths,capabilities,db}.rs`（~700 LOC，TUI 用）⟷ `ccr-ui/src-tauri/src/llmusage_adapter/{source,paths,capabilities,db}.rs`（~3200 LOC 超集，desktop 用）。`source.rs` 逐字重复，且 ccr-usage 的副本丢了 adapter 的两个测试用例。schema 升级/新 SourceKind 别名/新 FeatureKey 必须双处编辑，否则 TUI 与 desktop 静默分叉。
- TUI 影子结构：`crates/ccr-tui/src/tui/usage/app.rs:9-50,167-180` 的 `UsageProviderRow` 是 `ccr_usage::ProviderBreakdownDto`（queries.rs:3-15）的逐字段拷贝 + platform 标签 − cost_without_cache_usd，配 10 字段手工映射 `provider_row_from_shared`。
- TUI 数据层不可测：`load_usage_dataset()`（usage/app.rs:147-165）直连 `discover_llmusage_paths()`+`open_dashboard()`，无注入点；`UsageLoadState` 状态机与 `drain_task_messages` 错误分类（app.rs:115-144）零测试。
- 文档矛盾：根 CLAUDE.md 称 llmusage 为 "git dependency pinned rev"，llmusage_adapter/mod.rs 不变量是"绝不链接 upstream crate"（走 CLI + 只读 SQLite）。

### 要做的

1. ccr-usage 吸收 adapter 版投影的超集能力（含被丢的测试用例），DTO 加 source 标签（或等价 scope 字段）。
2. llmusage_adapter 的 source/paths/capabilities/db 改为委托 ccr-usage；adapter 只保留真正属于它的：llmusage CLI sync 调用、事件、Tauri DTO/错误映射。
3. 删除 TUI `UsageProviderRow` 与 `provider_row_from_shared`，直接消费 source-tagged DTO。
4. 给 TUI usage 数据加载引入注入 seam（loader 可替换），使 `UsageLoadState` 转移与错误分类可单测（顺带修复审查候选中的 TUI 可测性缺口）。
5. 澄清并修正 CLAUDE.md 与 mod.rs 的 llmusage 依赖表述矛盾，以 `llmusage_no_crate_guard` 测试的实际行为为准。

### 约束

- 严守 `llmusage-provider-adapter.md` 全部契约：schema 14 门控、provider filter 语义、`provider = null` → unattributed、`--provider-map` 仅在 activation log 存在时传、AppPaths 路径契约（Tauri 可用 `from_root` 保持现状）。
- upstream 边界不变：只走已安装 llmusage CLI + 只读 SQLite，不链接 upstream crate。
- Tauri 侧 DTO/错误类型仍归 adapter 所有（契约明文），收敛的是投影实现而非表现层类型。

## Acceptance Criteria

- [ ] `llmusage_adapter/{source,paths,capabilities,db}.rs` 不再含独立投影实现（删除或退化为对 ccr-usage 的薄委托）；`rg 'usage_bucket_30m'` 的 SQL 只命中 crates/ccr-usage。
- [ ] `SourceKind`/`FeatureKey`/schema 门控全仓仅 1 处定义；被丢的两个测试用例回归 ccr-usage。
- [ ] `UsageProviderRow` 与 10 字段手工映射删除。
- [ ] TUI `UsageLoadState` 状态转移与 SchemaUnsupported/FeatureUnavailable 错误分类有单元测试（无需真实 ~/.llmusage DB）。
- [ ] 契约要求的测试全绿：`cargo test -p ccr-usage`、`cargo test -p ccr-tui -- --test-threads=1`、`cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml llmusage_adapter -- --nocapture`、`--test llmusage_no_crate_guard`、`cd ccr-ui && bun run test:smoke -- tests/usage-dashboard-payload.smoke.test.ts`。
- [ ] CLAUDE.md llmusage 段落与 mod.rs 不变量表述一致。
- [ ] `llmusage-provider-adapter.md` 如有契约细化，同步更新（trellis-update-spec）。

## Notes

- 复杂任务：`task.py start` 前需补 design.md（ccr-usage 接口扩展形状、adapter 委托边界、TUI 注入 seam）与 implement.md。
- 依赖：无硬依赖，第一批可并行。
