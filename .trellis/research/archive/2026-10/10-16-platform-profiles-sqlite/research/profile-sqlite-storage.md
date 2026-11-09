# 各平台 profile 目录迁入 SQLite

分析日期：2026-10-16。只读仓库与本机 `C:\Users\lyh\.ccr`，未改产品代码。定稿在 `../design.md` 和 `../prd.md`。用户已确认停用活的 `profiles.toml`，并且 `profile open` 在没有 `$VISUAL` / `$EDITOR` 时失败。

## 问题

CCR 的 provider 目录今天写在每个平台的 `profiles.toml` 里。切换次数只是其中一列。目标是：每个已有目录的平台各一张 SQLite 表，一行一个 provider，保存 URL、计数，以及该 profile 的其余字段。

## 两层数据

| 层 | 本机路径 | 角色 |
| --- | --- | --- |
| CCR 目录 | `~\.ccr\platforms\<platform>\profiles.toml` | 可切换的 provider 清单、当前/默认指针、平台设置 |
| 上游运行时 | Claude `~\.claude\settings.json`；Codex `~\.codex\config.toml`；Grok `$GROK_HOME/config.toml`；Gemini / Droid 各自的 settings | 切换成功后写入的生效配置 |

本次范围是目录层。运行时文件仍由现有 apply 流程写入。文档：`docs/reference/platforms/index.md`。

## 目录文档的形状

真相在 `CcsConfig`（`crates/ccr-config/src/managers/config/ccs_config.rs`）和 `ConfigSection`（`managers/config/types.rs`）。

文档级字段：

- `default_config`、`current_config`。简化文档可以没有这两个键；空的 current 表示未激活。读路径不得把解析器的内存默认值当成激活。见 `.trellis/spec/ccr-config/backend/profile-repository.md`。
- `[settings]`：至少有 `skip_confirmation`（旧名 `yolo_mode`）和可选 `tui_theme`（`types.rs` 的 `GlobalSettings`）。

每个 profile 段是一个 `ConfigSection`。共享列包括 `description`、`base_url`、`auth_token`（`Secret`，落盘明文，Debug 掩码）、模型映射、`provider`、`provider_type`、`account`、`tags`、`usage_count`、`enabled`。未建模的键进 `other`（`#[serde(flatten)]`），平台代码再从 `ProfileConfig.platform_data` 读取。

`usage_count` 是成功激活次数，饱和加一，发生在激活日志提交之后。失败只产生 `usage_count_failed`，同一 operation id 重放不加。见 `crates/ccr-cli/src/application/profile_lifecycle.rs` 与 `.trellis/spec/ccr-cli/backend/profile-application-lifecycle.md`。

## 本机目录（不含密钥值）

| 平台 | profile 数 | 密钥字段 | 没有 usage_count 的段 | 平台扩展键 |
| --- | ---: | --- | ---: | --- |
| claude | 28 | 每段 `auth_token` | 0 | `auth_mode` |
| codex | 25 | 每段 `auth_token` | 0 | `auth_mode`、`model_reasoning_effort`、`requires_openai_auth` |
| grok | 7 | 每段 `api_key`，没有 `auth_token` | 0 | `api_backend`、`context_window`、`reasoning_effort`、`supports_backend_search` |
| gemini | 6 | 每段 `auth_token` | 6，也没有 `enabled` | 无 |
| droid | 1 | `auth_token` | 0 | `display_name`、`max_output_tokens` |

五个文件都有 `current_config`、`default_config` 和 `[settings] skip_confirmation`。Codex 与 Grok 的 current 为空。`opencode` 没有 `profiles.toml`。Qwen 在 `Platform` 枚举里，`is_implemented()` 为假，本机没有目录文件。

Claude 的 `usage_count` 合计 326。`~\.ccr\data.db` 的 `history` 里，`operation=Switch AND result=Success AND extra=claude` 只有 11 行。`anyrouter2` 在 TOML 中是 134，历史里是 1。另有 17 条成功切换没有平台字段。历史行数不能回填这个计数。

`~\.ccr-ui\ccr-ui.db` 的 `usage_records` 有 150891 行，是导入的模型用量。同库 `claude_profiles` 有 0 行，schema 是快照（`snapshot_json`），由 migration v9 创建（`crates/ccr-db/src/database/migrations.rs`）。它不是这份目录的存储。

## 为什么不能塞进现有库

- `~\.ccr\data.db`（`ccr-store`，约 2.5 MB）拥有 history、sessions、search。会话索引和 provider 密钥放在一起，会让同步、备份和权限绑在同一文件上。
- `~\.ccr-ui\ccr-ui.db`（约 475 MB）是桌面库，含用量导入。CLI/TUI 在不打开桌面应用时也要读目录。`ccr-usage` 对上游用量库只读。
- 新表若仍叫 `claude_profiles` 且放进 `ccr-ui.db`，会和空的快照表撞名。

更合适的文件是独立的 `~\.ccr\profiles.db`：只放目录和文档指针，密钥文件权限单独处理，同步时可以只带这一个库。表名按平台分开：`claude_profiles`、`codex_profiles`、`grok_profiles`、`gemini_profiles`、`droid_profiles`。这个文件选择仍取决于下面的手改/同步决定。

## 一行要装下的内容

共享列可以直接对应 `ConfigSection`：名称、描述、`base_url`、密钥、模型字段、provider、provider_type、account、tags、usage_count、enabled、排序。

扩展键不能丢。Grok 的密钥在 `api_key`，并且与 `auth_token` 互斥（`crates/ccr-cli/src/platforms/grok.rs` 的 `profile_inline_api_key`）。Codex 的 `wire_api`、`env_key`、`approval_policy`、`sandbox_mode` 等也在代码里读取，只是本机文件里目前只出现了其中一部分。未知键和 TOML 扩展类型（例如 datetime）今天靠增量合并保留（`repository.rs` 的 `apply_delta`）。每张表需要一个 `extra_json`，原样保存这些键，而不是只拷贝 Rust 结构体上的具名列。

文档指针不属于 provider 行。每个平台还要有一份单例状态：`current_config`、`default_config`、`settings`，以及文档顶层的未知键。

## 写入与补偿

普通修改走 `ConfigManager::mutate`：资源锁、内容版本、备份、原子替换、密钥权限。`profiles.toml` 的备份策略是 `~\.ccr\backups\<平台>\`、前缀 `profiles`，本机 Claude/Codex 各留 10 份。每次成功切换都会为了 `usage_count + 1` 再写一遍整份含密钥的文件。

激活补偿是文件日志 `WriteJournal`（`crates/ccr-core/src/core/write_journal.rs`），按路径和内容版本回滚。目录若改成 SQLite，这条日志不再覆盖 profile 行。切换、改名、启用、删除必须在同一个同步事务里更新表，失败时恢复行和指针，或返回现有的 `recovery_required`。计数仍在激活提交之后单独增加。

读路径（list、current、validate、export）今天禁止顺手建文件或补字段。Gemini 缺 `usage_count` / `enabled` 是合法的；`load_with_autofix` 才会写成 0 和 true。迁移不能借查询把缺省写回去。

## 仍依赖 TOML 文件的产品面

- `ccr <platform> profile open` 用编辑器打开 `profiles.toml`，文件不存在时从模板创建（`docs/reference/commands/claude.md`）。
- `ccr export` / `ccr import` 以 TOML 为交换格式（`docs/reference/platforms/migration.md`）。
- WebDAV 同步把 `platforms/claude` 等目录当内容；`profiles.toml` 不在排除名单（`crates/ccr-sync/src/sync/service.rs` 的 `should_exclude_from_sync`）。`data.db` 不在平台目录里，现有「同步 Claude 配置」不会带上它。
- 医生检查、修复、临时配置文档都把该文件当作永久配置位置。

改名今天在同一次 TOML 提交里改段名和 current/default。SQLite 以名字做主键时，改名必须同时改行键和指针。删除后再建同名 profile，今天从空计数开始；库里留下旧行就会继承旧次数。

## 已决定：停用活的 profiles.toml

用户于 2026-10-16 确认按推荐停用。导入成功后目录只存在于 SQLite。`ccr export` / `ccr import` 保留 TOML 交换格式，但不恢复活文件。同步改为传 `profiles.db`。

库文件放在 `~\.ccr\platforms\<platform>\profiles.db`，不放进 `data.db` 或 `ccr-ui.db`。这样现有按平台目录的同步不会把 Claude 和 Codex 的密钥打进同一个文件。每库两张表：`profiles`（一行一个 provider）和 `document`（current、default、settings 的单例）。`document` 不是第二份 provider 清单，只承接原来的文档头。

连接仍用现有 WAL。同步前执行 `wal_checkpoint(TRUNCATE)`，只上传主文件。`-wal` 和 `-shm` 排除。推送时删除远端残留的 `profiles.toml`，否则下次拉取会把旧目录下载回来；拉取也跳过这个文件名。`backups/` 已经不在同步范围内，移走的原 TOML 停在本机备份里。

`profile open` 已定为只使用会阻塞到退出的 `$VISUAL` / `$EDITOR`。没有这两个变量时命令失败，不调用系统关联程序。

## 建议的表形状（未定稿）

每张平台表一行一个 profile：

- 主键 `name`
- `ConfigSection` 的具名列，密钥列按明文存储但文件 ACL 与今天的 secret 文件一致；SQLite 的 `-wal` / `-shm` 也要同一 ACL
- `tags_json`、`extra_json`
- `usage_count INTEGER NOT NULL DEFAULT 0`
- `enabled INTEGER`，NULL 表示文档里原本缺省（Gemini）
- `sort_order`，保持 TOML 段顺序
- `updated_at`

另加每平台单例状态表，或在 `profiles.db` 里用一张 `profile_documents(platform PRIMARY KEY, current_config, default_config, settings_json, extra_json)`。单例表能避免把指针塞进某个 provider 行。

导入：逐平台读现有 TOML 一次，写入行和单例。导入完成前不改序列化，避免增量写入提前删掉 TOML 里的 `usage_count`。历史行不参与回填。
