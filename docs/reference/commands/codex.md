# `codex` - Codex 运行时与多账号管理

`ccr codex` 是 Codex CLI 的专项命令组，涵盖官方 OAuth 多账号管理、基于 SQLite 目录库的 Profile 路由、配额查询、会话垃圾箱、进程与运行时漂移修复以及历史命名空间桥接：

- `ccr codex auth ...`：官方 OAuth 多账号快照、同步、修复、重登录与导入导出
- `ccr codex profile ...`：Codex Profile 目录与运行时路由管理（完整 13 个公开动作）
- `ccr codex env [name]` / `quota` / `sessions`：环境变量导出脚本、配额查询与会话垃圾箱
- `ccr codex fix`：残留 `app-server` 进程清理与本地 Profile/运行时一致性诊断
- `ccr codex sync-history ...`：修复 `openai` / `custom` 命名空间切换后的历史可见性

## 入口

```bash
ccr codex
ccr codex help
ccr help codex auth
ccr help codex profile
```

不带子命令时，支持 TUI 的构建直接进入 Codex TUI 界面。

## Official Auth（`auth`）

| 命令 | 说明 |
|---|---|
| `ccr codex auth save <name> [-d <text>] [-f\|--force]` | 将当前 `~/.codex/auth.json` 保存为命名账号快照 |
| `ccr codex auth update <name> [-d <text> \| --clear-description] [--json]` | 更新或清空账号描述（不覆盖凭据快照） |
| `ccr codex auth list` | 列出所有已保存账号及当前登录状态 |
| `ccr codex auth sync` | 将当前运行期 OAuth tokens 回写至匹配的已保存账号 |
| `ccr codex auth repair <name>` | 从 `~/.codex/auth.json` 与备份中扫描修复过期 OAuth 快照 |
| `ccr codex auth switch <name> [--restart-daemon]` | 切换到指定官方账号；可选直接重启 `app-server` |
| `ccr codex auth delete <name> [-f\|--force]` | 删除已保存账号快照（不影响当前登录态） |
| `ccr codex auth rename <old> <new> [-f\|--force] [--json]` | 原子重命名账号文件、注册表与用量归因 |
| `ccr codex auth current [--json]` | 显示当前 `~/.codex/auth.json` 登录信息 |
| `ccr codex auth off [--json]` | 登出当前官方运行时登录 |
| `ccr codex auth relogin` | 本地移除 `auth.json` 并停止守护进程后运行 `codex login` |
| `ccr codex auth export [--no-secrets]` | 导出已保存官方账号至 JSON 文件 |
| `ccr codex auth import [--replace] [-f\|--force]` | 从 JSON 文件导入已保存官方账号 |

`ccr codex auth off` 与 `profile off` 相互独立，不修改 Profile 指针或 `config.toml` 路由。即使处于第三方 Profile 模式，它仍会清除文件存储下的运行期 `auth.json` 或对 keyring / auto 调用 `codex logout`。`--json` 可报告仍存在的 `profile_pointer`（属于提示而非失败，再次运行 `profile switch` 即可写回密钥）。

`ccr codex auth relogin` 用于换号或新增官方账号：由于 `codex login`、`codex logout` 与 Codex TUI `/logout` 均会在服务端吊销当前 refresh token，`relogin` 先删除本地 `auth.json`（不调用 `codex logout`）并停止 `app-server` 守护进程，再启动 `codex login`，使已保存同账号快照免受吊销。仅支持 `cli_auth_credentials_store = "file"`（keyring / auto 拒绝执行）；守护进程停止失败时立即中止，登录完成后用 `ccr codex auth save <name>` 保存。

## Profile Runtime（13 个公开动作）

权威存储位于 `<CCR_ROOT>/platforms/codex/profiles.db`（默认 `~/.ccr/platforms/codex/profiles.db`），内联密钥配对保存在平台密钥存储中并在导出/编辑时受控拼装。

| 命令 | 说明 |
|---|---|
| `ccr codex profile init [--json]` | 从内置非激活模板初始化 `profiles.db` 并在 `config.toml` 注册平台 |
| `ccr codex profile open [--json]` | 将 `profiles.db` 渲染为 `0600` 临时 TOML 投影并用 `$VISUAL` / `$EDITOR` 编辑导回 |
| `ccr codex profile current [--json]` | 查看当前 Codex Profile 与运行时状态 |
| `ccr codex profile list [--json]` | 列出全部 Codex Profiles（`--json` 含顶层 `revision`） |
| `ccr codex profile switch <name> [--json]` | 激活并将指定 Profile 应用至 Codex 运行时 |
| `ccr codex profile create <name> [...] [--json]` | 创建新的非激活 Profile（不改变 `current_config`） |
| `ccr codex profile edit <name> [...] [--json]` | 原子更新多个字段、重命名、设置推理强度或轮换/清除密钥 |
| `ccr codex profile enable <name> [--json]` | 校验并在 `profiles.db` 中原子启用指定 Profile |
| `ccr codex profile disable <name> [--force] [--json]` | 禁用指定 Profile；当前激活项需显式 `--force` |
| `ccr codex profile delete <name> [--force] [--json]` | 删除指定 Profile；当前激活项需 `--force`（先执行 `off` 清理运行时） |
| `ccr codex profile off [--json]` | 退出 Profile 模式，清理 Codex Profile 路由与运行期 `auth.json` |
| `ccr codex profile export [NAME]... [...] [--json]` | 纯只读导出 `ExchangeV1` 可交换 TOML |
| `ccr codex profile import [<FILE>\|-] [...] [--json]` | 合并或整体替换导入交换文件或历史 TOML |

### 初始化与编辑器投影（`init` / `open`）

- **`init [--json]`**：从内置非激活模板（`current_config = ""`）创建 `~/.ccr/platforms/codex/profiles.db` 并在 `~/.ccr/config.toml` 注册 `codex`，不生成驻留 `profiles.toml`，也不写入 `~/.codex/`。`--json` 输出 `{ ok, platform, profiles_file, created, registered }`。
- **`open [--json]`**：将 `profiles.db` 渲染为平台目录外的 `0600` 系统临时 TOML 文件，通过 `$VISUAL` 或 `$EDITOR` 打开，正常退出且有改动时校验并导回 `profiles.db`，随后删除临时文件。`--json` 输出 `{ ok, platform, profiles_file, created, registered, editor }`。

### 查询、切换与生命周期（`current` / `list` / `switch` / `enable` / `disable` / `delete` / `off`）

- **`current [--json]`**：显示 Codex 运行时卡片；`--json` 输出结构化运行态摘要（与 `ccr current --json` 的 `codex` 节点一致）。
- **`list [--json]`**：输出 `{ "current_profile": "..." | null, "revision": "..." | null, "profiles": [...] }`，顶层 `revision` 用于乐观并发控制（`--expected-revision`）。
- **`switch <name> [--json]`**：原子切换并应用 Codex 运行时路由与凭据；`--json` 输出 `{ ok, platform, profile, previous_profile, status, revision, warnings, recovery_paths? }`。
- **`enable <name> [--json]`** / **`disable <name> [--force] [--json]`** / **`delete <name> [--force] [--json]`**：原子管理启用标记与条目删除；`disable` 与 `delete` 默认保护当前激活项，传入 `--force` 时允许强制禁用或先执行 `off` 退出运行时后删除。
- **`off [--json]`**：退出 Codex Profile 模式，清理 `~/.codex/config.toml` 中的 CCR Profile 路由及 CCR 写入的运行期 `auth.json`。`--json` 输出 `{ ok, changed, previous_profile, runtime_mode, runtime_dirs, removed_auth_json }`。

### 创建与编辑（`create` / `edit`）

密钥统一从标准输入读取（`--auth-token-stdin`），不在命令行参数中传递明文密钥：

```bash
printf '%s' "$OPENAI_API_KEY" | ccr codex profile create relay \
  --base-url https://api.example.com/v1 \
  --model gpt-5.4 \
  --auth-mode openai_api_key \
  --set model_reasoning_effort=high \
  --tag work \
  --auth-token-stdin

ccr codex profile edit relay \
  --description "Work relay" \
  --model gpt-5.4 \
  --base-url https://api.example.com/v1 \
  --reasoning-effort high

printf '%s' "$OPENAI_API_KEY" | ccr codex profile edit relay --auth-token-stdin
ccr codex profile switch relay --json
```

- **`create` 支持参数**：`<name>`、`--description <TEXT>`、`--base-url <URL>`、`--auth-token-stdin`、`--model <MODEL>`、`--small-fast-model <MODEL>`、`--auth-mode <MODE>`、`--provider <NAME>`、`--provider-type <TYPE>`、`--account <ACCOUNT>`、可重复 `--tag <TAG>`、`--disabled`、可重复 `--set FIELD=VALUE`（如 `model_reasoning_effort=high`、`wire_api=responses`）、`--expected-revision <REV>`、`--json`。新建项保持未激活。
- **`edit` 支持参数**：`<name>`、`--rename <NEW_NAME>`、`--description <TEXT>`、`--model <MODEL>`、`--base-url <URL>`、`--provider <NAME>`、可重复 `--tag <TAG>`、互斥 `--enable` / `--disable`、`--reasoning-effort <LEVEL>`（写入 `model_reasoning_effort`）、可重复 `--set FIELD=VALUE`、可重复 `--clear FIELD`、互斥 `--auth-token-stdin` / `--clear-auth-token`、`--expected-revision <REV>`、`--no-apply`、`--json`。编辑当前激活项默认重放运行时（`--no-apply` 仅改存储），`--json` 不包含任何密钥。

### 导入与导出（`export` / `import`）

```bash
ccr codex profile export --no-secrets -o codex-team.toml
ccr codex profile import codex-team.toml --dry-run
ccr codex profile import codex-team.toml --on-conflict skip --dedup --json
```

- **`export [NAME]... [-t|--tag <TAG>] [-o|--output <FILE>] [--stdout] [--no-secrets] [-y|--yes] [--json]`**：纯只读内存中叠加 `profile_secrets.json` 并输出 `ExchangeV1`（`schema_version = 1`、`platform = "codex"`、`secrets_mode = "included" | "redacted"`、`[profiles.<name>]`）。默认文件名 `./ccr-codex-profiles-<YYYYMMDD-HHMMSS>.toml`（`0600` 权限）；`--stdout` 与 `-o`、`--json` 互斥；`--json` 写盘并输出元数据回执（非交互或 `--json` 下覆盖已有 `-o` 文件且未传 `-y/--yes` 时退出码为 `64`）。
- **`import [<FILE>|-] [--stdin] [--on-conflict skip|overwrite|rename] [--dedup] [--replace] [-y|--yes] [--dry-run] [--no-apply] [--expected-revision <REV>] [--json]`**：支持 `ExchangeV1`（校验 `platform = "codex"`）、`LegacyCatalog` 与 `LegacySimplified` 三种输入；默认为非破坏性合并（`--on-conflict skip`），`--replace` 与 `--on-conflict`、`--dedup` 互斥且需确认或 `-y/--yes`。缺密钥新增项设为 `enabled = false`（`missing_secrets`），兼容覆盖保留本地既有密钥（`preserved_local_secrets`），原子同步 `profiles.db` 与 `profile_secrets.json` 并生成 `0600` 预写备份；`--json` 返回 6+1 桶 `counts`、`missing_secrets`、`preserved_local_secrets`、`reapplied_profile`、`backup_path`、`revision` 与 `warnings`。

## 环境变量、配额与会话垃圾箱（`env` / `quota` / `sessions`）

| 命令 | 说明 |
|---|---|
| `ccr codex env [<name>]` | 输出当前或指定 Profile 的环境变量导出脚本 |
| `ccr codex quota [-a\|--account <name>] [--refresh] [--json]` | 查询所有或指定已保存账号的配额余量（5h 窗口 / 周限额） |
| `ccr codex sessions trash <session_id>... [--codex-home <dir>]` | 将指定会话移入 CCR 管理的垃圾箱 |
| `ccr codex sessions trash-list [--codex-home <dir>]` | 列出垃圾箱内可恢复的会话 |
| `ccr codex sessions restore <session_id>... [--codex-home <dir>]` | 从垃圾箱恢复指定会话 |

## `fix`

```bash
# 先切换目标 profile，再诊断；裸命令不写 runtime，也不调用上游 doctor
ccr codex profile switch future
ccr codex fix

# 显式修复可安全处理的本地漂移
ccr codex fix --repair-runtime

# 仅预览进程清理与 runtime 重放，不发送信号、不写文件
ccr codex fix --dry-run --repair-runtime

# 需要上游健康检查时再运行 codex doctor
ccr codex fix --doctor
```

进程阶段只匹配当前用户的原生 Codex / Node Codex wrapper `app-server`，不会误伤 `codex exec`、`codex resume`、`codex login` 或仅把 `codex app-server` 当作参数的其他工具。清理先发送 TERM，每 300ms 重查（最长约 3 秒），截止仍匹配的进程发送 KILL；只要发过信号，再等待约 1 秒做最终快照，宽限后新出现的身份记入 `respawned`（退出码 `2`）。每次发信号前重新核验 owner、PID 启动时间和 argv；无法安全读取时报告 `process_state = unavailable` 并停止发信号。

诊断区分 profile pointer、route、credential 与 Provider 有效性，仅比对本地保存密钥与实际凭据源，绝不打印密钥、掩码片段、长度或指纹。默认不调用上游 `codex doctor`（`provider_auth_validity = not_checked` 不代表失败）；`--repair-runtime` 不隐含 `--doctor`。

| 退出码 | 含义 |
|---|---|
| `0` | 未发现确定的本地漂移；Provider 有效性仍可能未检查 |
| `1` | CCR runtime inspection/repair 阶段失败 |
| `2` | `app-server` 仍存在，或无法安全完成进程发现/清理 |
| `3` | 本地 profile/runtime 漂移仍存在，或 doctor 期间快照发生变化 |
| `127` | 传入 `--doctor` 且 `codex` 不在 `PATH` 中 |

## `sync-history`

修复 `openai` / `custom` provider 命名空间切换后旧会话在 Codex CLI / App 中不可见的问题，并提供 `status`、`restore <backup_dir> [--restore-state]`、`prune-backups [--keep <N>]` 子命令。

```bash
# 显式写入某个 provider，默认只处理最近 7 天（可用 --max-age-days <N> 调整）
ccr codex sync-history --provider custom --dry-run
ccr codex sync-history --provider openai

# bridge 模式：把 openai/custom/缺失 provider 历史桥接到当前 runtime provider
ccr codex sync-history --bridge official-custom --dry-run
ccr codex sync-history --bridge official-custom --all-history

# 诊断 provider、SQLite、preview、cwd、Desktop 首屏限制与 encrypted_content
ccr codex sync-history status
```

- `--provider` 与 `--bridge` 互斥；`--bridge official-custom` 按当前运行时解析目标（官方/隐式 OpenAI 为 `openai`，第三方 Profile 为 `custom`）。
- `--all-history` 取消 7 天过滤；SQLite 修复默认仅触碰 `openai`、`custom` 与缺失 provider 行，额外命名空间通过可重复 `--include-provider <name>` 指定。
- 写入前自动备份 rollout 首行、`state_5.sqlite` 与 `.codex-global-state.json`（可用 `--keep <N>` 限制备份保留份数）；`--dry-run` 只输出计划；`encrypted_content` 仅统计告警，不解密或改写正文。

## 相关页面

- [配置模型](/guide/configuration)
- [CLI 工作流](/guide/cli-workflows)
- [`current`](./current)
- [`validate`](./validate)
- [`doctor`](./doctor)
- [`platform`](./platform)
