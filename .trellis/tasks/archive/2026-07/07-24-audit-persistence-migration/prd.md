# 持久化与 Migration 正确性

> 父任务：`07-24-audit-remediation` ｜ 覆盖：P1-05、P1-08、P2-10、P2-18 ｜ 报告 Epic D1-D5

## Goal

修复凭据文件权限降级、v3 迁移静默丢数据、迁移缺事务、崩溃持久性四个持久化正确性问题。

## 背景 / 证据（已核实）

### P1-05 secret 权限降级
- `crates/ccr-core/src/core/atomic_writer.rs:288-321` — `AsyncAtomicWriter::write_async` 用 `async_fs::write` 新建 temp（mode 受 umask 控制，典型 022 → 0644），rename 覆盖目标不保留原 mode/ACL
- `crates/ccr-cli/src/managers/settings.rs:201-215` — `save_atomic_async` 用该 writer 保存含 `ANTHROPIC_AUTH_TOKEN` 的 Claude settings

### P1-08 v3 静默丢数据
- `crates/ccr-db/src/database/migrations.rs:589,619` — `.filter_map(|r| r.ok())` 丢弃 row/pricing 解码错误
- `crates/ccr-db/src/database/migrations.rs:663` — `let _ = update_stmt.execute(...)` 忽略 UPDATE 错误
- `crates/ccr-db/src/database/migrations.rs:629` — 坏 JSON 静默跳过
- `crates/ccr-db/src/database/migrations.rs:567-571` — 上述失败后仍写 v3 marker 并 log completed

### P2-10 迁移缺事务
- `migrations.rs` v3/v4/v5 直接在 `conn` 上多语句执行，schema/data/marker 未置于同一显式 transaction

### P2-18 崩溃持久性
- `crates/ccr-core/src/core/atomic_writer.rs:358-361` — Unix async path fsync temp 后只 rename，未 fsync parent directory

## Requirements

### Secret writer（D1）
- [x] `AsyncAtomicWriter` 增加 `AsyncAtomicWriterOptions { secret, preserve_mode }`（对齐同步 writer 的 secret policy）
- [x] Unix：temp 以显式 `0o600`/更严格既有 mode 创建；Windows 保留既有 DACL
- [x] 所有已知 credentials/profile/settings writer 使用 `secret(true)`；`check-secret-writes.py` 已接入 `just lint-strict`

### 父目录 fsync（D2）
- [x] rename 后 Unix 打开 parent dir `sync_all()`；Windows 以 `MOVEFILE_WRITE_THROUGH` 为持久化边界并已文档化

### v3 修复迁移（D4）
- [x] 新增 v16 repair migration（不改已发布 v3）：事务内逐行处理缺失/不一致 extracted fields
- [x] 坏 JSON 写 coded `migration_rejections` 并计数；row/update error 传播
- [x] postcondition 验证 remaining/rejection/accounting；marker 仅在验证完成后写并含 counts

### 迁移事务框架（D3）
- [x] 引入 `apply_migration`：事务内执行 + postconditions + marker + commit；v3-v5 已迁移

## Acceptance Criteria

- [x] secret writer：WSL2 验证 umask 000/022/077 与 0400/0600/0644；Windows 实文件验证 DACL 保留
- [x] migration：row decode、malformed JSON、UPDATE trigger、DDL rollback、file-backed historical-v3 backup 与二次运行幂等均有测试
- [x] v3 partial-migration 的三类 silent error 模式归零
- [x] `just lint-strict` + `just test`（串行 workspace tests）通过

## Out of Scope

- 不修改冻结的公开 `CcrError` 变体
- 不重写 SQLite 数据层或无关历史迁移
- 不以模拟 mode 替代 Windows ACL 真实文件系统验收

## Notes

- 需保持既有原子写/文件锁契约；改动 atomic_writer 会广泛影响，先跑全量 `just test`
- 触发 rust-security-reviewer（secret mode）+ sqlite-migration-reviewer（迁移事务）复查
- repair migration 前自动 DB backup，repair 可幂等重跑（报告阶段 0 回滚策略）
