# 验证记录：10-06-auth-switch-reliability

## 自动化（2026-10-06，Windows 11，本机）

| 命令 | 结果 | 日志 |
| --- | --- | --- |
| `just fmt-check` | PASS | checks/fmt-check.log |
| `just lint-strict` | PASS | checks/lint-strict.log |
| `just test` | PASS（exit 0，41 个 test result 全部 ok） | checks/just-test.log |

## 验收映射

| AC | 测试 |
| --- | --- |
| AC1 | `codex_auth_service::tests::switch_writes_rotated_outgoing_tokens_to_snapshot`、`newer_snapshot_of_current_account_is_written_back_to_runtime`、`failed_switch_keeps_runtime_and_registry_consistent` |
| AC2 | `sync_does_not_write_when_tokens_match`（字节与 mtime 断言）、`e2e_rotation_observed_then_external_login_then_switch_back_succeeds`（runtime 路由 + 刷新后快照同步）；同步失败不阻断：`sync_runtime_with_saved_account_best_effort` 记 warn 并返回 NoOp（代码走查，无专门测试） |
| AC3 | `e2e_invalidated_snapshot_token_is_repaired_from_newer_backup`、`e2e_rotation_without_observation_reports_relogin_and_keeps_snapshot`、`openai_quota_core::tests::should_repair_tokens_covers_permanent_refresh_failures`、`relogin_marker_wraps_only_permanent_refresh_errors_once` |
| AC4 | `codex_oauth_token_service::tests::test_repair_does_not_overwrite_newer_snapshot_with_older_source`；备份共享池边界见 research/switch-failure-analysis.md §6 W2 |
| AC5 | `sync_identity_rules_never_write_another_account`、`sync_with_duplicate_account_id_targets_current_then_latest_used`、`newer_snapshot_of_non_current_account_is_left_alone` |
| AC6 | research/switch-failure-analysis.md；stub 场景 (a) `e2e_rotation_observed_then_external_login_then_switch_back_succeeds`，(b) `e2e_rotation_without_observation_reports_relogin_and_keeps_snapshot` |
| AC7 | registry/快照格式未变；新增日志只输出账号名与上下文标签；`just test` 含 10-05 既有测试 |
| AC9 | `ccr-cli application::auth_off::tests::codex_file_off_syncs_rotated_runtime_tokens_before_delete`、`ccr-tui codex_auth::ui::tests::relogin_quota_error_shows_local_logout_hint_in_both_languages` |

## 原生手工验证

| 项 | 状态 |
| --- | --- |
| 真实账号：codex 轮换后 TUI 切换，换出账号快照持有新 token | NOT_RUN |
| 真实账号：活动账号配额刷新后 codex 继续可用（无 refresh_token_reused） | NOT_RUN |
| 真实账号：TUI `o` 本地登出后 `codex login` 新账号，旧账号快照仍可切回 | NOT_RUN |

原因：需要真实 OpenAI 账号与网络，且 refresh_token 吊销不可逆；本任务不操作用户凭据。
