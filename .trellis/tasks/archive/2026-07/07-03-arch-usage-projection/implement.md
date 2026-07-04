# Implement: 统一 usage 投影

前置：design.md 已定边界。每步末尾的 verify 必须通过才进下一步。

## B1 ccr-usage 扩容（纯新增，不动 adapter/TUI）

- [ ] 1.1 `crates/ccr-usage/Cargo.toml` 加 `base64 = { workspace = true }`
- [ ] 1.2 `capabilities.rs`：FeatureKey 扩为 11 键全集；迁入 UnsupportedReason / FeatureCapability / required_columns 超集 / min_schema_version；新增 `DbCapabilities::detect`（原 populate_db_features + detect 的 DB 半边，含缺失/不可读降级）；迁入 adapter 5 个测试并与现有 2 个合并
- [ ] 1.3 `queries.rs`：迁入投影 DTO 全集（逐字节，serde 形状不动）+ generated_at；ProviderBreakdownDto 加 display_provider/output_tokens_total/cache_tokens_total 方法；新增 TaggedProviderBreakdown
- [ ] 1.4 `db.rs`：QueryFilter 加 model/project_hash + with_provider/without_source；SqlFilter 补 push_raw；迁入 9 个查询 + ensure_feature_for_filter + LogsQuery/LogsPage/SourceDiagnostics/DiagnosticsPayload + build_filter + cursor；新增 provider_breakdown_by_source；迁入 adapter 14 个 db 测试（fixture 超集版）并与现有 4 个合并
- [ ] 1.5 `paths.rs` 补 default_discovery 测试；`source.rs` 补 gemini/opencode 别名断言；`lib.rs` 导出新公开面
- [ ] verify: `cargo test -p ccr-usage -- --test-threads=1` 全绿

## B2 adapter 收敛（薄委托）

- [ ] 2.1 删 `source.rs`、`paths.rs`；mod.rs re-export ccr_usage 对应项；cli.rs import 核对
- [ ] 2.2 `capabilities.rs` 收缩：CapabilityReport::detect 委托 DbCapabilities + CLI 两键拼装；re-export FeatureKey 等；删 DB 检测原语与已迁测试；补 CapabilityReport 拼装测试
- [ ] 2.3 `db.rs` 收缩：Dashboard 薄 wrapper（10 方法委托 + shared_usage_error）；re-export QueryFilter/ReportTimezone/LogsQuery/LogsPage/build_filter；删 SQL/SqlFilter/已迁测试；补 shared_usage_error 5 变体测试
- [ ] 2.4 `queries.rs`：投影 DTO 改 re-export，表现层 DTO/to_*/max_rfc3339 留守；to_paginated_logs 签名对 ccr_usage::LogsPage
- [ ] 2.5 commands/usage.rs、claude_observer.rs、state.rs、main.rs 编译核对（预期近零改动）
- [ ] verify: `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml llmusage_adapter -- --nocapture` + `--test llmusage_no_crate_guard` + `commands::handler_registry` 全绿；`rg 'usage_bucket_30m' --type rust` SQL 仅命中 crates/ccr-usage

## B3 TUI

- [ ] 3.1 app.rs：删 UsageProviderRow/provider_row_from_shared；UsageDataset 改 TaggedProviderBreakdown；UsageLoader seam（with_loader + 默认真实 loader）；state_from_load_result 纯函数；load_usage_dataset 改 provider_breakdown_by_source
- [ ] 3.2 ui.rs：字段访问改 row.source / row.breakdown.*；现有测试改型
- [ ] 3.3 测试：state_from_load_result 6 分类；with_loader 注入状态机转移（Loaded/Empty/Unsupported/Error，tokio multi_thread）
- [ ] verify: `cargo test -p ccr-tui -- --test-threads=1` 全绿

## B4 文档 + 契约

- [ ] 4.1 根 CLAUDE.md llmusage 段落改写（无 crate 依赖、投影归 ccr-usage、升级流程）
- [ ] 4.2 adapter mod.rs 头注释补一句"投影归 ccr-usage"
- [ ] 4.3 trellis-update-spec：llmusage-provider-adapter.md 契约细化（design.md 列的 5 点）
- [ ] verify: 文档表述与 guard 测试/Cargo.toml 事实一致

## B5 全门禁 + 收尾

- [ ] 5.1 `just version-check` → `just fmt-check` → `just lint-strict`
- [ ] 5.2 `just test`（全 workspace）+ src-tauri 全量测试
- [ ] 5.3 `cd ccr-ui && bun run test:smoke -- tests/usage-dashboard-payload.smoke.test.ts` + `tests/api-facade-boundary.smoke.test.ts`
- [ ] 5.4 AC 逐条核对（prd.md 7 条）；journal；提交切分按 design.md 四段
- [ ] 5.5 task.py finish + 归档

回滚点：B1/B2/B3 各自独立提交，出回归按提交粒度回退。
