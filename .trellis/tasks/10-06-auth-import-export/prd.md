# Codex Auth 导入身份校验与导出加密默认值

## Goal

导入时校验 `account_id` 与 tokens 一致；评估是否默认使用加密导出。

## Authorization and Status

2026-10-06 用户要求为 10-06-reference-hardening 审计报告的后续任务提案 P1–P6 创建 Trellis 任务。本版为规划种子，未开始实施；需求与待决问题需经 brainstorm 收敛，复杂任务补齐 design.md 与 implement.md，并经用户评审后 `task.py start`。规划基线：dev @ 73b47563。证据锚点取自审计报告（.trellis/tasks/archive/2026-10/10-06-reference-hardening/research/codex-auth-audit.md），实施前需复核行号。

## Background and Evidence

- F16（低）：`codex_auth_service.rs:2001-2013` 导入时 `account_id` 直接取自导出元数据。被篡改或手工编辑的导出文件会让快照与注册表的身份不一致，影响 runtime 同步目标选择。
- F15（低）：`codex_auth_service.rs:1743-1760` 在 `include_secrets` 为真时导出完整 `auth_data` 明文。加密导出 `export_accounts_encrypted` 已存在，由调用方显式选择。

## Requirements（草案）

| ID | 要求 |
| --- | --- |
| R1 | 导入时从 tokens 推导身份（与 P1 规则一致）并与元数据比对；不一致时按确认的策略拒绝或告警。 |
| R2 | 评估默认加密导出：CLI、TUI、桌面端的入口与提示。 |

## Open Decisions（需用户确认）

- 身份不一致时拒绝导入，还是告警后以 tokens 推导值为准。拒绝会让部分既有导出文件无法导入。
- 是否把加密导出设为默认；明文导出是否保留显式开关。

## Acceptance Criteria（草案）

- [ ] 元数据 `account_id` 与 tokens 不一致的导入按确认策略处理，并有测试。
- [ ] 一致输入的既有导入测试（`test_import_accounts_*`）语义不变。
- [ ] 若改默认导出：EN/ZH 提示同步，既有导出测试按新默认值更新并说明原因。
- [ ] `cargo test -p ccr-codex`、`just lint-strict`、`just test` 通过。

## Dependencies

- 在 10-06-auth-identity-key（P1）之后实施。
