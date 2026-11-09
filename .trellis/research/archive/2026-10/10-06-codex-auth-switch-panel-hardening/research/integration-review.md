# 集成复核：Codex Auth 切换可靠性、用量面板与实现加固

复核日期：2026-10-06。复核基线：dev @ ab67bf68（三个子任务均已提交并归档）。

## 子任务提交

| 子任务 | 实现提交 | 归档提交 |
| --- | --- | --- |
| 10-06-auth-switch-reliability | f4bb26d6、ee78fd48、66a27ed1（spec） | e90eeb65 |
| 10-06-usage-panel-display | 1fe6984d、ab699a09（spec） | 7f6f7164 |
| 10-06-reference-hardening | cb56de7d、eb003ce2、3c044536、cc8d4006（spec） | e79d36cd |

## 跨子任务验收

| AC | 结论 | 证据 |
| --- | --- | --- |
| 三个子任务验收通过并归档 | 通过 | `.trellis/tasks/archive/2026-10/` 下存在 3 个子任务目录 |
| 子任务 1 不改面板数据契约 | 通过 | ee78fd48 在 `ccr-tui/src/tui/codex_auth/ui.rs` 只改 `quota_status_line` 的需重新登录提示；`app.rs` 只增加 load/reload 观测点同步调用 |
| 子任务 2 不改 ccr-codex 领域服务 | 通过 | 1fe6984d 只改 `crates/ccr-tui/src/tui/codex_auth/ui.rs`、`crates/ccr-tui/src/tui/ui.rs` |
| 子任务 3 无用户可见行为变化 | 通过（附说明） | 保留池仍为 10（`backup_retention_pool_keeps_ten_distinct_versions`）；配额错误文本对既有不 panic 的输入逐字节相同；需行为变更的发现记录为 P1–P6 后续任务提案（子任务 3 research/codex-auth-audit.md）。说明：`save_current` 发生 I/O 失败时，toast 前缀「保存失败：」不变，内层原因文本改由 AtomicWriter 生成（`crates/ccr-tui/src/tui/codex_auth/app.rs:1056-1060`）；仅影响失败路径，判定为类别 A |
| `just lint-strict`、`just test` 通过 | 通过 | lint-strict：`Sensitive persistence policy check passed`、`✅ 严格 Clippy 检查通过`；just test：rc=0，41 组 `test result: ok`，无 FAILED |
| Codex Auth TUI EN/ZH 与尺寸矩阵 | 通过 | `codex_auth_composed_layout_matrix_preserves_scope_quota_and_errors`、`codex_auth_composed_feature_states_keep_cost_capacity_and_keys_visible`、`relogin_quota_error_shows_local_logout_hint_in_both_languages` 均 ok；`tui::codex_auth::*` 39 个测试全部 ok |
| 未推送、未创建 PR、未发布 | 通过 | `dev...origin/dev [ahead 13]`；未执行 push / PR / release |

## NOT_RUN

- 3 个 `#[cfg(unix)]` 权限测试（`account_snapshot_backup_is_owner_only`、`save_writes_owner_only_registry`、`secret_runtime_and_backup_files_are_owner_only`）：复核机为 Windows，需 Linux 或 macOS CI。
- `just ci`。
- 真实账号的切换与外部登录序列（PRD Out of Scope）。

## 遗留

- 子任务 3 后续任务提案 P1–P6，未创建任务。
