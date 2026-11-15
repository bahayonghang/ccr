# `grok` - Grok Build Auth 与 Profile Runtime

`ccr grok` 管理 Grok Build 的官方 OAuth 会话快照（`auth`）与基于 SQLite 目录库的模型/第三方 Provider Profile（`profile`）。Profile 操作受控管理 `~/.grok/config.toml`（或 `$GROK_HOME/config.toml`）中的 `[model.custom]`、`[models].default` 和 `[models].default_reasoning_effort`；账号服务受控读取 `$GROK_HOME/auth.json`（默认 `~/.grok/auth.json`）。所有终端与 JSON 输出均不暴露密钥，且 CCR 不读取、写入、备份或校验 `mcp_credentials.json`。

## 入口

```bash
ccr grok help
ccr grok auth
ccr help grok auth
ccr help grok profile
```

## Official Auth（`auth`）

| 命令 | 说明 |
|---|---|
| `ccr grok auth` | 有 TUI 启动器时进入 Grok Auth 标签；否则打印帮助 |
| `ccr grok auth save <name> [--scope <scope>] [-f\|--force] [--json]` | 保存当前官方 OAuth 来源至账号库 |
| `ccr grok auth list [--json]` | 列出已保存账号与本地可用 OAuth 来源 |
| `ccr grok auth switch <name> [--json]` | 恢复已保存账号供新 Grok 会话使用 |
| `ccr grok auth delete <name> [-f\|--force] [--json]` | 删除已保存账号快照（不登出运行期会话；支持全局 `-y`） |
| `ccr grok auth current [--json]` | 报告官方会话文件是否存在（不输出 token） |
| `ccr grok auth off [--json]` | 登出当前官方运行时登录 |

`auth off` 与 `profile off` 相互独立，不修改 Profile 指针或 `[model.custom]`。登出会删除整个 `auth.json`（含其他 scope 与 API-key 缓存），但在删除前会先将可唯一识别的已保存账号回存最新凭据，CCR 账号库条目保持保留。

Grok Auth TUI 快捷键（保存在 `<CCR_ROOT>/platforms/grok/auth/accounts.json`）：
- `s`：按别名保存所选当前 OAuth scope 副本（多个来源时先选 scope，同名覆盖需确认）。
- `Enter`：确认切换账号（请先结束当前 Grok 会话；仅替换目标 scope 并先回存可识别原账号凭据）。
- `d`：确认删除已保存条目（运行期凭据不变）。
- `o`：确认登出整个运行时并保留已保存条目；`r` 重新读取本地状态。

单个合法 OAuth 来源自动选中；存在多个来源时先用 `ccr grok auth list --json` 查看精确 `scope` 并通过 `--scope` 指定。重名覆盖必须显式传 `--force`（全局 `-y` 不代替覆盖授权）；缺少 Grok 原生格式所需的 `user_id` 时切换会拒绝写入。

```bash
ccr grok auth save gmail
ccr grok auth list --json
ccr grok auth save work --scope '<scope from list>'
ccr grok auth switch gmail
ccr grok auth delete work --force
```

## Profile Runtime（13 个公开动作）

权威存储位于 `<CCR_ROOT>/platforms/grok/profiles.db`（默认 `~/.ccr/platforms/grok/profiles.db`）。

| 命令 | 说明 |
|---|---|
| `ccr grok profile init [--json]` | 从内置非激活模板初始化 `profiles.db` 并在 `config.toml` 注册平台 |
| `ccr grok profile open [--json]` | 将 `profiles.db` 渲染为 `0600` 临时 TOML 投影并用 `$VISUAL` / `$EDITOR` 编辑导回 |
| `ccr grok profile current [--json]` | 查看当前 Grok Profile 与运行时状态 |
| `ccr grok profile list [--json]` | 列出全部 Grok Profiles（`--json` 含顶层 `revision`） |
| `ccr grok profile switch <name> [--json]` | 激活并将指定 Profile 应用至 `~/.grok/config.toml` |
| `ccr grok profile create <name> [...] [--json]` | 创建新的非激活 Profile（不改变 `current_config`） |
| `ccr grok profile edit <name> [...] [--json]` | 原子更新多个字段、重命名或轮换/清除凭据 |
| `ccr grok profile enable <name> [--json]` | 校验并在 `profiles.db` 中原子启用指定 Profile |
| `ccr grok profile disable <name> [--force] [--json]` | 禁用指定 Profile；当前激活项需显式 `--force` |
| `ccr grok profile delete <name> [--force] [--json]` | 删除指定 Profile；当前激活项需 `--force`（先执行 `off` 恢复运行时） |
| `ccr grok profile off [--json]` | 退出 Profile 模式，移除 `[model.custom]` 与 `[models].default` 并恢复原默认推理强度 |
| `ccr grok profile export [NAME]... [...] [--json]` | 纯只读导出 `ExchangeV1` 可交换 TOML |
| `ccr grok profile import [<FILE>\|-] [...] [--json]` | 合并或整体替换导入交换文件或历史 TOML |

### 初始化与编辑器投影（`init` / `open`）

- **`init [--json]`**：从内置非激活模板（`current_config = ""`）初始化 `~/.ccr/platforms/grok/profiles.db` 并在 `~/.ccr/config.toml` 注册 `grok`，不创建驻留 `profiles.toml`，也不改动 `~/.grok/config.toml`。`--json` 输出 `{ ok, platform, profiles_file, created, registered }`。
- **`open [--json]`**：将 `profiles.db` 渲染为平台目录外的 `0600` 临时 TOML 文件，通过 `$VISUAL` 或 `$EDITOR` 编辑并在正常退出后校验导回 `profiles.db`，随后立即清理临时文件。`--json` 输出 `{ ok, platform, profiles_file, created, registered, editor }`。

### 查询、切换与生命周期（`current` / `list` / `switch` / `enable` / `disable` / `delete` / `off`）

- **`current [--json]`**：显示当前生效的 Grok Profile；`--json` 输出 `{ platform: "grok", profile, runtime_mode: "profile" | "grok_native", details? }`。
- **`list [--json]`**：输出 `{ "current_profile": "..." | null, "revision": "..." | null, "profiles": [...] }`，顶层 `revision` 可用于 `--expected-revision`。
- **`switch <name> [--json]`**：原子切换并应用 Profile；`--json` 输出 `{ ok, platform, profile, previous_profile, status, revision, warnings, recovery_paths? }`。
- **`enable <name> [--json]`** / **`disable <name> [--force] [--json]`** / **`delete <name> [--force] [--json]`**：原子管理启用状态与删除；`disable` 与 `delete` 默认保护当前激活项，`delete --force` 先执行 `off` 恢复进入 Profile 模式前的运行时配置再删除条目。
- **`off [--json]`**：退出 Grok Profile 模式，移除 `~/.grok/config.toml` 中的 `[model.custom]` 与 `[models].default` 并恢复进入 Profile 模式前捕获的默认推理强度。`--json` 输出 `{ ok, changed, previous_profile, runtime_mode }`。

### 创建与编辑（`create` / `edit`）

官方模型选择器（不接管认证）：

```bash
ccr grok profile create official --model grok-4
```

第三方 Provider Profile（内联密钥通过 `--api-key-stdin` 或可见别名 `--auth-token-stdin` 从标准输入传入，或使用互斥的 `--env-key <ENV_VAR>` 指定环境变量名）：

```bash
printf '%s' "$GROK_API_KEY" | ccr grok profile create relay \
  --base-url https://api.example.com/v1 \
  --model grok-4 \
  --api-backend responses \
  --context-window 1000000 \
  --reasoning-effort high \
  --supports-backend-search \
  --api-key-stdin

# 或改用环境变量名管理凭据（与 --api-key-stdin 互斥）
ccr grok profile create relay-env \
  --base-url https://api.example.com/v1 \
  --model grok-4 \
  --env-key GROK_RELAY_KEY

ccr grok profile switch relay --json
ccr grok profile current --json
```

- **`create` 支持参数**：`<name>`、`--description <TEXT>`、`--base-url <URL>`、互斥的 `--api-key-stdin`（可见别名 `--auth-token-stdin`）与 `--env-key <ENV_VAR>`、`--model <MODEL>`、`--provider <NAME>`、`--provider-type <TYPE>`、`--account <ACCOUNT>`、可重复 `--tag <TAG>`、`--api-backend <chat_completions|responses|messages>`、`--context-window <TOKENS>`、`--supports-backend-search [true|false]`、`--reasoning-effort <none|minimal|low|medium|high|xhigh|max>`、可重复 `--set FIELD=VALUE`（环境变量名必须用 `--env-key` 指定）、`--expected-revision <REV>`、`--disabled`、`--json`。新建条目保持未激活。
- **`edit` 支持参数**：`<name>`、`--rename <NEW_NAME>`、`--description <TEXT>`、`--model <MODEL>`、`--base-url <URL>`、`--provider <NAME>`、可重复 `--tag <TAG>`、互斥 `--enable` / `--disable`、互斥凭据参数 `--api-key-stdin` / `--auth-token-stdin` / `--env-key <ENV_VAR>` / `--clear-auth-token`（写入内联 `api_key` 会自动移除 `env_key`，写入 `env_key` 会自动移除 `api_key`）、`--api-backend <BACKEND>`、`--context-window <TOKENS>`、`--supports-backend-search [true|false]`、`--reasoning-effort <LEVEL>`、可重复 `--set FIELD=VALUE`、可重复 `--clear FIELD`、`--expected-revision <REV>`、`--no-apply`、`--json`。

```bash
ccr grok profile edit relay \
  --description "Relay" \
  --model grok-4 \
  --base-url https://api.example.com/v1 \
  --reasoning-effort high

printf '%s' "$GROK_API_KEY" | ccr grok profile edit relay --api-key-stdin
ccr grok profile edit relay --env-key GROK_RELAY_KEY
```

第三方 Profile 会将推理强度写入 `[model.custom].reasoning_effort`、派生 `[model.custom].supports_reasoning_effort = true`，并同步 `[models].default_reasoning_effort`；官方 Profile 仅设置全局默认值。编辑当前激活项默认自动重放运行时（`--no-apply` 仅改存储），把当前激活项设为禁用会被拒绝。

### 导入与导出（`export` / `import`）

```bash
ccr grok profile export --no-secrets -o grok-team.toml
ccr grok profile import grok-team.toml --dry-run
ccr grok profile import grok-team.toml --on-conflict skip --dedup --json
```

- **`export [NAME]... [-t|--tag <TAG>] [-o|--output <FILE>] [--stdout] [--no-secrets] [-y|--yes] [--json]`**：纯只读输出 `ExchangeV1`（`schema_version = 1`、`platform = "grok"`、`secrets_mode = "included" | "redacted"`、`[profiles.<name>]`）。默认写入 `./ccr-grok-profiles-<YYYYMMDD-HHMMSS>.toml`（`0600`）；`--stdout` 与 `-o`、`--json` 互斥；`--json` 写盘并打印回执（非交互或 `--json` 下覆盖已有 `-o` 文件且未传 `-y/--yes` 时退出码为 `64`）。
- **`import [<FILE>|-] [--stdin] [--on-conflict skip|overwrite|rename] [--dedup] [--replace] [-y|--yes] [--dry-run] [--no-apply] [--expected-revision <REV>] [--json]`**：支持 `ExchangeV1`（校验 `platform = "grok"`）、`LegacyCatalog` 与 `LegacySimplified` 三种输入；默认执行非破坏性合并（`--on-conflict skip`），`--replace` 需交互确认或 `-y/--yes` 且与 `--on-conflict`、`--dedup` 互斥。缺内联密钥的新增第三方条目自动设为 `enabled = false`（记入 `missing_secrets`，之后可用 `ccr grok profile edit <name> --api-key-stdin --enable` 补录启用），兼容覆盖保留本地既有密钥（`preserved_local_secrets`），写入前生成 `0600` 备份；`--json` 返回 6+1 桶 `counts`、明细列表、`backup_path`、`revision` 与 `warnings`。

## 凭据边界

- `api_key` 是 Grok Build 第三方端点的直接密钥字段：通过 `--api-key-stdin`（或 `--auth-token-stdin`）录入后会持久化在 CCR SQLite 目录、轮换备份以及激活时的 `~/.grok/config.toml` 中，但终端与 JSON 输出始终隐藏该值。
- `env_key` 仅填写环境变量名称（如 `GROK_RELAY_KEY`），严禁填入密钥明文。
- 官方 Profile 拒绝携带内联密钥或 `env_key`；Grok 自身的登录会话与 `XAI_API_KEY` 保持由 Grok 原生管理。
- Profile 命令不读写 `auth.json`；展示 URL 时自动剥离 userinfo、query 与 fragment。

## 相关页面

- [CCR Grok profiles 示例](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/grok-profiles.toml)
- [Grok config.toml 示例](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/grok-cli-config.toml)
- [`current`](./current)
- [`validate`](./validate)
- [`platform`](./platform)
