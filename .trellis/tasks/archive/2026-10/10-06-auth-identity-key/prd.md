# Codex Auth 身份模型改用 user_id::account_id 复合键

## Goal

把 runtime 与已保存账号的匹配从仅 `account_id` 改为 `chatgpt_user_id::chatgpt_account_id` 复合键，避免同一 ChatGPT workspace 下的不同用户互相覆盖快照。

## Authorization and Status

2026-10-06 用户已授权顺序实施。缺少 user_id 的规则已确认：先从已保存快照补全身份；完整 `user_id::account_id` 仍不可得时跳过 OAuth 同步。API key 账号的既有匹配行为保留。

2026-10-06 用户要求为 10-06-reference-hardening 审计报告的后续任务提案 P1–P6 创建 Trellis 任务。本版为规划种子，未开始实施；需求与待决问题需经 brainstorm 收敛，复杂任务补齐 design.md 与 implement.md，并经用户评审后 `task.py start`。规划基线：dev @ 73b47563。证据锚点取自审计报告（.trellis/tasks/archive/2026-10/10-06-reference-hardening/research/codex-auth-audit.md），实施前需复核行号。

## Background and Evidence（F12，中）

- 匹配仅用 `account_id`：`codex_auth_service.rs:682-692`、`:729-737`；`codex_oauth_token_service.rs:507-544`。
- codex-auth 的身份键为 `record_key = chatgpt_user_id::chatgpt_account_id`。
- 风险：Team/Enterprise workspace 下多个用户共享 `account_id`。10-06-auth-switch-reliability 已用「current_auth 优先，其次 last_used」限定同步目标，但不能区分用户，同步可能把用户 A 的 runtime tokens 写入用户 B 的快照。

## Requirements

| ID | 要求 |
| --- | --- |
| R1 | 从 id_token / access_token claims 读取 user_id，与 account_id 组成身份键。 |
| R2 | 注册表保存身份键；旧记录在首次观测时补全，补全不改变账号名与 current_auth。 |
| R3 | runtime 同步、切换写回、OAuth repair候选、配额路由与内存缓存按完整身份隔离；身份键不可得时OAuth同步为NoOp，不回退account_id单键。不完整runtime身份的对账保留current_auth；完整且不匹配或未登录时沿用原清理逻辑。 |
| R4 | 身份键与 tokens 不进入日志、错误文本与 DTO。 |
| R5 | 参考 cockpit-tools 补齐凭据生命周期：同一完整身份的 CCR 刷新互斥并在锁内重读来源；切换先锁住换出与目标身份，避免轮换结果因 runtime 被替换而丢失；save_current 使用同一份 runtime 内容生成快照与元数据。 |

## Decisions

- D1（用户确认）：完整 OAuth 身份不可得时为 NoOp，不回退到仅 account_id。
- D2（沿用已有默认）：同一完整身份的多个别名优先选择 current_auth，其次 last_used 最新者；时间相同保持有序账号遍历的确定性。
- D3（沿用草案默认）：不在 CLI/TUI、DTO、错误或日志展示身份键。
- D4（2026-10-06 用户继续实施并要求修复旧账号查询）：手动强制查询绕过配额缓存；有效 access token 先请求额度。仅在 token 过期或额度端点拒绝认证时触发 OAuth 刷新。失效 refresh token 不得阻止尚可用 access token 的查询。
- claims 字段与兼容旧记录的技术细节由源码研究和设计约束确定；不从 JWT 的未验证内容推导授权，只用于本地身份关联。

## Acceptance Criteria

- [x] 同 account_id、不同 user_id 的两个快照：runtime 只同步到 user_id 匹配的快照。
- [x] 双向同步、repair候选、切换前写回、配额路由与缓存均隔离共享workspace中的不同用户；同完整身份多个别名沿用确定性优先级。
- [x] 旧注册表记录（无 identity_key）只从各自快照补全，账号名与current_auth不因补全改变；只读planner不持久化，P2只读注册表不backfill，安全token同步可继续。
- [x] 缺失/冲突完整身份不改凭据文件；API key/provider既有匹配与命令语义保留，完整身份不进入Debug、错误、日志与公开DTO。
- [x] 成功OAuth合成夹具补完整用户claims；保留缺失身份NoOp专项用例，CLI auth-off轮换写回与TUI/P2回归通过。
- [x] 有效 access token、失效 refresh token 的手动查询成功且不触发刷新或凭据写入；缓存被绕过；过期及远端认证拒绝仍触发现有刷新/需重新登录路径。
- [x] 并发查询不重复消费同一旧 refresh token；等待者锁内重读已轮换凭据；刷新与切换交叉不丢失换出账号的新凭据。相同身份别名使用安全的新鲜来源，不按 workspace 单键关联。保存的快照与账号元数据来自同一读取。跨进程文件锁与既有叶锁顺序有明确证据。
- [x] `cargo test -p ccr-codex`、`just lint-strict`、`just test` 通过。

2026-10-06 本地验收：AC1–AC7 见 `research/account-lifecycle-check.md`；最终源码的 AC8 与跨子任务回归见父任务 `research/integration-validation.md` 和 `checks/ci-final-first-attempt.log`。最终 Codex 381 passed、2 ignored，TUI 253 passed，CLI 347 单元、12 集成与 1 doctest 通过。缺失完整身份的 NoOp 指跨文件关联；既有未知身份同文件刷新合同保留。跨进程证据覆盖文件锁与锁内重读，HTTP 生命周期使用同进程合成 loopback。真实 k12 恢复仍为 UNVERIFIED。

## Dependencies

- 在 10-06-auth-registry-schema（P2）之后实施。
- 10-06-auth-snapshot-naming（P5）与 10-06-auth-import-export（P6）依赖本任务。

## Out of Scope

- 快照文件命名迁移（P5）。
