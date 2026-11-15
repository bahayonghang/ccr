# `claude` - Claude Auth 与 Profile Runtime

`ccr claude` 管理两条相互独立的 Claude Code 运行时路径：官方订阅账号快照（`auth`）与基于 SQLite 目录库的 Profile 路由（`profile`）。

## 入口

```bash
ccr claude
ccr claude help
ccr help claude auth
ccr help claude profile
```

不带子命令时，支持 TUI 的构建直接进入 Claude Auth 标签；无 TUI 启动器的构建回退打印已保存账号列表。

## Official Auth

| 命令 | 说明 |
|---|---|
| `ccr claude auth save <name> [-d <text>] [-f\|--force]` | 保存当前官方登录凭据快照 |
| `ccr claude auth list` | 列出已保存的官方订阅账号 |
| `ccr claude auth switch <name>` | 切换至指定已保存官方账号 |
| `ccr claude auth delete <name> [-f\|--force]` | 删除已保存账号快照；`--force` 跳过确认 |
| `ccr claude auth current [--json]` | 显示当前官方登录信息 |
| `ccr claude auth off [--json]` | 登出当前官方运行时会话 |

Auth 快照不等同于 API-key Profile。`auth off` 与 `profile off` 相互独立：它不删除 CCR 已保存的账号快照，不修改 Profile 指针，也不调用 `profile off`。Windows / Linux 删除 `.credentials.json`，macOS 调用官方 `claude auth logout`；已保存快照仍可用 `auth switch` 恢复。

## Profile Runtime（13 个公开动作）

权威存储位于 `<CCR_ROOT>/platforms/claude/profiles.db`（默认 `~/.ccr/platforms/claude/profiles.db`）。

| 命令 | 说明 |
|---|---|
| `ccr claude profile init [--json]` | 从内置非激活模板初始化 `profiles.db` 并在 `config.toml` 注册平台 |
| `ccr claude profile open [--json]` | 将 `profiles.db` 渲染为 `0600` 临时 TOML 投影并用 `$VISUAL` / `$EDITOR` 编辑导回 |
| `ccr claude profile current [--json]` | 查看当前 Claude Profile 与运行时状态 |
| `ccr claude profile list [--json]` | 列出全部 Claude Profiles（`--json` 含顶层 `revision`） |
| `ccr claude profile switch <name> [--json]` | 激活并应用指定 Profile 到运行时 |
| `ccr claude profile create <name> [...] [--json]` | 创建新的非激活 Profile（不改变 `current_config`） |
| `ccr claude profile edit <name> [...] [--json]` | 原子更新多个字段、重命名或轮换/清除密钥 |
| `ccr claude profile enable <name> [--json]` | 校验并在 `profiles.db` 中原子启用指定 Profile |
| `ccr claude profile disable <name> [--force] [--json]` | 禁用指定 Profile；当前激活项需显式 `--force` |
| `ccr claude profile delete <name> [--force] [--json]` | 删除指定 Profile；当前激活项需 `--force`（先执行 `off` 清理运行时） |
| `ccr claude profile off [--json]` | 退出 Profile 模式，清理 CCR 运行时覆盖并返回官方认证模式 |
| `ccr claude profile export [NAME]... [...] [--json]` | 纯只读导出 `ExchangeV1` 可交换 TOML |
| `ccr claude profile import [<FILE>\|-] [...] [--json]` | 合并或整体替换导入交换文件或历史 TOML |

### 初始化与编辑器投影（`init` / `open`）

- **`init [--json]`**：若 `profiles.db` 尚未建库，则从内置示例模板写入初始目录（`current_config = ""`，所有模板条目保持未激活），同时确保 `claude` 已登记在 `~/.ccr/config.toml` 中。不会创建驻留的 `profiles.toml`，也不会改动 `~/.claude/settings.json`。`--json` 输出 `{ ok, platform, profiles_file, created, registered }`。
- **`open [--json]`**：要求已设置 `$VISUAL` 或 `$EDITOR`（未设置时直接报错，不创建任何文件）。命令将 `profiles.db`（不存在时先从模板建库）渲染为规范 TOML，写入平台目录之外的系统临时文件（所有者独享 `0600` 权限）；编辑器以零状态码退出且内容有变更时，经领域校验与目录版本检查写回 `profiles.db`，随后立即删除临时文件。`--json` 输出 `{ ok, platform, profiles_file, created, registered, editor }`。

### 查询、切换与生命周期（`current` / `list` / `switch` / `enable` / `disable` / `delete` / `off`）

- **`current [--json]`**：展示 Claude 运行时卡片；`--json` 输出与 `ccr current --json` 中 `claude` 节点一致的结构化状态摘要（不含完整密钥）。
- **`list [--json]`**：终端输出状态、名称、Provider、模型、认证来源与启用列；`--json` 输出：
  ```json
  {
    "current_profile": "work",
    "revision": "sha256:...",
    "profiles": [
      {
        "name": "work",
        "is_current": true,
        "enabled": true,
        "provider": "Example",
        "model": "claude-sonnet-4-5",
        "auth_source": "api_key"
      }
    ]
  }
  ```
  顶层 `revision`（目录未初始化时为 `null`）可直接传入 `create` / `edit` / `import` 的 `--expected-revision <REV>` 做乐观并发校验。
- **`switch <name> [--json]`**：校验目标 Profile 并原子写入 `~/.claude/settings.json` 与当前指针。`--json` 输出单行回执 `{ ok, platform, profile, previous_profile, status, revision, warnings, recovery_paths? }`。
- **`enable <name> [--json]`**：在平台操作锁内校验配置完整性并将 `enabled` 设为 `true`（不自动切换当前激活项；若因 `--no-secrets` 导入缺少密钥则会拒绝启用并提示补录密钥）。`--json` 输出 `{ ok, platform, name, message, enabled, current_profile }`。
- **`disable <name> [--force] [--json]`**：将 `enabled` 设为 `false`；若目标正处于当前激活状态，默认拒绝，需显式附加 `--force`。
- **`delete <name> [--force] [--json]`**：从 `profiles.db` 移除指定条目；若目标正处于激活状态，默认拒绝，附加 `--force` 时会先执行 `profile off` 清理运行时再删除。
- **`off [--json]`**：退出 CCR 管理的 Profile 模式，清空 `current_profile` 指针并移除 `~/.claude/settings.json` 中由 CCR 写入的 `ANTHROPIC_*` 覆盖（写入前快照备份至 `~/.ccr/backups/profile-off/`，中途失败自动回滚）。`ccr claude profile off --json` 输出契约：
  ```json
  {
    "ok": true,
    "changed": true,
    "previous_profile": "work",
    "runtime_mode": "official_auth",
    "remaining_suppressors": [
      {
        "kind": "process_env",
        "location": "ANTHROPIC_API_KEY",
        "confidence": "high",
        "evidence": "present_non_empty",
        "ownership": "external"
      }
    ],
    "warnings": []
  }
  ```
  其中 `remaining_suppressors` 报告退出 Profile 后仍会压制 Claude 官方登录但属于外部管理（CCR 不会自动删除）的认证来源；`warnings` 为空时省略。

### 创建与编辑（`create` / `edit`）

创建第三方 API Profile（密钥仅通过标准输入传入，避免进入 Shell 历史与进程参数列表）：

```bash
printf '%s' "$ANTHROPIC_AUTH_TOKEN" | ccr claude profile create relay \
  --base-url https://api.example.com \
  --model claude-sonnet-4-5 \
  --small-fast-model claude-haiku-4-5 \
  --auth-mode api_key \
  --provider Example \
  --tag work \
  --auth-token-stdin

ccr claude profile switch relay --json
ccr claude profile current --json
```

- **`create` 支持参数**：`<name>`、`--description <TEXT>`、`--base-url <URL>`、`--auth-token-stdin`、`--model <MODEL>`、`--small-fast-model <MODEL>`、`--auth-mode <subscription|api_key>`、`--provider <NAME>`、`--provider-type <TYPE>`、`--account <ACCOUNT>`、可重复 `--tag <TAG>`、`--disabled`、可重复 `--set FIELD=VALUE`（拒绝内联密钥字段）、`--expected-revision <REV>`、`--json`。新建条目不会修改 `current_config`；`--json` 输出 `{ ok, platform, name, message, enabled, current_profile, revision, set_fields }`。
- **`edit` 支持参数**：`<name>`、`--rename <NEW_NAME>`、`--description <TEXT>`、`--model <MODEL>`、`--base-url <URL>`、`--provider <NAME>`、可重复 `--tag <TAG>`（整体替换标签列表）、互斥的 `--enable` / `--disable`（禁止将当前激活 Profile 设为禁用）、可重复 `--set FIELD=VALUE`、可重复 `--clear FIELD`、互斥的 `--auth-token-stdin` / `--clear-auth-token`、`--expected-revision <REV>`、`--no-apply`、`--json`。

```bash
ccr claude profile edit relay \
  --description "Relay primary" \
  --set small_fast_model=claude-haiku-4-5 \
  --rename relay-2026

printf '%s' "$ANTHROPIC_AUTH_TOKEN" | ccr claude profile edit relay-2026 --auth-token-stdin
ccr claude profile edit relay-2026 --no-apply --description "仅更新存储"
```

编辑当前激活的 Profile 默认会自动重新应用运行时；传入 `--no-apply` 时仅更新 `profiles.db`。`--json` 仅输出变更字段名、新旧名称、revision 与是否重放运行时，绝不输出密钥。

### 导入与导出（`export` / `import`）

```bash
# 只读导出脱敏模板（可安全分享/入库）或含密钥私有备份（-o 自动按 0600 权限落盘）
ccr claude profile export --no-secrets -o claude-team.toml
ccr claude profile export relay-2026 -o relay-backup.toml
ccr claude profile export relay-2026 --stdout | ssh dev-box ccr claude profile import -

# 先预览再增量合并导入（默认 --on-conflict skip）
ccr claude profile import claude-team.toml --dry-run
ccr claude profile import claude-team.toml --on-conflict overwrite --dedup --json

# 破坏性整体替换（非交互或 --json 必须显式附加 -y/--yes）
ccr -y claude profile import relay-backup.toml --replace --json
```

- **`export [NAME]... [-t|--tag <TAG>] [-o|--output <FILE>] [--stdout] [--no-secrets] [-y|--yes] [--json]`**：
  - 纯只读导出 `ExchangeV1` 格式（`schema_version = 1`、`platform = "claude"`、`secrets_mode = "included" | "redacted"`、`[profiles.<name>]`），不对 `profiles.db` 产生锁或迁移副作用。
  - `[NAME]...`（任一不存在即报错）与可重复 `-t, --tag <TAG>` 互斥；默认包含内联密钥（仅限本机私有备份），未传 `-o` 时默认写入 `./ccr-claude-profiles-<YYYYMMDD-HHMMSS>.toml`（含密钥时自动收紧为 `0600`，拒绝符号链接/重解析点）。
  - `--no-secrets` 彻底剥离内联密钥、可疑扩展敏感字段与 URL 内嵌凭据，同时保留显式 `auth_mode`；分享或提交版本库前必须附加 `--no-secrets`。
  - `--stdout` 与 `-o`、`--json` 互斥（含密钥时请仅用于受信管道，切勿直接 `> file.toml` 重定向以免绕过 `0600` 权限保护）；`--json` 仍落盘写文件并向标准输出打印 `{ ok, platform, output_path, schema_version, secrets_mode, contains_secrets, exported_count, exported_names, warnings }`（目标文件已存在且处于非交互终端或 `--json` 模式但未传 `-y/--yes` 时，返回退出码 `64`）。
- **`import [<FILE>|-] [--stdin] [--on-conflict skip|overwrite|rename] [--dedup] [--replace] [-y|--yes] [--dry-run] [--no-apply] [--expected-revision <REV>] [--json]`**：
  - 兼容三种输入格式：`ExchangeV1`（严格校验 `platform = "claude"`）、`LegacyCatalog`（含 `default_config` / `current_config`）与 `LegacySimplified`（纯节表），自动拦截含脱敏占位符的伪密钥。
  - 默认采用非破坏性**合并模式**（`--on-conflict skip`；可选 `overwrite` 整条更新并保留 `usage_count`，或 `rename` 改名为 `<name>-imported` 入库；`--dedup` 跳过归一化配置与本地已有异名项完全一致的候选）。
  - `--replace` 执行整体替换，与 `--on-conflict`、`--dedup` 互斥，需交互确认或全局 `-y/--yes`（`--stdin`、`--json` 或非 TTY 下必填），并保护当前激活 Profile 不被删除或禁用。
  - 缺少内联密钥的新增项自动以 `enabled = false` 入库并记入 `missing_secrets`；兼容的覆盖更新自动保留本地既有密钥（`preserved_local_secrets`）；正式写入前自动生成 `0600` 备份；命中当前激活项时默认重放运行时（`--no-apply` 可跳过）。
  - `--json` 返回结构化报告，包含 6+1 桶统计 `counts`（`total_input`、`total_incoming`、`added`、`updated`、`unchanged`、`renamed`、`skipped_conflict`、`skipped_duplicate`、`skipped`、`removed`、`disabled_missing_secret`）、各桶名称明细、`missing_secrets`、`preserved_local_secrets`、`reapplied_profile`、`backup_path`、`base_revision`、`revision` 与 `warnings`。

## Auth Mode 边界

- `subscription` Profile 会清理 CCR 管理的 `ANTHROPIC_*` 及相关 Claude Code 环境覆盖。
- `api_key` Profile 将类型化字段写入 `~/.claude/settings.json` 的 `env` 块。
- 第三方端点应使用 `api_key`；当 `base_url` 与密钥同时存在时，CCR 会按运行时规则归一化明显失配的历史配置。
- 所有终端输出、错误诊断与 `--json` 回执均不回显完整密钥。

## 相关页面

- [配置模型](/guide/configuration)
- [CLI 工作流](/guide/cli-workflows)
- [`current`](./current)
- [`validate`](./validate)
- [`doctor`](./doctor)
- [`platform`](./platform)
