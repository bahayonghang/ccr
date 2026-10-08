# ~/.ccr 占用与现有清理命令

测量日：2026-10-08。测量时 `LLMUSAGE_HOME` 与 `CCR_ROOT` 均未设置。

## 顶层占用

`~/.ccr` 合计 5122.1 MB。

| 相对路径 | MB | 身份 |
| --- | ---: | --- |
| `ccr-ui` | 4068.7 | `ccr ui` 的用户目录安装。`justfile` 存在时走 `start_dev_mode`（`crates/ccr-cli/src/services/ui_service.rs:100-104`）。安装路径写死为 `home/.ccr/ccr-ui`（同文件 36-37 行），不读 `CCR_ROOT`。 |
| `ccr-ui/backend/target` | 3910.4 | 该安装里旧 Axum 后端的 Cargo 构建缓存。 |
| `ccr-ui/frontend/node_modules` | 153.6 | 同一安装的前端依赖。`frontend/src-tauri/target` 不存在。 |
| `analytics` | 943.6 | 见下方文件。 |
| `analytics/usage.db` | 454.8 | 仍在使用的用量归档。最后写入 2026-09-02。`-shm` 最后触及 2026-09-25。路径由 `get_usage_archive_db_path()` 固定（`crates/ccr-db/src/database/mod.rs:77-85`）。 |
| `analytics/usage.db.pre-migration-v16.20260727T155622460027000Z.bak` | 244.3 | 2026-07-27 迁移快照。 |
| `analytics/usage.db.pre-migration-v16.20260727T155638385550500Z.bak` | 244.3 | 同一次迁移的第二份快照。 |
| `llmusage/llmusage.db` | 101.3 | 旧库。`bin/`、`backups/`、`exports/` 为空。 |
| `platforms` | 3.3 | 配置与凭据，含 Codex auth 备份。 |
| `data.db` | 2.5 | 历史库（`crates/ccr-store/src/storage/database.rs:22-40`）。 |
| `backups` | 2.3 | profile 与 profile-off 备份。 |
| `logs` | 0.2 | 启动时已删除 14 天前的 CCR 日志（`crates/ccr-core/src/core/logging.rs:250-259`）。 |
| `checkin` 与其余配置文件 | <1 | 签到数据、`config.toml`、`sync.toml`、`ui_state.json`。 |

`ccr-ui` 去掉 `backend/target` 和 `frontend/node_modules` 后大约剩 4 MB 源码。

## 根目录解析

`resolve_ccr_root()`（`crates/ccr-db/src/database/mod.rs:54-64`）顺序：

1. `CCR_DATA_DIR`
2. `CCR_ROOT`
3. `~/.ccr`

该函数是 `ccr-db` 私有函数。`ccr clean storage` 在 CLI 内按同一顺序解析，不为此依赖 `ccr-db`。

## 已停用的 llmusage 位置

`AppPaths::discover()` 在未设置 `LLMUSAGE_HOME` 时使用 `~/.llmusage`（`crates/ccr-usage/src/paths.rs:10-16`）。测试 `default_discovery_uses_llmusage_root_not_legacy_ccr_root` 锁住不使用 `~/.ccr/llmusage`（同文件 86-103 行）。

`~/.llmusage` 不在这 5122 MB 内。同日测到 `llmusage.db` 1523 MB、`backups/` 4621.7 MB、`baselines/` 1106.4 MB。

## 现有 clean 命令

- 子命令：`CleanAction::{Planfiles, Backups}`（`crates/ccr-cli/src/cli/definitions.rs:511-518`）。
- 分发：`crates/ccr-cli/src/cli/dispatch.rs:98-128`。全局 `--yes` 与子命令 `--force` 合并为 `auto_yes || force`。
- 裸 `ccr clean` 打开菜单，默认是 planfiles（`crates/ccr-cli/src/commands/lifecycle/clean.rs:30-42`）。
- `ccr clean backups` 默认目录是 `~/.claude/backups`（`crates/ccr-cli/src/services/backup_service.rs:41-52`）。配置缺失时不跳过确认；配置损坏时在删除前返回错误（`clean.rs:131-138`）。
- 帮助文案在 `crates/ccr-cli/src/cli/help_config.rs:216-251`。`crates/ccr/tests/commands/help.rs:195-204` 锁定其中四句示例。
- 集成测试通过 `CARGO_BIN_EXE_ccr` 启动，并设置 `HOME`、`USERPROFILE`、`CCR_ROOT`（`crates/ccr/tests/commands/clean.rs:8-22`）。符号链接用例只在 Unix 编译（同文件 247-249 行）。

## 另一处数据库

`get_db_path()` 指向 `~/.ccr-ui/ccr-ui.db`（`crates/ccr-db/src/database/mod.rs:67-74`），不在 `~/.ccr` 下。
