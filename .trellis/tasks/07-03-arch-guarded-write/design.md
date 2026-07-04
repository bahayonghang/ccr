# Design: 统一 guarded write 模块

## 0. 探索结论修正（相对 prd.md 的事实校准）

实现前全量核实了 PRD 定位的调用点，两处事实需要修正，不影响任务成立：

1. **Unix 下经 sync `AtomicWriter` 写出的文件实际是 0o600（偶然行为）**：`tempfile::NamedTempFile` 在 Unix 上以 0o600 创建，rename 保留源文件权限。因此 sync.toml 在 Unix 上今天并非 world-readable。但这是 tempfile 的实现细节而非契约；`AsyncAtomicWriter` 走手工 temp 路径（`async_fs::write`，umask 默认 0o644）则确实泄露。本设计把"secret → 0o600"变成显式契约 + 测试，而不是依赖偶然。
2. **`ccr-cli` 下的 `config_file_handler.rs` / `budget_manager.rs` 等同名文件是 legacy re-export**（`pub use ccr_config::...`，受 `public-api-boundary.md` 冻结保护），不是重复实现，无需迁移。

另外发现 PRD 未列出的同类债务（**本任务不做**，记入 §8 遗留清单）：`ccr-cli/src/sync/commands.rs:694` 非原子写配置、`ccr-codex`/`ccr-skills` 多处直接 `AtomicWriter`/手搓 temp+rename。它们不在 PRD 验收的 5 crate 范围内。

## 1. 范围与验收口径

- **迁移范围（5 crate）**：`ccr-core`（新模块 + fileio/AtomicWriter 加深）、`ccr-config`、`ccr-sync`、`ccr-store`、`ccr-checkin`。
- **AC#1 的 rg 检查口径**：`fs::write|fs::rename` 在上述 5 crate 的**生产代码持久化路径**上只允许命中 guarded write / atomic_writer / lock 模块内部；`#[cfg(test)]`、锁文件操作、`remove_file` 类清理不算。其余 crate 记为后续任务。
- 不改变任何文件的磁盘格式与路径；只收敛"怎么写"。

## 2. 分层与模块边界

```
fileio (序列化层)      write_toml / write_json / *_async / *_opts
   │  serialize → bytes
   ▼
guarded_write (策略层)  lock → backup(轮换) → 物理写   ← 本任务新增的深模块
   │
   ▼
atomic_writer (物理层)  temp(secret→0600) → write → fsync → rename(Win 重试)
```

- **`ccr-core/src/core/guarded_write.rs`（新）**：唯一的"持久化策略"入口。加锁、备份轮换、调用物理层。
- **`atomic_writer.rs`（加深，不另起炉灶）**：
  - `AtomicWriter` 增加 `secret(bool)` builder 选项：temp 文件创建后、写内容**前** chmod 0o600（Unix；Windows no-op）。
  - sync/async 两个 writer 统一增加 **fsync**（`sync_all`）后再 rename——修复现状"rename 可能先于数据落盘"的窗口（现状仅 pricing_manager 手工做了 fsync）。
  - Windows 替换语义（MoveFileExW + REPLACE_EXISTING + WRITE_THROUGH + 重试）不变，遵守 `atomic-writer.md` spec。
- **`fileio`**：`write_toml/write_json`（含 async）改为委托 `guarded_write`（默认选项：无备份、非 secret）。新增 `write_toml_opts / write_json_opts`（含 async）透传 `WriteOptions`。**所有 fileio 调用方免费获得锁 + fsync**。

## 3. 接口契约

```rust
// ccr-core/src/core/guarded_write.rs（公共 API 文档英文）
pub const BACKUP_KEEP: usize = 10;

#[derive(Debug, Clone, Default)]
pub struct WriteOptions {
    pub backup: BackupPolicy,      // 默认 None
    pub secret: bool,              // 默认 false；true → 0o600（Unix）
    pub lock_timeout: Duration,    // 默认 10s（测试注入用）
}

#[derive(Debug, Clone, Default)]
pub enum BackupPolicy {
    #[default]
    None,
    /// 同目录：`{filename}.{tag_}{ts}.bak`，轮换匹配 starts_with(filename) && ends_with(".bak")
    SameDir { tag: Option<String> },
    /// 独立目录：`{prefix}.{ts}.{ext}.bak`，轮换匹配 starts_with(prefix) && ends_with(".bak")
    Dir { dir: PathBuf, prefix: String },
}

pub fn write_guarded(path: &Path, bytes: &[u8], opts: &WriteOptions) -> Result<()>;
pub async fn write_guarded_async(path: &Path, bytes: Vec<u8>, opts: WriteOptions) -> Result<()>; // spawn_blocking 包同步实现
/// 显式备份入口（决策点备份，如 restore/import 前）。源不存在或 policy=None → Ok(None)。
pub fn backup_guarded(path: &Path, policy: &BackupPolicy) -> Result<Option<PathBuf>>;
```

`write_guarded` 内部顺序（单次锁获取覆盖 2-4）：

1. 由目标路径派生锁名，`LockManager::with_default_path()`（`~/.claude/.locks`，`CCR_LOCK_DIR` 可覆盖）取 `FileLock`；
2. 按 `BackupPolicy` 备份 + 轮换（keep=10，源不存在则跳过）；
3. `AtomicWriter::new(path).secret(opts.secret).write(bytes)`（temp→0600→写→fsync→rename）；
4. 释放锁（RAII）。

**锁名派生**：`gw_{file_stem 消毒}_{fnv1a64(绝对路径小写):016x}.lock`。fnv1a64 内联实现（约 6 行，无新依赖；std 的 SipHash 每进程随机种子，不可用于跨进程锁名）。Windows 用小写归一（NTFS 大小写不敏感）；用 `std::path::absolute` 而非 canonicalize（目标首次写入时尚不存在）。符号链接别名不归一——现状的命名锁同样不处理，不回退。

## 4. 并发与死锁分析

- **guarded write 的内部锁是"写-写互斥"叶子锁**：锁名由路径派生、调用方永远不直接持有它 → 锁序恒为 `{调用方 RMW 锁 (ccr_config / platform_profiles_* / CONFIG_LOCK)} → {gw 路径锁}`，无环。
- **不触碰 `CONFIG_LOCK`**：`config_service.lock_config()` 已持有 CONFIG_LOCK（std Mutex 不可重入）再调 save → fileio → guarded write，若 guarded write 内部再取 CONFIG_LOCK 会立即死锁。guarded write 只取自己的文件锁。
- **RMW 事务性仍是调用方责任**（与现状一致）：guarded write 保证单次写的互斥与完整性，不保证 load→mutate→save 序列原子。`base.rs` 的命名锁、`config_service` 的 lock_config 全部保留。`folder_manager` 的 RMW 竞态（load 在锁外）是既有问题，本任务不修（记入 §8）。
- 双重锁开销（调用方命名锁 + gw 路径锁）为一次额外 open+flock，可忽略。

## 5. 备份命名兼容矩阵

| 调用点 | 现状命名 | 迁移后 policy | 新命名 | 可发现性 |
|---|---|---|---|---|
| config_file_handler.backup | `.ccs_config.toml.{tag_}{ts}.bak` 同目录 keep-10 | `SameDir{tag}` | **逐字节相同** | list_backups 过滤器不变 |
| config_service.import_config 内联备份 | `.ccs_config.toml.import_backup_{ts}.bak` 无轮换 | 改调 `config_manager.backup(Some("import_backup"))` | 相同，**新增轮换** | 同上 |
| platforms/base.rs backup_with_rotation | `{prefix}.{ts}.{ext}.bak` 于 backups_dir keep-10 | `Dir{dir, prefix}` | **逐字节相同** | 轮换匹配规则相同 |
| platform_config.backup | `config_{tag_}{ts}.toml.bak` 于 `~/.ccr/backups` | `Dir{dir, prefix: "config"或"config_{tag}"}` | `config.{ts}.toml.bak` / `config_{tag}.{ts}.toml.bak` | **list_backups 过滤器从 `starts_with("config_")` 放宽为 `starts_with("config")`**，新旧都可见 |

- `platform_config.cleanup_old_backups(keep_count)` 是公共维护 API（keep 可调），保留原实现不动；`Dir` 轮换只管写路径上的 keep-10。
- `platform_config.restore()` 现为 `fs::copy` 直写目标（非原子）→ 改为读备份字节 + `write_guarded`（无隐式备份，`config_service.restore_config` 已在外层显式备份 `pre_restore`）。

## 6. 各调用点迁移方案（8+ 全景）

| # | 调用点 | 现状 | 迁移后 | 删除的代码 |
|---|---|---|---|---|
| 1 | ccr-config `config_file_handler.rs` save/backup | fileio + 自制备份轮换 | save 不动（fileio 已 guarded）；backup 委托 `backup_guarded(SameDir)` | 内联 copy+轮换逻辑 |
| 2 | ccr-config `platforms/base.rs` | 命名锁 + `backup_with_rotation` + 裸 AtomicWriter ×2 | 命名锁保留；写改 `write_guarded(Dir policy)` ×2 | `backup_with_rotation` 整个函数 |
| 3 | ccr-config `platform_config.rs` save/backup/restore | fileio / fs::copy / fs::copy | save 不动；backup 委托 `backup_guarded(Dir)`；restore 改 `write_guarded` | 内联 copy 备份 |
| 4 | ccr-config `config_service.rs` import_config | 内联 fs::copy 无轮换 | `self.config_manager.backup(Some("import_backup"))` | 内联备份块 |
| 5 | ccr-sync `sync/config.rs` save | fileio，无锁，权限靠偶然 | `fileio::write_toml_opts(.., secret: true)` | — |
| 6 | ccr-sync `sync/folder_manager.rs` save_config | 自建 `<dir>/.locks` + fileio | 直接 fileio（内部已锁，统一锁目录） | 自建 LockManager 块（split-brain 消除） |
| 7 | ccr-sync `sync/service.rs` pull_file | `tokio fs::write` 非原子 | `write_guarded_async`（无备份、非 secret） | — |
| 8 | ccr-store `budget_manager.rs` save_config | 手工序列化 + 裸 fs::write | `fileio::write_toml` | 手工序列化+目录创建 |
| 9 | ccr-store `pricing_manager.rs` save_config | 手搓 tempfile+fsync+persist | `fileio::write_toml` | 手搓块（fsync 由物理层保证） |
| 10 | ccr-checkin `core/crypto.rs` save_key | 手搓 temp+rename+事后 chmod | `write_guarded(secret: true)`，错误映射 `CryptoError::KeyWriteError` | 手搓块（0600 提前到写内容前） |

行为变化点（除写法收敛外）：sync.toml/checkin key 权限成为显式契约；import 备份获得轮换；restore/pull_file 变原子；所有 fileio 写获得锁 + fsync。同一文件的写路径一次性切换，无新旧并存。

## 7. 测试设计（guarded_write 单元测试为主）

1. 基本写入 + 覆盖写（内容断言）。
2. `SameDir` / `Dir` 备份命名逐字节断言 + keep-10 轮换（写 15 次备份剩 10）。
3. `secret: true` → 文件 mode 0o600（`#[cfg(unix)]`；Windows 下 `#[cfg(windows)]` 跳过并注释说明）。
4. 并发互斥：预先手工持有派生锁 → `write_guarded(lock_timeout=100ms)` 返回 `LockTimeout`（锁名派生函数 `pub(crate)` 供测试）。
5. 多线程压测：N 线程各写不同完整 payload 循环 → 终态文件必为某个完整 payload（无撕裂）。
6. 崩溃安全代理测试：temp 创建失败（父路径是文件）→ 旧内容原样保留；沿用 atomic_writer 既有 Windows 重试耗尽测试。
7. `CCR_LOCK_DIR` 隔离：测试统一用 tempdir 覆盖锁目录（repo 约定 `--test-threads=1`，env 变更安全；ccr-core 有 TestLogEnv 夹具模式可参照）。
8. fileio 委托后既有 fileio/atomic_writer 测试全部保持绿色（回归面）。

## 8. 遗留债务（本任务明确不做，回写 spec 时记录）

- `ccr-cli/src/sync/commands.rs:694` 非原子写配置；`ccr-cli/platforms/{gemini,droid}.rs` 裸 fs::write settings；`ccr-codex`/`ccr-skills` 直接 AtomicWriter / 手搓 temp+rename 的调用点 → 后续增量迁移 guarded write。
- `folder_manager.add_folder` 等 RMW 序列 load 在锁外（既有竞态，与本任务的写互斥正交）。
- `AsyncAtomicWriter` 手工 temp 路径在 Unix 上的 umask 权限（直接使用者都在范围外 crate；本次仅给它加 fsync）。

## 9. 回滚

- B0（core 模块）单独提交；B1-B4 每 crate 一个提交，互不依赖（都只依赖 B0）；任一批次可独立 `git revert`。
- 磁盘兼容性：新备份命名与旧模式并存可发现，回滚后旧代码仍能列出新备份（`.bak` 后缀匹配不变）。锁目录统一后 `<config_dir>/.locks` 遗留目录无害，不清理。

## 10. Spec 同步

完成后更新 `.trellis/spec/ccr-core/backend/atomic-writer.md`：新增 fsync 契约、secret 权限契约、guarded write 场景（锁/备份/轮换）与"禁止在 5 crate 持久化路径手搓 fs::write"的 Wrong/Correct 对照（walk trellis-update-spec 流程）。
