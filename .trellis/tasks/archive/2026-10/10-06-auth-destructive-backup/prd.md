# Codex Auth 注册表备份策略与破坏性操作前备份

## Goal

为注册表与账号备份建立去重、同秒序号与可靠私有写；让删除和强制重命名在破坏性步骤前有可靠备份。保留全部旧备份。

## Authorization and Status

2026-10-06 用户已确认继续顺序实施；保留策略为「本轮保留全部旧备份，只修复去重、防覆盖与可靠备份」。本轮不清理超额历史，不设置新保留上限。删除前备份沿用已有无期限留存，本轮增加去重与失败阻断。错误沿用现有变体与 TUI EN/ZH 操作失败前缀。

2026-10-06 用户要求为 10-06-reference-hardening 审计报告的后续任务提案 P1–P6 创建 Trellis 任务。本版为规划种子，未开始实施；需求与待决问题需经 brainstorm 收敛，复杂任务补齐 design.md 与 implement.md，并经用户评审后 `task.py start`。规划基线：dev @ 73b47563。证据锚点取自审计报告（.trellis/tasks/archive/2026-10/10-06-reference-hardening/research/codex-auth-audit.md），实施前需复核行号。

## Background and Evidence

- F10（低）：`codex_registry_store.rs:72` 每次 `save` 前执行 `let _ = self.backup();`，`:83-101` 写 `auth_registry_{ts}.toml`；账号备份见 `codex_auth_service.rs:1332-1356`。`auth/backups` 下的注册表与账号备份没有清理，数量无上限。
- F11（低）：`codex_registry_store.rs:93-98`、`codex_auth_service.rs:1312-1330` 使用秒级文件名加 `fs::copy`，同秒覆盖；备份权限跟随源文件模式或目录 ACL。
- F21（中）：`codex_auth_service.rs:1288-1293` 的 `delete_account` 直接 `fs::remove_file`，不先备份。规范要求 backup-before-destructive-change。
- F22（中）：`codex_auth_service.rs:1507` 忽略 `backup_account_auth(new_name)` 的失败，随后 `:1509-1513` 删除冲突快照；`:1519-1520` 同样忽略源快照与注册表备份的失败。

## Requirements（草案）

| ID | 要求 |
| --- | --- |
| R1 | 注册表与账号备份：内容去重（命中返回既有路径）、同秒追加序号、私有原子写。可复用 `CodexConfigManager::backup_file` 的合同。本轮保留全部既有备份，不执行保留清理。 |
| R2 | `delete_account` 删除快照前创建备份。 |
| R3 | `rename_account --force` 在任一备份失败时中止，不删除任何快照。 |

## Decisions

- 用户确认：不运行备份清理，保留全部旧备份。
- 删除前备份与现有账号备份沿用无期限留存。将来建立有界保留需单独决定。
- 失败错误沿用现有变体和 EN/ZH 操作失败前缀，不包含凭据或完整身份键。

## Acceptance Criteria（草案）

- [x] 同内容连续保存注册表不新增备份文件；同秒不同内容生成不同文件。
- [x] 既有旧备份均保留；不运行数量或期限清理。
- [x] `delete_account` 后可从备份恢复快照字节。
- [x] 模拟备份失败时 `rename_account --force` 返回错误，源快照与目标快照均未删除。
- [x] `cargo test -p ccr-codex`、`just lint-strict`、`just test` 通过。

2026-10-06 本地验收：AC1–AC4 见 `research/independent-check.md`；最终 AC5 由父任务 `just ci` 覆盖，Codex 381 passed、2 ignored，Rust workspace 2038 passed、0 failed、16 ignored。备份字节验证支持本地恢复，不证明远端凭据有效或已执行恢复。Windows 大小写别名与后续 I/O 的多文件回滚限制仍保留，详见父任务 `research/integration-validation.md`。

## Dependencies

- 先于 10-06-auth-defense-in-depth（P4）实施：两者都修改 `rename_account`。
