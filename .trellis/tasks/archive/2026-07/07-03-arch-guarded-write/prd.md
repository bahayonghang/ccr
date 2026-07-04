# 统一 guarded write 模块

## Goal

把 lock→backup(轮换)→原子写(fsync+rename)→权限(0o600) 收进 ccr-core 一个深模块接口（在既有 `AtomicWriter` seam 上加深，而非另起炉灶）。消除 5 种原子写风格 / 4 套 backup 实现 / 2 个 lock 目录 split-brain；修复 secret 落盘 world-readable。审查候选 1（Strong，首推）。

## Requirements

### 现状（探索报告定位）

- 原子写 5 种风格：canonical `ccr-core::fileio::write_toml→AtomicWriter`（config_file_handler.rs:113、platform_config.rs:371、sync/config.rs:95、sync/folder_manager.rs:156）；手搓 `ccr-checkin/core/crypto.rs:110-131 save_key`（唯一设 0o600 的写入方）；非原子 `ccr-store/budget_manager.rs:72`、`pricing_manager.rs:94`、`ccr-sync/service.rs:265`（裸 fs::write）。
- backup 4 套独立实现且都不在 writer 内：config_file_handler.rs:132（同目录 .bak keep-10）、platforms/base.rs:32 backup_with_rotation、platform_config.rs:400 + cleanup_old_backups、config_service.rs:409-417（inline fs::copy 无轮换）。
- 锁分裂：base.rs 用 `LockManager::with_default_path()`（~/.claude/.locks）；sync/folder_manager.rs:141-147 自建 `<config_dir>/.locks` —— 两个锁目录互不互斥（split-brain）；sync/config.rs:94（含 WebDAV 密码）完全不加锁；各 manager 的 save 把锁责任下放给调用方注释。
- 安全缺陷：`AtomicWriter` 不设权限，经 fileio 写出的 secret（如 WebDAV 密码）world-readable。

### 要做的

1. ccr-core 提供单一 guarded write 接口（形如 `write_guarded(path, bytes, opts)`）：内部完成 加锁（统一锁目录）→ 备份+轮换（统一策略）→ temp 写 → fsync → rename → 按需 0o600。选项控制"是否备份/是否 secret 权限"。
2. 迁移全部 8+ 调用点：config_file_handler、platforms/base、platform_config、config_service、sync/config、sync/folder_manager、sync/service、budget_manager、pricing_manager、checkin crypto save_key。调用点自身的 backup/锁/权限代码删除。
3. 备份策略统一后，保留一种轮换规则（数量与目录在设计阶段定夺，兼容既有备份的可发现性）。
4. Windows 替换语义遵循既有 `atomic-writer.md` spec 契约。

### 约束

- 不改变各文件的磁盘格式与路径；只收敛"怎么写"，不动"写什么"。
- 分批迁移可接受，但同一文件的写入路径不得新旧并存超过一个子提交。
- 动手前读 `.trellis/spec/ccr-core/backend/atomic-writer.md` 与 `backend-guidelines.md`。

## Acceptance Criteria

- [ ] 全仓 `rg 'fs::write|fs::rename'` 在持久化路径上只命中 guarded write 模块内部（测试与非持久化临时文件除外）。
- [ ] 锁目录仅一个；并发写互斥有测试（通过 guarded write 接口测，不再依赖调用方自觉）。
- [ ] 备份+轮换行为在 guarded write 模块内有单元测试；4 套旧实现删除。
- [ ] secret 类文件（checkin key、sync.toml 等）落盘权限 0o600 有断言测试（Unix；Windows 跳过并注明）。
- [ ] 崩溃安全（temp+rename 不留半成品）有测试。
- [ ] `just lint-strict` 与 `just test` 通过；`cargo test -p ccr-core -p ccr-config -p ccr-sync -p ccr-store -p ccr-checkin -- --test-threads=1` 通过。
- [ ] `atomic-writer.md` spec 同步更新（trellis-update-spec）。

## Notes

- 复杂任务：`task.py start` 前需补 design.md（接口形状、选项集、迁移批次）与 implement.md。
- 与 07-03-arch-secret-newtype 天然衔接（该任务的 Secret 类型可作为 opts 中"secret 权限"的判定来源），但互不阻塞。
