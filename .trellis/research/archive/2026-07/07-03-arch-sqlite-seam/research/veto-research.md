# 否决式调研：单一 SQLite store seam（2026-07-05）

结论速览：**否决门触发，任务按 PRD 预案缩水**——三个 DB 文件分离是有意设计，跨栈 crate 级 seam 会制造新耦合且 pool 工厂本就已唯一；保留可落地的四件事：单池化、注入式访问试点、删 executor 死代码、删 checkin wholesale re-export。

## 前提 1（否决门）：三个 DB 文件分离是否有意？——有意，不合并

| DB 文件 | 所属栈 | 进程 | 证据 |
|---|---|---|---|
| `~/.ccr/data.db` | ccr-store `Database` | ccr CLI/TUI | sessions/search_history/history 表；生产消费方 `sessions/indexer.rs:25`、`history.rs:181`（`init_default`） |
| `~/.ccr-ui/ccr-ui.db` | ccr-db `database` | Tauri 桌面 | checkin/ui_state/ssh/monitoring 等；`get_db_path()` |
| `~/.ccr/analytics/usage.db` | ccr-db `database` | Tauri 桌面 | durable usage 归档；`get_usage_archive_db_path()` |

- **CLI vs 桌面分离**：不同进程、不同根目录（`~/.ccr` vs `~/.ccr-ui`）、不同生命周期；`data.db` 随 b3d22abe（workspace 拆分）迁入 ccr-store，`ccr-ui.db` 随 8838890c（引入 CCR UI）诞生，从未同库。
- **usage.db 拆分是显式 feature**：aa5af6c1（2026-04-20）"feat(usage归档): 持久化 usage 归档与会话摘要"，方向是把 usage 数据**从 ccr-ui.db 迁出**（`migrate_usage_archive_from_legacy_dbs`），commit 里带 Directive（迁移/legacy seed/repo 方法需同步维护）。合并即翻案。
- **合并 DB 文件的成本**：跨进程锁竞争（CLI 与桌面同库）、生命周期耦合（卸载 UI 不应影响 CLI 历史）、三套已发布 migration 历史不可重放。

→ 按 PRD 第 2 条预案：**共享 seam 代码、保持 DB 文件分离**。

## 前提 2："两套栈重复实现 pool+migrate+conn"——大部分推翻

- **pool 工厂已经全仓唯一**：`ccr-core/src/core/sqlite.rs`（65 行）持有 `DbPool/DbConnection/PoolConfig/create_sqlite_pool/create_memory_sqlite_pool` + 统一 PRAGMA（WAL/NORMAL/foreign_keys/busy_timeout/cache_size）。两套栈都消费它：`ccr-db/database/pool.rs:15-19`、`ccr-store/storage/database.rs:6`。所谓"重复实现 pool"实为 ccr-db/database/pool.rs 这个 137 行**纯别名+转发浅层**（3 个 type alias + DbError 包装 + info 日志）——可折叠删除。
- **migration runner 不可合并**：两套记账机制同名表不同 schema——ccr-store `migrations(id, name, applied_at)`（name-based，database.rs:97-101）vs ccr-db `migrations(version, name, applied_at)`（version-based，migrations.rs:2015）。PRD 约束禁止重写已发布迁移语义；合并 runner 高风险零收益，**各自保留**。
- **crate 级 seam 会制造新耦合**：依赖图上 CLI 侧（ccr / ccr-cli → ccr-store）不依赖 ccr-db；ccr-db 的依赖方仅 ccr-checkin 与 src-tauri。若 seam 放 ccr-db，则 ccr-store→ccr-db 把 89KB 迁移 + checkin 模型拖进每个 CLI 构建；若 seam 放 ccr-core——它已经存在（sqlite.rs 就是 seam）。
- ccr-error-freeze ADR 预写规则在缩水形状下依然满足：seam（ccr-core sqlite）说 primitive 错误（r2d2::Error），ccr-db 侧包成 DbError、ccr-store 侧包成 CcrError 各在自己边界 map_err；不新增 `impl From<DbError> for CcrError`；ccr-store 27 处 DatabaseError 构造点不动。

## 前提 3："GLOBAL_POOL 使 manager 层不可测"——成立（精确化）

- `AccountManager::create/update/list/delete`（加密→持久化→掩码路径）硬连 `database::with_connection`（GLOBAL_POOL，database/mod.rs:112-134）。其测试（account_manager.rs:285-438）只能**绕过 manager 直调 repo 函数**，harness 是本地 `static TEST_DB: Lazy<Mutex<Connection>>`（:294）——PRD 原话逐字成立。manager 方法路径零覆盖。
- 全局池生产消费面：ccr-checkin 5 个 manager（account/provider/record/balance/waf_cookie，约 45 处）+ ccr-db `managers/ui_state.rs`（8 处）+ `services/log_persistence.rs`（8 处，AppState.monitoring_logs 生产用）。`UsageImportService::new()`（GLOBAL_POOL 路径）**生产零调用**，仅测试（2142 行起的 tests 模块）；生产走 `with_pool(state.usage_db_pool)` 注入。checkin_service.rs 的全局调用也全在测试侧（:2110 起）。
- **新发现的 wart：同库双池**。main.rs:122+127 顺序调 `initialize()`（GLOBAL_POOL，默认 max_size 10，跑一遍迁移）+ `create_app_pool()`（AppState.db_pool，max_size 8，再跑一遍迁移）——同一个 `ccr-ui.db` 被两个池打开、迁移执行两遍。注释（main.rs:120-121）表明是"已知设计"，但两条访问路径（manager 层走全局、commands 层走 AppState）没有共享连接上限，纯属历史分叉。

## 前提 4："executor.rs 错位"——成立，且应删除而非移出

- `ccr-db/src/core/executor.rs`（300 行 Tokio 子进程 runner）：4 个导出函数 `execute_command/execute_binary/execute_binary_with_timeout/execute_binary_stream` **全仓零调用**（rg 全仓含 src-tauri 验证）；`ExecutorError` 除 ccr-checkin/core/error.rs:3 的 re-export 外零引用（07-03-arch-ccr-error 调研 inventory 亦记录"无(0 引用)"）。
- 来源：8838890c "feat(ui): 引入 CCR UI"——老 Web 全栈架构里后端 shell out 到 ccr CLI 的执行器；Tauri 化后改为直接链接库调用，executor 成遗物。
- 附带收益：`futures` 依赖与 tokio `process` feature 是 executor 独占（log_persistence 仅用 `tokio::sync`）——删除后可瘦 Cargo 依赖。
- PRD 说"移出 ccr-db（候选 ccr-core 或调用方 crate）"，证据说**没有调用方**：删除，不迁移。

## 前提 5："checkin 整体 re-export"——成立，删除成本极低

- `ccr-checkin/lib.rs:9 pub use ccr_db::database;` 的外部消费方 = **0**（rg `ccr_checkin::database` 全仓无命中）；消费者全是 crate 内部 `use crate::database`（5 个 manager 文件的生产 import + 各测试模块 + checkin_service.rs:2110 测试）。删墙 = 删 1 行 + 内部 import 改 `ccr_db::database`，无外部破坏面。
- `models::checkin` 的具名 re-export 是 spec 认可的契约（ccr-checkin guidelines "models::checkin re-exports come from ccr-db"），**保留**。

## 冻结契约核对

- `public-api-boundary.md` 冻结的是 `ccr` 根 re-export；ccr-db / ccr-checkin 是 workspace 内部 crate（无 publish 配置、消费方全在仓内），公共面可同步修改。`ccr::Database`（来自 ccr-store）不改名不动。
- AppState 池配置行为兼容约束：单池化后统一用 create_app_pool 的 max_size=8（见 design 取舍）。

## 缩水后的实施面（进 design.md）

1. **单池化**：`initialize_app_pool() -> Result<DbPool, DbError>`（建池→迁移→set GLOBAL→返回 clone），main.rs 一处调用替代 `initialize()+create_app_pool()` 两连击；删无消费方的 `initialize()`。ccr-ui.db 一个池、迁移一遍。
2. **pool.rs 折叠**：类型别名改为 database/mod.rs 直接 `pub use ccr_core::core::sqlite::{...}`，create_pool/create_memory_pool 移入 mod.rs，删 `pub mod pool`；src-tauri 4 处 `ccr_db::database::pool::DbPool` import 路径同步改。
3. **注入式访问**：`DbAccess` 枚举（Global | Pool(DbPool)）挂在 ccr-db::database，说 DbError；AccountManager 试点注入（`new()` 默认 Global 零调用方变更 + `with_db()` 注入），manager 级单测覆盖加密→持久化→掩码路径。其余 manager 机制可用、按需迁移（YAGNI）。
4. **删 executor.rs** + `ExecutorError` + ccr-checkin 的 re-export 项 + `futures`/tokio-process 依赖。
5. **删 `pub use ccr_db::database`**，内部 import 转 `ccr_db::database`。

复现命令（关键证据）：

```bash
rg -n "ccr_checkin::database" --glob '!target'            # 0 命中
rg -n "execute_binary|execute_command|ExecutorError" --glob '{crates,ccr-ui}/**/*.rs'
rg -n "database::(with_connection|get_pool|initialize)" crates ccr-ui/src-tauri
git log --oneline --diff-filter=A --follow -- crates/ccr-db/src/core/executor.rs   # 8838890c
git log --oneline -S "analytics" -- crates/ccr-db/src/database/mod.rs              # aa5af6c1
```

## 附：Step 3 单池化迁移审查记录（2026-07-05）

sqlite-migration-reviewer 子代理 dispatch 因环境故障不可用（代理侧对所有 Agent 调用返回
400 "1m 上下文"配置错误，与模型选择无关）；按该 reviewer 的检查清单内联执行等价审查：

1. 迁移执行次数 2→1：`run_all_migrations` 逐版本 `is_migration_applied` 守卫幂等，双跑从来只是冗余；usage.db 池自 aa5af6c1 起就是单跑先例。无对双跑的正确性依赖。
2. 池共享语义：r2d2 `Pool` Clone 为 Arc 共享同一池；`GLOBAL_POOL.set(pool.clone())` 与返回值同实例。OnceLock set-once + `let _ =` 容忍测试进程 `initialize_for_test` 先行；生产 main 单次执行。
3. 连接上限 10+8→8：`rg -A6 'with_connection('` 全消费面无嵌套调用（闭包体只调 repo 函数，连接需求深度恒为 1）；busy_timeout=5000 + r2d2 connection_timeout=30s 兜底；桌面负载下 8 连接充足，同库双池对 WAL checkpoint 反而更差。
4. 备份/restore/原子写顺序：diff 仅触池创建与 main.rs 启动段，零涉及。
5. `initialize_for_test()` 未动，测试隔离不变。
6. `database::initialize()` 生产残留：零（仅 target/ 下 libsqlite3-sys bindgen 的无关符号）。

结论：PASS。
