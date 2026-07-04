# 单一 SQLite store seam

## Goal

`ccr-store::storage::Database` 与 `ccr-db::database` 两套并行栈重复实现 pool+migrate+conn；`GLOBAL_POOL` 单例使 manager 层不可测；ccr-checkin 整体 re-export `ccr_db::database` 使 ccr-db 的 schema 成为其契约。收敛为一个注入式 store seam；ccr-db 给出 curated 面；executor.rs 移出 DB crate。**Speculative——动手前先做否决式调研**。审查候选 7。

## Requirements

### 前置调研（必须先做，结论可否决本任务）

1. 三个 DB 文件（`~/.ccr/data.db` ccr-store、`~/.ccr-ui/ccr-ui.db` + `~/.ccr/analytics/usage.db` ccr-db）的分离是否是有意设计（进程隔离/生命周期/备份策略差异）？查 git history 与任务归档。
2. 若分离有意：本任务缩水为"共享 seam 代码、保持 DB 文件分离"；若无意：评估合并成本。
3. 调研结论写入本 prd 并更新验收标准；若结论是"现状合理、不值得动"，以 ADR 形式记录进 spec 后关闭本任务（trellis-update-spec）。

### 现状（探索报告定位）

- `ccr-db/database/pool.rs`（137 LOC）：对 `ccr_core::core::sqlite` 的纯别名+转发，删除测试通过（浅模块）。
- 两套栈同一工厂：ccr-store Database（CcrError、inline migrations、per-instance pool）vs ccr-db database（DbError、run_all_migrations、GLOBAL_POOL 单例）→ 两套测试 harness。
- 可测性：ccr-db 逻辑只能经 GLOBAL_POOL（database/mod.rs:112-134）；account_manager 测试测的是 static Mutex<Connection> 后面的 repo，AccountManager 的加密+持久化路径无 manager 级覆盖。
- seam 泄漏：`ccr-checkin/lib.rs:9 pub use ccr_db::database;`，account_manager.rs:53,66,110 直调 `database::with_connection(checkin_repo::…)`。
- 错位：`ccr-db/core/executor.rs`（300 LOC Tokio 子进程 runner）与 SQLite 无关，虚增 DB crate 身份。

### 要做的（若调研通过）

1. 一个 store seam（pool 所有权、migration runner、with_connection/transaction），注入式而非全局单例；ccr-store 与 ccr-db 共用。
2. ccr-db 提供 curated lib.rs 面；ccr-checkin 停止 wholesale re-export，改走具名接口。
3. `executor.rs` 移出 ccr-db（归属设计阶段定，候选 ccr-core 或调用方 crate）。
4. 删除 pool.rs 浅层。

### 约束

- 迁移期间数据零丢失：涉及 backup/restore/原子写顺序的变更触发 sqlite-migration-reviewer 审查。
- 三个 DB 的 schema 与 migration 历史不合并重写；只收敛"跑迁移的代码"，不动已发布的 migration 脚本语义。
- AppState 连接池配置（ccr-ui/src-tauri/src/state.rs）行为兼容。

## Acceptance Criteria

- [ ] 前置调研结论已记录（含 git 证据），并据此更新了本验收清单。
- [ ] （若实施）pool+migrate+conn 的实现全仓仅 1 处；两套测试 harness 合一。
- [ ] （若实施）manager 层（如 AccountManager）可注入连接进行单测，GLOBAL_POOL 不再是唯一入口。
- [ ] （若实施）`rg 'pub use ccr_db::database' crates/ccr-checkin` 无命中；checkin 经 curated 接口访问。
- [ ] （若实施）executor.rs 不在 ccr-db；`cargo test -p ccr-db -p ccr-store -p ccr-checkin -- --test-threads=1` 通过。
- [ ] sqlite-migration-reviewer 审查通过；`just lint-strict`、`just test` 通过。
- [ ] （若否决）ADR 记入 spec，任务关闭理由完整。

## Notes

- 复杂任务：调研通过后、`task.py start` 前需补 design.md 与 implement.md。
- 依赖：错误类型统一与否受 07-03-arch-ccr-error 结论影响（CcrError vs DbError 的取舍），建议后者先出评估结论。
