# 添加 ccr clean storage 命令

## Goal

新增 `ccr clean storage`，清掉 CCR 根目录里占空间、又可以再生成或已经停用的构建缓存和数据库副本。命令先列出路径和字节数，用户确认后才删除。

本机 `~/.ccr` 在 2026-10-08 实测 5122.1 MB。现有 `ccr clean` 不扫描这个目录。用户确认默认删除集合为「缓存 + 停用副本」，本机大约可释放 4654 MB。

## Confirmed Facts

测量与代码锚点见 `research/storage-inventory.md`。这里只保留验收要用的事实。

- `ccr-ui/backend/target` 3910.4 MB，`ccr-ui/frontend/node_modules` 153.6 MB。二者都在 `ccr ui` 的用户目录安装里。删掉后源码还在，下次从该安装启动会重新构建。
- `analytics/usage.db` 454.8 MB 仍是用量归档（`crates/ccr-db/src/database/mod.rs:77-85`）。两份 `usage.db.pre-migration-v16.*.bak` 各 244.3 MB，是 2026-07-27 的迁移快照。
- `llmusage/llmusage.db` 101.3 MB 位于旧目录。当前发现逻辑使用 `~/.llmusage`（`crates/ccr-usage/src/paths.rs:10-16`、86-103 行）。测量时 `LLMUSAGE_HOME` 未设置。
- 根目录顺序是 `CCR_DATA_DIR`、`CCR_ROOT`、`~/.ccr`（`crates/ccr-db/src/database/mod.rs:54-64`）。
- `ccr clean planfiles` 清理当前工作目录的规划文件。`ccr clean backups` 清理 `~/.claude/backups` 里的旧 `.bak`。裸 `ccr clean` 的菜单默认仍是 planfiles。

## Requirements

- R1. 增加子命令 `ccr clean storage`。有待删项、且不是 `--dry-run`、也没有跳过确认时，先列出路径和字节数，再询问确认。
- R2. `--dry-run` 只打印清单，不删除。`--force` 与全局 `--yes` 跳过确认。`settings.skip_confirmation = true` 也跳过确认，并沿用 `ccr clean backups` 的提示。配置文件缺失时仍询问确认。配置文件损坏时，在删除前返回错误。
- R3. 存储根按 `CCR_DATA_DIR`、`CCR_ROOT`、`~/.ccr` 的顺序解析。每个候选路径都必须位于这个根之内。
- R4. 默认删除且只删除下面四类，且每一类都不是符号链接：
  - `<root>/ccr-ui/backend/target` 目录
  - `<root>/ccr-ui/frontend/node_modules` 目录
  - `<root>/analytics/` 直下、文件名匹配 `usage.db.pre-migration-*.bak` 的普通文件
  - `<root>/llmusage` 目录。`LLMUSAGE_HOME` 解析后与该目录是同一路径时，留下整个目录，并在结果里说明原因
- R5. 命令输出删除或将删除的个数，以及字节数。某个候选删除时返回 IO 错误，则留下该候选，写明路径，继续处理其余候选，最后以非零状态退出。这种保护性跳过（`LLMUSAGE_HOME` 命中、符号链接、路径不存在）不算失败。没有任何待删项时成功退出，并说明没有可清理项。用户取消时不删除任何文件，成功退出。
- R6. 裸 `ccr clean` 的菜单、默认项和 `ccr clean planfiles` / `ccr clean backups` 的目标保持不变。帮助文案补上 `ccr clean storage` 的示例，并保留 `help.rs` 已经锁定的旧示例句子。

下面这些路径不进入删除集合：

- `config.toml`、`sync.toml`、`sync_folders.toml`、`desktop-shell.json`、`ui_state.json`
- `platforms/`、`checkin/`、`skills/`、`locks/`、`history/`、`backups/`、`logs/`
- `data.db`
- `analytics/usage.db`、`usage.db-wal`、`usage.db-shm`
- `~/.llmusage`，以及 `LLMUSAGE_HOME` 指向的目录
- `~/.claude/backups`
- `~/.ccr-ui/ccr-ui.db`

## Out of Scope

- 不把 `storage` 加进裸 `ccr clean` 菜单。
- 不清理 `~/.llmusage` 和 `~/.ccr-ui/`。
- 不对保留的 SQLite 文件执行 `VACUUM`。
- 不删除整棵 `ccr-ui` 安装。
- 不在桌面端或 VS Code 插件里加入口，也不做定时清理。
- 不改 planfiles 与 backups 的删除规则。

## Acceptance Criteria

- [ ] AC1. `ccr clean storage --help` 说明 `--dry-run` 和 `--force`。`ccr clean --help` 仍包含 `help.rs` 锁定的 planfiles / backups 示例，并出现 `ccr clean storage --dry-run`。
- [ ] AC2. 在临时 `CCR_ROOT` 上，`--dry-run` 列出 R4 的四类路径和字节数，临时根里的文件都还在。
- [ ] AC3. `--force` 或确认后只删除 R4 的四类。`config.toml`、`platforms/`、`checkin/`、`data.db`、`analytics/usage.db` 及其 `-wal`、`-shm` 保持原样。
- [ ] AC4. 用户回答取消时不删除任何文件。
- [ ] AC5. `LLMUSAGE_HOME` 指向 `<root>/llmusage` 时，该目录保留，其余 R4 候选仍可删除，命令成功退出。
- [ ] AC6. 符号链接形式的 `backend/target` 或 `frontend/node_modules` 不被跟随、不被删除。这项集成测试与现有 planfiles 符号链接测试一样只在 Unix 编译。
- [ ] AC7. 聚合结果里只要有一个候选删除失败，命令就返回错误。其余成功候选不受影响。这条用结果聚合的单测证明，不在测试里锁真实文件。
- [ ] AC8. 测试使用临时目录，不读写本机 `~/.ccr`。`crates/ccr/tests/commands/clean.rs` 里现有 planfiles / backups 用例仍然通过。

## Technical Notes

- 命令锚点：`crates/ccr-cli/src/cli/definitions.rs`、`crates/ccr-cli/src/cli/dispatch.rs`、`crates/ccr-cli/src/cli/help_config.rs`、`crates/ccr-cli/src/commands/lifecycle/clean.rs`。
- 测试锚点：`crates/ccr/tests/commands/clean.rs`、`crates/ccr/tests/commands/help.rs:195`。
- 中英文命令说明：`docs/reference/commands/clean.md`、`docs/en/reference/commands/clean.md`。
