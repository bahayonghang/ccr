# Auth 加密导出入口评估

状态：CURRENT_SOURCE_REVIEWED；P6 已保留现有加密导出合同。日期：2026-10-06。导出格式、密码生命周期和公共 DTO 未改变。

| 入口 | 现有行为 | 本轮建议 |
| --- | --- | --- |
| CLI `codex auth export` | 包含凭据时调用 `export_accounts_encrypted`；`--no-secrets`仅输出元数据 | 保留现有强制加密默认与加密信封格式 |
| Codex Auth TUI | 没有Auth导入/导出入口 | 不新增菜单或密码交互 |
| Tauri | 明文bundle/对象/数组导入；没有Auth加密导出API | 不修改typed IPC/密码DTO；独立记录逐项提交边界。P6 服务层的 typed bundle 预检同样拒绝已知身份冲突；Tauri 既有逐项对象/数组提交边界保持不变 |
| 服务 `export_accounts(include_secrets)` | 内部API可明确选择是否带原始auth数据，加密导出API复用相同源 | 不把内部明文可选API误标为CLI默认；保持已有调用形状 |

CLI已经采用含凭据强制加密策略，无需以“改默认”名义新增明文flag或重复加密实现。P6 修正 plaintext import 的错误传播：服务拒绝现在返回 CLI `Err` 和非零退出状态；encrypted import 继续解密后复用同一服务预检。若未来增加桌面/TUI加密入口，需要明确密码内存持有、取消、错误脱敏、Secret DTO和typed IPC合同；这些不是现有默认值切换。

源码依据：`crates/ccr-cli/src/commands/codex/auth/export.rs:164` 的 `include_secrets` 分支与 `:172` 的 encrypted export 调用；`crates/ccr-cli/src/cli/subcommands/codex.rs:432` 的 `no_secrets` 参数；`crates/ccr-codex/src/services/codex_auth_service.rs:1965` 的 encrypted export、`:2009` 的 encrypted import 与 `:2029` 的共同 import 入口。Tauri bundle 分支位于 `ccr-ui/src-tauri/src/commands/codex_auth.rs:1077`；对象/数组循环位于 `:1444`，每项通过 `:1054` 调用服务层。P6 最终产品源码在 `2026-10-07 02:33:43 UTC` 冻结，SHA256 见 `implementation-validation.md`。

验证：已执行 `cargo test -p ccr-codex -p ccr-cli --all-features p6_ -- --skip export_bindings`，13 个 Codex 回归与 1 个 CLI 回归通过，原始回执为 `owner-final-targeted.log`。该回执先于审阅代理追加两条认证方式冲突用例；追加后既有矩阵单测与格式检查通过，详见 `implementation-validation.md`。其中 `p6_import_encrypted_conflict_uses_identical_preflight` 使用临时目录和合成密码，确认 encrypted import 进入同一冲突预检；`p6_plaintext_import_returns_service_rejection` 确认 plaintext service Err 返回非零错误码。该 CLI 测试调用内部命令函数，未执行安装后的 CLI 二进制。

NOT_RUN：真实账户加密导出、用户文件导入、TUI/桌面新增入口、公共协议迁移和托管 CI。合成回归不证明真实凭据可恢复或网络端点可用。
