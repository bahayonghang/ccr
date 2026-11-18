# SkillPort SSH 远程模式剖析（目标模型、认证、传输与交互）

- 调研票：[#197](https://github.com/bahayonghang/ccr/issues/197)，隶属地图 [#195](https://github.com/bahayonghang/ccr/issues/195)
- 参照仓库：SkillPort 本地只读副本 `D:\Documents\Code\Agents\skills-manage-windows`，HEAD `ea9fd958`（2026-11 归档 `11-05-ssh-skill-import-performance` 之后）
- 下文路径均相对该仓库根目录，`file:line` 指向该提交。只记录事实与出处；标注「推断」的条目不是源码直接陈述。
- SkillPort 的术语是 target（目标）。CCR 地图里对应的词是「执行环境 / SSH 主机」，本报告保留 SkillPort 原词以便对照源码。

## 0. 结论速览

- 目标模型是「全局唯一当前目标 + 按目标隔离的本机缓存库」。当前目标 id 存在本机 SQLite `settings` 表里，只有在 id 失效或配置被隔离时才自动回落 Local；连接失败不会回落。
- 认证只有两种：`key`（私钥路径）和 `password`。**没有独立的 ssh-agent 模式**，也不支持带 passphrase 的私钥。密码进系统凭据库（`keyring` crate），写入后回读校验，失败时退到 Windows DPAPI 或仅本次会话。
- 传输仍是**每次操作拉起一个系统 `ssh.exe` 子进程**。ADR-001 选定了 russh，但阶段 3 从未落地：`src-tauri/Cargo.toml` 里没有 `russh`，代码里也没有自管 `known_hosts`。已落地的是阶段 1（超时、进程树监管）和阶段 2（脚本批量化）。
- 切换到 SSH 目标前要先探测连通性，探测失败不切换。当前 SSH 目标不可达时，中央技能页不显示缓存库（fail-closed），只显示「无法连接」旗标和重试入口。
- 设置页已经是「一个分页一个路由」，左侧导航（窄屏变成横向条），远程目标独占「连接与同步」页。

## 1. 目标模型

### 1.1 字段

- `TargetKind = local | ssh | wsl`（`src-tauri/src/targets/model.rs:39-45`）。Local 是保留 id `"local"`，不落库（`model.rs:8`，`registry.rs:347-365`）。
- SSH 目标持久结构 `RemoteTargetConfig`（`model.rs:73-94`）：`id`、`label`、`host`、`username`、`port`、`authMethod`、`keyPath`、`credentialKey`、`protectedPassword`（可选，DPAPI 密文）、`remoteHome`、`remoteOs`、`symlinkEnabled`。明文 `password` 带 `#[serde(skip)]`，永不序列化（`model.rs:89-90`）。
- 新建时 id 为 `ssh-<uuid>` / `wsl-<uuid>`（`commands.rs:8`、`commands.rs:216`），端口缺省 22（`commands.rs:418`）。更新时 id 不可变（`commands.rs:434-437`，错误 `TargetIdImmutable`）。
- id 只允许 ASCII 字母数字、`-`、`_`（`model.rs:10-16`）。同一校验同时用于载入配置和拼缓存路径（`commands.rs:630-635`，测试 `config.rs:690-715`），用来防止 `../escape` 一类 id 逃出缓存目录。
- 对外 DTO `TargetSummary`（`model.rs:171-191`）不含密码。`keyPath` 只在 key 模式返回（`registry.rs:294-297`），另带 `cacheDbPath`、`credentialStatus`、`credentialError`、`isActive`。

### 1.2 持久化位置与格式

- 三个 key 写进本机 SQLite `settings` 表：`ssh_targets_v1`（SSH 目标 JSON 数组）、`wsl_targets_v1`、`active_target_id_v1`，另有 `target_config_quarantine_v1`（`model.rs:2-4`、`model.rs:17`；读写在 `persist.rs:370-388`）。
- 新建 / 更新是「先探测，后持久化」：`create_ssh_target_impl` 先跑 `probe_ssh_target` 拿到 `remote_home` / `remote_os`，OS 不是 Linux / Darwin 时直接拒绝，然后才持久化（`commands.rs:3-19`、`commands.rs:21-58`；OS 白名单在 `exec.rs:102-104`）。
- 持久化要求全部成功，否则整体回滚（模块注释 `persist.rs:1-3`）：
  - 新建：先存密码，再写目标列表，再打开该目标的缓存库。任何一步失败都会回滚 settings 快照、删凭据、丢弃缓存池（`persist.rs:26-63`、`persist.rs:267-281`）。
  - 更新：先拿到旧密码，再写新列表。如果从 password 改成 key，就删旧凭据，删除失败则回滚。之后丢弃并重开缓存池（`persist.rs:83-160`、`persist.rs:283-299`）。
  - 回滚用一个事务恢复 settings 快照（`config.rs:184-207`）。
  - 测试覆盖各个失败注入点（`tests.rs:1649-1835`）。

### 1.3 当前目标如何解析

- 每次需要时都从 settings 重读快照（`registry.rs:377-380`），解析为 `ActiveTarget::{Local, Ssh, Wsl}`（`model.rs:222-227`）。解析 SSH 目标时会顺带挂上可用密码（`registry.rs:406-408`）。
- `TargetContext` 把解析出的目标和对应 `DbPool` 绑成一次性快照（`model.rs:302-332`，`registry.rs:428-435`）。测试 `resolved_context_keeps_owned_target_while_missing_active_falls_back_to_local` 说明：快照拿到之后，即使 settings 里的 active id 变了，快照本身也不变（`tests.rs:427-449`）。
- `set_active_target_impl` 只校验 id 存在，然后写入 `active_target_id_v1`。它**不联网，也不丢缓存**（`commands.rs:343-368`）。连通性检查放在前端切换流程里（见 §4.3）。

### 1.4 切换时缓存与连接的失效规则

- **连接**：后端没有连接池。`open_ssh_target` 每次调用都新建一个 `ConnectedSshTarget`（包含目标副本、密码、askpass 助手、runner），打开时不探测（`askpass.rs:37-52`，测试 `tests.rs:658`）。每条命令都是一个新的 `ssh.exe` 进程（§3）。
- **缓存库池**：`TargetRegistry.pools` 以 target id 为键缓存 `DbPool`（`registry.rs:3-7`、`registry.rs:453-494`）。切换目标**不会**丢弃这些池，只在以下情况丢弃：
  - 更新 SSH / WSL 目标（`persist.rs:138`、`persist.rs:213`）
  - 删除目标（`commands.rs:339`）
  - 持久化回滚（`persist.rs:275-279`、`persist.rs:293-297`）
- **前端状态**：`AppShell` 监听 `activeTarget.id` 变化。变化后会重置 platform、central、skills、marketplace、skillsCli、projects 各 store，清空 GitHub 分支，然后全局重扫并重新加载项目（`src/components/layout/AppShell.tsx:108-135`）。首次加载不触发这条路径（`AppShell.tsx:111-122`）。
- **可达性状态保留**：`loadTargets` 重新拉取列表时，如果当前 SSH 目标的 id、host、port、username 都没变，而且上次结果是 reachable，就保留 reachable 不重新探测；否则进入 `checking` 并在后台探测（`src/stores/targetStore.ts:76-85`、`targetStore.ts:168-203`）。
- 探测结果用 `sshProbeSerial` 丢弃过期响应（`targetStore.ts:467-480`）。

### 1.5 瞬时失败不自动回落本地

- 架构文档明确写：瞬时失败不会把当前目标翻回 local，必须由用户显式切换（`docs/architecture/ssh-mode.md:67`）。
- 代码中只有两种情况会自动回落 Local：
  1. settings 里的 active id 指向不存在的目标。载入快照时会把 active 改写为 `local`，并在隔离状态里记 `active_target_reset=true`（`config.rs:98-119`）。
  2. 删除的正好是当前目标（`commands.rs:318-321`，`config.rs:174-179`）。
- 切换到 SSH 目标时探测失败：不调用 `set_active_target`，活动目标保持原值（`targetStore.ts:399-406`）。这是 `11-03-ssh-switch-reachability-flag` 任务的决策（`.trellis/tasks/archive/2026-11/11-03-ssh-switch-reachability-flag/prd.md` 的 Decision 与 Requirements 2）。

### 1.6 远程 HOME 探测

- 探测脚本一次输出 `HOME\t$HOME`、`OS\t$(uname -s)`，并执行 `mkdir -p "$HOME/.skillsmanage/skills"`，成功后打印 `MKDIR_OK`（`exec.rs:73-80`）。脚本经 stdin 交给远端 `sh -s --`（`exec.rs:682-689`，`process_api.rs:73-87`）。
- 解析规则（`exec.rs:711-742`）：
  - HOME 必须以 `/` 开头，否则报 `ProbeHomeMissing`。
  - 没有 `MKDIR_OK` 时报 `ProbeMkdirUnconfirmed`。
  - OS 缺失时记为 `unknown`。
- 探测结果在新建、更新、「改密码」成功后写回 `remoteHome` / `remoteOs`（`commands.rs:15-17`、`commands.rs:48-49`、`commands.rs:170-174`）。
- 「测试」按 id 发起且没有带新密码时，不写回 HOME（`commands.rs:96-117`，测试 `tests.rs:1893-1945` 断言 settings 原文不变）。
- 远程路径展开用保存下来的 `remote_home`（`src-tauri/src/paths.rs` 的 `expand_remote_home_path`）。

### 1.7 按目标的本机缓存库

- 路径为 `<app_data>/targets/<target_id>/db.sqlite`（`commands.rs:622-628`）。`app_data` 是 `~/.skillsmanage`（`paths.rs:14`、`paths.rs:255-263`），Windows 下兼容旧 Git Bash HOME（`paths.rs:527-557`）。
- 用远程 HOME 作为 POSIX 家目录来种子化内置 agent 路径（`src-tauri/src/db/migrations.rs:297-307`）。
- 用户指南的说法：每个目标保存自己的扫描结果与元数据；切换目标就是换一个缓存文件，不改动旧文件（`docs/guide/ssh-remote.md:9`，`docs/guide/settings.md:19-21`）。
- 设置页每行显示该目标的 `cacheDbPath`（`src/components/settings/RemoteTargetsSettingsControls.tsx:128-135`）。

## 2. 认证与凭据

### 2.1 认证方式

- `SshAuthMethod = key | password`，默认 `key`（`model.rs:47-53`）。**没有 agent 枚举值**。
- key 模式必须填 `keyPath`：请求校验见 `commands.rs:397-400`，载入校验见 `config.rs:275`。私钥只保存路径，不复制内容（`docs/guide/ssh-remote.md:16`）。
- key 模式传给 ssh.exe 的参数是 `-i <keyPath> -o BatchMode=yes -o PreferredAuthentications=publickey`（`exec.rs:226-235`）。没有设置 `IdentitiesOnly`。（推断：系统 ssh 可能仍会尝试 agent 里的密钥，但 UI 和模型都不把 agent 当作独立方式。）
- 任何请求带非空 `passphrase` 都会被拒，错误文案是「Passphrase-protected keys are not supported yet. Use ssh-agent or an unencrypted key.」（`commands.rs:388-395`，`error.rs:153-156`）。
- password 模式参数是 `BatchMode=no`、`PreferredAuthentications=password,keyboard-interactive`、`PubkeyAuthentication=no`、`NumberOfPasswordPrompts=1`（`exec.rs:236-245`）。

### 2.2 系统凭据库键名

- service 固定为 `"SkillPort SSH Targets"`（`model.rs:5`）。
- account（credential key）是 `<target_id>:ssh-password`（`commands.rs:637-639`）。已有目标优先用持久化的 `credentialKey` 字段（`commands.rs:641-651`）。
- 实现用 `keyring = "3"`（`src-tauri/Cargo.toml:46`，`cred.rs:202-246`）。

### 2.3 密码保存与读取顺序

- 保存：`set_password` 后立刻 `get_password` 回读并比较（`registry.rs:240-252`）。
  - 一致：状态 `Stored`，清掉 DPAPI 副本和会话缓存。
  - 回读不一致或写入失败：退到 Windows DPAPI（`CryptProtectData`，hex 编码后写入目标 JSON 的 `protectedPassword` 字段），同时放进会话缓存，状态仍为 `Stored`，但附带 error 文案（`registry.rs:217-238`、`cred.rs:60-187`）。
  - DPAPI 也失败（非 Windows 必然失败，见 `cred.rs:189-200`）：只放会话缓存，状态 `Session`。
- 读取顺序：系统凭据库 → DPAPI 副本 → 进程内会话缓存（`registry.rs:176-202`）。`load_target_password` 在没有任何来源时报 `PasswordUnavailable`，文案提示用户去设置页重新保存（`commands.rs:665-679`，`error.rs:168-171`）。
- 删除：清会话缓存、清 DPAPI 字段、删凭据库条目；`NoEntry` 视为成功（`registry.rs:265-277`，`cred.rs:237-245`）。删除目标时，如果删凭据失败，会回滚 settings 并恢复会话密码（`commands.rs:323-338`）。

### 2.4 凭据状态 `TargetCredentialStatus`

- 枚举有四个值：`stored`、`session`、`missing`、`unreadable`；`is_available()` 在 `stored` 和 `session` 时为真（`model.rs:55-71`）。
- 计算逻辑（`registry.rs:78-174`）：
  - 凭据库有非空值 → `Stored`
  - 凭据库为空或缺失：有 DPAPI → `Stored`；有会话 → `Session`；都没有 → `Missing`
  - 凭据库报错：有 DPAPI → `Stored` 附错误；DPAPI 报错且无会话 → `Unreadable`
- 只对 password 模式计算，key 模式不报告（测试 `tests.rs:1314`）。
- UI 文案：已保存 / 仅本次会话 / 需要密码 / 凭据库不可用（`src/i18n/locales/zh.json:415-418`）。徽章色调分别是 success / warning / neutral / destructive（`src/components/settings/remoteTargetsSettingsLayout.ts:21-32`）。
- 状态为 `session` 时，成功提示会额外说明重启后需重新输入（`commands.rs:653-663`）。

### 2.5 密码更新流程

- 专用命令 `update_ssh_target_password_impl`（`commands.rs:151-209`）：
  1. 只接受 password 模式的目标，否则报 `NotPasswordAuth`。
  2. 把新密码挂到目标上，先探测。
  3. 探测成功且 OS 受支持：写回 HOME / OS，保存密码，写列表。
  4. 探测失败：返回 `ok:false`，**不保存密码**。
- 「测试」带 id 和新密码时同样「探测成功才保存」（`commands.rs:88-117`）。只带 id 时，用已存的凭据或会话密码探测，探测结束就丢弃，不落盘（`commands.rs:96-100`，测试 `tests.rs:1893-1945`）。
- 编辑表单里密码留空表示沿用已保存的密码，占位文案是「SSH 密码（留空沿用已保存密码）」（`commands.rs:453-475`，`RemoteTargetsSettingsControls.tsx:391-397`）。

### 2.6 askpass

- Windows：把**应用自身 exe** 设为 `SSH_ASKPASS`，并设置 `SSH_ASKPASS_REQUIRE=force`、`DISPLAY=skillport`、`SKILLPORT_SSH_ASKPASS_HELPER=1`，密码放在子进程环境变量 `SKILLPORT_SSH_PASSWORD` 里（`askpass.rs:54-63`，`exec.rs:246-255`）。应用启动时如果检测到 marker 环境变量，就把密码打印到 stdout 后退出（`exec.rs:52-71`）。
- 非 Windows：在临时目录写一个 0700 权限的 sh 脚本，内容是打印该环境变量。每次创建前清理超过 1 小时的同前缀残留，`Drop` 时删除（`askpass.rs:65-190`）。
- 阶段 1 文档记录的旧问题：每次连接都写一个临时脚本，Windows 下常有残留（`plans/ssh-perf/01-phase1-stability.md:107-125`）。现在 Windows 路径改为复用 exe，已无临时文件。

### 2.7 配置隔离（quarantine）

- 载入快照时 SSH 和 WSL 两个域分别校验（`config.rs:59-147`）：
  - 必须是 JSON 数组。
  - 对象只允许白名单字段（SSH 见 `config.rs:9-22`；**多出任何键都算 `invalid_schema`**）。
  - 能反序列化，id 合法、不重复、不等于 `local`。
  - 必填字段非空，端口不为 0，key 模式有 `keyPath`（`config.rs:258-331`）。
- 某个域失败时，**整域**改写为 `[]`。同时记录一条事件：域、时间、原因码（`invalid_json | invalid_schema | duplicate_id | reserved_id`）、原文字节数、原文 SHA-256，**不保存原文**。若 active 指向被清掉的目标，就回落 local。所有改写在同一次 `set_settings` 中提交（`config.rs:76-139`）。
- 一个域损坏不影响另一个域（测试 `config.rs:411-512`）。写事务失败时原 settings 全部保留（`config.rs:536-576`）。同一份坏原文重复载入时，保留首次时间戳（`config.rs:514-534`）。
- 回读隔离状态时，会再校验元数据格式；不合规就返回默认空状态，防止把伪造内容（可能含密文）经 IPC 吐出（`config.rs:231-256`，测试 `config.rs:658-688`）。
- UI 在远程目标区顶部显示警告条：标题「已隔离无效的目标配置」，逐条列出 `域 · 时间 · 字节数 · SHA-256 前 12 位`；如有回落，再加一句「原当前目标已失效，应用已回退到本机。」。状态读取失败时显示另一条警告（`src/components/settings/RemoteTargetsSettingsSection.tsx:157-204`，`zh.json:442-446`）。

## 3. 传输

### 3.1 russh 落地程度：未落地

- `src-tauri/Cargo.toml` 和 `src-tauri/Cargo.lock` 都没有 `russh`（`grep russh` 无结果）。SSH 仍通过系统 `ssh.exe` 执行：Windows 优先使用 `%WINDIR%\System32\OpenSSH\ssh.exe`，否则用 PATH 里的 `ssh`（`exec.rs:131-145`），每条命令都从 `base_command()` 重新拼参数（`exec.rs:201-260`）。
- ADR-001（2026-05-06，状态「已采纳」）选择 russh、否决 ControlMaster 和 ssh2（`plans/ssh-perf/decisions.md:3-41`）：
  - 否决 ControlMaster：Windows OpenSSH 的命名管道行为不一致，仍要 spawn，取消要靠 kill。
  - 否决 ssh2：依赖 C 库，打包困难。
  - 后果：删除 askpass，自管 known_hosts，需要兼容多种私钥格式。
- ADR-006 规定落地后不保留 ssh.exe 兜底（`decisions.md:122-145`）。
- 后续归档设计明确写着该阶段「was never implemented」，并改为「条件性子任务」。需要单独做依赖与版本调研、兼容矩阵、回滚方案和真机门槛（`.trellis/tasks/archive/2026-07/07-11-remote-update-performance/design.md:163-175`）。2026-08 的清理提速任务把 russh 列为未落地，不在范围内（`.trellis/tasks/archive/2026-08/08-17-ssh-leftover-cleanup-speed/research/leftover-cleanup-bottleneck.md:80`）。
- 阶段 3 计划草案（仅计划，未实现）（`plans/ssh-perf/03-phase3-russh.md`）：
  - 依赖写的是 `russh = "0.45"`、`russh-keys`，可选 `russh-sftp`，并注明「须查最新版」（`:29-41`）。
  - `SshClientPool` 按 target 复用连接，每个 target 最多 4 个 channel，提供 `invalidate` / `close_all`（`:62-83`、`:186-194`）。
  - `channel.exec` 替代子进程（`:85-112`）。
  - 自管 known_hosts（`:114-123`）。
  - key 模式用 `authenticate_publickey`，password 模式用 `authenticate_password`，passphrase 从凭据库读（`:125-143`）。
  - 用 `CancellationToken` 关闭 channel（`:178-182`）。
  - 断线后自动 invalidate 并重连（`:203`）。

### 3.2 超时与取消（已落地，进程级）

- ssh 参数：`ConnectTimeout=10`、`ServerAliveInterval=15`、`ServerAliveCountMax=3`（`exec.rs:7-9`、`exec.rs:208-222`）。
- Rust 侧按命令类别设硬截止时间和输出上限（`runner.rs:32-92`）：

  | 类别 | 截止 | stdout / stderr 上限 |
  | --- | --- | --- |
  | `probe` | 30 s | 1 MiB |
  | `standard` | 120 s | 8 MiB |
  | `bulk_transfer` | 15 min | 32 MiB |
  | `bounded_read` | 120 s | max_bytes + 1 |

- `ProcessRunner`（`runner.rs:240-354`）：
  - 用 `tokio::select!` 让「完成 / 取消 / 超时」三者竞争。
  - 超时、取消、超出输出上限时，终止整个进程树：Windows 用 Job Object `KILL_ON_JOB_CLOSE`（`process_tree.rs` 开头的 windows 段），然后在 5 s 内回收子进程（`runner.rs:388-406`）。
- 取消信号是 `ProcessCancellation::Atomic(&AtomicBool)`，每 50 ms 轮询一次（`runner.rs:94-119`）。**多数单条命令传的是 `Never`**，例如 `exists`、`inspect_path`、`write_file`、`copy_dir`、`read_file_bounded`（`exec.rs:321-442`）。只有 `run_script_cancellable` 和 `run_command_with_stdin_bytes_cancellable` 接受取消（`process_api.rs:101-146`）。
- 错误映射到类型化变体：`ProcessTimedOut{transport,class,timeout_ms}`、`ProcessCancelled`、`ProcessOutputLimitExceeded`、`ProcessTerminationFailed`（`process_api.rs:11-52`，`error.rs:53-88`）。
- `CommandRunner` 是可注入接口，测试里用 `FakeRunner` 断言参数形状（`runner.rs:235-238`，`tests.rs:1343-1585`）。

### 3.3 批量化脚本协议（已落地）

- 通用做法：脚本体经 stdin 交给远端 `sh -s -- 'arg1' ...`，参数用单引号转义（`exec.rs:682-689`、`exec.rs:703-705`）。
- 探测 5 合 1（§1.6）。有界读：远端先 `wc -c` 取大小，超限用退出码 44 表示，大小非数字用 45，否则 `dd` 读取 `max+1` 字节；本地再校验一次长度（`exec.rs:10-50`）。
- 扫描批量化设计（`plans/ssh-perf/02-phase2-batching.md`）：
  - 第一次 SSH 用 `find` 找出全部 `SKILL.md`，按行输出 `ROOT_OK / SKILL / ROOT_MISS`（`:55-88`）。
  - 第二次 SSH 用 `\x01PATH\x02…\x03` 和 `\x04EOF\x05` 控制字符分隔批量读出内容（`:90-113`）。
  - ADR-002 说明选控制字符分隔的理由：不用 tar/base64，因为体积多约 30%；不用 jsonl，因为转义成本高（`decisions.md:45-65`）。
- 原问题量化：每次扫描 100-200 次握手，端到端 60-200 s（`decisions.md:11`，`plans/ssh-perf/00-overview.md:9-17`）。

### 3.4 known_hosts

- 现状：只传 `StrictHostKeyChecking=accept-new`（`exec.rs:223-224`），**没有** `UserKnownHostsFile`，因此用的是系统 ssh 默认的 `~/.ssh/known_hosts`。`src-tauri/src` 下没有任何 `known_hosts` 实现。
- 计划（ADR-004，未实现）（`decisions.md:84-103`）：
  - 应用自管 `<app_data>/known_hosts`，与 `~/.ssh` 隔离。
  - 首连 accept-new，之后严格匹配。
  - 「信任并替换」前展示指纹。
  - 后果：系统 ssh 已信任的主机，在应用内仍要首次确认。
- 风险登记 R3：不匹配时返回结构化错误 `KnownHostsMismatch{host, fingerprint_expected, fingerprint_got}`；UI 用对话框展示新旧指纹，trust 时把旧指纹备份到 `known_hosts.bak`（`plans/ssh-perf/risks.md:28-35`）。

### 3.5 远端写入

- `write_file` 先 `mkdir -p` 父目录，再 `cat > path`，内容走 stdin（`exec.rs:389-402`）。**没有临时文件加 rename 的原子写**。`copy_dir` 用 `cp -R`，删除用 `rm -rf --`（`exec.rs:429-454`）。

## 4. 测试连接与错误呈现

### 4.1 `test_ssh_target_with_probe`

- 签名把探测函数作为参数注入（`commands.rs:68-73`），方便测试替换。
- 分支：
  - 请求带 id：载入已存目标。带新密码时标记「成功后保存」，否则挂上可用密码（凭据库、DPAPI、会话）。
  - 请求不带 id：由表单字段临时构造目标，未保存表单也能测（`commands.rs:74-94`、`commands.rs:498-543`）。
- 返回值：
  - 连接、认证或探测失败返回 `Ok(SshTargetTestResult{ok:false, message})`，而不是 `Err`（`commands.rs:129-148`）。
  - 远端 OS 不受支持时返回 `ok:false`，但带回 HOME 和 OS（`commands.rs:129-139`）。
  - 只有目标不存在、字段缺失、带 passphrase 等前置校验失败才返回 `Err`。
- 前端「测试」按钮：结果写进区块内的状态行（`targetMessage`），同时弹 toast。成功文案是 `连接成功：{{os}} {{home}}`（`src/pages/settingsViewActions.ts:312-340`）。

### 4.2 `error.rs` 错误分类

- `TargetsError`（`thiserror`）按分区组织：IO、DB、JSON、SSH 命令传输、WSL 传输、连接探测、目标注册与 CRUD、凭据、Windows DPAPI（`error.rs:8-203`）。
- 模块注释说明：Display 文案刻意沿用历史字符串，IPC 层会把它字符串化，前端原样显示在 toast 里（`error.rs:1-6`）。
- 认证失败的分类：stderr 含 `permission denied` 时，按认证方式生成提示（`exec.rs:269-312`）。
  - key 模式：检查本机私钥路径和远端 `authorized_keys`。
  - password 模式：检查密码和 sshd 是否允许 password / keyboard-interactive。
  - 提示后附 `Raw ssh error: …`。
  - stderr 为空时显示 `ssh exited without stderr.`。
- 阶段 3 计划提出的类型化分类（协议不兼容 / 认证失败 / 网络问题）没有落地（`risks.md:5-13`）。

### 4.3 可达性提示（`CentralSkillsSshReachability`）

- 状态机 `SshReachability = idle | checking{targetId} | reachable{targetId} | unreachable{targetId,message}`（`src/lib/sshLibraryGate.ts:1-5`）。
- 门闩 `sshLibraryIsLive` / `centralSshLibraryGate`（`sshLibraryGate.ts:18-46`）：
  - 目标列表还没解析出来时为 `pending`。
  - 非 SSH 目标直接放行。
  - SSH 目标只有 `reachable` 且 targetId 对得上才放行。
  - targetId 对不上时一律按 `checking` 处理，即 fail-closed。
- 切换到 SSH 目标：先调用 `test_ssh_target({id})`，成功后才 `set_active_target`，并直接标记为 reachable（`targetStore.ts:375-431`）。
- 应用启动时如果当前目标已是 SSH，`loadTargets` 会触发后台探测（`targetStore.ts:175-203`）。
- 中央技能页的行为由测试固定下来（`src/test/pages/CentralSkillsSshReachability.test.tsx`）：
  - 目标未解析：显示「正在加载技能...」，不显示库和旗标（`:58-68`）。
  - `checking`：显示「正在检查连接」面板，隐藏缓存库的卡片、计数和路径（`:70-79`）。
  - `reachable`：显示库，不显示旗标（`:81-88`）。
  - `unreachable`：页头出现「无法连接」旗标，title 是错误信息；同时显示不可达空状态，带「重试连接」和「打开 SSH 设置」两个按钮，库保持隐藏（`:90-106`）。
  - 已可达的目标即使 store 里留有切换失败的错误，也照常显示库（`:108-118`）。Local / WSL 永远显示库（`:120-133`）。
  - 不可达记录属于别的目标：按 checking 处理，不显示旗标（`:135-147`）。
  - 重试成功后重新加载技能并重扫，重载失败不外露（`:149-178`）；重试仍失败则不重载（`:180-200`）。
  - 用例全程断言密码不出现在页面上。
- 组件实现（`src/components/central/SshReachabilityStatus.tsx`）：
  - 旗标是玻璃按钮，destructive 色调，带警告三角图标，点击跳转 `/settings/connections?section=remote-targets`（`:7`、`:22-41`）。
  - 错误信息只有在 ≤160 字、无换行时才放进 title（`:9-20`）。
  - 面板居中，标题加说明加两个按钮（`:43-102`）。
  - 说明文案（`zh.json:1419-1422`）：
    - 检查中：「正在确认这个 SSH 目标能否连通。连通前不显示已保存的技能库。」
    - 不可达：「这个 SSH 目标目前连不上。连通前不显示已保存的技能库。」
- 设计来由：用户截图里一个 IP 已经错误的目标仍被标成 Active，并显示技能数量和远程路径（`11-03-ssh-switch-reachability-flag/prd.md` Confirmed Facts 第 1 段）。同一份 PRD 记录：Dashboard、Marketplace 等其他页面**仍读缓存**，不受这个门闩约束（Out of Scope 一节）。

## 5. 界面与交互

### 5.1 设置分区方式（`settingsPages.ts` / `SettingsView.tsx` / `settingsPageSections.tsx`）

- 设置页有 6 个分页，每页一个路由 `/settings/<id>`（`src/components/settings/settingsPages.ts:30-79`）：
  - 外观 `appearance`（默认页）
  - 连接与同步 `connections`，只含 `remote-targets` 一个区块
  - 平台管理 `platforms`
  - 集成与密钥 `integrations`
  - 技能来源 `skill-sources`
  - 帮助与关于 `about`
- 中文页名和描述见 `zh.json:2894-2915`。「连接与同步」的描述是「管理本机、SSH 与 WSL 目标，并启动本机到远程同步。」
- 旧链接兼容（`settingsPages.ts:85-154`）：`?section=` 和 `#hash` 映射到新分页，未知子路径落到默认页。默认页的规范路径是 `/settings`，其他页会被 replace 成规范路径（`SettingsView.tsx:249-262`）。
- 布局（`SettingsView.tsx:454-476`）：
  - 外层在 `lg` 以上是左右两栏；左侧 `SettingsSideNav`，宽 `lg:w-64`。
  - 导航是一个玻璃面板：大屏竖排、sticky；窄屏变成横向可滚动条。当前项只用 `aria-current="page"` 加文字样式标出，不用填充色或侧边条（`src/components/settings/SettingsSideNav.tsx:19-49`）。
  - 右侧 `main` 独立滚动，内容最大宽 `max-w-6xl`，顶部是 `PageHeader`（标题加描述）。
  - 切页时滚动位置回到顶部（`SettingsView.tsx:376-379`）。
- 带 `section=remote-targets` 进入时，平滑滚动到远程目标区（`SettingsView.tsx:381-398`）。
- `renderSettingsPage` 按 pageId `switch` 出整页组件；对话框（加目录、平台、本机→远程同步）挂在页外（`src/pages/settingsPageSections.tsx:168-216`）。
- 设置规则（`docs/guide/settings.md:17-22`）：改动立即提交，没有单独的保存按钮；密钥保存后只显示掩码，不再明文回显。

### 5.2 远程目标区（`RemoteTargetsSettingsSection.tsx` 与 `RemoteTargetsSettingsControls.tsx`）

区块外壳是 `SettingsSection`：Server 图标，标题「目标管理」，描述「在本机技能、SSH 远程用户目录和 WSL 发行版之间切换。」（`RemoteTargetsSettingsSection.tsx:150-155`，`zh.json:371-372`）。从上到下依次是：

1. **隔离警告条**（如有，§2.7）。
2. **操作结果行**：一行小字，成功为主色，失败为 destructive 色，`role=status`（`:205-212`）。
3. **主机列表**：一个圆角卡片，行间有分隔线，第一行是 Local（`:214-244`）。每行用 `RemoteTargetRow` 渲染（`RemoteTargetsSettingsControls.tsx:72-248`），`xl` 以上左右两栏：
   - 左栏：
     - Server 图标、名称、种类徽章（本机 / SSH / WSL）、「当前」徽章（success 色）。
     - 描述行 `认证方式 • user@host:port • OS • HOME`（`remoteTargetsSettingsLayout.ts:45-73`）。
     - 等宽字体的缓存库路径。
     - password 模式的 SSH 行**总是**附带「改密码」子行（判定条件只看 `kind==='ssh' && authMethod==='password'`，见 `src/lib/targetKind.ts:21-28`）：凭据状态徽章、错误说明、密码输入框、「更新密码」按钮。输入为空或正在更新时按钮禁用（`RemoteTargetsSettingsControls.tsx:136-153`、`:261-316`）。
   - 右栏动作按钮依次是：
     - 本机→远程同步（非本机行）
     - 编辑 / 取消（同一个按钮切换）
     - 测试（玻璃样式，进行中显示转圈）
     - 切换（只在非当前行显示，进行中显示转圈）
     - 删除（`InlineConfirmAction`，点一次变成「确认删除」，再点才执行）
     - Local 行没有编辑、测试、删除（`:156-221`）。
   - 编辑态：行下方展开内联表单，左缩进对齐（`:223-245`）。字段依次是认证方式分段按钮（SSH key / 账号密码）、名称、主机、用户、端口；凭据输入框按认证方式切换为「SSH 密钥路径」文本框或密码框；最后是保存和取消（`:327-407`）。网格在 `md` 两列、`xl` 六列（`remoteTargetsSettingsLayout.ts:5-10`）。
4. **新建卡片**：两张浅底卡片。
   - 「添加 SSH 目标」：字段与编辑表单相同，底部并排「测试」和「添加 SSH」两个按钮，互斥禁用（`RemoteTargetsSettingsSection.tsx:246-259`，`RemoteTargetsSettingsControls.tsx:462-560`）。
   - 「添加 WSL 发行版」：发行版下拉框加刷新按钮，已添加的发行版做标记（`RemoteTargetsSettingsSection.tsx:261-278`）。
   - 认证方式分段按钮是胶囊形的两格切换，选中格为主色描边加浅底，带 `aria-pressed`（`src/components/settings/RemoteTargetFormPrimitives.tsx:81-117`）。
5. **底部提示**：四行小字，依次说明支持的认证方式与存储位置、何时需要重新保存密码、Windows 私钥路径与查找 IP 的方法、WSL 的说明（`RemoteTargetsSettingsSection.tsx:281-286`，`zh.json:420-421`）。

状态管理（`targetStore.ts:32-74`）：

- 「正在进行」类状态都按 target id 分开记录：`isCreating`、`updatingTargetId`、`testingTargetId`（新建表单用 `"new"` / `"new-wsl"`）、`updatingPasswordTargetId`、`switchingTargetId`、`deletingTargetId`。
- 变更后重拉列表；重拉失败时设 `requiresTargetReload`（`targetStore.ts:112-121`）。

### 5.3 快速切换器（`TargetQuickSwitcher.tsx`）

- **位置**：TopBar 右侧，在「导入技能」按钮之后、快捷键按钮之前（`src/components/layout/TopBar.tsx:136-153`）。`sm` 以下隐藏（`TargetQuickSwitcher.tsx:96`）。
- **触发器**：玻璃胶囊小按钮，内容是 Server 图标、当前目标名（本机显示「本机」）和下拉箭头（展开时旋转），最大宽 `max-w-44`，名称截断。
  - 当前目标是远程类时，加主色浅底和描边（`:94-101`）。
  - title 提示：SSH 显示 `user@host`，WSL 显示发行版名，本机显示「本机」（`:87-93`）。
- **菜单**：宽 `w-72`，右对齐，非模态（`:84`、`:114`）。
  - 每个目标一项：Server 图标、名称、当前项的「当前」徽章、第二行描述（本机说明 / `WSL - 发行版` / `认证方式 - user@host:port`）。
  - 右侧：切换中显示转圈；非当前项显示「切到这里」（`:115-168`）。
  - 任一切换进行时，所有项都禁用（`:117-123`）。
- **空状态**：只有本机一项时，追加一行「尚未配置远程目标。」（`:170-174`）。
- **失败**：菜单保持打开（受控 `open`），菜单内用 `role=alert` 显示错误，同时弹 toast 作为第二通道（`:59-74`、`:176-180`）。
- **「管理目标…」**：分隔线后的最后一项，点击后关闭菜单并跳到 `/settings`（`:76-79`、`:182-189`）。注意：跳的是 `/settings`，会落到默认的「外观」页，**不会**直接到「连接与同步」。11-03 PRD 也记录了「管理入口目前只去 `/settings`」；而失败旗标跳的是 `/settings/connections?section=remote-targets`（§4.3）。

## 6. 文档与代码不一致

| 文档说法 | 代码事实 |
| --- | --- |
| 字段名为 `private_key_path`（可选，缺省回落 ssh-agent）、`password_ref`、`remote_root`（`docs/architecture/ssh-mode.md:9-16`） | 实际字段是 `keyPath`（key 模式必填）、`credentialKey`、`remoteHome`；没有 agent 回落（§2.1） |
| 远程库是经 SSH 打开远端 `~/.skillsmanage/db.sqlite`（`ssh-mode.md:39-40`） | 实际是本机 `~/.skillsmanage/targets/<id>/db.sqlite`（§1.7） |
| registry 缓存连接，切换目标时失效（`ssh-mode.md:43`） | registry 只缓存 DbPool，不缓存连接；切换目标不丢池（§1.4） |
| SSH 目标禁用 symlink 安装（`docs/guide/ssh-remote.md:24`） | `remote_target_symlink_allowed` 在 OS 为 Linux / Darwin 时返回 true（`exec.rs:106-116`） |

## 7. 可借鉴 / 不适用 / 已知坑

### 可借鉴

| 条目 | 出处 |
| --- | --- |
| 当前目标 id 单独持久化；Local 是保留 id，不落库；只有 id 失效或删除当前目标时才回落本地，连接失败不回落 | `src-tauri/src/targets/config.rs:98-119`；`commands.rs:318-321`；`docs/architecture/ssh-mode.md:67` |
| 先探测后持久化：新建 / 更新 / 改密码都先跑探测，拿到 HOME 与 OS，失败就不保存 | `commands.rs:3-58`、`commands.rs:151-209` |
| 单次探测脚本同时取 HOME、OS 并确认目录可写，HOME 必须是绝对 POSIX 路径 | `exec.rs:73-80`、`exec.rs:711-742` |
| 切换到 SSH 前先探测，失败不切换，菜单留在原处显示错误 | `src/stores/targetStore.ts:375-431`；`TargetQuickSwitcher.tsx:59-74` |
| 可达性 4 态状态机加 targetId 绑定，加序号丢弃过期探测，fail-closed 门闩 | `src/lib/sshLibraryGate.ts:1-46`；`targetStore.ts:467-510` |
| 只在失败时出现的可点击旗标，直达连接设置；正常和检查中不显示 | `src/components/central/SshReachabilityStatus.tsx:22-41`；`.trellis/tasks/archive/2026-11/11-03-ssh-switch-reachability-flag/prd.md` |
| 按目标隔离的本机缓存库，路径由校验过的 id 拼出 | `commands.rs:622-635`；`model.rs:10-16`；`config.rs:690-750` |
| 密码进系统凭据库，键名 `<id>:ssh-password`；写后回读校验；状态分 stored / session / missing / unreadable 四种并在 UI 显示 | `model.rs:5`、`model.rs:55-71`；`commands.rs:637-651`；`registry.rs:78-263` |
| 编辑时密码留空沿用旧值；单独的「改密码」流程探测成功才保存 | `commands.rs:453-475`、`commands.rs:151-209`；`RemoteTargetsSettingsControls.tsx:391-397` |
| 新建 / 更新 / 删除要么全部成功，要么回滚（settings 快照、凭据、缓存池），并有故障注入测试 | `persist.rs:1-331`；`commands.rs:287-341`；`tests.rs:1649-1835` |
| 配置隔离：白名单字段校验，坏域整域清空；只记录字节数和 SHA-256，不保留原文；回读时再校验以防泄密 | `config.rs:59-147`、`config.rs:231-331`；`RemoteTargetsSettingsSection.tsx:157-204` |
| 按命令类别设截止时间和输出上限，超时 / 取消时杀整个进程树（Job Object） | `runner.rs:32-92`、`runner.rs:240-406` |
| 可注入的 `CommandRunner` / 探测函数参数，测试只断言参数形状，不依赖真机 | `runner.rs:235-238`；`commands.rs:68-73`；`tests.rs:1343-1585` |
| 「测试」按 id 发起时用已存或会话凭据，探测完即丢弃，不落盘 | `commands.rs:88-100`；`tests.rs:1893-1945` |
| 认证失败按认证方式给出可操作提示，并附原始 stderr | `exec.rs:269-312` |
| 设置页一个分页一个路由；侧栏导航在窄屏变横条；旧 `?section=` 链接兼容；可按区块锚点滚动 | `src/components/settings/settingsPages.ts:30-158`；`SettingsSideNav.tsx:19-49`；`SettingsView.tsx:381-398` |
| 主机列表行内展开编辑，删除二次确认，进行中状态按 id 区分 | `RemoteTargetsSettingsControls.tsx:72-248`；`targetStore.ts:32-74` |
| 切换器的空状态提示、错误 `role=alert` 加 toast 双通道、切换时全部项禁用 | `TargetQuickSwitcher.tsx:115-180` |
| 批量脚本协议：一次 SSH 跑完多条探测或读取，用控制字符分隔 | `plans/ssh-perf/02-phase2-batching.md:55-113`；`plans/ssh-perf/decisions.md:45-65` |
| russh 选型理由与 ADR-006「不保留 ssh.exe 兜底」；阶段 3 计划中的连接池、channel 上限、断线后 invalidate | `plans/ssh-perf/decisions.md:3-41`、`decisions.md:122-145`；`plans/ssh-perf/03-phase3-russh.md:62-112`、`03-phase3-russh.md:186-206` |
| ADR-004 自管 known_hosts 加不匹配时的结构化错误与指纹对话框（计划，未实现） | `decisions.md:84-103`；`plans/ssh-perf/risks.md:28-35` |

### 不适用

| 条目 | 出处 |
| --- | --- |
| 认证只有 key / password 两种，没有 agent 模式；私钥只认路径 | `model.rs:47-53`；`commands.rs:397-400` |
| askpass 方案（自身 exe 或临时 sh 脚本，密码放子进程环境变量）只服务于 ssh.exe 子进程，换成 russh 后会整体删除 | `askpass.rs:37-190`；`exec.rs:52-71`、`exec.rs:246-255`；`03-phase3-russh.md:143-147` |
| WSL 也作为同类目标出现在切换器和设置区 | `model.rs:96-105`；`RemoteTargetsSettingsSection.tsx:261-278`；`TargetQuickSwitcher.tsx:41-52` |
| 远程技能扫描、symlink 安装、本机→远程技能同步、中央库门闩，都与技能管理绑定 | `exec.rs:106-116`；`RemoteTargetsSettingsControls.tsx:157-163`；`docs/guide/ssh-remote.md:21-31` |
| 远端探测会 `mkdir -p ~/.skillsmanage/skills`（应用自己的目录约定） | `exec.rs:73-80` |
| Windows DPAPI 密文写进目标 JSON 作为凭据库失败时的兜底（非 Windows 只能落到会话） | `cred.rs:60-200`；`registry.rs:217-238` |
| 远程目标配置以 JSON 字符串存进应用 SQLite `settings` 表 | `persist.rs:370-388` |
| 设置页的分页划分（外观 / 连接与同步 / 平台管理 / 集成与密钥 / 技能来源 / 关于）对应的是 SkillPort 自己的功能 | `settingsPages.ts:30-79` |

### 已知坑

| 条目 | 出处 |
| --- | --- |
| 每条命令拉起一次 `ssh.exe`：每次扫描 100-200 次握手，端到端 60-200 s；阶段 2 的批量化也只是缓解 | `plans/ssh-perf/decisions.md:11`；`plans/ssh-perf/00-overview.md:9-17` |
| russh 方案已经「采纳」半年仍未落地，后来降级为需要单独立项的条件性阶段（涉及认证、passphrase、known_hosts、算法兼容、取消、打包、测试接口） | `.trellis/tasks/archive/2026-07/07-11-remote-update-performance/design.md:163-175` |
| 计划里写的 russh 版本 `0.45` 注明「须查最新版」，不能直接照抄 | `03-phase3-russh.md:33-41` |
| 只设 `accept-new`，未指定 `UserKnownHostsFile`，会写入并依赖用户系统的 `~/.ssh/known_hosts`；自管方案没有实现 | `exec.rs:223-224`；`decisions.md:84-103` |
| 错误的 IP 仍被标成 Active 并显示缓存库：`set_active_target` 只写 id，不联网；后来靠前端探测和门闩补救 | `11-03-ssh-switch-reachability-flag/prd.md`（Confirmed Facts）；`commands.rs:343-368` |
| 按 id 测试曾经没挂会话密码，导致测试失败而真实连接能成功；现已修复并有测试 | `11-03…/prd.md`（Confirmed Facts）；`commands.rs:92-94`；`tests.rs:1893` |
| 中央库门闩只覆盖中央技能页，Dashboard、Marketplace 等页面在不可达时仍显示缓存 | `11-03…/prd.md`（Out of Scope） |
| 「管理目标…」跳到 `/settings`，落在外观页；失败旗标跳的是连接页，两个入口不一致 | `TargetQuickSwitcher.tsx:76-79`；`SshReachabilityStatus.tsx:7` |
| 多数单条命令不可取消（`ProcessCancellation::Never`），只能等截止时间 | `exec.rs:321-442`；`process_api.rs:101-146` |
| 远端写入是 `cat > path`，不是原子写；写到一半中断会留下半截文件 | `exec.rs:389-402` |
| Windows 下 askpass 临时脚本残留（已改为复用 exe 解决） | `plans/ssh-perf/01-phase1-stability.md:107-125`；`askpass.rs:54-63` |
| 不支持带 passphrase 的私钥；PuTTY `.ppk` 等格式在 russh 路线下也需要专门提示 | `error.rs:153-156`；`risks.md:16-26` |
| 白名单校验很严：配置里多出任何一个键就会清空整个域（推断：版本升级或降级引入新字段时可能误伤） | `config.rs:305-314`、`config.rs:76-85` |
| 架构文档与代码多处不一致（字段名、远程库位置、连接缓存、symlink） | 本报告 §6；`docs/architecture/ssh-mode.md`；`docs/guide/ssh-remote.md:24` |
| 切换目标后清空数据再重扫会闪空；计划改成先显示旧数据再刷新（stale-while-revalidate），但 SSH 中央库最终选择了 fail-closed，两者取舍相反 | `plans/ssh-perf/decisions.md:107-118`；`plans/ssh-perf/04-phase4-ux.md:37-55`；`11-03…/prd.md`（Decision） |
| 扫描进行中切换目标会让操作日志的目标标签错位；计划是扫描期间禁止切换 | `plans/ssh-perf/risks.md:121-126` |
