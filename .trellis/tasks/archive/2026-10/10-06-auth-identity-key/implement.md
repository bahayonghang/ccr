# 实施计划：完整 OAuth 身份

## 前置与白名单

P2 独立检查通过后启动。用户已确认缺少完整身份时跳过同步。

产品白名单：`crates/ccr-codex/src/models/codex_auth.rs`；`services/mod.rs`、必要的窄身份解析模块、`codex_auth_service.rs`、`codex_oauth_token_service.rs`、`codex_quota_service.rs`、`openai_quota_core.rs`；新增账户字段所需的结构字面量与合成夹具：`codex_registry_store.rs`、`codex_usage_estimation.rs`、`platforms/codex.rs`、`crates/ccr-tui/src/tui/codex_auth/ui.rs`。CLI/Tauri 只核对不暴露新键，不增加 DTO 字段。相关 spec 由主会话写回。

必要外部测试构造点：`crates/ccr-cli/src/commands/codex/auth/current.rs` 的 CurrentAuthInfo 夹具，以及 `crates/ccr-cli/src/application/auth_off.rs` 的 `codex_file_off_syncs_rotated_runtime_tokens_before_delete` 合成JWT。仅补完整用户claims/内部构造字段，不改变CLI生产行为或删除同步断言；补CLI JSON未泄漏身份键的断言。

独立检查补充（2026-10-06）：配额请求后的刷新写回必须复核请求源与实际目标，并通过versioned guarded write拒绝内容变化。`WriteRuntime`执行点必须复核完整身份与新鲜度，并把`codex_runtime_service.rs`纳入必要白名单，增加crate内限定auth同步CAS入口；沿用现有gw路径叶锁、runtime_switch备份和私有原子写，不改变公开CommitPlan/DTO，不引入通用事务。CAS冲突不得用旧auth备份覆盖新runtime。外部不遵守CCR锁的写入仍有最后竞争窗口，证据明确保留该限制。

## 顺序

1. 共享 claims 解析和注册表可选身份键；测试有效、缺失、空值、坏 JWT、冲突 claims，Debug 不含身份值。
2. 从各自快照推导旧记录身份；读取计划保持只读，执行观测点保存安全补全。测试 name/current_auth 不变，P2 只读不写。
3. 完整身份筛选与别名优先级；覆盖 runtime/current_auth/切换前同步；选中后重新验证快照身份。
4. 约束 OAuth repair 候选完整身份，保持新鲜度和原子写规则。
5. 配额 runtime 路由和缓存按完整身份隔离；HTTP 头与用量账本保留 account_id。
6. OAuth 成功测试和刷新 stub 补合成 claims；不删除缺失身份用例、不降低覆盖或改串行门。
7. 运行定向回归与 crate 级测试、clippy、fmt；独立 Trellis check 对照全部条款。
8. 用户新增旧账号查询要求：共享 quota core 中有效 access token 的强制配额查询直接 GET，保留过期/认证拒绝刷新；增加缓存绕过、失效 refresh 与无写入回归。此补充不增加公开 API，不进行真实 OAuth 刷新或登录切换。
9. 用户要求深入对照 cockpit-tools：必要白名单增加 crate-private `codex_auth_refresh_lock.rs`；在 quota/auth/oauth 三个既有服务中补窄凭据操作锁及持锁内部入口、锁内来源重读与同身份新鲜来源；save_current 使用同一内容生成元数据。保留公开形状、API key/provider 与 P2 只读拒绝前置。验证并发同来源、别名同链、换出刷新/切换交叉、锁后身份变化、不同用户隔离与保存同源；保存首败证据，不新增watcher或后台守护。

## 验收

- 同 account_id 不同 user_id 的同步、repair 与配额缓存不交叉。
- 双向令牌同步、Unchanged、重复完整身份别名、失败切换、API key/provider、P2 只读行为符合设计。
- 旧记录从自有快照补全；缺完整身份 NoOp 不改文件；只读 plan 不持久化。
- 不向日志、错误、公共 DTO 输出完整身份键。

命令：`cargo test -p ccr-codex --all-features -- --skip export_bindings`、`cargo test -p ccr-tui -- --skip export_bindings`（构造点变更时）、`cargo clippy -p ccr-codex --all-targets --all-features -- -D warnings`、`just fmt-check`。最终 workspace `just lint-strict`、`just test`、`just ci` 在所有子任务冻结后运行。

## 风险控制

若真实源码证明现有 API/DTO 必须扩大或身份规则有歧义，先报告主会话，不猜测。仅操作合成临时夹具；不读真实凭据，不迁移快照目录，不提交或归档。
