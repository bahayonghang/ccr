# Codex Auth 快照文件名与身份键解耦

## Goal

评估以身份键派生快照文件名，账号名只作展示与别名。本任务先做评估；是否实施由评估结论与用户决定。

## Authorization and Status

2026-10-06 已按父任务顺序完成评估与独立检查。评估建议暂缓迁移，产品布局未改变；迁移决定与实施仍需用户单独授权。评估证据见 `research/snapshot-naming-assessment.md` 和 `research/independent-check.md`。

2026-10-06 用户要求为 10-06-reference-hardening 审计报告的后续任务提案 P1–P6 创建 Trellis 任务。本版为规划种子，未开始实施；需求与待决问题需经 brainstorm 收敛，复杂任务补齐 design.md 与 implement.md，并经用户评审后 `task.py start`。规划基线：dev @ 73b47563。证据锚点取自审计报告（.trellis/tasks/archive/2026-10/10-06-reference-hardening/research/codex-auth-audit.md），实施前需复核行号。

## Background and Evidence（F13，低）

- `codex_auth_service.rs:186-188` 用 `format!("{}.json", name)` 生成快照路径；`:1582-1610` 校验账号名的字符集与长度。
- 审计结论：当前校验阻止路径穿越与非法字符，没有安全缺口。本任务属于设计变更。
- codex-auth 以 record_key 派生文件名并做 base64url 编码。

## Requirements（草案）

| ID | 要求 |
| --- | --- |
| R1 | 评估报告：收益、迁移成本、回滚方案、与重命名和导入导出的交互。 |
| R2 | 若实施：文件名由 P1 身份键派生（base64url）；现有快照自动迁移，迁移前备份，失败可回滚。 |
| R3 | 若实施：重命名账号不再移动快照文件。 |

## Open Decisions

- 是否实施（评估后由用户决定）。
- 迁移触发时机：加载时自动迁移，或显式命令。

## Acceptance Criteria（草案）

- [x] 评估报告存在，并给出实施或不实施的结论与依据。
- [ ] 若实施：旧布局快照迁移后，切换、同步、配额可用；迁移失败时回滚到原布局。
- [x] `cargo test -p ccr-codex`、`just lint-strict`、`just test` 通过。

2026-10-06 本地验收：评估与独立检查 PASS。父任务最终 `just ci` 验证现有布局下的集成，Codex 381 passed、2 ignored，Rust workspace 2038 passed、0 failed、16 ignored；该结果不作为迁移验收。迁移条件不适用且保持未勾选，迁移决定与实施未授权。详见父任务 `research/integration-validation.md`。

## Dependencies

- 依赖 10-06-auth-identity-key（P1）的身份键。
