# 初始化建议断言的 CI 修复

状态：SCOPED_RETEST_PASS。仅修改 `crates/ccr/tests/commands/help.rs` 的既有断言，保留其他工作区改动。

父回执 `2026-10-07-isolation-just-ci-first.txt` 保留首个正式失败：root commands 138 passed、1 failed；失败为 `help::initialized_ccr_init_output_does_not_recommend_retired_platform_init`。原断言要求 `profile init`，当前已批准的 lifecycle/init 展示只推荐平台列表，原始输出包含分组后的 `下一步`、`查看平台列表`、`ccr platform list`。

断言改为匹配该完整分组与命令行，继续检查输出不含 retired `ccr platform init`。没有修改生产代码、退出码或夹具行为。

| 验证命令 | 结果 | 原始回执 |
| --- | --- | --- |
| `cargo test -p ccr --all-features --test commands help::initialized_ccr_init_output_does_not_recommend_retired_platform_init -- --exact --skip export_bindings` | exit 0；1 passed、0 failed、0 ignored，默认并行 | `2026-10-07-gate-help-init-retest.txt` |
| `rustfmt --edition 2024 --check crates/ccr/tests/commands/help.rs` | exit 0 | `2026-10-07-gate-help-init-fmt.txt` |
| `git diff --check -- crates/ccr/tests/commands/help.rs` | exit 0 | `2026-10-07-gate-help-init-diff.txt` |

完整 `just ci` 复测由主会话负责。首失败保留，不将局部复测作为完整 CI 通过。未提交、推送或归档。
