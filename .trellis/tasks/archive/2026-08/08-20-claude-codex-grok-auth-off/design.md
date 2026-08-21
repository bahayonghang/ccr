# Design: Claude / Codex / Grok auth off

审阅处置见 `research/review-disposition.md`。

## Architecture

单一写核：`auth_off_for_platform(Platform) -> AuthOffResult`，与 `profile_off_for_platform` 并列，互不调用。

```
CLI / TUI / Tauri / VS Code CLI wrapper
        │
        ▼
auth_off_for_platform
        │
        ├── inspect store
        ├── file 且无凭据文件 → Ok(changed=false, 不 spawn)
        ├── file store → backup + delete + commit + 删除本次 backup 目录
        └── keychain/keyring/auto → spawn official logout
```

禁止各面复制删除列表或各自 spawn。

位置：`crates/ccr-cli/src/application/auth_off.rs`，由 `application/mod.rs` 导出。

## Result contract

```text
AuthOffResult {
  platform,
  changed: bool,
  path: File | NativeLogout,
  profile_pointer: Option<String>,  // 未改；Codex 有指针时填，供提示
  warnings: Vec<String>,            // 无密钥
}
```

CLI `--json`：`ok`, `changed`, `path`, `profile_pointer`, `warnings`。`path` 为 `"file"` / `"native_logout"`。

`changed`：

| 路径 | 含义 |
| --- | --- |
| file | 本次删除了至少一个凭据文件 |
| native | 本次成功 spawn 了官方 logout（退出码 0）。不可观察登录态，故第二次仍可能 `changed=true` |

`needs_auth_off(platform) -> bool` 供 UI `can_auth_off`。前端不得读 home 文件。

| 平台 | `needs_auth_off` 为 true |
| --- | --- |
| Claude Win/Linux | `.credentials.json` 存在且为非空对象或诊断为已登录 |
| Claude macOS | 始终 true（Keychain 不可观察；off 必须幂等成功） |
| Codex `file` | `login_prep_codex_dirs()` 任一目录存在 `auth.json` |
| Codex `keyring`/`auto` | 始终 true |
| Grok | `$GROK_HOME/auth.json` 存在（存在性读取，不解析 token） |

## File vs native

### Claude

- `cfg(target_os = "macos")`：spawn `claude auth logout`。不读不写 Keychain。不额外改 `~/.claude.json`。
- 其它 OS：删除 `ClaudeRuntimePaths.credentials_file`。文件不存在 → `changed=false`。不修改 onboarding。

### Codex

- 读 `cli_auth_credentials_store`，沿用 `CredentialStoreKind`。
- `File`：删除 `CodexPlatform::login_prep_codex_dirs()` 返回的每个目录下的 `auth.json`。不改 `config.toml`。
- `Keyring` / `Auto`：spawn `codex logout`。
- 有 profile 指针时仍执行清除，JSON 带 `profile_pointer` 与 warning。退出码 0。

### Grok

- 存在性读取、备份、删除 `$GROK_HOME/auth.json`（未设则 `~/.grok/auth.json`）。不解析 token。
- 不读不写 `mcp_credentials.json`、`config.toml`、CCR profiles。
- spec/文档原句「never read, written, backed up, or validated」改为：CCR 仅允许 auth off 对 `auth.json` 做存在性读取、secret 备份与删除；其余操作仍禁止。`mcp_credentials.json` 仍四项全禁。

## Backup

`$CCR_ROOT/backups/auth-off/<platform>-<ts>/`，Unix 目录 `0o700`。快照用 `AtomicWriter.secret(true)`。RAII 对齐 `ProfileOffBackup`：未 `commit` 则还原。

- Claude：credentials 文件（若存在）。
- Codex file：`login_prep_codex_dirs()` 中各 dir 的 `auth.json`。
- Grok：`auth.json`。
- Native：不建 backup。

成功 `commit` 之后删除本次快照目录（D10）。`changed=false` 时不建目录。

`auth_off.rs` 不加入 `scripts/quality/check_secret_writes.py` 的 `SENSITIVE_MODULES`（与 `profile_off.rs` 一致）。删除 OpenCode 服务文件时必须同时删掉该脚本中的两条 OpenCode 路径。

## Native logout spawn

| 平台 | argv |
| --- | --- |
| Claude macOS | `claude auth logout` |
| Codex | `codex logout` |

约束：继承 PATH；不走 shell；stdin 关闭；stdout/stderr 不回显；超时 15s；二进制缺失或非零 → `Err`；不传 flags；不调用 `login`。

Tauri 非 local：`ensure_local_env`（`ccr-ui/src-tauri/src/commands/settings_raw.rs`），返回 `unsupported_environment`，不 spawn、不写盘。

复用 `codex fix --doctor` 的 PATH 与超时杀进程模式，不复制 doctor JSON 解析。

## Grok Auth 读写面

新建 `GrokAuthService`：

- `current()`：已登录 / 未登录（`auth.json` 是否存在）。禁止输出 token。
- `off()`：调用 `auth_off_for_platform(Grok)`。

CLI：

```text
ccr grok auth current [--json]
ccr grok auth off [--json]
ccr grok auth          # 有 TUI launcher → Grok Auth 标签；否则帮助
```

`ccr grok` 无子命令仍打印 grok 帮助。

Clap：`GrokAction` 增加 `Auth { action: Option<GrokAuthAction> }`。`GrokAuthAction`：`Help`、`Current { json }`、`Off { json }`。

## TUI 页签与 tui.toml

变更文件必须包括 `crates/ccr-config/src/managers/tui_config.rs`（经 `ccr_cli::managers` 再导出）。

- 新增 `TuiTabId::GrokAuth`（`grok_auth`）。
- `TuiTabId::OpencodeAuth` 按 `Usage` 先例保留为 deprecated：`load()` 过滤并 warn，不得因旧 `tui.toml` 含 `opencode_auth` 而反序列化失败并丢掉用户排序。
- `DEFAULT_TAB_ORDER` 仍为 6 项：把 `OpencodeAuth` 槽换成 `GrokAuth`。默认显示顺序保持现有前五项，第六项由 OpenCode Auth 变为 Grok Auth：
  `codex_profile, claude_profile, grok_profile, codex_auth, claude_auth, grok_auth`。
  不重排 Claude/Codex 页签。`load()` 在过滤后把缺失的 `GrokAuth` 按默认顺序追加。
- `validate_tab_order` 仍要求与 `DEFAULT_TAB_ORDER` 等长且无缺项；过滤后的 `OpencodeAuth` 不计入。
- `crates/ccr-tui/src/tui/app.rs`：`tab_config_id` 映射 `TabVariant::GrokAuth` → `TuiTabId::GrokAuth`；删除 `OpenCodeAuth` 运行时页签。

TUI 其它：

- 删除 `tui/opencode_auth/`、`TuiLaunchers.opencode_auth`、`run_opencode_auth_tui`。
- 增加 `tui/grok_auth/`。`TuiLaunchers.grok_auth`：`ccr grok auth` 无嵌套动作时进入该标签。
- Auth 页 `o`：`auth_off_for_platform`。Profile 页 `o` 仍是 profile off。
- Grok Auth 只渲染会话状态和 off。

## Tauri / ccr-ui

新命令（`spawn_blocking` + `ensure_local_env`）：

- `claude_auth_off`
- `codex_auth_off`
- `grok_auth_off`
- `grok_auth_current`

三联注册：`handler_registry` / 生成绑定 / `generate_handler`。不要手改 `generated/`。随后 `just tauri-command-inventory`。

Domain：Claude/Codex 走现有 `domains/{claude,codex}.ts`；Grok auth 走 `domains/grok.ts`。禁止往 `tauri.ts` 加新 `invoke()`。

UI：

- current DTO 增加 `can_auth_off`（来自 `needs_auth_off`）。禁止复用 profile 的 `can_off`。
- 处理函数 `handleAuthOff`，禁止复用 `handleOff`。
- 命令面板 Auth 页 `__auth_off`，不占用 Profiles `__off`。
- Claude/Codex Auth：诊断/运行时区增加「登出官方会话」。确认 `type=warning`。
- 新页 `GrokAuthView.vue`，路由 `/grok/auth`，subnav 加 Auth。确认 `type=danger`。
- i18n：`auth.off*` / `auth.confirmOff*`。

## VS Code

只加服务层，对齐 `execProfileOff`：`buildPlatformAuthOffArgs` / `execAuthOff` + argv 测试。不改 `package.json`，不为 Grok 打开 profile 写入口。

## OpenCode 删除

删除：

- `Commands::OpenCode`、`cli/subcommands/opencode.rs`
- `crates/ccr-cli/src/commands/opencode/`
- `dispatch_opencode`
- `CommandDispatcher::show_version()` 中 OpenCode 两行（`dispatch.rs` 约 801、806-809）
- help_config 中 opencode auth 文案
- `TuiLaunchers.opencode_auth` 及 `main.rs` 注入
- `crates/ccr-tui/src/tui/opencode_auth/`
- `OpenCodeAuthService` / `OpenCodeQuotaService` / `OpenCodeUsageService` 及仅被其使用的 `models/opencode_auth.rs`
- `scripts/quality/check_secret_writes.py` 中两条 OpenCode `SENSITIVE_MODULES`
- `crates/ccr/tests/commands/help.rs` 的 opencode 用例
- `crates/ccr-cli/tests/dispatch_routing.rs` 的 `opencode_auth` launcher 字段与相关用例
- 所有指向 `docs/reference/commands/opencode` 的链接，包括：
  - `docs/reference/commands/opencode.md` 与英文页（删除文件）
  - `docs/{,en/}reference/commands/index.md`
  - `docs/{,en/}reference/commands/tui.md`
  - `docs/{,en/}reference/commands/version.md`
  - `docs/{,en/}guide/cli-workflows.md`
  - `docs/{,en/}reference/architecture.md`
  - `docs/.vitepress/config.mjs` 中英侧边栏（约 62、238 行）
- Tauri `command_exec.rs` 里 `ccr opencode` 面板条目

保留：ccr-ui `/opencode/*` 与 Tauri `opencode_*` settings/agents/MCP 命令。磁盘上用户 OpenCode 数据不删。

`ccr opencode` 变为未知命令。

## Compatibility

- `profile off` 行为不变。
- Claude/Codex `auth save|switch|delete|current` 不变。
- 旧 `tui.toml` 含 `opencode_auth`：过滤该项、追加 `grok_auth`、保留其余自定义顺序。
- Grok `auth.json` 边界仅对 auth off 放开存在性读取、备份、删除。
- OpenCode 配置文件不迁移、不删除。

## Trade-offs

| 选择 | 代价 |
| --- | --- |
| native `changed=true` 表示 spawn 成功 | 第二次 off 不能 `changed=false` |
| 成功后删除 backup | 成功 logout 后 CCR 不能从 backup 恢复官方凭据 |
| 默认 tab 只替换第六槽 | 不实现「Claude Auth 排第一」的展示顺序 |
| Grok 确认 `danger` | 与 Claude/Codex `warning` 文案分级不同 |

## Rollback

- 文件路径：未 commit 时 Drop 还原；已 commit 则 backup 目录已删，只能官方重新登录或 `auth switch`（Claude/Codex）。
- Native：无法回滚钥匙串。
- UI：可先藏按钮。
- OpenCode 删除：git 还原。

## Tests

- `cargo test -p ccr-cli auth_off -- --test-threads=1`：file 幂等、`changed=false` 不 spawn；写入失败回滚；成功后 backup 目录消失；spawn 缺失二进制 → Err；Codex `needs_auth_off` 覆盖 `login_prep_codex_dirs()` 第二目录。
- `cargo test -p ccr-config`：`TuiTabId` 默认顺序含 `GrokAuth`；旧 toml `opencode_auth` 可 load 且排序其余项保留。
- CLI 集成：claude/codex/grok auth off；Claude 第三方 profile 后托管 env 仍在；Grok 第三方后 `[model.custom]` 仍在。
- `cargo test -p ccr --test commands -- help -- --test-threads=1`：`ccr version` 不含 opencode。
- TUI：无 OpenCode 页签；Grok Auth `o`；Profile `o` 仍是 profile off。
- UI smoke：`can_auth_off`；Claude/Codex warning、Grok danger；确认取消。
- VS Code argv。
- `just tauri-bindings-check`
- `just tauri-command-inventory` 然后 `just tauri-command-inventory-check`
- `just frontend-check-quick`
- `cd ccr-vscode && npm test`
