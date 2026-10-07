# P1 实施与验证

状态：IMPLEMENTED_LOCAL_CHECKS_PASS，等待主会话独立 Trellis check。日期：2026-10-06。工作区 HEAD：`c524ac07e77e94587399d969546ed75f13cfaf19`。P2 的现有未提交改动保留。

## 实施结果

- 共享 `OAuthIdentity` 读取 id/access JWT 顶层及 `https://api.openai.com/auth` 下的 `chatgpt_user_id`；不同有效用户 claims 冲突时返回未知身份。账号上下文优先 `tokens.account_id`，其次 access/id claims。不使用 `sub` 或 email 推导用户。JWT 解码只用于本地关联。
- 注册表增加可选 `identity_key`，缺失字段默认为 None，保存 None 时省略。账号 Debug 与身份对象 Debug 隐藏新增身份值。`CurrentAuthInfo`、CLI/Tauri DTO 和导出 DTO 字段保持原合同。
- 读取和规划只从各账号自有快照临时推导身份。执行观测点补全该账号的缺失键；缺失或无效快照保持 None。既有键与自有快照冲突时跳过关联，诊断不含键。
- runtime 匹配、current_auth 对账、双向同步、切换前写回和 repair 候选使用完整身份。相同完整身份别名依次选择 current_auth、最近 last_used、既有有序集合顺序。规划读取选中快照后再次检查身份；快照写入入口另行检查身份。
- 完整 runtime 身份不足时令牌同步为 NoOp，凭据文件保持不变。执行入口可独立补全其他可证明的自有快照元数据；因此 NoOp 不承诺所有注册表文件均不变。`sync_current_auth_registry` 在有效 OAuth runtime 身份未知时保留原 current_auth，返回 None；已知完整身份未匹配时及未登录状态沿用原指针清理逻辑。
- P2 只读注册表跳过身份补全，完整身份匹配的双向令牌同步继续。既有只读测试使用无 identity_key 的旧记录，验证注册表字节不变及两个同步方向。
- 配额路由在观测点同步后重新读取注册表及身份。配额缓存按完整身份隔离；身份不足时按既有凭据指纹隔离，不使用 account_id/email 单键。私有配额快照增加 id_token，并实现脱敏 Debug。
- 原新鲜度、Unchanged 字节/mtime、原子私有写、失败切换、API key/provider 行为保留。HTTP `ChatGPT-Account-Id` 和 usage ledger 仍使用 account_id。快照路径保持原合同。

## P1 修改文件

- `crates/ccr-codex/src/models/codex_auth.rs`
- `crates/ccr-codex/src/services/mod.rs`
- `crates/ccr-codex/src/services/codex_auth_identity.rs`（新增）
- `crates/ccr-codex/src/services/codex_auth_service.rs`
- `crates/ccr-codex/src/services/codex_oauth_token_service.rs`
- `crates/ccr-codex/src/services/codex_quota_service.rs`
- `crates/ccr-codex/src/services/openai_quota_core.rs`
- `crates/ccr-codex/src/services/codex_registry_store.rs`（仅测试结构字面量）
- `crates/ccr-codex/src/platforms/codex.rs`（仅测试结构字面量及完整合成身份夹具）
- `crates/ccr-tui/src/tui/codex_auth/ui.rs`（仅测试结构字面量）
- `crates/ccr-cli/src/application/auth_off.rs`（仅轮换写回测试 JWT 夹具）
- `crates/ccr-cli/src/commands/codex/auth/current.rs`（仅公共 JSON 字段数量与身份字段不泄漏断言）

`codex_usage_estimation.rs` 使用反序列化构造账号，不需改动。P2 的 `ccr-tui/.../app.rs` 产品改动未由本实施修改。相关 spec 与任务状态由主会话管理。

## 验证

全部测试使用临时目录、合成凭据及 loopback stub。保留默认测试并行度与 `--skip export_bindings`。

| 命令 | 结果 | 退出码 |
| --- | --- | --- |
| `cargo check -p ccr-codex --all-targets --all-features` | PASS | 0 |
| `cargo test -p ccr-codex --all-features -- --skip export_bindings` | 最终 316 passed，0 failed，2 ignored；doc tests 0 | 0 |
| `cargo test -p ccr-tui --all-features -- --skip export_bindings` | 252 passed，0 failed | 0 |
| `cargo test -p ccr-cli --all-features application::auth_off -- --skip export_bindings` | 11 passed，371 filtered out | 0 |
| `cargo test -p ccr-cli --all-features commands::codex::auth::current -- --skip export_bindings` | 2 passed，380 filtered out | 0 |
| `cargo clippy -p ccr-codex --all-targets --all-features -- -D warnings` | PASS | 0 |
| `cargo clippy -p ccr-cli -p ccr-tui --all-targets --all-features -- -D warnings` | PASS | 0 |
| `just fmt-check` | Rust workspace、Tauri Rust、11 个 JSON 格式和 5 个格式检查测试通过 | 0 |
| P1 产品范围 `git diff --check` | PASS；仅现有 LF/CRLF 转换提示 | 0 |

新增回归覆盖 claims 来源/缺失/冲突/坏 JWT、身份 Debug、注册表追加字段兼容、同 workspace 不同用户的读/规划/双向同步/切换、旧记录执行补全、缺失快照不借用 runtime 身份、repair 跨用户候选、配额路由/缓存用户隔离和未知身份凭据隔离。既有别名优先级测试补充同 last_used 的确定顺序断言。既有轮换、Unchanged、失败切换、API key/provider、P2 只读测试全部保留。

## 首次失败与修复记录

- 首次编译失败：导入构造点引用块内 `auth_path`，该绑定不在构造点作用域。改为 `self.account_auth_path(&name)`；随后 check 退出 0。
- 新增隔离测试首次完整运行：315 passed，1 failed，2 ignored，退出 1。唯一失败是测试硬编码账号顺序。实际 `left: ["a", "b", "invalid", "missing"]`，期望 `right: ["a", "b", "missing", "invalid"]`。原因：既有 TOML 加载顺序。修正测试期望后完整重跑为 316 passed，0 failed，2 ignored，退出 0。
- 首次失败完整输出由工具保留于 `C:/Users/lyh/AppData/Local/rtk/tee/1791296706_cargo_test.log`；上面的结果和断言差异在本文件保留。
- 主会话已复制该原始输出到本任务 `research/first-codex-failure.log`，SHA256：`53eebc54b57dc87a76b784103ef9f1ca5b638e81c8abb224ab009e1f507f2d73`，保留失败与修正后的独立证据。

## NOT_RUN 与后续

- 独立 Trellis check：NOT_RUN，由主会话接续。
- 所有子任务冻结后的 workspace `just lint-strict`、`just test`、`just ci`：本实施未重复运行，由主会话统一执行。P2 历史 PASS 不作为当前 P1 workspace 验收。
- Unix 专属权限测试：本机 Windows，NOT_RUN。P1 未改变权限实现。
- 真实账户、外部 OAuth 端点、原生登录/登出验证：NOT_RUN，仅合成与本地 stub。
- 无提交、归档、push、真实凭据读取或快照路径迁移。
