# Verification — 10-06-usage-panel-display

日期：2026-10-06。原生终端验证：NOT_RUN（仅 TestBackend 渲染与快照目视检查 EN/ZH 140×40、100×22）。

## 门禁

| 命令 | 结果 | 证据 |
| --- | --- | --- |
| `just fmt-check` | exit 0 | `fmt-check.log` |
| `just lint-strict` | exit 0 | `lint-strict.log` |
| `just test` | exit 0，41 个 test result 全部 ok | `just-test.log` |
| `cargo test -p ccr-tui` | 250 passed | 包含于 `just-test.log` |

## AC 映射

| AC | 证据 |
| --- | --- |
| AC1 | `full_usage_panel_states_cost_mode_once_and_never_truncates_the_source_url`：宽度 60/68/88 下 "Std assumed" 恰好 1 次，表格行不含计价状态词；矩阵测试断言 "Stdassumed"/"Std假设" 与 "UNPRICED" ≤1 次。 |
| AC2 | `insufficient_samples_capacity_folds_and_missing_token_classes_are_omitted`（EN/ZH）：样本不足时容量区 1 行（联合状态不同则 2 行）；分类全缺时不渲染，部分缺失时不出现 N/A。URL 行仅在宽度 ≥ 68 时出现且为完整 URL；矩阵 180×50 Estimate 断言 URL 存在，任何出现 "https://" 的尺寸断言完整 `AUTH_PRICE_SOURCE`。 |
| AC3 | `codex_auth_composed_feature_states_keep_cost_capacity_and_keys_visible`：六尺寸 × EN/ZH 断言错误、范围、partial（"partly priced"/"部分估值"）、unpriced（"UNPRICED"/"未定价"，无 "$0.00"）、联合剩余与快捷键可见；宽度 ≥140 断言 "total10M/" 与 "samples3/3·"。 |
| AC4 | 同上矩阵测试 EN/ZH 通过；`cargo test -p ccr-tui` 250 passed。 |
| AC5 | `git diff --stat` 仅 `crates/ccr-tui/src/tui/codex_auth/ui.rs`、`crates/ccr-tui/src/tui/ui.rs`；`usage_panel_data` 契约未变。 |
| AC6 | `just lint-strict`、`just test` exit 0。 |
