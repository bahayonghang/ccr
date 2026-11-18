# GPUI 桌面端 SSH 栈与执行环境准入现状盘点

- 来源工单：bahayonghang/ccr#196（地图 #195）
- 调研基线：`origin/dev` @ `8d498849`（`main` 尚无 `crates/ccr-app` 与 `ccr-desktop`）
- 性质：事实盘点，不做方案取舍。行号均指上述基线。

## 结论速览

1. **SSH 只活在设置页**。`RemoteView` 选中主机后，壳层把除「全局设置」外的所有目的地正文替换为「此页面依赖本地 CCR 服务，在 SSH 环境不可用」（`ccr-desktop/src/shell/mod.rs:883-887`）。
2. **平台目的地 × SSH = 全部未支持**。除设置页 SSH 面板的 4 个明文文件外，没有任何平台目的地能读写远程文件。所有 feature 构造 `RequestKey` 时都硬编码 `EnvironmentId::Local(..)`，只有 `features/environments/remote.rs:216-220` 会构造 `EnvironmentId::Ssh`，且只喂给 `ssh_action`。
3. **准入闸门写法不统一**。`admit_local_only` 只在 `runtime/environments.rs:14` 被调用一次；其余约 30 处适配器用 `matches!(key.environment, EnvironmentId::Local(_))` 自行判断，返回各自的 `Unsupported` / `LocalOnly` 错误。
4. **文件访问全是本地路径**。平台适配器经 `ccr_cli::application::platform_roots::PlatformRoots::capture()` 解析根目录，再交给 `ccr-app` 资源层（`platform_settings::Document` / `read_text`，内部是 `std::fs` + `write_guarded_versioned`）或 `ccr-cli` / `ccr-config` 领域服务。没有一层接受远程根。
5. **「会话」是逻辑授权，不是连接**。`Workspace.session: Mutex<Option<Session>>` 最多持有 1 个主机的 `session_id`；每次 probe / connect / read / save / detect 都会新起 `ssh-keyscan` / `ssh` / `sftp` 子进程（`BatchMode=yes`）。
6. **只有 agent / 已有 identity_file 能认证**。桌面端新增主机时写 `identity_file: None`（`ccr-app/src/environments/ssh.rs:211`），表单不提供私钥路径；编辑时保留已有值。密码一律在连接前被拒绝（`AUTH_UNSUPPORTED`）。
7. **Tauri 端不依赖 `ssh::Workspace`**。它只用 `ssh_security` 的校验 / OpenSSH 命令构造 / known_hosts 写入，以及 `ssh_session` 的进程内信任闸门。替换 `Workspace` 的传输不会直接破坏 Tauri；改 `ssh_security` 的公开 API 会。

---

## 1. 共享后端 `crates/ccr-app/src/environments/`

### 1.1 文件职责

| 文件 | 职责 | 依据 |
|---|---|---|
| `mod.rs` | 汇总导出；`RunningEnvironment::Local`（唯一变体）；`inspect()` 生成 `EnvironmentReport`（OS、WSL 探测、已保存 SSH 主机**名称**、壳能力）；`InspectOptions::live()` 读 `existing_ui_database_file()`；`SshNames` 不连接 | `mod.rs:1-150` |
| `surface.rs` | `EnvironmentSurface<T>`：按 `EnvironmentId` 分槽保存 cache / draft / job，`switch()` 递增 generation；`admit_local_only`；`read_local_source`（caller 与 owner 必须是同名 Local） | `surface.rs:44-132` |
| `ssh_session.rs` | 进程内重连 / 断开决策（纯函数，不开 socket）：`boundary_before_connect`、`ProcessHostTrust`（`BTreeSet<(host,port)>`）、`reconnect_gate`、`allow_reconnect(_target)`、`plan_disconnect` / `disconnect_error`；错误码常量 | `ssh_session.rs:8-164` |
| `ssh_trust.rs` | 指纹文本比较：`classify_host_key` → `New` / `Matched` / `Mismatch`；`decide_host_key` 返回的 `may_store` / `may_trust` / `may_accept` 恒为 false | `ssh_trust.rs:16-87` |
| `ssh_security.rs` | 输入校验（`SshTarget`、`RemotePosixPath`）；OpenSSH / SFTP 命令构造；SFTP batch 计划；`run_openssh_command`；`app_known_hosts_path`；`parse_keyscan_output`；`SshTrustService`（challenge 存储）；`persist_known_host(_with_locks)`；`classify_ssh_failure` | `ssh_security.rs:28-693` |
| `ssh.rs` | 桌面端 SSH 所有者 `Workspace`：主机 CRUD、probe / confirm / connect / disconnect、4 个固定配置文件的 SFTP 读写；内部 `Transport` trait，唯一实现 `OpenSsh` | `ssh.rs:20-632` |
| `ssh/detect.rs` | 已连接会话上的远端 CLI 检测（`command -v` 依次查 claude / codex / agy / opencode），用 `CliSession` 绑定 `session_id` | `ssh/detect.rs:4-97` |
| `ssh/fixture.rs` | 测试 / `test-fixtures` 特性下的内存传输 | `ssh.rs:634-635` |
| `wsl.rs` | WSL 列表与执行探测（本工单不展开） | `mod.rs:21-25` |

### 1.2 会话模型

- `Workspace` 字段：`database: Option<PathBuf>`、`known_hosts: PathBuf`、`trust: SshTrustService`、`session: Mutex<Option<Session>>`、`serial: tokio::sync::Mutex<()>`、`transport: Arc<dyn Transport>`（`ssh.rs:132-142`）。
- `Session { id, host: SshHost, session_id: Uuid }`（`ssh.rs:71-76`）。全进程只有 1 个槽位，`retained_session_count()` 只会返回 0 或 1（`ssh.rs:282-289`）。
- 所有公开操作都先拿 `serial` 锁，彻底串行化（`ssh.rs:192, 244, 294, 332, 364, 397, 551, 576`；`detect.rs:52`）。
- 会清空会话的情形：`probe`、`confirm`、`connect` 入口（`ssh.rs:333, 366, 398`）；`save_host` / `delete_host` 命中当前主机（`ssh.rs:235-237, 250-252`）；`read` / `save` / `detect_cli` 的任何失败（`ssh.rs:565, 599, 619, 623`；`detect.rs:92-94`）；`disconnect`（`ssh.rs:293-296`）。
- 没有持久连接。`connect` 用 `ssh … printf nonce` 验证一次后只记录 `Session`（`ssh.rs:426-446`）。之后每次读写各起一个 `sftp -b -` 子进程（`ssh.rs:453-478`），CLI 检测每个工具起一个 `ssh` 子进程（`detect.rs:62-88`）。
- 子进程走 `ProcessGateway` 的 `ProcessDescriptor::openssh()` / `sftp()` / `ssh_keyscan()`，超时 15 s（`ccr-app/src/jobs/process/gateway.rs:144-197`）。
- OpenSSH 公共参数：`BatchMode=yes`、`ConnectTimeout=N`、`StrictHostKeyChecking=yes`、`GlobalKnownHostsFile=none`、`UserKnownHostsFile=<应用私有路径>`；有 `identity_file` 时追加 `-i`（`ssh_security.rs:74-120`）。

### 1.3 主机密钥信任流程（probe → confirm → connect）

1. **probe**（`ssh.rs:331-357`）：清空会话 → 读主机行 → `ssh-keyscan -T 5 -p <port> -t ed25519,ecdsa,rsa <host>`（`ssh.rs:297-330`）→ `parse_keyscan_output` 取第一条受支持的 key 并算出 `SHA256:` 指纹（`ssh_security.rs:392-438`）→ 读 DB 表 `ssh_known_hosts` 里存的指纹 → `classify_host_key` → `SshTrustService::register` 生成 challenge UUID。返回 `Probe { challenge_id, host, port, fingerprint, stored_fingerprint, status }`。
2. **confirm**（`ssh.rs:359-395`）：只接受 `challenge_id`，不接受渲染层回传的 key 数据。`consume` 一次性有效，TTL 120 s，有 Reused / Expired 墓碑（`ssh_security.rs:24, 497-558`）。之后先写应用私有 known_hosts 文件（`persist_known_host_with_locks`：读 → 去掉同 marker 行 → 追加 → `write_guarded_versioned` CAS，最多 4 次，`secret: true`，见 `ssh_security.rs:570-636`），再 `upsert_known_host` 写 DB。
3. **connect**（`ssh.rs:396-447`）：校验 `remote_home` → 再 keyscan 一次 → 与 DB 指纹比较。只有 `Matched` 能继续，`Mismatch` 返回 `ssh_host_key_mismatch`，其余返回 `ssh_host_key_untrusted`。随后**再次写入** known_hosts 文件，再用 nonce 回显验证认证，成功后才记录 `Session`。
4. 桌面端 UI 的 `EnvironmentView::plan_connection` 也调用 `boundary_before_connect`，但只传全空的 facts（`features/environments/mod.rs:65-103`），只是一个本地闸门。

信任存在两处：DB 表 `ssh_known_hosts`（比较用）和应用私有 known_hosts 文件（OpenSSH 的 `StrictHostKeyChecking` 用）。confirm 时两处都写；connect 时只重写文件。

### 1.4 远程读写：版本令牌与备份语义

- 固定 4 个文件，路径相对 `remote_home`：`.claude/settings.json`、`.codex/config.toml`、`.gemini/antigravity-cli/settings.json`、`.opencode/opencode.json`（`ConfigFile::path`，`ssh.rs:45-62`）。
- **read**（`ssh.rs:545-569`）：要求当前会话就是该主机 → `sftp get` 到本地临时目录 → 大于 2 MiB 拒绝（`ssh.rs:489-496`）→ `Document { environment, file, content, version: content_version_token(content), session_id }`。
- **save**（`ssh.rs:570-631`）：
  1. `document.session_id` 必须等于当前会话，否则 `ssh_session_changed`；
  2. 内容 ≤ 2 MiB；Codex 按 TOML 解析，其余按 JSON 解析，失败为 `ssh_invalid_document`；
  3. 重新下载当前远程内容，`content_version_token` 不等于 `document.version` 时返回 `ssh_conflict`；
  4. 把旧内容上传到 `<file>.bak.<uuid>`（远程备份），失败就中止；
  5. 上传新内容。
- **upload 原子性**（`ssh.rs:499-544`；计划见 `ssh_security.rs:324-353`）：本地暂存用 `write_guarded`（`secret: true`）→ batch 1：`-mkdir` 各级父目录 + `put` 到 `<path>.ccr-tmp-<uuid>` → batch 2：`rename` 临时文件到目标。rename 失败映射为 `ssh_write_outcome_unknown`。任一步失败都会执行 `-rm` 清理 batch；清理本身失败时返回 `ssh_cleanup_failed`。
- 读写失败（除 `ssh_conflict` / `ssh_invalid_document` 的 UI 侧处理外）都会清空会话（`ssh.rs:564-567, 598-601, 618-624`）。

### 1.5 错误码集合

所有错误都是 `&'static str` 或带前缀的 `String`。

| 来源 | 错误码 |
|---|---|
| `ssh_session.rs:8-12` | `ssh_host_key_untrusted`、`ssh_not_connected`、`ssh_auth_unsupported`、`ssh_cancelled`、`ssh_host_key_mismatch` |
| `surface.rs:10` | `local_only_environment`（`LOCAL_ONLY`） |
| `ssh.rs` / `ssh/detect.rs` | `ssh_database_unavailable`、`ssh_invalid_target`、`ssh_invalid_remote_path`、`ssh_host_missing`、`ssh_not_connected`、`ssh_cancelled`、`ssh_network_error`、`ssh_cleanup_failed`、`ssh_host_key_invalid`、`ssh_challenge_invalid`、`ssh_trust_write_failed`、`ssh_host_key_mismatch`、`ssh_host_key_untrusted`、`ssh_local_staging_failed`、`ssh_read_failed`、`ssh_file_too_large`、`ssh_write_outcome_unknown`、`ssh_session_changed`、`ssh_invalid_document`、`ssh_conflict`、`ssh_cli_output_invalid` |
| `ssh_security.rs`（`String`，前缀加说明） | `ssh_invalid_port`、`ssh_invalid_host`、`ssh_invalid_user`、`ssh_invalid_identity_file`、`ssh_invalid_remote_path`、`ssh_process_timeout`、`ssh_process_output_limit`、`ssh_known_hosts_path`、`ssh_host_key_invalid`、`ssh_host_key_missing`、`ssh_challenge_invalid` / `_reused` / `_expired` / `_unknown`、`ssh_known_hosts_lock_failed` / `_join_failed` / `_invalid_utf8` / `_read_failed` / `_write_failed` / `_conflict` |
| `classify_ssh_failure`（按 stderr 文本归类） | `ssh_host_key_mismatch`、`ssh_host_key_untrusted`、`ssh_auth_unsupported`、`ssh_network_error`（`ssh_security.rs:646-683`） |
| 桌面适配器 `runtime/environments.rs` | `ssh_fixture_offline`、`ssh_environment_mismatch`；并把 `ssh_cleanup_failed` 映射为 `Completion::CleanupFailed`，`ssh_cancelled` 映射为 `Completion::Cancelled`（`:203-209`） |
| 桌面 UI `remote.rs` 消息键 | `ssh_busy`、`ssh_failed`、`ssh_trusted`、`ssh_connected`、`ssh_document_ready`（`remote.rs:242, 301, 305, 325, 345`；文案表 `:390-443`） |

## 2. 持久化（`ccr-db`）

### 2.1 表结构（迁移 v6）

`crates/ccr-db/src/database/schema.rs:326-356`，`migrations.rs:912-951`：

- `ssh_hosts(id TEXT PK, name, host, port INTEGER DEFAULT 22, username, identity_file TEXT NULL, remote_home TEXT NULL, created_at, updated_at, last_connected_at TEXT NULL)`；索引 `(updated_at DESC)`、`(host, port)`。
- `ssh_known_hosts(host, port, key_type, fingerprint, confirmed_at, PK(host, port))`；索引 `(confirmed_at DESC)`。
- 模型：`SshHost`、`CreateSshHostRequest`、`UpdateSshHostRequest`、`SshKnownHost`（`models/ssh.rs:8-52`）。模型里没有认证方式字段，也没有密码字段。

### 2.2 仓储（`database/repositories/ssh_repo.rs`）

- `read_saved_host_names`：只读打开，只 `SELECT name`（`:18-41`）。
- `open_host_store(path, write)`：读时只读且不建库；写时建父目录，再跑 `run_initial_migration` + `run_migration_v6`（`:45-63`）。
- `list_hosts` / `get_host` / `insert_host` / `update_host` / `delete_host`（`:65-129`）；`set_last_connected_at`（`:131-141`）；`get_known_host` / `upsert_known_host`（`:143-178`）。
- 时间解析失败时回退为 `Utc::now()`（`:210-214`）。

### 2.3 字段的实际使用

| 字段 | GPUI 桌面端（`ccr-app::environments::ssh`） | Tauri 端 |
|---|---|---|
| `identity_file` | `HostInput` 没有这个字段。新建写 `None`（`ssh.rs:211`），编辑时沿用已有行的值（`ssh.rs:203-222`），连接时传给 `SshTarget` 生成 `-i`（`ssh.rs:260-268`）。表单文案：「新主机使用 SSH agent；编辑时保留已有身份文件设置」（`remote.rs:522`） | `ssh_add_host` 接受并写入（`commands/ssh.rs:489-560`，`:539`）；启动时随 `SshHostConfig` 注册（`commands/environment.rs:146-155`）；`auth.rs:230-234` 优先级为「显式 identity_file > 发现的密钥 > Agent」 |
| `remote_home` | 必填。`RemotePosixPath::root` 只接受 `~`、`/` 或绝对 POSIX 路径（`ssh_security.rs:187-208`）；缺省时显示为 `~`（`ssh.rs:188`）；作为 4 个文件和 `.bak` 的根 | `SshHostConfig.remote_home`，用作 `config_path` 的根 |
| `last_connected_at` | **从不写入**（`ccr-app` 中没有对 `set_last_connected_at` 的调用） | 连接成功后 `set_last_connected_at`，同时改 `updated_at`（`commands/ssh.rs:383-399`） |

### 2.4 路径

- 主机库：`ccr_db::database::ui_database_path()` = `~/.ccr-ui/ccr-ui.db`，不受 `CCR_ROOT` 影响（`ccr-db/src/database/mod.rs:69-78`）。桌面端在 `Services::ssh_workspace` 中按需打开（`runtime/environments.rs:76-82`）；Tauri 端用 `db_pool`（`main.rs:188-189`）。**两端共用同一库文件和同一张 `ssh_hosts` 表。**
- 应用私有 known_hosts：`app_known_hosts_path()` = `$CCR_ROOT/ssh/known_hosts`，未设置时为 `~/.ccr/ssh/known_hosts`（`ssh_security.rs:375-383`）。两端共用。
- Fixture 启动时为 `<registry 父目录>/ssh.db` 与 `<…>/ssh/known_hosts`（`runtime/environments.rs:83-90`）；Benchmark 启动直接返回 `ssh_fixture_offline`。

## 3. 环境准入（`ccr-app::runtime` → `ccr-desktop/src/runtime/*`）

### 3.1 原语

- `EnvironmentId { Local(String), Wsl(String), Ssh(String) }`；`RequestKey { environment, platform, generation, request_id }`；`ResponseGate` 只比较完整 key（`ccr-app/src/runtime.rs:45-73`）。
- `Platform` 共 7 个：Claude、Codex、Grok、Gemini、Qwen、Droid、OpenCode（`runtime.rs:11-19`）。
- `EnvironmentSurface` 只在 `EnvironmentView` 中实例化（`features/environments/mod.rs:43`）。生产代码里没有任何地方调用 `switch()`，`note_draft` / `note_job` 也只在测试中出现，所以 active 恒为 `Local("local")`。
- `ccr-app` 中除 `runtime.rs` 与 `environments/` 外，没有任何模块引用 `EnvironmentId`。资源层（`platform_settings`、`resources::*`、`codex_agents`、`sessions`、`usage`）全部只接受本地 `Path`；例如 `platform_settings::read_text` 直接 `std::fs::read`（`ccr-app/src/platform_settings.rs:157-158`），写入走 `write_guarded_versioned`（`:143`）。

### 3.2 调用方现状

- **feature 层**：除 `remote.rs:216-220` 外，所有 `RequestKey` 都是 `EnvironmentId::Local("<用途标签>")`，例如 `features/configs/mod.rs:166`、`features/platforms/mod.rs:223, 365`、`features/mcp/mod.rs:241, 280, 366`、`features/workbench.rs:155`、`features/resources/*`、`features/insights/*`、`features/sync/mod.rs:114, 258`、`features/checkin/*`、`runtime/preferences.rs:185`。
- **壳层遮罩**：`RemoteView::is_remote()` 等于 `selected.is_some()`，表示「已选主机」，不要求已连接（`remote.rs:83-85`）。为真时：`go()` 不再 `park` 本地视图（`shell/mod.rs:382-384`），`activate()` 直接返回（`:412-414`），非平台目的地的工具栏和 lens 胶囊隐藏（`:810-812, 829-842`），非 Settings 正文替换为不可用提示（`:883-887`）。

### 3.3 适配器逐个清单

文件访问方式记号：

- **PR** = `PlatformRoots::capture()` 解析本地根（`ccr-cli/src/application/platform_roots.rs:19-41`）
- **APP** = 经 `ccr-app` 资源层
- **CLI / CFG** = 经 `ccr-cli` / `ccr-config` 领域服务
- **FS** = 适配器内直接 `std::fs`
- **DB** = SQLite

| 适配器（`ccr-desktop/src/runtime/…`） | 入口 | 准入闸门 | 文件访问 | 归类 |
|---|---|---|---|---|
| `mod.rs` `configurations` | 配置方案列表（Claude） | `Local && Claude`（`:300-307`） | CFG：`ConfigReadService::for_claude()` | 仅本地 |
| `profiles.rs` | `profile_rows` / `profile_command` | `Local`（`:342, :381`），再 `domain_platform` 只放行 Claude / Codex / Grok（`:185-192`） | CLI + CFG：`ProfileLifecycleContext::new`，`ConfigService` | 仅本地 |
| `profiles/overview.rs` | `grok_overview` | `Local && Grok`（`:62`） | CLI：`GrokPlatform` | 仅本地 |
| `platforms.rs` | `platform_request`（读 / patch / source / provider / account / quota） | `Local`（`:1032-1040`） | PR → APP `platform_settings::Document`；备份 `~/.ccr/backups/platform-settings`；锁在 `roots.locks`（`:252-309, 815-934`） | 仅本地 |
| `platforms.rs` | `platform_oauth_start` / `_control` | `Local && Codex && Auth`（`:955-963, 1004-1008`） | CLI：`codex_oauth::NativeSession` | 仅本地 |
| `platforms/diagnostics.rs` | `codex_auth_diagnostic` | `Local && Codex`（`:69`） | PR + 本机进程 / 端口探测 | 仅本地 |
| `platforms/layers.rs` | `platform_layers_request` | `Local`（`:153`） | APP `platform_settings::layers` + FS `symlink_metadata`（`:5-20`） | 仅本地 |
| `platforms/rolling.rs` | `codex_rolling_usage` | `Local && Codex`（`:102`） | `ccr_codex::CodexUsageService` 扫描本地会话 | 仅本地 |
| `platforms/{accounts,grok,providers,quota}.rs` | 由 `execute` 分派 | 继承 `platform_request` 的闸门 | PR + CLI / APP | 仅本地 |
| `resources.rs` | `resource_request`（系统提示词、Claude 扩展、旧版导出） | `Local`（`:117`） | PR → APP `system_prompts` / `claude_extensions` / `config::source::save_export` | 仅本地 |
| `resources/claude_files.rs` | `claude_resource_request` | `Local && Claude`（`:130`） | PR → APP `resources::claude_files` / `claude_collections` / `Document` | 仅本地 |
| `resources/codex_agents.rs` | `codex_agents_read` / `start_codex_agents` | `Local && Codex`（`:210`） | PR → APP `codex_agents::{AgentStore, SourceStore}` | 仅本地 |
| `resources/platform_files.rs` | `platform_files_read` / `commit_platform_file` | `Local && 平台匹配`（`:54`） | PR → APP `resources::platform_files`（`~/.config/opencode/*`、`~/.gemini/*`） | 仅本地 |
| `resources/templates.rs` | `template_request` | `Local`（`:32`） | APP `resources::prompt_presets` | 仅本地 |
| `mcp.rs` | `mcp_request` / `mcp_project_request` | `read_catalog` 中 `Local`（`:409`） | PR → APP `platform_settings::read_text`（`:180`） | 仅本地 |
| `mcp/editor.rs` | `mcp_edit_request` / `mcp_mutate_request` | `Local`（`:95, :224`） | APP `Document::new(.., roots.locks)`（`:139`） | 仅本地 |
| `mcp/batch.rs` | `mcp_batch_*` | `Local`（`:114`） | 同 editor | 仅本地 |
| `mcp/transfer.rs` | `mcp_copy_*` | 经 `read_catalog` / editor 闸门（`:173`） | 同 editor | 仅本地 |
| `insights.rs` | `load` / `load_workbench` / `refresh_archive` | `Local`（`:447, :643, :758`） | `ccr_usage::AppPaths` + `~/.ccr/analytics/usage.db` + 本地 provider home（`:46-62`）；APP `sessions` / `usage` | 用量库与环境无关；会话读的是本地 provider home |
| `insights/claude_workbench.rs` | `claude_workbench` | `Local && Claude`（`:58`） | 同上 | 仅本地 |
| `insights/{capabilities,import,observer,policies,session_operations,…/delivery}.rs` | 用量能力、导入、observer、会话导出 / 回收 / 恢复 | `Local`（`capabilities.rs:10`、`import.rs:247`、`observer.rs:325, 351`、`policies.rs:70`、`session_operations.rs:119, 133, 170`、`delivery.rs:32, 83`） | 本地 llmusage / 归档库 / provider home | 用量与环境无关；会话操作仅本地 |
| `checkin.rs` + `checkin/*` | 签到读取 / 操作 / 凭据 | `Local`（`checkin.rs:190`、`operations.rs:290`、`credentials.rs:21`） | DB `~/.ccr-ui/ccr-ui.db`（`checkin.rs:152-157`） | 与环境无关（有本地闸门） |
| `sync.rs` + `sync/operations.rs` | WebDAV 同步配置与操作 | `Local`（`sync.rs:236`、`operations.rs:199`） | `ccr_sync::SyncFolderManager::with_default()` 的 `sync_folders.toml`（`sync.rs:210-224`） | 与环境无关（有本地闸门） |
| `ui_state.rs` | 命令收藏 / 历史 | `Local`（`:128, :196`） | DB（`UiStateManager`） | 与环境无关（有本地闸门） |
| `preferences.rs` | GPUI 偏好 | 无闸门，key 固定为 `Local("desktop-gpui-preferences")`（`:183-190`） | FS + `AtomicWriter`，`desktop-gpui/preferences.json` | 与环境无关 |
| `monitoring.rs` / `monitoring/system.rs` | 全局监控 / 系统信息 | `Local`（`monitoring.rs:331`、`system.rs:135`） | DB + 本机系统探测 | 与环境无关（本机） |
| `jobs.rs` | 命令工作台（运行 `ccr` CLI） | `Local`（`:93`） | 本机子进程 | 仅本地（本机进程） |
| `install.rs` | 安装 llmusage 等 | `Local && !fixture`（`:219-224`） | CLI `InstallService` | 仅本地（本机） |
| `tools.rs` | 版本检测 / SkillPort 桥 / 更新 ccr | `Local`（`:103`） | APP `tools` | 仅本地（本机） |
| `benchmark.rs` / `benchmark/stream.rs` | 基准测试源 | `Local`（`stream.rs:68`） | fixture 布局 | 测试专用 |
| `environments.rs` `load` | 环境报告 | `admit_local_only`（`:14`）；强制 `query_wsl = false`（`:31-32`） | DB 只读主机名 | 本地 |
| `environments.rs` `ssh_action` | SSH 面板全部操作 | Probe / Connect / Read / Save / DetectCli 要求 `key.environment == Ssh(target)`，否则 `ssh_environment_mismatch`（`:121-133`）；fixture 一律 `ssh_fixture_offline`（`:136-149`） | `ccr-app::environments::ssh::Workspace` | **已支持 SSH**（仅 4 个文件） |

不经过 runtime 适配器的：`features/converter.rs` 只对输入文本做内存转换（key 固定 Local，`:78`），与环境无关。

## 4. 平台目的地 × 环境支持矩阵

供「作用范围决策票」使用。目的地与 lens 来自 `shell/mod.rs:36-50, 77-94`、`shell/platform_navigation.rs:46-78`。

- **本地**列：当前代码可用。
- **SSH 现状**列：选中主机后能否使用。✗ 表示被壳层遮罩，适配器也会拒绝。
- **WSL** 列：`EnvironmentId::Wsl` 在生产代码中从不构造，WSL 探测默认关闭，所以全列都是 ✗，表中不重复。

### 4.1 平台目的地（随执行环境变化的配置）

| 目的地 / lens | Claude Code | Codex | Grok | OpenCode | Antigravity（Gemini） | SSH 现状 | 适配器 / 访问方式 |
|---|---|---|---|---|---|---|---|
| 工作台（Primary） | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | `insights` / `claude_workbench`、`profiles`、`rolling`、`grok_overview`；本地 |
| 配置方案 Profiles | ✓ | ✓ | ✓ | — | — | ✗ | `profiles.rs`；CLI + CFG |
| 账户 Auth | ✓ | ✓（含 OAuth、诊断） | ✓ | — | — | ✗ | `platforms.rs`；PR → APP |
| 设置（`Lens::Accounts`） | ✓ | ✓ | ✓ | ✓ | ✓（旧版 settings 只读，Patch / Source 返回 Unsupported，`platforms.rs:848, 879`） | ✗（设置页 SSH 面板可明文编辑同一文件） | `platforms.rs`；PR → APP |
| 供应商 Providers | — | ✓ | — | ✓ | — | ✗ | `platforms.rs`；PR → APP |
| 模型 Models | — | ✓ | — | — | — | ✗ | `platforms.rs`；PR → APP |
| MCP | ✓ | ✓ | — | — | ✓ | ✗ | `mcp*.rs`；PR → APP |
| 指令 Files | ✓ | ✓ | — | ✓ | ✓ | ✗ | `resources/*`；PR → APP |
| 扩展 Extensions | ✓ | ✓ | — | ✓ | ✓ | ✗ | `resources.rs` / `codex_agents` / `platform_files`；PR → APP |
| 用量 Usage（平台内） | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | `insights`；本地 llmusage 库 |
| 会话 Sessions | ✓ | ✓ | — | — | — | ✗ | `insights` / `session_operations`；本地 provider home |

**设置页 SSH 面板唯一覆盖的远程文件**（= 本地「设置」lens 主文件的远程副本）：

| 远程文件 | 对应平台 | 本地「设置」lens 主文件 |
|---|---|---|
| `~/.claude/settings.json` | Claude | 同 |
| `~/.codex/config.toml` | Codex | 同 |
| `~/.gemini/antigravity-cli/settings.json` | Antigravity | 同 |
| `~/.opencode/opencode.json` | OpenCode | **不同**：本地主文件是 `~/.config/opencode/opencode.json`，`~/.opencode/config.json` 只是 fallback（`platforms.rs:277-281`） |

Grok 没有远程文件。

### 4.2 非平台目的地

| 目的地 | 本地 | SSH 现状 | 语义归类 | 依据 |
|---|---|---|---|---|
| 概览 Dashboard（含 Monitoring） | ✓ | ✗ | 与环境无关（本机 / 应用数据） | `monitoring.rs`、overview |
| 会话 AgentSessions | ✓ | ✗ | 随环境变化（读 provider home） | `insights` |
| MCP 管理 McpManager（含转换器） | ✓ | ✗ | MCP 随环境变化；转换器与环境无关 | `mcp.rs`、`converter.rs` |
| 命令 Commands（收藏 / 最近） | ✓ | ✗ | 执行在本机；收藏 / 历史与环境无关 | `jobs.rs`、`ui_state.rs` |
| 签到 Checkin | ✓ | ✗ | 与环境无关 | `checkin.rs` |
| 同步 Sync | ✓ | ✗ | 与环境无关 | `sync.rs` |
| 用量 Usage（含安装 llmusage） | ✓ | ✗ | 与环境无关（本地用量库）；安装在本机 | `insights`、`install.rs` |
| 全局设置 Settings | ✓ | ✓（唯一可用） | 偏好与环境无关；SSH 面板 = 远程 4 文件 | `settings.rs`、`remote.rs` |

## 5. 桌面端 UI 与状态

### 5.1 `features/environments/mod.rs`（`EnvironmentView`）

- 只读诊断块：本地 OS 行、WSL 行、已保存 SSH 名称行、信任说明、托盘 / 隐藏 / 通知能力行，外加一个「发送系统通知」按钮（`mod.rs:495-584`）。
- `refresh` 走 `runtime::environments::load`；fixture 直接调用 `inspect(fixture)`（`mod.rs:75-140`）。
- 持有的 `EnvironmentSurface` active 永远是 `Local("local")`。

### 5.2 `features/environments/remote.rs`（`RemoteView`）

- 状态：`hosts`、`selected: Option<Host>`、`connected: bool`、`file: ConfigFile`、`drafts: HashMap<(host_id, path), Draft>`、`probe`、`busy`、`generation`、`message`、`host_form` + 5 个输入框（名称 / 主机 / 端口 / 用户 / 远程主目录），以及删除 / 读取 / 保存三个确认标志（`remote.rs:30-54`）。
- 交互：主机列表按钮（选中项加 ●）；「使用本地环境」实际发送 `SshAction::Disconnect`（`:453-460`）；编辑 / 删除主机；「检查主机密钥」→ 显示指纹 → 「信任此主机密钥」；「连接」；4 个文件切换按钮；「读取远程明文」；Textarea 编辑；「备份后保存到远程」并二次确认；「取消操作」（`:446-865`）。
- `remember` / `restore` 只在内存里按 `(host_id, file)` 保存 / 恢复编辑器草稿（`:97-116`），不持久化选中主机。
- **选中主机不持久化**：`selected` 初始化为 `None`（`:60`），`DesktopPreferences` 只有 theme / language / confirm_quit / window / typography（`runtime/preferences.rs:92-98`）。重启后总是回到本地，主机列表在首次进入设置页时懒加载（`refresh`，`:92-96`；由 `shell/mod.rs:409-411` 触发）。
- 选择变化会记一条监控事件（`record_selection` → `record_environment_selection`，key 为 `Local("environment-selection-monitoring")`，`:143-150`；`runtime/monitoring.rs:355`）。
- `remote/detect.rs`：「检测远端 CLI」按钮，结果按 generation 和 host 双重校验（`detect.rs:22-49`）；离开设置页时 `clear_cli_on_leave` 会取消检测并把 `connected` 置为 false（`detect.rs:6-21`，由 `shell/mod.rs:376-379` 调用）。

### 5.3 `shell/settings.rs`

- 5 个分区：外观 / 字体与字号 / 语言 / 工作台偏好 / 诊断与环境（`settings.rs:25-71`）。
- 所有分区在同一个 `overflow_y_scroll` 容器里依次渲染。左侧导航点击后 `scroll_to_top_of_item` 并聚焦锚点；滚动时 `follow_settings_scroll` 反推当前高亮分区（`:183-213, 275-326`）。
- 「诊断与环境」分区 = `EnvironmentView` + `RemoteView` 上下拼接（`:321-323`）。
- `open_environment_settings` 跳转到 Settings 后定位到该分区（`:215-229`），供远程遮罩页上的按钮使用（`shell/mod.rs:887`）。

### 5.4 状态栏「本地 · …」的来源

`shell/mod.rs:1085-1107`：

- `is_remote()` 为真：显示 `RemoteView::status()` = `"SSH · <主机名>"`（`remote.rs:86-91`）。注意 `"Local"` 分支实际不会被用到。
- 否则按优先级：slow fixture →「本地 · 演示数据 · 受控慢读取」；fixture →「本地 · 演示数据」；`overview.requests_configured()` →「本地 · 已配置用量来源」；其余 →「本地 · 未配置用量来源」。
- 颜色（`shell/status.rs:48-56`）：remote 用 info，fixture 用 warning，已配置用 success，其余用 muted。

也就是说，「本地 · …」后半段表达的是**用量来源 / 数据源**状态，不是执行环境本身。

## 6. Tauri 端依赖面（`ccr-ui/src-tauri`）

### 6.1 依赖点

- `Cargo.toml:24`：`ccr-app = { path = "../../crates/ccr-app" }`。
- `src/ssh/security.rs:2`：`pub use ccr_app::environments::ssh_security::*;`。整个 `ssh_security` 公开 API 直接透传，文件其余部分是测试。
- 用到的 `ssh_security` 符号：
  - `src/commands/ssh.rs:21-24`：`HostKeyStatus`、`ScannedHostKey`、`SshTarget`、`SshTrustService`、`app_known_hosts_path`、`classify_host_key`、`parse_keyscan_output`、`persist_known_host`、`run_openssh_command`
  - `src/ssh/connection.rs:10-13`：`app_known_hosts_path`、`classify_ssh_failure`、`openssh_error`、`posix_single_quote`、`run_openssh_command`
  - `src/platform/ssh.rs:9-12`：`RemotePosixPath`、`SftpWritePlan`、`SshTarget`、`app_known_hosts_path`、`openssh_error`、`posix_single_quote`、`run_openssh_command`、`sftp_read_batch`、`sftp_write_plan`
  - `src/main.rs:231-233`：`SshTrustService::new()` 注入 Tauri state
- 用到的 `ccr_app::environments` 符号：`ProcessHostTrust`、`ReconnectGate`、`allow_reconnect`、`allow_reconnect_target`、`disconnect_error`、`plan_disconnect`、`reconnect_gate`（`commands/ssh.rs:10-13`），`DisconnectReport`（`:842`），`ProcessHostTrust`（`state.rs:13, 134`）；WSL 相关见 `platform/wsl.rs:268-299`。
- 进程层：`src/process/gateway.rs:2-3` 透传 `ccr_app::jobs::process::{ProcessDescriptor, ProcessGateway, …}`。`run_openssh_command` 的参数类型就是这个 `ProcessDescriptor`。
- **不依赖** `ccr_app::environments::ssh::{Workspace, Host, HostInput, Document, ConfigFile, CliStatus}`。Tauri 有自己的 `SshEnvironment`（`platform/ssh.rs`，实现 `ExecutionEnvironment`，读写走 `sftp_read_batch` / `sftp_write_plan`，见 `:213-260`）、`EnvironmentRegistry`（`state.rs:125`）、`SshConnectionManager`（`ssh/connection.rs`）。
- 共享持久化：同一个 `ssh_repo`、`~/.ccr-ui/ccr-ui.db` 和 `app_known_hosts_path()`（`commands/ssh.rs:14-15`、`main.rs:278-300`）。

### 6.2 与桌面端的行为差异（事实）

- Tauri 连接成功后 `set_last_connected_at` 并 `switch_by_id` 激活 SSH 环境；失败时回退到 local（`commands/ssh.rs:322-466`）。
- Tauri 接受 `password` 参数，在内存 `ssh_password_cache` 中缓存（TTL 600 s，`state.rs:33, 131`；`connection.rs:33-66`），但连接测试仍用 `BatchMode=yes` 的 OpenSSH（`connection.rs:115-121`）。
- Tauri 有 `ssh_list_keys`，扫描 `~/.ssh` 密钥（`commands/ssh.rs:832`；`ssh/auth.rs:1-7`）。

### 6.3 IPC 命令（`commands/handler_registry.rs:849-861`）

`ssh_list_hosts`、`ssh_add_host`、`ssh_connect`、`ssh_reconnect`、`ssh_disconnect`、`ssh_get_connection_state`、`ssh_probe_host_fingerprint`、`ssh_confirm_host_fingerprint`（需后端 challenge，`:345, 419`）、`ssh_read_config`、`ssh_write_config`、`ssh_detect_cli`、`ssh_test_connection`、`ssh_list_keys`。

### 6.4 传输替换时必须保持的接口（依上述依赖面）

1. `ccr_app::environments::ssh_security` 中 Tauri 引用的全部公开符号，以及它们的签名和错误码前缀：
   - `SshTarget::{new, host, port, destination, ssh_command, sftp_command}`
   - `RemotePosixPath`、`SftpWritePlan`、`sftp_read_batch`、`sftp_write_plan`
   - `run_openssh_command(Command, &ProcessDescriptor, Option<&[u8]>)`
   - `app_known_hosts_path`、`persist_known_host`、`parse_keyscan_output`、`ScannedHostKey`
   - `SshTrustService::{new, register, consume}`
   - `classify_host_key`、`HostKeyStatus`（serde `snake_case`）
   - `classify_ssh_failure` / `SshFailureKind::code`、`openssh_error`、`posix_single_quote`
2. `ccr_app::environments::{ProcessHostTrust, ReconnectGate, reconnect_gate, allow_reconnect, allow_reconnect_target, plan_disconnect, disconnect_error, DisconnectReport}`。
3. `ccr_app::jobs::process::ProcessDescriptor::{openssh, sftp, ssh_keyscan, ssh_keygen}` 与 `ProcessGateway`。
4. 共享数据契约：`ssh_hosts` / `ssh_known_hosts` 表结构、`~/.ccr-ui/ccr-ui.db` 路径、known_hosts 文件路径与行格式（`host` 或 `[host]:port`，后接 `key_type key_data`，见 `ssh_security.rs:599-644`）。

`ssh::Workspace` 及其 `Transport` trait 只被 `ccr-desktop` 使用，可以在不改 Tauri 的前提下替换。
