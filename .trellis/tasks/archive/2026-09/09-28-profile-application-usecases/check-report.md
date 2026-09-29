# T02 独立检查报告

日期：2026-09-28。检查角色：`/root/check_t02`。任务：`09-28-profile-application-usecases`。

**结论：T03 所需的 apply/update/outcome、补偿、rename 和三端提交语义契约通过独立检查。T04 的只读验证前置契约仍通过。** 本结论限定于下列证据，不代表父任务、完整 `just ci`、完整生成物门禁或原生交互验收通过。没有修改任务生命周期、提交、推送或归档。

使用 `trellis-check` 规范执行独立检查；测试程序启动异常时使用 `ccr-gate-recovery` 定位最窄失败。实现者交接前只读，交接后按主代理授权修复 T02 局部问题。既有用户 `.tmp` 文件保持原样；保留其他任务既有修改。T01 repository 仅按授权等价委托共享路径 helper。

## Findings (fixed)

### C02-01：发布后的失败未登记补偿

- 文件：`crates/ccr-core/src/core/atomic_writer.rs:213`、`crates/ccr-core/src/core/guarded_write.rs:225`。
- 问题：原 journal 只在整次 atomic write 成功后登记。文件替换成功之后，目录同步等步骤仍可失败，已发布内容会遗漏于补偿集合。
- 修复：实现者加入内部 `write_with_commit` publication callback；物理替换后立即登记，再执行可能失败的后置步骤。
- 独立验证：Windows 和 Linux 的 `journal_restores_post_publish_failure` 均通过，断言发布可见但返回失败后，rollback 恢复原字节。

### C02-02：删除后的补偿未保留原文件权限

- 文件：`crates/ccr-core/src/core/atomic_writer.rs:50`、`crates/ccr-core/src/core/write_journal.rs:18`。
- 问题：仅保存字节不能恢复删除前的 Unix mode、Windows DACL 和 read-only 属性。
- 修复：实现者以不可 Debug/Serialize 的内存 `Preimage` 保存内部 `FileMetadata`。恢复临时文件先设置权限/DACL，再写 payload；Windows 发布后恢复 read-only。
- 独立验证：Windows DACL/read-only 删除恢复、Linux 0400 删除恢复通过；guarded backup 和 atomic writer 回归均通过。

### C02-03：非激活编辑的警告文案和 TUI 本地化不一致

- 文件：`crates/ccr-cli/src/application/profile_lifecycle.rs:69`、`crates/ccr-tui/src/tui/profile_backend.rs:9`。
- 问题：inactive edit 的附属记录失败也会产生 warning；原文案无条件称已激活。TUI 直接消费领域英文文本会绕过现有双语呈现。
- 修复：实现者把领域警告改为已提交操作的中性描述；TUI 以状态和 `activation_committed` 选择本地化 toast。
- 独立验证：inactive edit 保持原计数且不声称激活；英中 warning toast 均包含避免重复激活提示。React 实际 hooks 覆盖 warning/recovery、refetch 和不重复 activation。

### C02-04：Windows 路径别名绕过 guarded leaf lock

- 文件：`crates/ccr-core/src/core/guarded_write.rs:308`、`:326`；`crates/ccr-core/src/core/write_journal.rs:40`；`crates/ccr-config/src/managers/config/repository.rs:76`。
- 问题：交接版 journal 已统一 ordinary/verbatim/case 路径，叶锁仍对原始 absolute 路径哈希。两个别名可取得不同叶锁；rollback 也可能绕过外部 writer 已持有的锁。
- 红例：新增 `guarded_write_obeys_windows_alias_leaf_lock` 和 `journal_rollback_obeys_windows_alias_leaf_lock`，旧叶锁算法 **0 passed / 2 failed**。第一个断言别名 writer 必须超时且不改变字节；第二个通过通道控制 rollback 并断言持锁期间不能完成。
- 本检查者修复：将 T01 已有组件级路径规范化等价提取到 core 的 `normalized_resource_path`。repository、Windows journal 和 Windows 叶锁共同使用；保留原非 Windows 叶锁哈希及 stem 算法，不解析 symlink，不要求目标存在。
- 绿例：**2 passed / 0 failed**。完整 Windows journal 13 项、T01 repository 的普通/verbatim/UNC 身份、真实 alias 多进程和备份策略回归通过。Linux 最终源码复验通过。
- 证据：`check-alias-leaf-red.{json,log}`、`check-alias-leaf-green.{json,log}`。

### C02-05：提交后的详情读取可能把成功降为普通错误

- 文件：`crates/ccr-cli/src/application/profile_switch.rs:33`、`crates/ccr-cli/src/application/profile_lifecycle.rs:226`、`ccr-ui/src-tauri/src/commands/claude_profiles.rs:111`。
- 修复：实现者将展示详情读取改为 best effort，将必要路径解析前移。Claude mutation DTO 返回标识和 outcome，不再提交后重读完整 profile，不返回原始凭据。
- 独立验证：三 adapter 的附属失败及同 operation ID 重放通过；源 profile 删除后重放仍可读取已提交结果。DTO/Debug/tracing 的合成 secret sentinel 断言通过。生成后的 Grok action DTO 包含 `ProfileOutcome`，前端 type-check 通过。

### C02-06：CLI 终端呈现缺少实际输出断言

- 文件：`crates/ccr/tests/commands/claude_profile.rs:369`。
- 问题：共享领域 contract 不能单独证明 CLI 的真实 stdout 和退出状态。
- 本检查者修复：新增 `claude_profile_switch_reports_committed_history_warning`。测试仅使用临时 home/CCR root 和合成 profile，以 `data.db` 目录触发历史存储失败，运行实际 `ccr claude profile switch proxy` 二进制。
- 独立验证：**1 passed**。命令 exit 0；stdout 明确已生效及勿重复激活，不声称历史已记录；runtime、registry、current 指向 proxy；计数仅增加一次；只有一个 operation record，记录 `activation_committed=true` 和 `history_failed`；stdout/stderr 无原始 token。
- 证据：`check-cli-warning-output.{json,log}`。

### 同步的规范

- `.trellis/spec/ccr-core/backend/atomic-writer.md`：共享资源身份 owner、Windows 实际别名争用测试、journal 与叶锁一致；移除已完成的 Gemini/Droid 直接写入债务条目。
- `.trellis/spec/ccr-config/backend/profile-repository.md`：repository 等价委托 core 身份 helper。
- `.trellis/spec/ccr-cli/backend/profile-off-login-prep.md`：TUI auth switch 仍以 off 错误中止；profile apply 不再独立执行 off。
- `.trellis/spec/ccr-tui/backend/backend-guidelines.md`：Grok/TUI 经过 application adapter，成功计数、历史及 outcome 呈现归属明确。

## AC 与下游依赖

| 验收 | 独立证据 | 结论范围 |
| --- | --- | --- |
| AC1 | 三 adapter 的 disabled/deleted/invalid preflight、prepare 后删除、每个 declared write 故障、post-publication 故障、外部新版本保留；Windows/Linux journal 元数据与 CAS 回归 | 相关契约通过；不声称 OS 多文件事务 |
| AC2 | 相同 fixture 的 CLI application、TUI backend、desktop service 测试；相同 ID 重放不重复计数/历史，源删除后仍可重放；真实 CLI 单次 warning 操作 | 相关契约通过 |
| AC3 | 真实 CLI stdout/exit；TUI 英中 presenter；React hooks 的 warning/recovery 和不重复 activation；secret sentinel | 相关契约通过；native WebView 和交互终端未验收 |
| AC4 | production 调用边界 guard；CLI/Tauri 一个同步 worker 包含完整 journal 生命周期，TUI 直接调用共享 owner；共享用例没有终端 I/O/exit | 相关契约通过 |
| AC5 | Claude/Codex rename 同一 harness 经 application 和实际 desktop update helper；逐写故障、current/default、新旧名、secret store、unknown TOML datetime、reserved name、inactive edit | 相关契约通过 |

必要 cleanup 仍由平台 apply 的 owner 完成。早期独立 67 项平台测试包含 Claude `test_subscription_profile_apply_clears_only_ccr_managed_overrides` 和 Grok `third_party_official_third_party_round_trip_keeps_entry_state`；Codex 18 项包含 bearer roundtrip、official overlay 与 provider env-key 转换。最终三端 harness 再次验证共同持久状态。没有把取消独立 TUI off 当作取消这些规则。

**T03 可在这些已验证契约上迁移 adapter。** 保留 `apply_profile(ApplyProfileRequest)` / `update_profile(platform, old, target, patch)` 责任边界；不要在调用层重组 off/save/delete/apply，也不要把提交后的警告转为普通未生效错误。T04 scoped 前置通过结论见 `check-checkpoint.md`。任务状态由主代理处理。

## Verification

下表记录实际执行结果。零匹配 integration target 不计入通过数。`guarded_write` 的 Windows 26 项包含一个 journal alias 测试，与 13 项 journal 有重叠。

| 命令或范围 | 结果 | 日志前缀 |
| --- | --- | --- |
| Windows core `alias_leaf_lock`，`--features test-support` | 红 0/2；绿 2/2 | `check-alias-leaf-{red,green}` |
| Windows core `write_journal`，`--features test-support` | 13 passed | `check-journal-alias-final` |
| Windows core `guarded_write`，`--features test-support` | 26 passed | `check-guarded-alias-final` |
| Windows core `atomic_writer`，`--features test-support` | 11 passed | `check-atomic-alias-final` |
| Windows `ccr-config repository_tests` | 20 passed / 1 ignored child helper；三个跨进程父测试通过 | `check-repository-alias-final` |
| Windows `ccr-cli profile_contract` | 9 passed | `check-cli-alias-final` |
| Windows `ccr-tui profile_backend` | 8 passed | `check-tui-alias-final` |
| Windows desktop `application_`，`.cargo/tauri-ci.toml` | 10 passed（9 项 T02 + 1 项 ACL）；跳过 export_bindings | `check-desktop-alias-final` |
| 实际 CLI warning 二进制测试 | 1 passed | `check-cli-warning-output` |
| root 的 Linux MSRV 1.95 core journal，test-support | 9 passed；最终五文件指纹前后相同 | `root-linux-write-journal-faults-final-alias` |
| root 的 Linux core guarded write | 24 passed；最终五文件指纹前后相同 | `root-linux-guarded-write-final-alias` |
| root 的 Linux core atomic writer | 9 passed；最终五文件指纹前后相同 | `root-linux-atomic-writer-final-alias` |
| root 的 Linux repository | 17 passed / 1 ignored child helper；两个跨进程父测试通过 | `root-linux-repository-final-alias` |
| `bun run test:smoke -- tests/profiles tests/api` | 33 files / 171 passed；随后修改仅为 Rust helper、测试和规范 | `check-frontend-contract-final` |
| `bun run type-check` | pass | `check-frontend-types-final` |
| `bun run test:i18n` | 24 passed；4506 leaf keys | `check-i18n-final` |
| `bun run lint:style` / `bun run check:style-lines` | pass / pass | `check-style-final` / `check-style-lines-final` |
| `bun run lint:ci` | fail：两个受保护用户 `.tmp` 文件的 5 条 no-console | `check-frontend-lint-final` |
| `CARGO_NET_OFFLINE=true just lint-strict` | pass；workspace/all-targets/all-features，warnings/unwrap denied，secret-write guard pass | `check-lint-strict-final` |
| 新增 CLI 测试后 `cargo clippy -p ccr --all-targets --all-features -- -D warnings -D clippy::unwrap_used` | pass | `check-root-clippy-final` |
| Tauri strict clippy | 复用独立 check_t11 的通过证据；没有重复运行 | `../09-28-desktop-control-oauth-lifecycle/check-clippy.log` |
| `just fmt-check` / `git diff --check` | pass / pass | `check-format-final` / `check-diff-final` |

所有本检查者启动的命令会话均已退出。测试使用 `--offline --locked` 和现有 `--test-threads=1` 直接测试约定；不改项目并行正式门禁。桌面命令使用命令局部 `CCR_SKIP_ICON_GENERATION=1`。本检查者没有运行 DTO export/normalize；完整生成窗口仍由 check_t11 主持。

## Findings (not fixed)

1. **正式 frontend lint 仍失败。** `ccr-ui/.tmp-desktop-probe.mjs` 有 3 条、`ccr-ui/.tmp-insights-visual.mjs` 有 2 条 `no-console`。两文件是受保护的既有用户文件，不属于 T02；没有修改、删除或排除。分项 style/type-check 通过不能替代正式 lint 通过。
2. **完整 bindings guard 尚未由本报告授予通过。** Grok outcome 类型及 T02 frontend 编译已检查；整体生成物差异和最终 guard 归 check_t11/T10。没有以手动改生成物或跳过受影响项目解除该门禁。
3. **Windows 测试程序有一次启动异常，原因未查明。** 绿例首次编译后 executable 返回 `0xc0000005`，直接 `--list` 和 `--nocapture` 也失败。之后 `--list` 和原 cargo 命令通过，没有清共享 target、改 global wrapper 或增加 nocapture 来掩盖失败。失败时未采集 exe SHA，不能声称失败/成功文件哈希相同。成功 `--list` SHA 为 `a71b52c1f7a6058fdd47d73da6db1bbe726ce1642d32e0e655cc8a315344866d`。保留 `check-alias-leaf-startup-failure.{json,log}` 和 `check-core-startup-list-retry.json`。
4. **验证边界：** 没有执行完整 `just ci`、macOS、真实账户、native WebView 或交互 TUI 验收。React 使用合成 IPC，真实 CLI 测试使用隔离 home。备份轮换不参与 journal rollback；进程中断仅保留恢复标记；非合作进程及 symlink 别名不获得全局事务保证。这些是已公开范围，不作为已测试成功。

没有剩余阻塞 T03 前置行为的已证实 T02 局部缺陷。完整交付门禁和原生验收仍由父任务跟踪，不能仅凭 scoped 结论完成父任务。

## 最终核心源码指纹

下列 SHA-256 与 root 最终 Linux 复验前后指纹一致。完整记录为 `check-final-source-fingerprints.json`。

| 文件 | SHA-256 |
| --- | --- |
| `Cargo.lock` | `ca1f03e85b5e0d888109ac8af7e7943fc1ceecd20a0f6eeb6a60052c5155db79` |
| `crates/ccr-core/src/core/atomic_writer.rs` | `54d4b2773e9f386a773b8aee9c59bec96895514f9608588caec7d372e2cffbd4` |
| `crates/ccr-core/src/core/guarded_write.rs` | `5c7dc32c51340360a2cffb5019821aa134ff94ebbc24b3069ec34aaff92ae3cc` |
| `crates/ccr-core/src/core/write_journal.rs` | `8d57b9a1f307acb1fc75636014dd78b8b8978142d5537610b81cd449d6f730a5` |
| `crates/ccr-config/src/managers/config/repository.rs` | `be8c61e0a0cc03748457d694e911e458e0106e0287125d49176aae5f109643cb` |
