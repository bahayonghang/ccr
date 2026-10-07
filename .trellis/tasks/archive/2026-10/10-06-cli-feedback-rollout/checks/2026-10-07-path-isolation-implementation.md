# 已批准的 Windows 路径隔离实施

状态：IMPLEMENTED_SCOPED_CHECKS_PASS，独立安全复查待完成。HEAD `528d4bae1b145fd74d7bbc97454caac249a4b896`；在现有展示改动上实施，未提交。

## 变更与边界

- `crates/ccr-core/src/core/logging.rs`：非空 `CCR_ROOT` 选择 `<CCR_ROOT>/logs`。unset/empty 保留 `dirs::home_dir()/.ccr/logs`。不改变 logger 初始化、filter、writer、14 天期限、权限、bridge 或 redaction。
- `crates/ccr-cli/src/managers/conflict_checker.rs`：Claude 设置文件使用 `ClaudeRuntimePaths::from_env().settings_file`；Codex 使用 `CodexConfigManager::resolve_codex_dir()/settings.json`；Gemini 依次使用非空 HOME、USERPROFILE、dirs fallback。提取字段、冲突等级、建议和读取次数保持原有实现。
- `crates/ccr/tests/commands/project_init.rs`：子进程设置临时 `CCR_ROOT`。
- `crates/ccr/tests/commands/help.rs`：`run_help` 设置临时 `CCR_ROOT`。已确认 `version_subcommand_omits_opencode_entry` 以 `version` 子命令调用该 helper，main 在 dispatch 前初始化 logger；`--help`/`--version` 在 Clap 阶段退出，单独调用不需修复。
- `crates/ccr-core/tests/log_path_isolation.rs`：隔离 test executable 子进程验证两个 logger 入口，使用合成日志和跨根哨兵。没有调用产品 `ccr.exe`。

`CARGO_BIN_EXE_ccr` 调用扫描未发现其他进入 logger 且缺少 CCR_ROOT 的根进程夹具。纯日志 fallback 单测只获取并比较路径，不读取、初始化或清理实际 fallback 目录。ConflictChecker 单测设置全部三个平台的合成路径，未读取真实配置。没有删除、恢复或迁移真实日志/账号。Windows 权限方法仍为 no-op。

## 已运行检查

| 命令 | 结果 | 原始回执 |
| --- | --- | --- |
| `cargo test -p ccr-core --lib core::logging -- --skip export_bindings` | exit 0，13 passed；2 个新增路径单测 | `2026-10-07-path-isolation-core-logging-first.txt` |
| `cargo test -p ccr-cli --lib managers::conflict_checker -- --skip export_bindings` | exit 0，5 passed；新增优先级、合成冲突只读、缺失文件检查 | `2026-10-07-path-isolation-conflict-first.txt` |
| `cargo test -p ccr-core --test log_path_isolation -- --skip export_bindings` | exit 0，2 passed；1 个 intentional ignored child probe，两项 parent 各实际执行该 probe | `2026-10-07-path-isolation-log-process-first.txt` |
| `just fmt-check` | exit 0 | `2026-10-07-path-isolation-fmt-first.txt` |
| `cargo clippy -p ccr-core -p ccr-cli -p ccr --all-targets --all-features -- -D warnings -D clippy::unwrap_used` | exit 0，包含所改 package/targets 的编译检查 | `2026-10-07-path-isolation-clippy-first.txt` |
| scoped `git diff --check` | exit 0 | `2026-10-07-path-isolation-scoped-diff-first.txt` |

两个日志子进程均设置 `CCR_LOG_LEVEL=off`。选定根中 15 天 managed 文件删除，13 天 managed 文件保留，15 天 unmanaged 文件保留，UTC daily 文件创建。相邻 CCR 根与合成 HOME 中 managed 旧日志的字节、mtime、目录项数保持不变。现有 14 天清理规则保持原样。

## 待完成

- 独立 trellis-check 核对路径与清理作用域；checker 验证后解除当前展示测试的临时 Windows ignore。
- 产品 `ccr.exe` 回归、原生终端矩阵、完整相关套件及 `just ci`：此实施阶段 NOT_RUN，交由主会话按原计划继续。
- 最终路径 spec、全局源码清单和任务验收：主会话负责。提交、推送、归档未授权。

回退只恢复上述批准源码/测试及主会话对应规范。保留当前所有首次回执，不修改真实用户数据。
