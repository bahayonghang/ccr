# ccr clean storage 设计

## Boundaries

命令属于 `ccr-cli` 的现有 `clean` 家族。新增 `CleanAction::Storage`。规划文件和备份清理的函数、菜单数组 `CLEAN_TARGETS`、以及 `dispatch.rs` 里 `action == None` 的三条分支保持原样。

新逻辑放在 `crates/ccr-cli/src/commands/lifecycle/clean_storage.rs`。`clean.rs` 里的 `confirm_cleanup` 改为 `pub(super)`，供新文件复用。`lifecycle/mod.rs` 与 `commands/mod.rs` 再导出 `clean_storage_command`。

不把路径解析放进 `ccr-db`。`resolve_ccr_root()` 是私有函数。CLI 按同一环境变量顺序自己解析。

## Plan

纯函数根据根目录和 `LLMUSAGE_HOME` 产生候选列表。每个候选有种类、路径和字节数。

候选规则：

| 种类 | 路径 | 纳入条件 |
| --- | --- | --- |
| `UiBackendTarget` | `<root>/ccr-ui/backend/target` | 是目录，且 `symlink_metadata` 不是符号链接 |
| `UiFrontendModules` | `<root>/ccr-ui/frontend/node_modules` | 同上 |
| `UsageMigrationBak` | `<root>/analytics/usage.db.pre-migration-*.bak` | 只看 `analytics` 这一层的普通文件，文件名以前缀 `usage.db.pre-migration-` 开始、以 `.bak` 结束，且不是符号链接 |
| `LegacyLlmUsage` | `<root>/llmusage` | 是目录且不是符号链接；`LLMUSAGE_HOME` 非空且规范化后与该目录相同，则改为保护项，不进入删除列表 |

不存在的路径不产生候选，也不算错误。符号链接产生一条保护记录，删除阶段不处理它。字节数只累加候选目录里的普通文件；遍历时不进入符号链接目录。

`LLMUSAGE_HOME` 为空时不保护 `<root>/llmusage`。两边路径都存在时用 `std::fs::canonicalize` 比较。一边尚不存在时比较组件规范化后的绝对路径。

## Execution

1. 解析根目录。`CCR_DATA_DIR` 优先于 `CCR_ROOT`。两者都为空时用 `dirs::home_dir().join(".ccr")`。拿不到主目录则返回现有的配置错误。
2. 读取确认设置，顺序与 `clean_backups_command` 相同：`ConfigService::with_default()`，缺失配置视为不跳过，其他加载错误立刻返回。
3. 生成候选。没有删除候选时打印「没有可清理的存储项」并成功返回。保护项可以同时打印。
4. `--dry-run` 用 `ColorOutput::key_value` 打印每条路径和「将释放空间」，不询问，不删除。
5. 需要确认时复用 `confirm_cleanup`。取消时打印「已取消清理操作」并成功返回。`--force`、全局 `--yes`、`skip_confirmation` 跳过这一步。
6. 逐个删除。目录用 `remove_dir_all`，文件用 `remove_file`。一个候选的 IO 错误记入失败列表，循环继续。
7. 结果一行说明已删除个数，并用字段打出释放字节数。有失败项时，在打印失败路径之后返回 `Err`。只有保护项、没有 IO 失败时成功返回。

输出遵守 `.trellis/spec/ccr-cli/backend/cli-output-presentation.md`：成功用 `ColorOutput::success`，保护用警告或字段，错误走现有 `Result`。不打印 `platforms/` 或 `checkin/` 的内容。不新增 JSON 模式。

## Help

`CLEAN_LONG_ABOUT` 增加第三类目标「CCR 根目录里的构建缓存和停用数据库副本」。`CLEAN_AFTER_LONG_HELP` 增加示例 `ccr clean storage --dry-run`，并写明边界：不删除 `analytics/usage.db`、`data.db`、配置、凭据和签到数据；不跟随符号链接；`LLMUSAGE_HOME` 指向旧目录时保留该目录。

保留这些已锁定句子：

- `打开菜单: ccr clean`
- `ccr clean planfiles --dry-run`
- `ccr clean planfiles --all --dry-run`
- `ccr clean --all`
- `ccr clean backups --dry-run`

`Commands::Clean` 的文档注释补上 `ccr clean storage --dry-run`。`definitions.rs` 里现有 clean 解析测试补一条 storage 解析，不改 planfiles / backups 的断言。

## Docs

`docs/reference/commands/clean.md` 与 `docs/en/reference/commands/clean.md` 各加一节 `storage`。说明四类目标、确认、`--dry-run`、`--force`，以及保留的数据库。不改 planfiles 与 backups 两节的规则。

## Compatibility

- 裸 `ccr clean`、`ccr clean --all`、`ccr clean --days`、`ccr clean --dry-run` 的旧入口不变。
- 本机 `CCR_ROOT` 未设置时，根就是 `~/.ccr`，和 `UiService` 的安装目录重合。设置了 `CCR_ROOT` 或 `CCR_DATA_DIR` 时，只清理那个根里面的 `ccr-ui` 缓存。不额外去删真实主目录里的另一份安装。
- 删除 `backend/target` 和 `frontend/node_modules` 之后，从该安装运行 `ccr ui` 会重新安装依赖并编译。仓库里的 Tauri 桌面端不使用这棵 `target`。

## Trade-offs

- 只点名这两个缓存目录，不扫描任意名为 `target` 或 `node_modules` 的目录，避免误删同步进来的其他工程。
- 迁移快照只认 `analytics` 一层的文件名。不删除其他 `.bak`，那些仍由 `ccr clean backups` 或用户自己的备份目录负责。
- 删除失败后继续处理其余候选。用户能一次看到全部失败路径。已经删掉的候选不会回滚。
- 不在集成测试里制造文件锁。Unix 可以删掉仍被打开的文件，Windows 共享锁不稳定。失败后的退出码由聚合单测锁定。

## Rollback

`--dry-run` 是删除前的检查点。执行之后：

- `backend/target` 与 `frontend/node_modules` 由下次该安装的 `ccr ui` 重建。
- 两份迁移 `.bak` 没有产品内恢复命令。保留的是 `analytics/usage.db`。
- `<root>/llmusage` 没有产品内恢复命令。正在使用的库在 `~/.llmusage` 或 `LLMUSAGE_HOME`。

不提供回收站。误删后的恢复来自用户自己的文件备份。
