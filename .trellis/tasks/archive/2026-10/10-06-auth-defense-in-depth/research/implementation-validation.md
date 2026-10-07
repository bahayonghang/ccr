# P4 Codex / TUI 实施与验证

状态：P4_SCOPED_PASS / WORKSPACE_PENDING。当前源码的 Codex/TUI 完整包、scoped strict Clippy、格式、敏感写入与 scoped whitespace 检查均通过。最终独立检查见 `independent-check.md`，源码与收据 SHA256 见 `independent-hashes.json`。核心 API 源码与独立检查由 core owner / root 管理，见 `core-implementation.md`、`core-independent-check.md`。未提交、归档、安装或推送。

## 实施范围

- `utils.rs` 的权限帮助函数返回 `Result<()>`，使用原始文件字节版本和核心 owner-only API；Windows 不依赖 `USERNAME` 或外部命令。所有原有调用者传播或明确处理结果。registry/model-provider store 只修改必要签名和 `?`。
- Auth/OAuth 两个 Unchanged 执行入口，以及 quota-only 没有 token 更新的准备路径，验证完整身份、匹配账号和原始字节版本后加固 runtime / snapshot 元数据。planner 保持只读；冲突返回 NoOp / None 或固定错误。
- snapshot / quota token CAS 回写先按原始版本加固已有目标，再发布新字节。runtime 版本提交保持原 backup→版本校验提交次序，权限步骤位于 backup 后、payload publication 前。普通 runtime 写入与 secret store / 既有恢复分支传播权限失败，保持原补偿结构。
- rename 私有帮助函数仅注入 `fs::rename` 操作；失败时使用私有版本校验原子写。已有目标先加固，目标发布成功后才删除源。没有 `fs::copy` 写凭据再加固的回退。
- Auth JSON、tokens、profile secret、原始 auth map / runtime config、OAuth refresh request/response 使用脱敏 `Debug`。registry/account 的原始 TOML `extra` 值也隐藏；未知字段的磁盘往返仍保留。已存在的 `OAuthIdentity`、`OpenAiQuotaSnapshot` Debug 防护保留。磁盘 serde 未改变；export account 的 `auth_data` 经已脱敏的 `CodexAuthJson` 传递保护。
- HTTP 错误只含状态及五个固定诊断码：`token_invalidated`、`refresh_token_reused`、`refresh_token_invalidated`、`refresh_token_expired`、`invalid_grant`。未知 code、message、body 和成功状态中的非法 JSON 值不回显。固定 invalidated 短语在内部映射为 `token_invalidated`，保留 403 后 refresh / retry。
- TUI 只补重新登录状态的正式 EN/ZH matrix 测试及 fixture。原本地退出 / 登录固定提示保持；没有改通用布局或增加产品流程。

产品文件为 `crates/ccr-codex/src/utils.rs`、`models/codex_auth.rs`、services 的 `codex_auth_service.rs`、`codex_oauth_token_service.rs`、`codex_quota_service.rs`、`codex_runtime_service.rs`、`codex_registry_store.rs`、`codex_model_provider_store.rs`、`openai_quota_core.rs`，以及 `crates/ccr-tui/src/tui/codex_auth/ui.rs`、`tui/ui.rs`（后者仅 R5 测试）。保留 P1/P2/P3、用户及其他角色改动。

## 首败与修复记录

| 原始收据 | 结果与处理 |
| --- | --- |
| `codex-first-failure.log` | exit 101；0 passed / 7 failed。合成 marker 证明 Debug、任意 code / body 回显与 Unchanged 宽 DACL 缺口；未发现或声明实际凭据日志泄漏。 |
| `codex-p4-retest.log` | exit 0；13 passed。包括两个 Unchanged 入口、quota-only 路径、raw-version 冲突、rename 回退及 publication 前失败源保留。之后增加 403 / 非法成功响应值两项回归，随完整包通过。 |
| `tui-matrix-baseline.log` | exit 101；新测试引用私有 `app` 模块，编译错误 E0603；改为既有 Codex Auth test fixture 内的 setter。 |
| `tui-matrix-baseline-retest.log` | exit 101；0 passed / 1 failed。Chinese buffer 包含双宽字符的占位空白，直接字符串比较失败；按现有矩阵方式去除占位空白再比较。 |
| `tui-matrix-baseline-corrected.log` | exit 0；1 passed。实际 EN/ZH 六尺寸加 60x18 的重新登录状态和错误色可见；没有修复通用布局回归的声明。 |
| `codex-validation.log` | exit 101；366 passed / 1 failed / 2 原有 ignored。新增 runtime 权限 leaf lock 放在 backup 前，阻塞 P1 的 backup→CAS 回归；权限步骤移到 backup 后。 |
| `runtime-cas-corrected-retest.log` | exit 0；原 failing CAS 回归 1 passed。外部新版本保留，不回写旧备份。 |
| `codex-corrected-validation.log` | exit 101；366 passed / 1 failed / 2 原有 ignored。P3 loopback fixture 的 accepted stream 未明确切回 blocking，Windows 读取返回 WouldBlock (10035)。 |
| `loopback-blocking-corrected-retest.log` | exit 0；1 passed。accepted stream 显式 `set_nonblocking(false)`，保留既有 5s timeout、握手和默认并行。Auth/OAuth/Quota/OpenAI core 同形检索仅发现该处；core crate 同形检索未发现其他 `set_nonblocking(true)`。P3 历史 PASS 收据保持原样。 |
| `clippy-validation.log` | exit 101；5 个测试源码 lint 错误：只读权限恢复、两处多余 `as_deref`、两处未处理读取长度。修复后 `clippy-corrected-validation.log` exit 0；没有 suppress lint。 |
| `extra-debug-first-failure.log` | exit 101；0 passed / 1 failed。两个合成 `extra` marker 均出现在 Debug 中。自定义 registry/account Debug 隐藏未知值；`codex-extra-and-lint-corrected-retest.log` exit 0，16 passed。 |
| `codex-final-validation.log` | 修复 extra Debug 前的历史包门：exit 0；367 passed / 2 ignored。该收据不覆盖后续 extra Debug 改动。最终当前源码包门见下表。 |

## 当前检查

| 命令 | 结果 |
| --- | --- |
| `cargo test -p ccr-codex --all-features -- --skip export_bindings` | exit 0；368 passed / 0 failed / 2 原有 ignored；doc 0；默认并行；`independent-codex-validation.log`。覆盖最终 16 个 `p4_` 测试。 |
| `cargo test -p ccr-tui --all-features -- --skip export_bindings` | exit 0；253 passed / 0 failed / 0 ignored；doc 0；默认并行；`independent-tui-validation.log`。 |
| `cargo clippy -p ccr-codex -p ccr-tui --all-targets --all-features -- -D warnings -D clippy::unwrap_used` | exit 0；Lint / TypeCheck PASS；`independent-clippy-validation.log`。 |
| `just fmt-check` | exit 0；5 格式工具测试、11 JSON 配置文件、Rust workspace 与 Tauri Rust 格式；`independent-fmt-validation.log`。 |
| `python scripts/quality/check_secret_writes.py` | exit 0；Sensitive persistence policy check passed；`independent-secret-writes-validation.log`。 |
| scoped `git diff --check --` | exit 0；仅 LF/CRLF advisory；`independent-diff-validation.log`。 |

Rustfmt 修复命令与验证命令分开。未降低测试并行度或删除原有回归。全部新数据位于合成临时目录；所有 HTTP fixture 仅访问 loopback。

## 行为和平台边界

- Windows native consumer 探针确认 protected DACL、仅当前进程 SID 的 Allow FullControl；只检查 SID 相等与权限，不输出 SID。planner 不改变 DACL；Unchanged / quota-only 执行后 bytes 与 mtime 保持。
- stale runtime 或 snapshot 原始字节版本会返回安全跳过；错误身份的 quota source 不加固。metadata 两文件逐一执行，runtime 可能先收紧后遇到 snapshot conflict / permission error。没有两文件 ACL 事务，也不承诺错误时全部 ACL 无副作用。
- rename 的 publication 前失败保留源字节和目标旧字节。publication 后同步失败可能已经留下完整目标，源保留；源删除失败会留下完整源、目标。成功 `fs::rename` 后权限失败则可能已移动文件。最终 registry 发布失败仍可能留下文件 / registry 部分状态；P3 前置备份用于恢复。
- TUI 正式 Ratatui TestBackend 包含 EN/ZH 的 80x24、100x22、100x30、120x22、140x40、180x50 与 60x18 compact degrade；断言实际 cells / error color。状态行完整文本的固定诊断码与退出 / 登录提示由原双语回归覆盖；狭窄卡片仍按既有 clipping 合同显示省略号。没有 native interactive TUI 外观验收声明。
- Unix native 权限、真实账号 k12 / khanh、已安装二进制、真实 OAuth 服务器、Linux/macOS 均为 NOT_RUN。父任务统一运行 workspace `just lint-strict`、`just test`、`just ci`；这些门尚不能由子任务包门替代。
- Windows case-only 账号别名保护尚待用户决策，未修改该行为。外部 Codex / 编辑器不参与 CCR 锁协议。

## 审计和回滚

原始失败收据保留，不覆盖为成功日志。最终源码、原始收据与检查报告 SHA256 写入 `independent-hashes.json`；正式门以该源版本为准。源码冻结时间为 2026-10-06 20:47:41 America/Chicago（2026-10-07 01:47:41 UTC）。之后产品源码变更需要新验证收据。

源码回滚仅撤销 P4 hunks，权限 API 签名和消费者必须一起回滚，保留 P1/P2/P3 及其他改动。测试 fixture 的 blocking 修复单独可识别。没有执行 Git 回滚或真实数据写入。

数据恢复必须先验证完整账号身份，再以私有原子写恢复匹配的 P3 snapshot / registry 备份；OAuth 服务器凭据有效性不由文件恢复保证。普通 runtime 操作沿用原备份和补偿边界，权限 metadata 没有新内容补偿层；发生权限或恢复失败时可能仍有已完成的文件变化，调用者收到错误。
