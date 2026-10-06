# Codex Auth 注册表版本门与未知字段保留

## Goal

防止较旧版本 CCR 加载并重写注册表时丢失较新版本写入的字段。

## Authorization and Status

2026-10-06 用户要求为 10-06-reference-hardening 审计报告的后续任务提案 P1–P6 创建 Trellis 任务。本版为规划种子，未开始实施；需求与待决问题需经 brainstorm 收敛，复杂任务补齐 design.md 与 implement.md，并经用户评审后 `task.py start`。规划基线：dev @ 73b47563。证据锚点取自审计报告（.trellis/tasks/archive/2026-10/10-06-reference-hardening/research/codex-auth-audit.md），实施前需复核行号。

## Background and Evidence（F14，中）

- `models/codex_auth.rs:291-308` 的 `CodexAuthRegistry` 没有 `#[serde(flatten)]` 扩展字段。
- `codex_registry_store.rs:45-55` 加载时不检查 `version`。
- codex-auth 使用 `schema_version` 门，较新版本拒绝加载。

## Requirements（草案）

| ID | 要求 |
| --- | --- |
| R1 | 二选一或组合：(a) `schema_version` 门，遇到较新版本拒绝写入并给出提示；(b) `#[serde(flatten)] extra` 保留未知字段并原样写回。 |
| R2 | 现有注册表文件保持可加载；保存仍走 `AtomicWriter::secret(true)` 与写前备份。 |
| R3 | 不新增 `CcrError` 变体；提示文本 EN/ZH 同步。 |

## Open Decisions

- 选择 (a)、(b) 或两者组合。(a) 会让旧版本 CCR 拒绝操作，属于用户可见行为。
- `CodexAuthRegistry` 结构字面量构造点的影响范围（需 `rg` 统计）。

## Acceptance Criteria（草案）

- [ ] 若选 (b)：含未知字段的注册表经加载、保存后，未知字段仍在。
- [ ] 若选 (a)：较新 `schema_version` 的注册表不被覆盖。
- [ ] 既有注册表测试通过；`cargo test -p ccr-codex`、`just lint-strict`、`just test` 通过。

## Dependencies

- 10-06-auth-identity-key（P1）依赖本任务。
