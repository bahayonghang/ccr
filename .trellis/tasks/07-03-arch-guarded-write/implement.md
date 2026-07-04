# Implement: 统一 guarded write 模块

> 前置：已读 `.trellis/spec/ccr-core/backend/atomic-writer.md`、`backend-guidelines.md`。
> 批次约束：B0 先行，B1-B4 依赖 B0 但互相独立；同一文件写路径一次切换到位。

## B0 — ccr-core 深模块（地基）

- [ ] `atomic_writer.rs`：`AtomicWriter` 增加 `secret(bool)` builder；temp 创建后写内容前 chmod 0o600（`#[cfg(unix)]`）；sync/async 写路径 rename 前增加 `sync_all()`（fsync）。
- [ ] 新建 `core/guarded_write.rs`：`WriteOptions` / `BackupPolicy` / `BACKUP_KEEP=10` / `write_guarded` / `write_guarded_async`（spawn_blocking）/ `backup_guarded` / `pub(crate) lock_resource_name`（内联 fnv1a64）。
- [ ] `core/mod.rs` + `lib.rs` 导出新模块（公共 API 文档英文，实现注释中文）。
- [ ] `fileio.rs`：`write_toml/write_json`（含 async）委托 guarded write 默认选项；新增 `write_toml_opts/write_json_opts`（含 async）。
- [ ] 按 design §7 写单元测试（备份命名逐字节、轮换 keep-10、secret 0600、LockTimeout、多线程无撕裂、崩溃代理、CCR_LOCK_DIR 隔离）。
- [ ] 验证：`cargo test -p ccr-core -- --test-threads=1` && `cargo clippy -p ccr-core --all-targets --all-features -- -D warnings`
- [ ] ⏸ 回滚点：单独 commit `feat(core)`

## B1 — ccr-config 迁移

- [ ] `config_file_handler.rs`：`backup()` 委托 `backup_guarded(SameDir{tag})`，删除内联 copy+轮换；`list_backups` 不动。
- [ ] `platforms/base.rs`：`save_profiles_to_toml` / `update_current_config` 改单次 `write_guarded(Dir{backups_dir, "profiles"})`（命名 RMW 锁保留）；删除 `backup_with_rotation`。
- [ ] `platform_config.rs`：`backup()` 委托 `backup_guarded(Dir{~/.ccr/backups, "config"|"config_{tag}"})`；`restore()` 改读字节 + `write_guarded`；`list_backups` 过滤器 `starts_with("config_")` → `starts_with("config")`；`cleanup_old_backups` 不动。
- [ ] `config_service.rs` `import_config`：内联备份块 → `self.config_manager.backup(Some("import_backup"))`。
- [ ] 备份命名回归测试：新旧命名均被 list_backups 发现。
- [ ] 验证：`cargo test -p ccr-config -- --test-threads=1` && clippy 同上
- [ ] ⏸ 回滚点：commit `refactor(config)`

## B2 — ccr-sync 迁移

- [ ] `sync/config.rs` `save()`：改 `fileio::write_toml_opts(secret: true)`；加 Unix 0600 断言测试（Windows cfg 跳过）。
- [ ] `sync/folder_manager.rs` `save_config()`：删除自建 `<dir>/.locks` LockManager 块，直接 fileio（锁已内置）。
- [ ] `sync/service.rs` `pull_file()`：`fs::write` → `write_guarded_async`（默认选项）。
- [ ] 验证：`cargo test -p ccr-sync -- --test-threads=1` && clippy
- [ ] ⏸ 回滚点：commit `refactor(sync)`

## B3 — ccr-store 迁移

- [ ] `budget_manager.rs` / `pricing_manager.rs` `save_config()`：删除手工序列化/手搓 tempfile 块，改 `fileio::write_toml`。
- [ ] 验证：`cargo test -p ccr-store -- --test-threads=1` && clippy
- [ ] ⏸ 回滚点：commit `refactor(store)`

## B4 — ccr-checkin 迁移

- [ ] `core/crypto.rs` `save_key()`：改 `write_guarded(secret: true)`，`CcrError → CryptoError::KeyWriteError` 映射；key 权限 0600 断言测试（Unix）。
- [ ] 验证：`cargo test -p ccr-checkin -- --test-threads=1` && clippy
- [ ] ⏸ 回滚点：commit `refactor(checkin)`

## B5 — 收尾全量检查

- [ ] AC#1 扫描：`rg -n 'fs::write\(|fs::rename\(' crates/ccr-core crates/ccr-config crates/ccr-sync crates/ccr-store crates/ccr-checkin` 生产代码仅命中 guarded_write/atomic_writer/lock 内部与 `#[cfg(test)]`。
- [ ] `just version-check` → `just fmt-check` → `just lint-strict` → `just test`
- [ ] `cargo test -p ccr-core -p ccr-config -p ccr-sync -p ccr-store -p ccr-checkin -- --test-threads=1`
- [ ] Spec 更新（trellis-update-spec）：atomic-writer.md 增 fsync/secret/guarded 契约 + 遗留债务清单（design §8）。
- [ ] 对照 prd.md Acceptance Criteria 逐条勾验；journal 记录；最终 commit。

## 全局回滚策略

任一批次失败且无法快速修复 → `git revert` 该批次 commit；B0 回滚需先回滚已合入的 B1-B4（它们依赖新 API）。磁盘产物（新命名备份、统一锁目录）对旧代码无害。
