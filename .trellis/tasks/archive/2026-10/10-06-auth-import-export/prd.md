# Codex Auth 导入身份校验与导出加密默认值

## Goal

导入时校验 `account_id` 与 tokens 一致；评估是否默认使用加密导出。

## Authorization and Status

2026-10-06 用户确认：「拒绝冲突输入，保留现有加密导出规则」。P1 通过后按既定顺序实施。CLI 含凭据导出已强制加密；本轮保持该规则，不新增 TUI/Tauri 导出入口或密码 DTO。

2026-10-06 用户要求为 10-06-reference-hardening 审计报告的后续任务提案 P1–P6 创建 Trellis 任务。本版为规划种子，未开始实施；需求与待决问题需经 brainstorm 收敛，复杂任务补齐 design.md 与 implement.md，并经用户评审后 `task.py start`。规划基线：dev @ 73b47563。证据锚点取自审计报告（.trellis/tasks/archive/2026-10/10-06-reference-hardening/research/codex-auth-audit.md），实施前需复核行号。

## Background and Evidence

- F16（低）：`codex_auth_service.rs:2001-2013` 导入时 `account_id` 直接取自导出元数据。被篡改或手工编辑的导出文件会让快照与注册表的身份不一致，影响 runtime 同步目标选择。
- F15（低）：`codex_auth_service.rs:1743-1760` 在 `include_secrets` 为真时导出完整 `auth_data` 明文。加密导出 `export_accounts_encrypted` 已存在，由调用方显式选择。

## Requirements（草案）

| ID | 要求 |
| --- | --- |
| R1 | 导入时从 tokens 推导身份（与 P1 规则一致）并与元数据比对；已知身份不一致时在写入前拒绝。 |
| R2 | 评估默认加密导出：CLI、TUI、桌面端的入口与提示。 |

## Decisions

- 用户确认：拒绝身份冲突输入，保留现有加密导出规则。
- 保留 Merge/no-force 跳过已有账号的语义；待处理条目先整体校验，再写入。

## Acceptance Criteria（草案）

- [x] 元数据 `account_id` 与 tokens 不一致的导入按确认策略处理，并有测试。
- [x] 一致输入的既有导入测试（`test_import_accounts_*`）语义不变。
- [ ] 若改默认导出：EN/ZH 提示同步，既有导出测试按新默认值更新并说明原因。
- [x] `cargo test -p ccr-codex`、`just lint-strict`、`just test` 通过。

2026-10-06 本地验收：身份冲突与整包预检、锁后身份/内容版本复核、既有 Merge/Replace/metadata-only 语义及 CLI 错误传播见 `research/independent-check.md`。最终源码通过父任务 `just ci`，Codex 381 passed、2 ignored，CLI 347 单元、12 集成与 1 doctest 通过，Rust workspace 2038 passed、0 failed、16 ignored。第三项为条件不适用：用户选择保留现有加密导出，未改变默认值；入口评估见 `research/encrypted-export-assessment.md`。Replace/no-force 无新增快照前像保证，多条目 I/O 与注册表发布仍非事务，详见父任务 `research/integration-validation.md`。

## Dependencies

- 在 10-06-auth-identity-key（P1）之后实施。
