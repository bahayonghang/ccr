# 各平台 profile 迁入 SQLite

需求来源：`prd.md`。本机字段清单：`research/profile-sqlite-storage.md`。

## 所有权

`ccr-config` 继续拥有目录的读、修订令牌、局部编辑和迁移。SQLite 放在这里，因为 `ccr-store` 已经依赖 `ccr-config`，目录不能再反向依赖会话库。`ccr-core` 已有 `rusqlite` 和密钥文件 ACL，目录库复用这两处，不新建 crate。

每个平台一个文件：

`CCR_ROOT/platforms/<platform>/profiles.db`

`CCR_ROOT` 的解析顺序保持现有规则：`CCR_ROOT`，否则 `~/.ccr`。不使用 `CCR_DATA_DIR` 的 `data.db`，也不使用 `~\.ccr-ui\ccr-ui.db`。

`PlatformPaths::profiles_file` 保留为遗留 TOML 路径，只给迁移和「文件已移走」的判断使用。新增 `profiles_db()`，指向同目录的 `profiles.db`。资源锁改按 `profiles.db` 的规范化路径计算，顺序仍是操作锁、资源锁、库事务。

## 表

`schema_version` 为 1。

```sql
CREATE TABLE document (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    current_config TEXT,
    default_config TEXT,
    settings_toml TEXT,
    extra_toml TEXT NOT NULL DEFAULT '',
    revision TEXT NOT NULL,
    migration_phase TEXT NOT NULL CHECK (migration_phase IN ('data_authoritative', 'source_retired')),
    migrated_at TEXT
);

CREATE TABLE profiles (
    name TEXT PRIMARY KEY,
    description TEXT,
    base_url TEXT,
    auth_token TEXT,
    model TEXT,
    small_fast_model TEXT,
    default_opus_model TEXT,
    default_sonnet_model TEXT,
    default_haiku_model TEXT,
    default_fable_model TEXT,
    default_opus_model_name TEXT,
    default_sonnet_model_name TEXT,
    default_haiku_model_name TEXT,
    default_fable_model_name TEXT,
    subagent_model TEXT,
    custom_model_option TEXT,
    custom_model_option_name TEXT,
    effort_level TEXT,
    claude_code_auto_compact_window TEXT,
    api_timeout_ms TEXT,
    claude_code_disable_nonessential_traffic TEXT,
    provider TEXT,
    provider_type TEXT,
    account TEXT,
    tags_json TEXT,
    usage_count INTEGER,
    enabled INTEGER,
    extra_toml TEXT NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL,
    updated_at TEXT NOT NULL
);
```

`document.current_config` 和 `default_config` 为 NULL 时，导出的 TOML 不写这两个键。空字符串表示键存在且为空。`settings_toml` 为 NULL 表示没有 `[settings]`。`usage_count`、`enabled` 为 NULL 表示段里没有该键。展示层现有的 `unwrap_or(0)` / 默认启用可以继续，但只有 `load_with_autofix` 能把 NULL 写成 0 和 true。

`extra_toml` 是一张只含未建模键的 TOML 表。Grok 的 `api_key`、Codex 的 `auth_mode`、`model_reasoning_effort`、`requires_openai_auth`，以及 datetime 都放这里。具名列和 `extra_toml` 在同一次事务里更新。`provider_type` 存 `official_relay` 或 `third_party_model`。`tags_json` 是 JSON 数组。`sort_order` 按原 TOML 段顺序。

`revision` 是规范文档的内容令牌，类型仍是字符串，桌面 raw IPC 不用改字段名。每次成功提交都换成新令牌。冲突时返回现有的版本冲突错误，不写备份、不改行。

密钥列是明文，和今天的 TOML 一样。语句参数不进入日志。权限不靠打开连接后补救，规则见「权限」。

## 迁移

CLI、TUI 和 Tauri 在分发命令之前调用 `migrate_catalogs()`。`ConfigManager::load`、`snapshot`、list、current、validate 不调用它。仅有遗留 TOML 时，这些读接口返回「尚未迁移」，不建库，也不把缺文件当成空目录。

阶段写在 `document.migration_phase`，并且和导入行同一次提交。`data_authoritative` 表示库已是目录权威，`migrated_at` 仍为空。`source_retired` 表示遗留 TOML 已不在活路径，同时写入 `migrated_at`。权威提交之前不移动 TOML。不使用 checkpoint 作为迁移完成条件。

本次尝试若要新建库，先写仅当前用户可读的 `profiles.db.attempt`，里面是本次 id，然后再创建 `profiles.db`。没有这个标记的已有库不属于本次尝试。

在目录资源锁内按下面的持久状态处理：

| 库 | attempt 标记 | 阶段 | 遗留 TOML | 动作 |
| --- | --- | --- | --- | --- |
| 无 | 无 | 无 | 无 | 不创建任何文件 |
| 无 | 无 | 无 | 有 | 写标记、建库，把行和 `data_authoritative` 一次提交。提交前失败则只删除本次库、WAL、SHM 和标记，TOML 不动 |
| 无 | 有 | 无 | 有 | 删除残留标记，按上一行重试 |
| 有 | 有 | 无 document 行 | 有 | 标记证明库是未完成尝试。删除该库、WAL、SHM 和标记，再按仅有 TOML 重试 |
| 有 | 无 | 无 `data_authoritative` | 有或无 | 不删除、不导入、不移动。返回迁移错误，文件保持原样 |
| 有 | 任意 | `data_authoritative` | 有 | 只把 TOML 移入 `backups/<platform>/`。移动失败则库和 TOML 都留下，下次仍走本行 |
| 有 | 任意 | `data_authoritative` | 无 | 只把阶段改成 `source_retired`。不重建 TOML，不重新导入 |
| 有 | 任意 | `source_retired` | 有 | 只移动 TOML |
| 有 | 任意 | `source_retired` | 无 | 结束。之后读写不打开 TOML |

解析器继续承认简化文档没有 current。迁移不调用 autofix。历史表不参与。键是否存在以源 TOML 为准，不用带默认值的 typed 投影反推。

中断与错误注入点：权威提交之前失败、权威提交之后且移动之前、移动过程中、移动成功但 `source_retired` 尚未写上、移动失败、以及一块没有 attempt 标记的已有库。后一块必须保持不动。

## 事务 owner

一次操作只有一个 `CatalogTransaction`，放在调用线程的 thread-local 上，不实现 `Send`。它持有该平台唯一的连接和至多一个未提交的 `BEGIN IMMEDIATE`。生命周期在 `execute` 期间持有目录资源锁；内层发现同一 `profiles.db` 已有活动事务时直接使用它，不再取锁，不再 `BEGIN` 或 `COMMIT`。

没有活动事务时，`ConfigManager::mutate`、`patch`、`clear_current_if_present` 和 `load_with_autofix` 自己取锁、开始并提交短事务。无变化的 mutate 不提交新修订，只补权限。SQLite 不允许嵌套 `BEGIN`，同一库同时只有一个写事务，所以平台代码不能再 `ConfigManager::new` 出第二条连接。

活动事务上的 load 使用同一连接，能看见本事务里尚未提交的 enable、改名和 current。`base::mutate_profiles`、`mutate_profiles_with_current` 和 `update_current_config` 都走这个入口。`enable_profile` 闭包里的 `manager.mutate`（`profile_lifecycle.rs`）以及 Grok `apply_profile` 里的 `load_profiles` 和 `update_current_config`（`grok.rs`）加入外层事务。Grok 自己的操作锁名称不变，但它不得再开目录事务。

## 平台读入口

`profiles_file` 只留给迁移判断遗留 TOML 是否还在。下面的调用点改为仓库读入口，不再 `fs::read_to_string`：

| 调用点 | 之后 |
| --- | --- |
| `base::load_profiles_from_toml` | 读活动事务或已提交的库。库不存在且没有遗留 TOML 时保持今天的空集合。仅有遗留 TOML 且阶段不是 `source_retired` 时返回尚未迁移，不返回空集合 |
| `base::load_current_profile_marker` | 读 document。NULL current 得到 `None`。不再把缺 TOML 文件当成无标记的成功 |
| Claude、Codex、Grok、Gemini、Droid 的 `load_profiles_from_file` | 调用上面的仓库入口。路径分别在 `claude.rs`、`codex.rs`、`grok.rs`、`gemini.rs`、`droid.rs` |
| `base::update_current_config`，以及 Claude、Droid 的 `update_current_config_in_profiles` | 加入活动事务，或在没有外层事务时提交自己的短事务 |
| `base::save_profiles_to_toml`、`mutate_profiles`、`mutate_profiles_with_current` | 写库。函数名可以暂留，但不得创建或覆盖活的 `profiles.toml` |

注册表优先、运行时匹配和空 current 的平台规则不改。验证要走真实的 `PlatformConfig` 或 service adapter，在 TOML 已经移走之后做列表和切换。只测 `ConfigManager` 不够。

## 读写

初始化改为 `ensure_initialized()` 写库。仓库里的示例 TOML 仍是初始化模板和导入样例，不是用户目录。

导出把库序列化成规范 TOML。导入解析 TOML 后按现有 merge/replace 规则写库。两条路径都不创建 `profiles.toml`。

桌面 raw get 返回这份规范 TOML 和 `revision`。raw save 仍先做语法和语义检查、空文档拒绝、当前 profile 消失时要求 `force`。通过后按令牌替换整份 `document` 和 `profiles`。注释和空白不保留。

`profile open` 在创建临时文件之前就要求 `$VISUAL` 或 `$EDITOR`。未设置时直接失败，不调用系统关联程序。临时文件放在系统临时目录，不放进平台目录，也不命名为 `profiles.toml`；创建时先套权限，再写入含密钥的规范 TOML。编辑器阻塞到退出。退出码为 0 且内容有变化才导入。无论成功、编辑器失败还是内容无效，临时文件都删除。无效内容不写库，并返回现有的校验错误。

## 切换补偿

`operation_paths` 去掉 `profiles_file`。`WriteJournal::verify` 因此不再哈希数据库文件。WAL 下文件字节本来就会变，放进文件日志会产生假冲突。

`execute` 仍先把操作记录写成 `Interrupted`。日志开始并核对 `revision` 之后打开目录事务。运行时写入和目录修改都发生在文件预映像仍可回滚、目录事务尚未提交的窗口里。同一事务写入一行 operation id，内容是即将成功的结果。

最终提交点按这个顺序：

1. `CatalogTransaction::commit`。失败则目录事务回滚，接着 `journal.rollback()` 恢复运行时文件，再把操作记录写成 `activation_committed = false`。这个记录若也写失败，结果是 `RecoveryRequired` 加上 `operation_record_failed`，不得伪装成已应用。同一 operation id 再次进入时，目录里没有该 operation id，文件记录也不是成功，因此会重新执行，而不是返回旧的成功。
2. 只有第 1 步成功，才 `journal.commit()` 丢掉文件预映像。此时目录和运行时文件都已提交。
3. 然后把同一成功结果写入 JSON 操作记录。写入失败时目录和文件保持已提交，返回 `AppliedWithWarning` 与 `operation_record_failed`。重放先看 JSON；若它仍是 `Interrupted` 或无法表示成功，再读目录里的 operation id，返回已提交结果，不第二次改 current，也不加计数。

进程在第 1 步之前被杀死时，未提交事务随连接关闭而回滚，操作记录保持 `Interrupted`。第 1 步和第 3 步之间被杀死时，目录里的 operation id 已经是重放依据。

计数不在这个事务里。它是第 3 步之后的独立短事务，使用 `MIN(COALESCE(usage_count, 0) + 1, 4294967295)`。失败只追加 `usage_count_failed`。重放读到已提交的 operation id 后不再打开它。

验证要注入最终 `COMMIT` 失败，并在该窗口用同一 operation id 重放。只注入运行时写入失败不够。

## 同步

`should_exclude_from_sync` 增加 `profiles.toml`、`profiles.db-wal`、`profiles.db-shm`。拉取因此不会把旧 TOML 或 WAL 副文件下载回来。

推送 `profiles.db` 时不读取活文件。整个快照窗口持有该平台的目录资源锁，使协作进程里的切换和计数写进不来：

1. `wal_checkpoint(TRUNCATE)`。检查返回行的 busy。busy 不为 0，或 pragma 报错，则本次不发布、不 DELETE 远端 TOML。
2. 仍在锁内用 SQLite backup API 把库复制到新建的临时快照。临时文件先套 owner-only 权限再接收页面。随后对快照执行 `integrity_check`，并读出 `document.revision` 与 profile 行。
3. 快照完成后可以释放目录锁，再上传快照字节。上传的不是 `fs::read(profiles.db)` 的结果。
4. 只有上传成功，才对远端 `profiles.toml` 发 WebDAV DELETE。404 视为完成。其他删除错误使本次推送失败。上传失败、busy 或快照失败都不发 DELETE。

这处 DELETE 只针对平台目录下的 `profiles.toml`，不做成通用远程清理。

拉取不把已打开的 `profiles.db` 改名。`profiles.db` 从普通 PullTransaction 的 active 改名路径里分开。staging 先按「权限」写成仅当前用户可读，再只读打开并做 `integrity_check`、schema 和 `document` 检查。目录锁内：

- 活库不存在：本次创建新文件。先套权限，再用 backup API 从已验证的 staging 装入。失败时只删除这次创建的文件。
- 活库存在：在已有连接上 `BEGIN IMMEDIATE`，用 staging 的 `document` 和 `profiles` 替换活内容，revision 改成快照的 revision，然后 `COMMIT`。任一步失败就 `ROLLBACK`，旧行和旧 revision 保持不变。不改名活文件，所以已打开的连接不会改指向备份文件。

持有旧读事务的连接可以读完旧快照，但目录锁不允许协作写入。该读事务结束后的新事务必须看到快照 revision。busy 超时则拉取失败，旧库不变。

## 权限

新增 `restrict_owner_only(path)`，只设置 Unix `0600` 或 Windows 当前用户 DACL，不读取内容，也不比较文件令牌。不用 `enforce_secret_permissions_versioned` 处理库、WAL 或 SHM，那个 helper 会按内容令牌决定是否改权限，并在 Windows 上保留已有 DACL。

各点的顺序都是先收紧、再写第一份密钥字节：

| 时机 | 责任 |
| --- | --- |
| 新建 `profiles.db` 或 attempt 标记 | 创建空文件时即套权限，然后才让 SQLite 写页 |
| 连接启用 WAL | `PRAGMA journal_mode=WAL` 产生 `-wal` / `-shm` 后立刻套权限，然后才 `INSERT` 密钥 |
| 迁移导入 | 同上，权威提交前的密钥写入发生在权限之后 |
| 普通同步 staging | 默认 `OpenOptions` 写明文之前先套权限 |
| 敏感同步解密 staging | Unix `0600` 之外，Windows 也用同一 helper 设置 DACL |
| 拉取安装的新库 | 先套权限，再 backup 进页面 |
| `profile open` 临时 TOML | 先套权限，再写规范 TOML |
| 移走遗留 TOML | 使用 rename，保留源文件已有 ACL，不经默认权限的副本 |

打开连接之后可以再套一次权限，但这不能覆盖上面任一点。验收分别覆盖 Unix mode 和 Windows DACL。当前平台测不到的一侧记 `NOT_RUN`。Debug 和日志另用密钥哨兵检查。

## 文档

行为落地后更新 `.trellis/spec/ccr-config/backend/backend-guidelines.md` 和 `profile-repository.md` 里的活路径描述，以及 `docs/reference/platforms/` 中把 `profiles.toml` 写成当前存储位置的句子。示例文件和 import/export 说明保留，并写明 TOML 是交换格式。

## 回退

这个切割没有双读。回退手段是：停用新二进制，把 `backups/<platform>/` 里移走的 TOML 放回 `profiles.toml`，并删掉对应的 `profiles.db` 及其 WAL 副文件。迁移之后新产生的切换次数只在库里，回退会丢掉迁移后的计数和编辑。
