# 仓库现状：auth 面 vs profile off

调研日期：2026-08-20。只记录仓库事实。

## 1. 已有命令面

| 平台 | Auth 组 | Profile off | Auth off |
| --- | --- | --- | --- |
| Claude | `save` `list` `switch` `delete` `current` | 有 | 无 |
| Codex | 上列 + `update` `sync` `repair` `rename` `export` `import` | 有 | 无 |
| Grok | 无 `ccr grok auth` | 有 | 无 |

Clap 锚点：

- `crates/ccr-cli/src/cli/subcommands/claude.rs` `ClaudeAuthAction`
- `crates/ccr-cli/src/cli/subcommands/codex.rs` `CodexAuthAction`
- `crates/ccr-cli/src/cli/subcommands/grok.rs` 只有 `GrokAction::Profile`

`ProfileOffActionArgs` 仅 `--json`（`cli/subcommands/profile_args.rs:72`）。

## 2. `profile off` 与官方凭据的边界

共享核：`crates/ccr-cli/src/application/profile_off.rs`。契约：`.trellis/spec/ccr-cli/backend/profile-off-login-prep.md`。

| 平台 | profile off 清什么 | 明确不清什么 |
| --- | --- | --- |
| Claude | `CCR_MANAGED_KEYS`、registry/`profiles.toml` 指针 | `.credentials.json`、已保存账号、用户 `ANTHROPIC_API_KEY` |
| Codex | 第三方路由、`forced_login_method` 等；有指针/第三方 runtime 时删除运行期 `auth.json` | 无指针且无第三方 runtime 时的官方 `auth.json`；已保存账号 |
| Grok | `[model.custom]`、`[models].default`，恢复入口 reasoning | `auth.json`、`mcp_credentials.json`、`XAI_API_KEY` |

Grok 文档契约：`docs/reference/commands/grok.md` 写明 CCR **不会读取或写入** `auth.json` / `mcp_credentials.json`。`platforms/grok.rs` 只操作 `$GROK_HOME/config.toml`。

Codex 非 file store：`CodexAuthService` / `CodexRuntimeService` 拒绝写入 `auth.json`，提示使用 `codex login` / `codex logout` 或把 `cli_auth_credentials_store` 改为 `file`。

Claude macOS Keychain：诊断为 `unobservable`；账号快照/切换在 macOS 上 unsupported。见 `.trellis/spec/ccr-cli/backend/claude-auth-runtime.md`。

## 3. Auth 现有写路径

Claude：

- `switch_account` 覆盖运行时 `.credentials.json`（Windows/Linux）
- `delete_account` **不**修改 `.credentials.json`（`commands/claude/auth/delete.rs`）
- TUI / UI 切换官方账号前先 `profile_off_for_platform(Claude)`

Codex：

- `switch` 在 file store 下替换 `~/.codex/auth.json`
- `delete` 不改当前登录
- TUI / UI 切换前先 `profile_off_for_platform(Codex)`
- `current` / `list` 在 keyring/auto 下提示 `codex logout`

Grok：无账号快照、无 Auth TUI、无 Auth 视图。ccr-ui 只有 Profiles / Home / Settings。`GrokAuthModeDto` 是 profile 的 `InlineApiKey | EnvKey | Session`，不是官方账号。

## 4. 表面缺口

TUI：

- Profile 页 `o` = profile off；Auth 页 footer **故意省略** profile off（`tui/ui.rs` `footer_omits_profile_off_on_auth_tabs`）
- Auth 页无独立 logout / auth off 键
- 无 Grok Auth 标签

ccr-ui：

- Profiles 与 Claude/Codex Auth 诊断区已有 **profile** Off 横幅（`ProfileOffBanner` / `can_off`）
- `ClaudeAuthView` / `CodexAuthView` 无“登出当前官方登录”按钮
- 无 Grok Auth 页
- 命令面板 `__off` 绑定的是 profile off

VS Code：

- 已有 `execClaudeProfileOff` / `execCodexProfileOff` / `execProfileOff`
- 无 auth off 包装。上一次 profile off 任务把 VS Code 列为范围外

## 5. Codex 文件共用

Codex 的 profile 第三方 key 与官方 OAuth 共用 `auth.json`。`profile off` 在 login-prep 路径会删该文件；官方模式下故意保留。`auth off` 若无条件删 `auth.json`，在仍处于第三方 profile 时会拆掉当前 profile 的运行时 key，直到再次 `profile switch`。

## 6. 可复用能力

- 备份/回滚模式：`ProfileOffBackup`（`$CCR_ROOT/backups/profile-off/`）
- Claude 路径：`ClaudeRuntimePaths.credentials_file`
- Codex 路径：`CodexConfigManager.auth_path()`，以及 `login_prep_codex_dirs()`（含 `CODEX_HOME` 重定向时的默认 home）
- Grok home：`GROK_HOME` 否则 `~/.grok`
- Tauri 命令三联：`commands.rs` `COMMANDS`、`useTauri.ts` `COMMANDS`、`lib.rs` `generate_handler`
- 领域门面：Claude/Codex 有 `ccr-ui/src/api/domains/{claude,codex}.ts`；Grok 无

## 7. 与官方 logout 的能力差

| 情况 | CCR 今日能否自己完成等价登出 |
| --- | --- |
| Claude Win/Linux `.credentials.json` | 能删文件；不能复制 `/logout` 的 first-launch reset |
| Claude macOS Keychain | 不能；现有契约是 unobservable / 切换 unsupported |
| Codex `file` `auth.json` | 能删文件（profile off 已在 login-prep 路径删除） |
| Codex `keyring` / `auto` | 不能写；已提示用户跑 `codex logout` |
| Grok `auth.json` | 技术上能删文件，但产品边界明确禁止读写 |
| Grok `mcp_credentials.json` | 同样禁止；官方 logout 文档未承诺会清它 |
