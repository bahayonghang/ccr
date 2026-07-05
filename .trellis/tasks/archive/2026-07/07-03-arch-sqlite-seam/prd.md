# 单一 SQLite store seam

## Goal

`ccr-store::storage::Database` 与 `ccr-db::database` 两套并行栈重复实现 pool+migrate+conn；`GLOBAL_POOL` 单例使 manager 层不可测；ccr-checkin 整体 re-export `ccr_db::database` 使 ccr-db 的 schema 成为其契约。收敛为一个注入式 store seam；ccr-db 给出 curated 面；executor.rs 移出 DB crate。**Speculative——动手前先做否决式调研**。审查候选 7。

## 前置调研结论（2026-07-05，全文见 `research/veto-research.md`）

**否决门触发，任务缩水为"共享 seam 代码、保持 DB 文件分离"**：

1. **三个 DB 文件分离是有意设计**：CLI（`~/.ccr/data.db`）与桌面（`~/.ccr-ui/ccr-ui.db`）不同进程不同根目录；`usage.db` 是 aa5af6c1 显式拆出的 durable archive（迁移方向就是从 ccr-ui.db 迁出）。不合并。
2. **"重复实现 pool" 大部分推翻**：pool 工厂已全仓唯一（`ccr-core::core::sqlite`，两套栈都在用）；真正的浅层是 ccr-db/database/pool.rs（137 行纯转发）。migration runner 两套记账机制同名表不同 schema（name-based vs version-based），按约束不合并。跨栈 crate 级 seam 会新增 ccr-store→ccr-db 耦合（CLI 构建拖入 89KB 迁移+checkin 模型），否决。
3. **GLOBAL_POOL 不可测成立**：AccountManager 加密→持久化→掩码路径零 manager 级覆盖（测试绕过 manager 直调 repo）。另发现同库双池 wart：main.rs 对 ccr-ui.db 同时开 GLOBAL_POOL（10 conns）+ AppState.db_pool（8 conns），迁移跑两遍。
4. **executor.rs 应删除而非移出**：4 个导出函数全仓零调用，`ExecutorError` 仅剩 ccr-checkin re-export（0 引用）；系 8838890c 老 Web UI shell-out 架构遗物。附带可卸 `futures` 依赖与 tokio `process` feature。
5. **checkin wholesale re-export 外部消费方为 0**：删除只需内部 import 从 `crate::database` 改 `ccr_db::database`。

错误规则遵循 ccr-error-freeze ADR：seam 说 primitive/DbError，ccr-store 在自己边界 map_err 桥接，不新增 `impl From<DbError> for CcrError`。

## Requirements（缩水后）

1. **单池化**：ccr-ui.db 在 Tauri 进程内只开一个池（`initialize_app_pool()` 建池→迁移→登记 GLOBAL→返回池给 AppState），迁移只跑一遍；删除无消费方的 `initialize()`。
2. **注入式访问**：`DbAccess`（Global | Pool）挂在 `ccr_db::database`，manager 层可注入连接池做单测；AccountManager 试点（`new()` 默认全局零调用方变更），补加密→持久化→掩码的 manager 级测试。
3. **删除 executor.rs** 及 `ExecutorError`、ccr-checkin 对它的 re-export、独占依赖（futures / tokio "process" feature）。
4. **删除 `pub use ccr_db::database`**（ccr-checkin/lib.rs:9），内部改走 `ccr_db::database` 具名路径；`models::checkin` 具名 re-export 保留。
5. **删除 pool.rs 浅层**：类型与工厂折进 database/mod.rs（`pub use ccr_core::core::sqlite::{...}`），src-tauri import 路径同步更新。

### 约束

- 迁移期间数据零丢失：涉及 backup/restore/原子写顺序的变更触发 sqlite-migration-reviewer 审查。
- 三个 DB 的 schema 与 migration 历史不合并重写；只收敛"跑迁移的代码"，不动已发布的 migration 脚本语义。
- AppState 连接池配置（ccr-ui/src-tauri/src/state.rs）行为兼容（单池后统一 max_size=8，取舍记录进 design.md）。
- `ccr::Database`（ccr-store）在冻结墙上，不改名不动签名。

## Acceptance Criteria

- [x] 前置调研结论已记录（含 git 证据），并据此更新了本验收清单。
- [x] 单池化：Tauri 进程对 ccr-ui.db 仅存在一个池实例；`rg 'database::initialize\(\)'` 生产代码无命中；迁移在 app 池上仅执行一遍。
- [x] `DbAccess` 注入入口存在且 AccountManager 可注入内存池单测；新增 manager 级测试覆盖 create（加密+插入+可解密+掩码）与 list 不解密路径；GLOBAL_POOL 不再是 manager 层唯一入口。
- [x] `rg 'pub use ccr_db::database' crates/ccr-checkin` 无命中；ccr-checkin 内部经 `ccr_db::database` 具名路径访问。
- [x] executor.rs 不存在于 ccr-db；`rg 'ExecutorError' crates` 无命中；ccr-db 不再依赖 `futures`。
- [x] `crates/ccr-db/src/database/pool.rs` 已删除，浅层折叠后 src-tauri 编译通过（cargo check 0 错误）。
- [x] `cargo test -p ccr-db -p ccr-store -p ccr-checkin -- --test-threads=1` 通过（并经 `just test` 全仓覆盖）。
- [x] sqlite-migration-reviewer 审查通过（子代理 dispatch 因环境故障不可用，按其检查清单内联执行等价审查，记录见 research 附录；结论 PASS）；`just lint-strict`、`just test` 通过。
- [x] 缩水/否决理由回写 spec（trellis-update-spec）：ccr-db guidelines（seam 契约与单池初始化 + Decision Record）、三库分离 + runner 不合并的决策记录、ccr-checkin guidelines（具名路径 + DbAccess 注入模式）。

## Notes

- 复杂任务：调研通过后、`task.py start` 前需补 design.md 与 implement.md。
- 依赖：07-03-arch-ccr-error 已出结论（CcrError 冻结 ADR），本任务错误取向按 ADR 行事，无阻塞。
