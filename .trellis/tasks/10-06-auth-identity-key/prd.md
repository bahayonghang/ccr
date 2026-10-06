# Codex Auth 身份模型改用 user_id::account_id 复合键

## Goal

把 runtime 与已保存账号的匹配从仅 `account_id` 改为 `chatgpt_user_id::chatgpt_account_id` 复合键，避免同一 ChatGPT workspace 下的不同用户互相覆盖快照。

## Authorization and Status

2026-10-06 用户要求为 10-06-reference-hardening 审计报告的后续任务提案 P1–P6 创建 Trellis 任务。本版为规划种子，未开始实施；需求与待决问题需经 brainstorm 收敛，复杂任务补齐 design.md 与 implement.md，并经用户评审后 `task.py start`。规划基线：dev @ 73b47563。证据锚点取自审计报告（.trellis/tasks/archive/2026-10/10-06-reference-hardening/research/codex-auth-audit.md），实施前需复核行号。

## Background and Evidence（F12，中）

- 匹配仅用 `account_id`：`codex_auth_service.rs:682-692`、`:729-737`；`codex_oauth_token_service.rs:507-544`。
- codex-auth 的身份键为 `record_key = chatgpt_user_id::chatgpt_account_id`。
- 风险：Team/Enterprise workspace 下多个用户共享 `account_id`。10-06-auth-switch-reliability 已用「current_auth 优先，其次 last_used」限定同步目标，但不能区分用户，同步可能把用户 A 的 runtime tokens 写入用户 B 的快照。

## Requirements（草案）

| ID | 要求 |
| --- | --- |
| R1 | 从 id_token / access_token claims 读取 user_id，与 account_id 组成身份键。 |
| R2 | 注册表保存身份键；旧记录在首次观测时补全，补全不改变账号名与 current_auth。 |
| R3 | runtime 同步、切换写回、配额路由按身份键匹配；身份键不可得时的回退规则需与 ccr-codex backend-guidelines 的 runtime ↔ snapshot token 同步合同一起评审。 |
| R4 | 身份键与 tokens 不进入日志、错误文本与 DTO。 |

## Open Decisions

- claims 中 user_id 的字段名，以及缺失时的回退：回退到仅 account_id，还是视为 NoOp。
- 同一身份键对应多个已保存账号时的处理。
- 是否需要在 CLI/TUI 展示身份键（默认不展示）。

## Acceptance Criteria（草案）

- [ ] 同 account_id、不同 user_id 的两个快照：runtime 只同步到 user_id 匹配的快照。
- [ ] 旧注册表记录（无 user_id）首次观测后补全，既有同步测试语义不变。
- [ ] `cargo test -p ccr-codex`、`just lint-strict`、`just test` 通过。

## Dependencies

- 在 10-06-auth-registry-schema（P2）之后实施。
- 10-06-auth-snapshot-naming（P5）与 10-06-auth-import-export（P6）依赖本任务。

## Out of Scope

- 快照文件命名迁移（P5）。
