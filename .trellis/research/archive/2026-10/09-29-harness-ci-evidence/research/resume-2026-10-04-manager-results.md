# T07 managers 续作实施与局部验证

状态：批准范围已实施，6 个 Windows 局部门槛通过。父任务聚合门槛和独立复核由主会话执行。历史失败原因仍未查明。

## 变更与输入核对

- 当前 HEAD：`c50716b5ed1bcd1c5b4950a746a8621258d0e0b5`。历史 exact/suite 诊断 HEAD 为 `fe9d65972376dbb4c332fde9860c1e42ff2f2005`。两次提交不同。
- 应用前 14 个诊断输入的当前字节哈希与历史 exact/suite 的 before/after 四份清单全部相同。该结论只覆盖 14 个文件，不能扩展为整个仓库或 HEAD 未变化。
- 已重算批准候选 SHA-256：`a7082c421018a9b54167b3069ec3000cc21210bdebe3ca135cf6d7f52e775738`。`git apply --check` 退出 0，随后应用原候选。
- `crates/ccr/tests/managers/general.rs` 新增 5 个 `let _env = crate::setup_ccr_test_env();`，`legacy_registry.rs` 新增 1 个 `let _env = setup_ccr_test_env();`。六个绑定均位于用例第一行，持有现有 RAII guard 至用例结束。其他用例、断言、生产锁逻辑、fixture 源码不变。
- `.gitignore` 新增精确锚定规则 `/.trellis/tasks/09-29-harness-ci-evidence/research/tauri-advisory-commands.json`。文件保留，未解析或输出内容。大小为 1,164,069 bytes。
- 忽略规则应用前后 JSON SHA-256 相同：`fd50a44aab9e7d9ab269425ceaa6c691004db9390147cf3be9ad273c7d43bd96`。`git check-ignore -v` 退出 0，命中 `.gitignore:108` 的新增精确规则。

输入核对见 `resume-2026-10-04-manager-baseline.json`；应用及忽略核对见 `resume-2026-10-04-manager-ignore.json`。

## 局部门槛

环境：Windows 11 本机；Cargo `1.98.0 (797e8a9bc 2026-08-05)`；rustc `1.98.0 (88d9e12ae 2026-08-18)`。执行日期为 2026-10-04，所有回执记录实际 UTC 开始/结束时间。

| 回执后缀 | 实际命令 | 退出码 | 结果 |
| --- | --- | --- | --- |
| `exact` | `cargo test -p ccr --all-features --test managers legacy_registry::legacy_registry_fields_load_without_becoming_clean_write_routing_truth -- --exact --skip export_bindings` | 0 | 1 passed / 0 failed / 16 filtered |
| `suite` | `cargo test -p ccr --all-features --test managers -- --skip export_bindings` | 0 | 17 passed / 0 failed |
| `platforms` | `cargo test -p ccr --test platforms -- --nocapture --skip export_bindings` | 0 | 35 passed / 0 failed |
| `commands` | `cargo test -p ccr --test commands -- sync_content --nocapture --skip export_bindings` | 0 | 3 passed / 0 failed / 114 filtered |
| `default` | `cargo test -p ccr --test managers -- --nocapture --skip export_bindings` | 0 | 17 passed / 0 failed |
| `clippy` | `cargo clippy -p ccr --all-targets --all-features -- -D warnings` | 0 | PASS；测试计数不适用 |

每个命令对应 `resume-2026-10-04-manager-<后缀>.json`、`.stdout.log` 和 `.stderr.log`。JSON 保存 argv、OS、开始/结束 HEAD、时间、退出码、实际测试计数、原始日志大小和 SHA-256。每个命令执行前后 14 个输入哈希相同。

runner 为 `resume-2026-10-04-manager-run.py`。仅为 Cargo 子进程复制的环境设置 `CCR_ROOT`、`CCR_LOCK_DIR` 和 `RUSTUP_AUTO_INSTALL=0`；未设置全局或系统环境，未设置线程覆盖，未加 serial 注解，未重试。六个命令的临时夹具清理均完成。测试保持默认并行和 `--skip export_bindings`。

三个受改源码/规则文件及新 runner 的 scoped `git diff --check` 退出 0。最终源码差异为 general +5、legacy_registry +1；`.gitignore` 增加一条规则及分隔空行。

原始 `.log` 文件受仓库既有 `*.log` 规则忽略，本子代理未暂存或强制加入。原始日志与 JSON 均保留本机；交付记录需要由主会话明确处理日志纳入范围。

## 验收边界

本子代理未运行 `just version-check`、`just fmt-check`、`just lint-strict`、`just test`、governance 或完整 `just ci`；上述入口由主会话负责。未修改 task/prd/design/implement/approval/spec，未提交、归档或推送。

新证据证明批准六行的当前局部门槛通过，不能证明旧失败的精确清理交错或根因。原 CI 失败、历史诊断和独立审查证据全部保留。

macOS、当前最终 SHA 的 hosted CI、fresh-client hook/trust 属于独立证据边界，本子代理未运行。T07 与父任务完成状态仍由正式门槛和独立复核决定。
