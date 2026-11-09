# Design：SQLite seam 缩水实施（单池化 + 注入式访问 + 死代码清除）

前提：否决式调研已推翻"整体收敛"（见 `research/veto-research.md`）。本设计只覆盖缩水后的五件事。seam 的身份认定：**`ccr-core::core::sqlite` 就是全仓 SQLite seam**（pool 工厂 + PRAGMA 统一），本任务不新建 seam crate，不移动它，只消除它上面的浅层与旁路。

## 1. 边界与契约

```
ccr-core::core::sqlite        ← seam（不动）：DbPool/DbConnection/PoolConfig/create_sqlite_pool/create_memory_sqlite_pool
   ↑                    ↑
ccr-db::database        ccr-store::storage::Database   ← 两个消费端各自保留
   （DbError 边界）          （CcrError 边界，不动）
   ↑
ccr-checkin managers（DbAccess 注入式） / src-tauri（AppState 池）
```

- 错误方向按 ccr-error-freeze ADR：seam 说 primitive（r2d2/rusqlite error），ccr-db 包 DbError，ccr-store 包 CcrError，各自边界 map_err；**不新增** `impl From<DbError> for CcrError`。
- `ccr::Database`（← ccr-store）在 public-api 冻结墙上：类型名、方法签名零变化；public_api_compat 快照必须零漂移。
- ccr-db / ccr-checkin 是 workspace 内部 crate（消费方仅 ccr-checkin、src-tauri，均在仓内），公共面允许同步修改。

## 2. 单池化（行为变化点，migration reviewer 重点）

### 现状
main.rs 对 `~/.ccr-ui/ccr-ui.db` 开两个池：`initialize()`（GLOBAL_POOL，max_size=10，跑迁移）→ `create_app_pool()`（AppState.db_pool，max_size=8，再跑迁移）。manager 层走全局池，commands 层走 AppState 池，连接上限互不知晓。

### 目标形状
```rust
// crates/ccr-db/src/database/mod.rs
/// 创建应用连接池并登记为全局池（同一实例）。
/// GLOBAL_POOL 与返回值共享同一池：manager 层 with_connection() 与
/// AppState 直取连接看到同一连接上限与迁移状态。
pub fn initialize_app_pool() -> Result<DbPool, DbError> {
    let pool = create_app_pool()?;          // 建池(max_size=8) + run_all_migrations（仅此一遍）
    let _ = GLOBAL_POOL.set(pool.clone());  // r2d2 Pool 是 Arc 语义，clone 共享实例
    Ok(pool)
}
```
main.rs：`initialize()` + `create_app_pool()` 两连击 → `let db_pool = ccr_db::database::initialize_app_pool()?;` 一处调用。`initialize()` 无其余生产消费方，删除。

### 不变量与取舍
- `with_connection`/`transaction`/`get_pool`/`is_initialized`/`shutdown`/`initialize_for_test` 签名与语义全部不动。
- **连接上限 10+8 → 8**：桌面负载低，WAL 下写路径本就单写者；busy_timeout=5000 + r2d2 connection_timeout=30s 兜底。同库双池对 WAL checkpoint/锁竞争反而更差，单池是净改善。
- 迁移从"同库跑两遍幂等"变为"跑一遍"——`is_migration_applied` 幂等语义未曾依赖双跑，零数据风险；usage.db 池（`create_usage_archive_pool`）不受影响。
- 失败模式不变：池建失败/迁移失败仍在 setup 阶段返回 Err → Tauri 启动报错，先于任何写入。
- 回滚：单 commit revert 即回到双池，无 schema 变化无数据迁移，可安全回退。

## 3. 注入式访问 DbAccess + AccountManager 试点

### 形状
```rust
// crates/ccr-db/src/database/mod.rs
/// 注入式数据库访问句柄：默认全局池；测试/嵌入场景注入独立池。
#[derive(Clone, Default)]
pub enum DbAccess {
    #[default]
    Global,
    Pool(DbPool),
}
impl DbAccess {
    pub fn with_connection<F, T>(&self, f: F) -> Result<T, DbError>
    where F: FnOnce(&rusqlite::Connection) -> Result<T, rusqlite::Error>;
    pub fn transaction<F, T>(&self, f: F) -> Result<T, DbError>
    where F: FnOnce(&rusqlite::Transaction<'_>) -> Result<T, rusqlite::Error>;
}
```
- 放 ccr-db::database（说 DbError），实现委托：Global 分支调既有自由函数，Pool 分支即注入池上的同构逻辑（提取共用私有 helper 避免两份 map_err）。
- 用 enum 不用 trait：无 dyn 开销、Clone 廉价（Pool 是 Arc）、错误类型单一；调用方零泛型污染。

### 试点范围（明确不做的）
- **只改 AccountManager**（PRD 点名的加密+持久化未覆盖路径）：加字段 `db: DbAccess`；`new(checkin_dir)` 默认 Global（全部现有调用方零变更）；新增 `with_db(checkin_dir, db)`；方法体 `database::with_connection(...)` → `self.db.with_connection(...)`（约 15 处机械替换）。
- 其余 4 个 checkin manager、ui_state、log_persistence **不迁移**：机制已就位（验收是"GLOBAL_POOL 不再是唯一入口"），等各自需要 manager 级测试时再改，避免 60+ 点位翻改膨胀 diff。
- account_manager.rs 测试重写为 manager 级：注入 `create_memory_pool()`（执行 `CREATE_TABLES_SQL` 后包 `DbAccess::Pool`），经 `AccountManager::with_db` 走真实方法路径（create→加密入库→get 可解密→掩码不泄露；list 不解密；update/delete）。现有 `static TEST_DB` + 直调 repo 的 harness 在本文件内被替换（其他 manager 的同款 harness 不动）。

## 4. executor.rs 删除（零调用死代码）

- 删 `crates/ccr-db/src/core/executor.rs`、`core/mod.rs` 的 `pub mod executor;`、`core/error.rs` 的 `ExecutorError`。
- ccr-checkin/core/error.rs:3 re-export 收窄为 `{DbError, MigrationError}`。
- Cargo.toml：删 `futures`；tokio 去掉 `process` feature，按 log_persistence 实际需要收敛（生产仅 `sync`；`#[tokio::test]` 所需 rt/macros 依 workspace 定义放 dev 侧或保留），以 `cargo check/test -p ccr-db` 为准。
- 不迁移不保留：全仓（含 src-tauri）零调用证据在 research 文档。

## 5. pool.rs 折叠 + checkin 去 wholesale re-export（零行为变化）

- database/mod.rs 顶部 `pub use ccr_core::core::sqlite::{DbConnection, DbPool, PoolConfig};`（类型同一性保持——本就是 ccr-core 类型的别名）；`create_pool`/`create_memory_pool` 连同 3 个测试移入 mod.rs；删 `pub mod pool`。
- import 路径更新：ccr-db 内部 `pool::{...}` 引用；src-tauri 4 处 `ccr_db::database::pool::DbPool` → `ccr_db::database::DbPool`（state.rs:14、services/usage.rs:1028、commands/usage.rs:353,1058）。
- ccr-checkin：删 lib.rs:9 `pub use ccr_db::database;`；`crate::database` → `ccr_db::database`（5 个 manager 生产 import + 各测试模块 + checkin_service.rs:2110，机械替换）；`models::checkin` 具名 re-export 保留。

## 6. 分步提交与验证（每步独立回滚点）

| # | 步骤 | 性质 | 验证 |
|---|---|---|---|
| 1 | 删 executor.rs + ExecutorError + 依赖瘦身 | 死代码删除 | `cargo test -p ccr-db -p ccr-checkin -- --test-threads=1` |
| 2 | pool.rs 折叠 + src-tauri import | 零行为 | 同上 + `just tauri-check` |
| 3 | 单池化 initialize_app_pool + main.rs | 行为变化 | 同上 + `just tauri-check`；**sqlite-migration-reviewer 审查本步 diff** |
| 4 | DbAccess + AccountManager 试点 + manager 级测试 | 测试能力 | `cargo test -p ccr-db -p ccr-checkin -- --test-threads=1` |
| 5 | checkin 去 wholesale re-export | 零行为 | `cargo test -p ccr-checkin -- --test-threads=1` + `just tauri-check` |

收尾全量：`just version-check` → `just fmt-check` → `just lint-strict` → `just test`；public_api_compat 零快照变化；spec 回写（ccr-db guidelines：单池初始化 + DbAccess 契约 + executor 移除；决策记录：三库分离/双 runner 保留理由）。

## 7. 风险清单

- **单池连接上限收敛**（18→8）：见 §2 取舍；若真出现池饥饿，PoolConfig 调 max_size 是一行改动。
- **OnceLock 双 set**：新路径只 set 一次；`initialize_for_test` 在测试进程内先到先得的既有语义不变。
- **tokio feature 收敛踩空**：以编译+测试为准，宁可少收敛不引入功能倒退。
- **checkin import 机械替换遗漏**：`rg 'crate::database' crates/ccr-checkin` 收敛到 0 作为完成判据。
