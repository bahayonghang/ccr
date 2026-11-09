# P1 独立检查

状态：`AC1_AC6_LOCAL_PASS`；`AC7_CRATE_PASS_WORKSPACE_NOT_RUN`。产品源码已冻结。日期：2026-10-06。环境：Windows，分支 `dev`，HEAD `c524ac07e77e94587399d969546ed75f13cfaf19`。角色：`trellis-check`。

## 范围与证据边界

已读取完整原生 hook 输出、check.jsonl、PRD、design.md、implement.md、源码研究，以及 Codex/Core/CLI/TUI 适用规范。按主会话批准保留 P2 已验收的未提交改动。审查代理独占本轮 Auth/OAuth/Quota/Runtime 修复；旧账号查询核心由另一实施代理独占 `openai_quota_core.rs`，审查代理核对最终核心与回归。

检查覆盖共享 claims、旧记录自有快照关联、读取与执行边界、双向同步、换出写回、repair、配额路由和缓存、异步刷新写回、CLI auth-off 与公开映射。新增写回保护不改变公开 `CodexRuntimeCommitPlan`、`RuntimeSyncPlan` 或 CLI/Tauri DTO 形状。唯一新增 RuntimeService 入口为 `pub(crate) commit_synced_auth_versioned`，只处理 auth 同步。

本轮修复后的完整 Codex 测试和严格 Clippy由审查代理执行。此前 CLI/TUI 完整或定向结果来自 `implementation-validation.md`；当前源码的 workspace 正式门由主会话执行。没有读取真实凭据、修改真实账号、提交、归档或 push。

## 逐 AC 结论

| AC | 结论 | 源码与测试证据 |
| --- | --- | --- |
| AC1 同 workspace 不同用户 | PASS | `codex_auth_identity.rs:25` 同时读取 id/access 的顶层及 auth namespace 用户 claims；`codex_oauth_token_service.rs:574` 从自有 snapshot 推导身份并验证已存 key；`:625` 只筛选完整匹配集合。`complete_identity_isolates_users_and_backfills_only_at_execution` 验证 user A runtime 不改 user B snapshot。 |
| AC2 全路径隔离与别名顺序 | PASS，已修写入保护 | planner `:444` 在选中后再次解析 snapshot；runtime 执行 helper `codex_auth_service.rs:1190` 再验身份并重新规划新鲜度，最终由 `codex_runtime_service.rs:322` CAS 写入。snapshot 入口 `codex_oauth_token_service.rs:300` 再验身份、注册表与当前新鲜度，并进行 CAS。repair `:665` 与候选解析 `:178` 使用完整 key。quota route `codex_quota_service.rs:515` 同步后重读 registry/runtime/snapshot；cache `openai_quota_core.rs:670` 使用完整身份，未知身份使用 refresh/access 指纹。别名回归 `codex_auth_service.rs:4544` 保留 current_auth、last_used、平手插入顺序断言。双向同步、switch-out、repair 和请求期间换用户回归均通过。 |
| AC3 旧记录与只读边界 | PASS | `saved_identity:574` 只读取该账号自己的 snapshot，缺失/坏快照不借 runtime。`backfill_identity_keys:598` 只保存缺失键，不改账号名/current_auth；read-only registry 跳过。read_auth_snapshot 与 plan_runtime_sync 没有写入。`complete_identity_isolates_users_and_backfills_only_at_execution` 比较 planner 后 registry 原字节，并覆盖 missing/invalid snapshot；P2 的 `token_sync_continues_in_both_directions_with_read_only_registry` 保留 registry bytes 和双向同步断言。 |
| AC4 缺身份、兼容与输出边界 | PASS，限本任务新增身份字段 | 缺失、坏 JWT、裸 sub/email、空值或冲突有效 user claims 返回未知身份；OAuth 同步 NoOp 不改凭据。有效 OAuth runtime 身份未知时 `sync_current_auth_registry:737` 保留原 current_auth；已知未匹配与未登录保留原清理规则。API/provider 指纹分支保留。新身份对象、账号 identity_key 与私有 quota snapshot Debug 脱敏；固定身份/CAS拒绝错误没有 key。CLI current JSON 断言原 5 字段及无 identity_key/user_id；Tauri `CodexAuthAccountMetadata::from` 与 export DTO仍使用显式原字段映射。 |
| AC5 合成夹具、CLI/TUI/P2回归 | PASS / 当前跨 crate 正式门待主会话 | OAuth成功夹具补完整合成 user claims，缺身份 NoOp 用例保留。最终 Codex 328 passed、0 failed、2 ignored，包含 P2 只读、双向 token、失败切换、API/provider与新回归。实施报告保留 TUI252、CLI auth_off11/current2通过证据；本轮未重复整包 TUI/CLI测试，主会话后续 workspace门确认最终跨 crate 源码。 |
| AC6 手动查询有效 access | PASS | `openai_quota_core.rs:226` 提前刷新只判断到期，force仅跳过cache。五个 loopback 回归覆盖有效 access + 永久失效 refresh成功、cache bypass、到期刷新、401后刷新重试，以及到期/401 +永久刷新失败。断言 GET、token POST、持久化次数和失败无成功cache。真实 k12 恢复不能由合成测试推断。 |
| AC7 正式检查 | CRATE_PASS / WORKSPACE_NOT_RUN | 最终完整 Codex328/0/2；scopedstrict Clippy、TypeCheck、fmt、敏感写入和diff通过。`just lint-strict`、`just test`、`just ci` 未由本审查代理执行，主会话统一运行。 |

## Findings (fixed)

1. 文件：`crates/ccr-codex/src/services/codex_quota_service.rs:353`、`:599`。
   - 问题：四个异步刷新持久化调用只在请求前选择凭据源。请求期间目标换为同 workspace 的另一用户时，旧请求结果会覆盖新用户；刷新结果身份也未复核。
   - 修复：私有 wrapper携带并推进 expected snapshot，兼容同一次请求的多次刷新；写入前比较 id/access/refresh/account四字段，校验已知完整身份与新文档身份，然后以原文内容版本执行 secret CAS。未知身份仍能对未改变的同文件凭据刷新。更新保持原有模式/私有权限政策，无新增备份。
   - 回归：最初 `update_auth_file_rejects_refreshed_tokens_for_another_user` 先失败后通过；`quota_refresh_write_rejects_runtime_changed_during_request` 覆盖另一用户和同用户新凭据；`quota_refresh_request_cannot_overwrite_a_replacement_user` 用 loopback响应前修改文件，覆盖 active/saved/default三条路径，证明新目标和原快照字节保留。

2. 文件：`crates/ccr-codex/src/services/codex_auth_service.rs:1190`、`crates/ccr-codex/src/services/codex_runtime_service.rs:322`。
   - 问题：WriteRuntime直接执行旧计划，未复核执行时身份或新鲜度，最终普通提交没有 expected-version保护。
   - 修复：执行前读 runtime版本、检查计划完整身份并重新plan；只有同账号与同 auth 的 WriteRuntime继续。新增crate内auth-only CAS入口，保留file-store检查、runtime_switch备份、既有gw路径叶锁及secret写。冲突返回NoOp，禁止恢复旧auth备份覆盖新runtime；config不参与该入口。
   - 回归：旧计划后的user变化、同user较新runtime、snapshot换user均被拒绝；仍有效计划成功。RuntimeService回归覆盖冲突前无备份、成功时备份字节正确，以及备份完成后在叶锁释放前改变runtime，最终CAS冲突不回滚。

3. 文件：`crates/ccr-codex/src/services/codex_oauth_token_service.rs:300`。
   - 问题：snapshot身份复核后使用普通AtomicWriter，未保护读后变更；计划到执行入口之间出现同身份较新snapshot时可覆盖新tokens。
   - 修复：入口比较doc与当前snapshot有效时间（last_refresh，否则mtime），拒绝旧来源；最终使用无备份secret CAS。
   - 回归：`snapshot_write_rejects_a_stale_plan_for_the_same_identity` 保留已更新的snapshot；既有repair旧来源与跨用户候选、双向sync继续通过。

4. 文件：上述Auth/OAuth测试fixture。
   - 问题：新增guarded写读取CCR_LOCK_DIR。旧显式tempdir测试未持有TestCodexEnv，可与P2的envfixture并发，导致P2整个home字节扫描读取仍被锁定的文件，Windows错误33。
   - 修复：Auth共用fixture及OAuthsetup持有TestCodexEnv到操作结束；唯一已持env的调用使用with_env_held入口，避免重复取锁。没有改变默认并行度、扫描断言、忽略项或门。
   - 回归：Auth服务定向先74/7失败，修复后81/0通过，最终完整Codex通过。

5. 文件：`crates/ccr-codex/src/services/openai_quota_core.rs:226`、手动查询测试与到期fixture。
   - 问题与修复：有效access曾因force先刷新而被失效refresh阻断。独立实施代理移除force提前刷新条件；原到期cache测试的全局未来now会删除其他并行用例条目，改为仅将自有条目写成实际过期时间。
   - 审核：生产仅该条件变更；接口/TTL不变。新增回归的次数断言、真实now到期断言和默认并行保留。实施首败与核心重跑见 `old-account-quota-implementation.md`。

## Findings (not fixed)

- 产品范围内没有未修复发现。任意不遵守CCR锁的外部文件写入仍存在CAS检查至替换之间的最后竞争窗口；跨文件registry/snapshot/runtime没有新增事务。本次没有扩大到通用事务或外部Codex进程协调。
- 规范写回由主会话负责。报告写入时backend-guidelines:260仍描述WriteRuntime走普通commit_plan；需同步auth-only CAS入口、执行重新plan、snapshot新鲜度/CAS和force仅绕cache合同。已提前通知主会话，审查代理没有编辑spec。
- 前序审计的原认证模型tokens Debug与远端响应体预览保护归P4任务；本轮没有扩大该范围，也没有声称所有旧Debug/错误实现均完成脱敏。新增完整身份key未进入当前公开映射或新增诊断。

## Verification

| 审查代理命令 | 最终结果 | 退出码 |
| --- | --- | --- |
| `cargo test -p ccr-codex --all-features services::codex_auth_service -- --skip export_bindings` | PASS：81 passed | 0 |
| `cargo test -p ccr-codex --all-features services::codex_runtime_service -- --skip export_bindings` | PASS：3 passed | 0 |
| `cargo test -p ccr-codex --all-features quota_refresh_request_cannot_overwrite_a_replacement_user -- --skip export_bindings` | PASS：1 test，3请求目标路径 | 0 |
| `cargo test -p ccr-codex --all-features -- --skip export_bindings` | PASS：328 passed、0 failed、2 ignored；2 suites，15.46s | 0 |
| `cargo clippy -p ccr-codex --all-targets --all-features -- -D warnings -D clippy::unwrap_used` | PASS，最终源码 | 0 |
| `just fmt-check` | PASS：workspace/Tauri Rust、11 JSON、5格式测试 | 0 |
| `python scripts/quality/check_secret_writes.py` | PASS | 0 |
| P1产品范围 `git diff --check` | PASS，仅LF/CRLF转换提示 | 0 |

Lint：PASS（scoped）。TypeCheck：PASS（最终Clippy全targets/allfeatures）。Tests：PASS（最终Codex完整包及定向回归）。保留默认测试并行与`--skip export_bindings`。

## 首次失败与修复过程

- 配额跨用户回归首次失败：0 passed、1 failed，断言写入应被拒绝但旧函数返回成功。原RTK输出 `C:/Users/lyh/AppData/Local/rtk/tee/1791297385_cargo_test.log`；主会话复制为 `research/first-quota-race-failure.log`。
- 将snapshot writer更换为guarded writer后，首次编译发现测试helper仍需AtomicWriter。将该import移到测试模块；没有增加生产未使用import。
- Auth定向首轮74 passed、7 failed。Windows error33发生在P2 fixture_tree读取锁文件，原因及修复见上。原输出 `C:/Users/lyh/AppData/Local/rtk/tee/1791298187_cargo_test.log`；随后同命令81/0。
- 最终完整包首轮327 passed、1 failed、2 ignored。新增备份后CAS测试未写file-store配置，服务以默认auto拒绝，测试等待backup超时。原输出 `C:/Users/lyh/AppData/Local/rtk/tee/1791298715_cargo_test.log`。
- 补file-store后该定向测试又选中AtomicWriter临时文件，最终读路径返回NotFound；等待条件改为实际`auth.runtime_switch.*.json.bak`文件。随后runtime3/0及完整包328/0/2通过。保留失败与独立重跑，没有删除回归或放宽测试门。
- 旧账号核心的修前1/4、cache夹具修前15/1及修后16/0原日志由实施代理在本任务保存，见其报告。当前完整包通过不改变这些历史失败记录。

## NOT_RUN 与限制

- WORKSPACE_NOT_RUN：审查代理没有执行`just lint-strict`、`just test`、`just ci`。主会话完成正式门后才可更新AC7；本报告不宣称父任务全部验收。
- NOT_RUN：本轮完整TUI/CLI包重跑、Unix专属权限测试、额外Windows ACL实测、原生终端UI与已安装二进制替换。
- NOT_RUN（本审查代理）：真实账号、真实OAuth端点或凭据文件操作。主会话报告真实k12的无刷新额度GET返回401，同路径其他账号返回200，且无较新匹配backup；该只读诊断不等于k12恢复，本轮合成PASS不替代有效认证。
- 已知两个Codexbenchmark忽略项保留。没有调整覆盖阈值、数据集、并行规则或正式门。

## 冻结源码SHA256

| 文件（services/下） | SHA256 |
| --- | --- |
| codex_auth_service.rs | `2A603463379789892BF40C1C0BEDDC7AF89792729C976465086A9E6536CDFBF5` |
| codex_oauth_token_service.rs | `6251FBB8EADCB75456137AD63B8DBC81DF6A0C79CB7959ABB74380683624C6CA` |
| codex_quota_service.rs | `3A8A2911F22CA45AB2F26C637FE09DE5B51BB12403FB5B62027C59053105B294` |
| codex_runtime_service.rs | `2FF21705B1280FF73070A4B01C59B2900E2237171B36C270831A9DE4018CD3AE` |
| openai_quota_core.rs | `5AB1778F682EF90F552ADC3594267D1C6919921F53DC868A667350C8CA8ED328` |
