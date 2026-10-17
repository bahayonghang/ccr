# 各平台 profile 迁入 SQLite

## Goal

把 CCR 的 provider 目录从每个平台的 `profiles.toml` 迁到该平台自己的 SQLite 文件。Claude、Codex、Grok、Gemini、Droid 各一份库、各一张 provider 表。一行是一个 provider，包含 URL、切换计数和其余字段。切换、列表、编辑、排序、导入导出和同步都读写这套库。上游 CLI 的运行时文件仍由现有切换流程写入。

用户价值：目录和切换次数不再靠重写整份含密钥的 TOML 保存，同时现有次数不会被历史行数或模型用量替换。

## Confirmed Facts

- 今天的目录是 `~\.ccr\platforms\<platform>\profiles.toml`，模型是 `CcsConfig` / `ConfigSection`（`crates/ccr-config/src/managers/config/ccs_config.rs`、`types.rs`）。读写契约在 `.trellis/spec/ccr-config/backend/profile-repository.md`。
- 规划时的本机快照是 Claude 28、Codex 25、Grok 7、Gemini 6、Droid 1，Claude 的 `usage_count` 合计 326。审阅没有重读这些文件。验收比较迁移前现场基线，不把这些数字写成固定断言。Qwen 未实现，`opencode` 没有该文件。字段清单见 `research/profile-sqlite-storage.md`。
- `usage_count` 是成功激活次数。规划时 `~\.ccr\data.db` 里带平台标记的 Claude 成功切换只有 11 行，不能回填。`~\.ccr-ui\ccr-ui.db` 的 `usage_records` 是模型用量，其中空的 `claude_profiles` 是快照表。这两项同样是快照，不是验收常量。
- Grok 的密钥在扩展键 `api_key`，与 `auth_token` 互斥（`crates/ccr-cli/src/platforms/grok.rs`）。未知键和 TOML 扩展值由 `repository.rs` 的增量合并保留。
- `current_config`、`default_config`、`[settings]` 是文档头。键不存在和空 current 都表示未激活，二者要能区分，避免无关编辑把简化文档补成已激活。
- 激活补偿在 `crates/ccr-cli/src/application/profile_lifecycle.rs`。`profiles.toml` 在 `operation_paths` 里。计数在日志提交之后单独增加；失败是 `usage_count_failed`；同一 operation id 重放不加。进程中断只留下 `Interrupted` 记录，内存中的 `WriteJournal` 预映像不会跨进程恢复。
- `profile open` 仅支持 Claude、Codex、Grok。设置了 `$VISUAL` / `$EDITOR` 时阻塞到进程退出；否则系统关联程序立即返回（`crates/ccr-cli/src/commands/platform/profile_open.rs`）。
- 桌面端 Claude / Codex 的 raw get/save 按文件字节做 CAS（`ccr-ui/src-tauri/src/commands/profile_lifecycle.rs`）。
- WebDAV 按文件名过滤，今天会上传并下载 `profiles.toml`。`backups/` 已排除。连接默认 WAL（`crates/ccr-core/src/core/sqlite.rs`）。现有同步客户端没有 WebDAV DELETE。

## Requirements

- R1 五个已有目录的平台各一个 `platforms/<platform>/profiles.db`。库内 `profiles` 一行一个 provider，`document` 一行保存文档头。Qwen 与没有目录的平台不预建库。
- R2 `profiles` 的具名列覆盖描述、`base_url`、密钥、模型字段、provider、provider_type、account、tags、`usage_count`、enabled，并保留段顺序。未建模的键放 `extra_toml`，用 TOML 值往返，保留 datetime 这类扩展值。
- R3 `document` 用 NULL 表示「键原本不存在」，用空字符串表示「键存在但为空」。settings 同样区分缺省和显式值。
- R4 命令入口在业务读之前做一次性迁移。计数以 TOML 的 `usage_count` 为准；缺失保持 NULL。不用 history 或 `usage_records` 回填。`load` / `snapshot` 自己不迁移、不建库、不补 0。仅有遗留 TOML 时，入口迁移会建库并进入权威阶段；同一状态下不经入口、直接调用仓库读接口必须返回「尚未迁移」，并且不建库。
- R5 迁移用持久阶段，不靠重导或删除已有库来填中间状态。`data_authoritative` 与导入行在同一次提交里生效，之后这份库是目录权威，提交前不移动 TOML。提交成功后才把 TOML 移入备份；移动成功后才写成 `source_retired`。`source_retired` 或已是 `data_authoritative` 且 TOML 还在时，只移动文件。TOML 已经不在而阶段仍是 `data_authoritative` 时，只补 `source_retired`。本次尝试在权威提交前创建的库、WAL、SHM 可以在失败时删除；没有本次尝试标记的已有库不得删除，也不得覆盖导入。权威提交之后移动失败时，库和 TOML 都留在原地，下次只重试移动。
- R6 切换成功并提交后，只给目标行做 `u32` 饱和 `+1`。`4294967295` 再加仍是 `4294967295`。失败仍是 `usage_count_failed`。重放不加。改名同时改主键和 current/default。删除行后，同名重建不继承旧计数。段顺序用 `sort_order` 保留。
- R7 一次激活只有一个目录事务 owner。平台 adapter 的 load、mutate 和 `update_current_config` 加入该事务，不另开连接、不嵌套 `BEGIN`、不再取已持有的目录资源锁，并且能读到本事务里尚未提交的 enable 和改名。`profiles.db` 不进入 `WriteJournal` 的文件校验。SQLite `COMMIT` 的结果确定之前，文件日志的预映像仍可回滚。`COMMIT` 失败时回滚文件，操作记录里的 `activation_committed` 保持 false，同一 operation id 再次进入不得把它当成已成功。`COMMIT` 成功后才释放文件预映像，再把同一结果写入操作记录；记录写入失败时目录和运行时文件保持已提交，返回 `operation_record_failed`，重放改读随目录事务提交的 operation id，不再执行一遍。计数仍是这次激活提交之后的独立事务。
- R8 列表、当前 profile、校验、TUI 和桌面端通过平台 adapter 显示同一套字段。Gemini 和 Droid 保持 legacy adapter，不为此新增公共 profile 命令。仓库只读接口不创建库、不补 enabled、不写 current。入口迁移和仓库只读接口的可观察结果按「两者都缺、仅 TOML、已权威、损坏或不可读」分开，不能用一层的结果代替另一层。
- R9 `ccr export` / `ccr import` 仍用 TOML 交换，读写对象是 SQLite，不创建活的 `profiles.toml`。桌面 raw get 返回规范 TOML 和修订令牌；raw save 用该令牌做 CAS，冲突不写库。
- R10 `profile open` 只接受 `$VISUAL` / `$EDITOR`。编辑器退出后把临时 TOML 导入 SQLite 并删除临时文件。未设置编辑器时命令失败并提示设置，不调用系统关联程序，不留下临时文件。临时文件不放在 `platforms/<platform>/`，也不命名为 `profiles.toml`。
- R11 同步上传的是在目录资源锁内生成、并通过完整性检查的稳定快照，不是 checkpoint 之后对活文件的直接读取。`wal_checkpoint(TRUNCATE)` 返回 busy 或错误、快照失败、或上传失败时，不发布这次快照，也不删除远端 `profiles.toml`。上传成功后才 DELETE 远端 `profiles.toml`；404 视为已完成，其他删除错误使本次推送失败。不同步 `-wal`、`-shm` 和 `profiles.toml`。拉取不改名一个已打开的库文件：经验证的快照在目录锁内用事务导入活库；导入失败时旧行和旧 revision 保持不变，成功后的新事务读到该快照的 revision。
- R12 密钥列按明文保存，等价于今天的 TOML。日志、错误和 Debug 不输出密钥。库文件、WAL、SHM、同步 staging、拉取安装中的新文件和编辑器临时文件，都在写入第一份密钥字节之前交给不比较内容令牌的 owner-only 权限 helper。打开连接后的补权限不能代替这次先收紧。Unix 目标是 `0600`，Windows 目标是仅当前用户的 DACL。无法在当前平台实测的那一侧标 `NOT_RUN`，不用读源码代替实测。

## Decisions

- D1 停用活的 `profiles.toml`。导入后目录只在 SQLite。密钥不写回活 TOML。运行时文件不改。用户于 2026-10-16 确认。
- D2 未设置 `$VISUAL` / `$EDITOR` 时，`profile open` 失败并提示设置编辑器。用户于 2026-10-16 确认。

## Out of Scope

- 替换 Claude、Codex、Grok、Gemini、Droid 的上游运行时文件。
- 把 history、sessions 或模型用量并进 profile 库。
- 用历史行数重算切换次数。
- 为 Qwen 或 OpenCode 新建目录。
- 保留 TOML 注释和空白。结构化编辑今天就会重排内容；迁库后 raw 往返同样只保证值。
- 新旧二进制同时写同一目录。

## Acceptance Criteria

- [ ] A1（R1、机制：每平台 `profiles` 行数）五个平台导入后的 profile 数等于各自迁移前 TOML 基线，不使用规划快照里的 28/25/7/6/1。
- [ ] A2（R2、机制：具名列和 `extra_toml`）扩展键与密钥和基线一致，Grok 的 `api_key` 仍在扩展键里。
- [ ] A3（R4、机制：按键导入 `usage_count`）每个平台的切换次数合计等于迁移前 TOML 基线，不按 history 改写，也不把 326 写成断言常量。
- [ ] A4（R4、机制：可空列按键导入）Gemini 基线里缺失的 `usage_count` / `enabled` 保持 NULL。
- [ ] A5（R4、机制：入口 `migrate_catalogs`）CLI、TUI、Tauri 分发前迁移。两者都缺时不创建文件。仅有 TOML 时导入并进入 `data_authoritative`。
- [ ] A6（R8、机制：仓库读接口）不经入口直接 load / snapshot / list / current / validate：两者都缺时不建库；仅有 TOML 时返回尚未迁移且不建库；`source_retired` 时读库且不打开 TOML；损坏或不可读时返回错误且不改写。
- [ ] A7（R3、机制：`current_config`、`default_config`、`settings_toml` 的 NULL 与空字符串）缺省 current 在迁移和无关编辑后仍不写出该键。显式空字符串仍导出为空值。settings 同样区分缺省和显式值。
- [ ] A8（R2、机制：`sort_order`）导出的段顺序与迁移前 TOML 的段顺序一致。
- [ ] A9（R6、机制：激活提交后的独立更新）成功切换使目标行加一且只加一次。计数写失败时激活仍成功并带 `usage_count_failed`。同一 operation id 重放不加。`usage_count = 4294967295` 时再切换仍是该值；NULL 第一次成功切换变成 1。
- [ ] A10（R6、机制：显式切换写入 document）显式切换把 current 写成目标名。
- [ ] A11（R7、机制：单一 `CatalogTransaction` 与提交顺序）SQLite `COMMIT` 失败时，运行时文件回到操作前，操作记录的 `activation_committed` 仍为 false，同一 operation id 再次调用不返回已成功。`COMMIT` 成功后文件预映像才释放。enable 之后的 apply、改名之后的 apply、以及内层 `update_current_config` 能看见本事务未提交的变更；失败回滚这些变更。
- [ ] A12（R6、机制：主键更新与删除）改名后按新名字读到原计数。删除后再建同名，计数是 NULL。
- [ ] A13（R2、机制：`extra_toml` 随主键移动）改名后扩展键仍在新行。
- [ ] A14（R5、机制：迁移阶段）`source_retired` 之后目录里没有活的 `profiles.toml`，备份里有原文件，随后经平台 adapter 的 list / switch 不读该文件。权威提交前失败时 TOML 留在原位，且不留下可当成权威的库。权威提交后移动失败时库保留、TOML 留在原位，重试只移动。已有且无本次尝试标记的库不被删除或覆盖。
- [ ] A15（R9、机制：规范 TOML 与 `revision`）export 来自库。import 写回库且不留下活的 `profiles.toml`。raw save 的旧令牌被拒绝且不写库。
- [ ] A16（R10、机制：阻塞编辑器与临时文件）设置了 `$EDITOR` 时，退出后把修改写入库并删除临时文件。`$VISUAL` 和 `$EDITOR` 都未设置时命令失败，临时文件和系统关联程序都不出现。
- [ ] A17（R12、机制：先收紧权限再写字节）新建库、WAL、SHM、同步 staging、编辑器临时文件在第一份密钥字节之前只有当前用户可访问。Unix 断言 `0600`，Windows 断言 owner-only DACL；当前平台做不到的一侧记录 `NOT_RUN`。Debug 和日志不包含密钥哨兵。
- [ ] A18（R8、机制：五个平台的 `PlatformConfig` 读入口）迁移完成后，Claude、Codex、Grok、Gemini、Droid 的列表和 current 来自各自的库，字段与仓库读到的一致。不给 Gemini 或 Droid 增加新的公共 profile 命令。
- [ ] A19（R11、机制：锁内 backup 快照与事务导入）checkpoint busy、快照失败或上传失败时不上传新目录，也不 DELETE 远端 `profiles.toml`。上传成功后的 DELETE 遇到 404 视为完成。拉取时保持一个已打开的目录会话：导入失败则旧 revision 不变；导入成功后，新的读事务看到快照 revision。不把已打开的 `profiles.db` 改名替换。
