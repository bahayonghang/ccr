# P2 独立验收

状态：AC1–AC6 PASS；AC7 LOCAL_PASS / WORKSPACE_NOT_RUN。产品代码已冻结，正式 workspace 门由主会话执行。

日期：2026-10-06。环境：Windows，分支 `dev`，HEAD `c524ac07e77e94587399d969546ed75f13cfaf19`。角色：`trellis-check`。

## 范围与证据边界

已读取原生注入的完整 hook 输出、任务 check.jsonl、PRD、design.md、implement.md，以及 Codex、Core 与 TUI 适用规范。补读了注入超预算的 TUI 规范尾部。

审查覆盖模型、store、6 个服务命令、2 个后台写入、双向 token 同步与 TUI toast/Profile 退出顺序。另核对 CLI 包装器、桌面端 save/switch/delete/rename 包装器，以及导出 DTO 和桌面 Account DTO。未修改 P1/P3 等后续任务、父任务、公开签名或错误变体。

完整包测试与实施阶段 Clippy 结果来自 `implementation-validation.md`。审查代理未重复完整测试。本次局部修复之后，审查代理执行了定向回归与最终 scoped Clippy、格式、敏感写入和空白检查。

## 逐 AC 结论

| AC | 结论 | 实际代码与测试证据 |
| --- | --- | --- |
| AC1 | PASS | `models/codex_auth.rs:272`、`:314` 的 flatten `extra` 保留顶层和账号字段；`codex_registry_store.rs:175` 往返断言完整 TOML 表相等，覆盖嵌套表、数组、日期、浮点和 usage_ledger；`codex_auth_service.rs:2339` 验证 force 替换仅清除被替换账号的旧扩展字段，保留其他账号、顶层字段与版本。 |
| AC2 | PASS | 默认版本仍为 1.0；`codex_registry_store.rs:222` 覆盖缺失版本、1.0、1.7、主版本空白和不可解析值；`:175` load/save 保持 1.7，`:323` 保留缺失文件的既有默认值测试。 |
| AC3 | PASS | `save_current:910`、`switch_account:1182`、`delete_account:1294`、`update_account_description:1459`、`rename_account:1496/1508`、`import_accounts:1920` 在首个文件副作用前检查；`codex_auth_service.rs:2181–2259` 对 2.0/abc × 有/无快照目录比较整个临时 home 的目录集合与文件字节，覆盖 force rename 和同名 rename；只读 list/账号读取仍可用。 |
| AC4 | PASS | `codex_auth_service.rs:752`、`codex_oauth_token_service.rs:359` 仅吞掉具有固定前缀的 ConfigError，记录 warn 并返回成功，其他错误继续传播；`:2265`、OAuth service `:723` 验证注册表字节与备份集合不变；Auth service `:2301` 验证两个只读版本下 runtime→snapshot 与 snapshot→runtime 同步均继续。 |
| AC5 | PASS | `codex_registry_store.rs:94` 在模型解析失败后用 TOML 表读取版本；`:287` 覆盖 2.0/abc 与 `accounts = 42` 的升级提示；`:305` 比较支持版本、缺失版本和 TOML 语法失败的原始错误文本。 |
| AC6 | PASS，已修复 1 个局部问题 | `app.rs:38` 在 TUI 线程本地化；保存/删除/重命名/切换均接入，外层 toast 文本不变。`:1306` 的预检早于 `profile_off`。`:1736`、`:1768` 覆盖 EN/ZH × 2.0/abc/含分隔文本的非法版本，验证完整版本值与实际 switch toast，Profile/runtime/config/registry 字节保留。 |
| AC7 | LOCAL_PASS / WORKSPACE_NOT_RUN | 既有测试除空 `extra` 构造字段外未改语义；未新增 CcrError 变体；store 保留文件锁、写前备份与 `AtomicWriter::secret(true)`。完整包测试证据为 Codex 308 passed/2 ignored、TUI 252 passed；本次最终 scoped Clippy 和格式通过。`just lint-strict`、`just test`、`just ci` 在本报告写入时尚未由审查代理执行。 |

## 调用链与规范同步

- `ensure_managed_auth_supported` 仅读取凭据存储配置。save 的 auth 读取和名称验证、rename 的名称验证、import 的输入解析均无文件写入。
- switch 的版本门早于 `ensure_current_runtime_supports_openai_switch`，后者通过 `current_profile_name` 调用 `load_or_create_default`。版本门也早于 switch-out 同步、runtime commit、Profile 更新和注册表保存。
- TUI 使用无副作用的服务预检后才调用原 Profile 退出步骤。注入的退出回调仅用于测试可观测顺序，生产仍调用 `profile_off_for_platform(Platform::Codex)`。
- store.save 的兜底按设计在锁内执行；后台只读操作可以创建锁目录和空锁文件，不创建注册表备份，不改注册表/快照/runtime/config 字节。用户命令的副作用前检查不触发该锁基础设施。
- CLI 包装器在服务写命令之前只有读取、进程检测、用户输入和输出。桌面端 save/switch/delete/rename 直接调用受保护服务，后续缓存与托盘更新在服务成功后执行。
- `export_accounts` 使用 `CodexAuthExportAccount` 显式字段映射；桌面 `CodexAuthAccountMetadata::from` 使用显式字段映射。未知字段不会进入这两个 DTO。没有模板、平台生成配置或绑定导出变更。
- 主会话新增的 backend spec 已覆盖未知字段保留、主版本规则、命令预检、后台跳过、双向同步、TUI Profile 退出前预检及所需测试。规范与最终代码一致。

## Findings (fixed)

- 文件：`crates/ccr-codex/src/services/codex_registry_store.rs:45`。
- 问题：`registry_read_only_version` 使用 `split_once(" 由")`。版本值为 `abc 由future` 时，服务错误包含完整值，提取结果只有 `abc`，TUI 也显示截断值。
- 修复：以最后一个完整固定提示分隔文本 ` 由更新版本的 CCR 写入` 提取版本，保持服务消息格式和公共签名。
- 回归：在既有 store 往返测试与 TUI EN/ZH、本地化 switch toast 测试中增加 `abc 由future`。两个定向测试先失败，分别断言 `Some("abc")` 和英文 toast 中的 `abc`；修复后通过。

## Findings (not fixed)

无未修复的范围内产品问题。workspace 正式门属于待执行验证，不计为产品缺陷。未扩展到已规划的身份、备份策略、权限、快照命名或导入导出行为变更。

## Verification

| 审查代理执行的命令 | 结果 | 退出码 |
| --- | --- | --- |
| `cargo test -p ccr-codex --all-features registry_read_only_message_preserves_version_in_display_errors -- --skip export_bindings`（补测试后、修复前） | FAIL：版本截断为 abc；0 passed、1 failed | 1 |
| `cargo test -p ccr-tui --all-features registry_read_only_errors_are_localized_with_original_version -- --skip export_bindings`（补测试后、修复前） | FAIL：英文 toast 版本截断为 abc；0 passed、1 failed | 1 |
| `cargo test -p ccr-codex --all-features registry_read_only_message_preserves_version_in_display_errors -- --skip export_bindings`（修复后） | PASS：1 passed、309 filtered | 0 |
| `cargo test -p ccr-tui --all-features read_only -- --skip export_bindings`（修复后） | PASS：2 passed、250 filtered | 0 |
| `cargo clippy -p ccr-codex -p ccr-tui --all-targets --all-features -- -D warnings -D clippy::unwrap_used` | PASS：最终 Codex/TUI 与 ccr-cli 依赖通过 Rust 类型检查和严格 lint | 0 |
| `just fmt-check` | PASS：JSON 5 tests/11 files；workspace 与 Tauri cargo fmt | 0 |
| `python scripts/quality/check_secret_writes.py` | PASS | 0 |
| `git diff --check -- crates/ccr-codex crates/ccr-tui` | PASS | 0 |

Lint：PASS（scoped）。TypeCheck：PASS（scoped，最终 Clippy）。Tests：PASS（定向）；完整包通过证据来自实施代理，发生在本次文本提取修复之前。

## NOT_RUN 与限制

- NOT_RUN：审查代理未执行 `just lint-strict`、`just test`、`just ci`；主会话执行这些门之后才能关闭 AC7 与跨 crate 最终验收。
- NOT_RUN：Unix-only 权限测试，本机为 Windows。
- NOT_RUN：真实账户、在线 quota/refresh、原生终端视觉、个人凭据目录行为。全部新增夹具使用临时目录与合成凭据。
- IGNORED：实施完整 Codex 测试保留既有两个 benchmark ignore，不改覆盖规则或默认并行度。
- 此保护仅适用于包含本修复的 CCR 版本；已发布旧版本仍可能丢弃未知字段。

审查代理只新增本报告，并修改上述版本提取函数与 3 个既有测试的输入矩阵。未提交、未归档、未 push。
