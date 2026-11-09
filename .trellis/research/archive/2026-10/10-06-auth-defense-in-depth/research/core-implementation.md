# P4 Core 实施证据

状态：SCOPED_CORE_CHECK_PASS / INDEPENDENT_CHECK_PENDING / WORKSPACE_NOT_RUN。2026-10-06。

修改边界：旧 `enforce_secret_permissions_versioned` 保留 Windows DACL，无法执行 P4 owner-only 加固。新增独立带版本校验的元数据入口；已有 secret writer 与版本帮助函数的权限策略保留。文件修改限于 `atomic_writer.rs`、`guarded_write.rs` 和必要导出。本任务不修改 journal 协议、错误变体、依赖或真实凭据。

先添加 Windows native 宽 DACL 回归探针，调用旧 preserve helper 记录运行时首败。随后同一探针切换到新 owner-only API。验证 current-process token SID、protected DACL、唯一 Allow FullControl ACE、内容/mtime/file identity 保持。测试子进程使用 `env_remove("USERNAME")`，不修改宿主环境。

## 最终源码

- `crates/ccr-core/src/core/atomic_writer.rs:633` 将既有 current-process token SID 描述符构造提取为私有 `owner_only_windows_dacl`。`secret_windows_dacl` 仍为已有目标捕获原 DACL；新目标沿用原 SID 构造。没有更改既有 writer 的权限合同。
- `atomic_writer.rs:721` 增加模块内 `enforce_owner_only_windows_permissions(&File)`。从描述符读取 DACL，并以 `SetSecurityInfo` 设置已验证 HANDLE 的 protected DACL。DACL 只有当前进程令牌用户的 Allow FullControl ACE。owner/group/SACL 和文件属性不变；未使用 `USERNAME`、`icacls` 或外部权限进程。
- `crates/ccr-core/src/core/guarded_write.rs:197` 增加公开 `enforce_owner_only_permissions_versioned(path: &Path, expected_token: &str, lock_timeout: Duration) -> Result<bool>`。`core::guarded_write` 已是公开模块，无需修改导出。叶锁内打开文件、读取原始字节并比较 BLAKE3 版本。Windows 打开权限为 `GENERIC_READ | WRITE_DAC`。缺失/已读取内容版本冲突返回 false；打开、锁、读取和权限失败传播既有错误。
- Windows 在同一文件句柄上修改 DACL；Unix 复用 `secret_unix_mode`：0644 转为 0600，既有 0400/0600 保留。没有 payload 写入、替换或备份。元数据加固沿用 `write_journal::before_write` 的版本/故障边界，且不增加内容回滚条目。
- 生产改动只有上述两个文件。原 `enforce_secret_permissions_versioned` 保持原实现。没有新依赖或 `CcrError` 变体。

## 回归与原始失败

| 收据 | 命令及结果 |
| --- | --- |
| `core-first-failure.log` | `cargo test -p ccr-core --all-features owner_only_permissions_tighten_windows_dacl_without_replacement -- --skip export_bindings --nocapture`；EXIT 101，0 passed / 1 failed。测试调用旧 preserve helper，Everyone ACE 未收紧到 current-process token SID；bytes/mtime/file identity 均保持。 |
| `core-first-retest.log` | 同一 native 探针改用新 owner-only API；EXIT 0，1 passed / 0 failed。 |
| `core-metadata-retest.log` | 扩展 8 项后默认测试 EXE 启动以 `0xc0000005` / `STATUS_ACCESS_VIOLATION` 退出；无 running-tests 输出。直接 EXE `--list` 也出现相同退出。原始失败保留。 |
| `core-native-startup-noincremental.log` | 仅该命令临时 `CARGO_INCREMENTAL=0`，`cargo test -p ccr-core --all-features owner_only_permissions -- --list`；EXIT 0，列出 8 项。命令结束恢复原环境。 |
| `core-metadata-noincremental-retest.log` | 同一临时设置，`cargo test -p ccr-core --all-features owner_only_permissions -- --skip export_bindings --nocapture`；EXIT 0，8 passed / 0 failed。 |
| `core-validation-noincremental.log` | 临时设置下完整 package 测试；EXIT 0，unit 130 passed / 0 failed；doc 6 passed / 6 既有 ignored。 |
| `core-validation-default-first.log` | 恢复默认 `cargo test -p ccr-core --all-features -- --skip export_bindings`；EXIT 0，unit 130 passed / 0 failed；doc 6 passed / 6 既有 ignored。默认并行度保留。 |
| `core-clippy-validation.log` | `cargo clippy -p ccr-core --all-targets --all-features -- -D warnings -D clippy::unwrap_used`；EXIT 0。 |
| `core-fmt-validation.log` | `rustfmt --check --edition 2024`，限定两个所改文件；EXIT 0。 |
| `core-diff-validation.log` | `git diff --check --` 两个所改文件；EXIT 0。原有 LF→CRLF 提示保留，没有空白错误。 |

Windows 原生新增组包含 7 个实际操作测试和 1 个仅在子进程环境执行的入口。USERNAME 测试启动真实测试子进程，使用 `env_remove("USERNAME")` 与独立 `CCR_LOCK_DIR`；子进程确实执行 owner-only API 和 SID 检查。正常宿主运行该入口时返回，不将入口本身另算一项原生功能证明。

原生断言覆盖：protected 标志、唯一非继承 Allow FullControl ACE、ACE SID 与独立 `OpenProcessToken` / `GetTokenInformation` / `GetLengthSid` 结果相等、重复加固不替换文件、不创建备份、bytes/mtime/卷与 file index 保持、缺失/版本冲突无权限变化、叶锁超时、原生 OWNER_RIGHTS `WRITE_DAC` 拒绝传播、journal 故障/版本冲突阻止元数据修改、加固无内容 journal 条目。SID 与凭据不打印。

## 启动异常边界

Application Error 事件仅提取该 core EXE 的故障字段：模块为 core EXE，exception `c0000005`，offset `0x3b61c0`。没有写入其它应用事件或系统日志正文。

首败与恢复默认 PASS 的 `ccr_core-b15e5fcb53867454.exe` SHA-256 相同：`5d93cfa6611173a8a25ebac7c763cb46ff17aa530fe984f268bc0c24b6ef61b0`。源码身份也相同。默认命令重新编译后生成相同字节。因此启动异常原因未查明；不标注为产品 FFI 根因、编译器根因或坏增量缓存修复。此前发送的“默认 EXE SHA 变化”判断已向主会话和 consumer owner 更正。没有删除/移动 target、修改全局环境/配置或更改依赖。首次失败的源码/EXE/日志身份保存在 `core-native-startup-failure-hashes.json`；该 JSON 只记录身份，未声称保留失败 EXE 副本。

## 冻结、平台与回滚

默认完整 core、strict clippy、owned fmt/diff 完成后已通知主会话及 Codex/TUI owner 释放 Cargo。后续只写证据，无 core 源码修改。最终源码及原始收据 SHA-256 见 `core-implementation-hashes.json`；首个 old-helper 回归源码身份另见 `core-first-failure-sources.json`。

Windows native：PASS，合成临时文件与真实子进程。Unix native / Unix cross-compilation：NOT_RUN；仅维护相应测试源。最终 workspace、`just ci`、native TUI、安装的 CCR 和真实 k12 登录/额度查询：NOT_RUN，归父任务或用户后续操作。

叶锁只协调遵守相同锁协议的 CCR writer；非合作外部 writer 仍可在读取或权限操作期间修改内容或替换路径。元数据使用已打开句柄，不声称锁提供跨应用事务。Windows 打开所需 WRITE_DAC 失败可能先于版本比较并传播权限错误。保持既有平台支持范围，未做其它目标的运行证明。

源码回滚需与 Codex consumer 同步移除新 API 调用，再撤回两个 core 文件中的 P4 片段；保留其它既有修改。加固后的权限不会由无内容 journal 自动放宽。合成临时文件由测试清理；未读取或修改真实账户，真实文件没有新增恢复操作。
