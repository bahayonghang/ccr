# Codex Auth 注册表备份策略与破坏性操作前备份

## Goal

为注册表与账号备份建立去重、同秒序号与有界保留；让删除和强制重命名在破坏性步骤前有可靠备份。

## Authorization and Status

2026-10-06 用户要求为 10-06-reference-hardening 审计报告的后续任务提案 P1–P6 创建 Trellis 任务。本版为规划种子，未开始实施；需求与待决问题需经 brainstorm 收敛，复杂任务补齐 design.md 与 implement.md，并经用户评审后 `task.py start`。规划基线：dev @ 73b47563。证据锚点取自审计报告（.trellis/tasks/archive/2026-10/10-06-reference-hardening/research/codex-auth-audit.md），实施前需复核行号。

## Background and Evidence

- F10（低）：`codex_registry_store.rs:72` 每次 `save` 前执行 `let _ = self.backup();`，`:83-101` 写 `auth_registry_{ts}.toml`；账号备份见 `codex_auth_service.rs:1332-1356`。`auth/backups` 下的注册表与账号备份没有清理，数量无上限。
- F11（低）：`codex_registry_store.rs:93-98`、`codex_auth_service.rs:1312-1330` 使用秒级文件名加 `fs::copy`，同秒覆盖；备份权限跟随源文件模式或目录 ACL。
- F21（中）：`codex_auth_service.rs:1288-1293` 的 `delete_account` 直接 `fs::remove_file`，不先备份。规范要求 backup-before-destructive-change。
- F22（中）：`codex_auth_service.rs:1507` 忽略 `backup_account_auth(new_name)` 的失败，随后 `:1509-1513` 删除冲突快照；`:1519-1520` 同样忽略源快照与注册表备份的失败。

## Requirements（草案）

| ID | 要求 |
| --- | --- |
| R1 | 注册表与账号备份：内容去重（命中返回既有路径）、同秒追加序号、私有原子写、有界保留。可复用 `CodexConfigManager::backup_file` 的合同。 |
| R2 | `delete_account` 删除快照前创建备份。 |
| R3 | `rename_account --force` 在任一备份失败时中止，不删除任何快照。 |

## Open Decisions（需用户确认）

- 注册表与账号备份的保留数量。当前无上限；设上限会删除旧备份。
- `delete_account` 备份的保留期。用户要求删除的凭据会额外留存，需要决定数量或期限。
- `rename_account --force` 中止时的错误文本（用户可见，EN/ZH）。

## Acceptance Criteria（草案）

- [ ] 同内容连续保存注册表不新增备份文件；同秒不同内容生成不同文件。
- [ ] 保留数量符合确认值。
- [ ] `delete_account` 后可从备份恢复快照字节。
- [ ] 模拟备份失败时 `rename_account --force` 返回错误，源快照与目标快照均未删除。
- [ ] `cargo test -p ccr-codex`、`just lint-strict`、`just test` 通过。

## Dependencies

- 先于 10-06-auth-defense-in-depth（P4）实施：两者都修改 `rename_account`。
