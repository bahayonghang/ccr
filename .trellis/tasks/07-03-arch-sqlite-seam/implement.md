# Implement：SQLite seam 缩水实施

前置：`research/veto-research.md`（证据）、`prd.md`（缩水后需求）、`design.md`（形状与取舍）。五步各自独立提交、独立回滚；步骤内"验证"全绿才进下一步。

## Step 1：删除 executor.rs 死代码 + 依赖瘦身

- [ ] 删 `crates/ccr-db/src/core/executor.rs`
- [ ] `crates/ccr-db/src/core/mod.rs`：删 `pub mod executor;`
- [ ] `crates/ccr-db/src/core/error.rs`：删 `ExecutorError` 枚举
- [ ] `crates/ccr-checkin/src/core/error.rs:3`：re-export 收窄为 `{DbError, MigrationError}`
- [ ] `crates/ccr-db/Cargo.toml`：删 `futures`；tokio 去 `process` feature（按编译结果收敛，dev 测试需求优先保编译）
- 验证：`cargo test -p ccr-db -p ccr-checkin -- --test-threads=1`；`rg 'ExecutorError|execute_binary' crates ccr-ui/src-tauri` 零命中
- 提交：`refactor(db): [AI] 🧹 删除零调用的 executor.rs 及 ExecutorError`

## Step 2：pool.rs 浅层折叠

- [ ] `database/mod.rs`：`pub use ccr_core::core::sqlite::{DbConnection, DbPool, PoolConfig};`；移入 `create_pool`/`create_memory_pool`（保留 DbError 包装与日志）及 pool.rs 的 3 个测试
- [ ] 删 `crates/ccr-db/src/database/pool.rs` 与 `pub mod pool;`
- [ ] ccr-db 内部 `pool::` 引用改直引；src-tauri 4 处 `ccr_db::database::pool::DbPool` → `ccr_db::database::DbPool`（state.rs / services/usage.rs / commands/usage.rs ×2）
- 验证：`cargo test -p ccr-db -- --test-threads=1`；`just tauri-check`
- 提交：`refactor(db): [AI] 🧹 折叠 pool.rs 纯转发浅层进 database/mod.rs`

## Step 3：单池化（行为变化点）

- [ ] `database/mod.rs`：新增 `initialize_app_pool() -> Result<DbPool, DbError>`（create_app_pool → set GLOBAL_POOL(clone) → 返回）；删 `initialize()`
- [ ] `ccr-ui/src-tauri/src/main.rs`：`initialize()+create_app_pool()` 两调用 → `initialize_app_pool()` 一处；启动日志相应微调
- [ ] 确认 `with_connection/transaction/get_pool/is_initialized/shutdown/initialize_for_test` 未动
- 验证：`cargo test -p ccr-db -p ccr-checkin -- --test-threads=1`；`just tauri-check`；`rg 'database::initialize\(\)'` 生产零命中
- [ ] **sqlite-migration-reviewer 审查本步 diff**（池生命周期与迁移执行次数变化）
- 提交：`refactor(db): [AI] ♻️ ccr-ui.db 单池化：GLOBAL_POOL 与 AppState 共享同一池实例`

## Step 4：DbAccess 注入 + AccountManager 试点

- [ ] `database/mod.rs`：`DbAccess` 枚举（`#[default] Global` | `Pool(DbPool)`）+ `with_connection`/`transaction`（与自由函数共用私有 helper）
- [ ] `account_manager.rs`：加 `db: DbAccess` 字段；`new()` 走 Global 默认；新增 `with_db()`；方法体 `database::with_connection` → `self.db.with_connection`
- [ ] 重写 account_manager 测试为 manager 级：内存池 + `CREATE_TABLES_SQL` + `with_db` 注入；覆盖 create（加密入库→get 解密→掩码不泄露）、list 不解密、update、delete、按 provider 查询；移除本文件 `static TEST_DB` harness
- [ ] 其余 manager / ui_state / log_persistence 不动（design §3）
- 验证：`cargo test -p ccr-db -p ccr-checkin -- --test-threads=1`
- 提交：`feat(db): [AI] ✨ DbAccess 注入式访问 + AccountManager manager 级单测试点`

## Step 5：checkin 去 wholesale re-export

- [ ] 删 `crates/ccr-checkin/src/lib.rs:9` `pub use ccr_db::database;`
- [ ] `crate::database` → `ccr_db::database`：5 个 manager 生产 import、各测试模块引用、checkin_service.rs:2110
- 验证：`cargo test -p ccr-checkin -- --test-threads=1`；`just tauri-check`；`rg 'pub use ccr_db::database' crates/ccr-checkin` 与 `rg 'crate::database' crates/ccr-checkin` 双零命中
- 提交：`refactor(checkin): [AI] 🧹 移除 ccr_db::database wholesale re-export，改具名路径`

## 收尾（Phase 2.2 末轮全量 + Phase 3）

- [ ] 全量：`just version-check` → `just fmt-check` → `just lint-strict` → `just test`（public_api_compat 零快照变化）
- [ ] spec 回写（trellis-update-spec）：
  - `ccr-db/backend/backend-guidelines.md`：单池初始化契约（initialize_app_pool）、DbAccess 注入模式、executor/ExecutorError 已删、pool 类型出处
  - 决策记录：三库分离有意 + migration runner 双套保留理由（防未来审查重复提议）
  - `ccr-checkin/backend/backend-guidelines.md`：database facade 描述更新（具名路径，无 wholesale re-export）
- [ ] prd.md 验收清单勾选；journal 记录；`task.py complete` + 归档提交

## 回滚

任一步失败：`git revert <该步 commit>` 即可，步骤间无 schema/数据迁移耦合；Step 3 revert 后回到双池行为（已知可用状态）。
