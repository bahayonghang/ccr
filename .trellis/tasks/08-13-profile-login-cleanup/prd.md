# Codex/Claude/Grok profile 登录清理

## Goal

用户在 Claude / Codex / Grok 用过第三方或 API-key profile 之后，能用统一的 `ccr <platform> profile off` 清掉 profile 切换写入的运行时配置，使官方登录不再被残留凭据、路由或 `forced_login_method` 压制。已保存的 profile 定义和官方账号快照保留。

TUI 在 profile 应用和官方账号切换时自动执行同一清理。ccr-ui 在 Profiles 横幅和 Auth 诊断区提供显式按钮。

## User Value

切回官方账号或执行 `claude login` / `codex login` / Grok 原生登录时，不必手工改 `settings.json`、`auth.json`、`config.toml`。

## Decisions

| ID | 决定 |
| --- | --- |
| D1 | 语义为登录预备清理：加强现有 `profile off`，不新增子命令，不删除官方凭据后强制重登 |
| D2 | 完善 `ccr claude\|codex\|grok profile off`，三家同一语义 |
| D3 | TUI profile 应用与 Claude/Codex auth 切换时先执行该 off；Profile 页另提供 `o` |
| D4 | ccr-ui 三家 Profiles + Claude/Codex Auth 有显式 Off |
| D5 | 用户自有 `ANTHROPIC_API_KEY` / `apiKeyHelper` / 云厂商 env / `primaryApiKey` 只诊断不删 |
| D6 | VS Code 本期不接线 |
| D7 | UI 放置：Grok 同构横幅（Header 与 StatStrip 之间）+ 命令面板 `__off` + Auth 诊断/运行时区同一动作 |

## Confirmed Facts

| 平台 | 运行时文件 | switch 写入、会挡住登录的内容 | 现有 off 缺口 |
| --- | --- | --- | --- |
| Claude | `~/.claude/settings.json` `env`；`profiles.toml` / registry 指针 | `CCR_MANAGED_KEYS`。`crates/ccr-cli/src/platforms/claude.rs:317-325` | 无指针时即使托管 env 仍在也 `unchanged`。`profile_off.rs:142-150`。Clap `Off` 无帮助。`cli/subcommands/claude.rs:121-137` |
| Codex | `~/.codex/config.toml`、`auth.json`、`profile_entry_auth_state.json` | `model_provider=custom`、bearer、`forced_login_method`、API-key 写入并删 OAuth。`ccr-codex/src/platforms/codex.rs:1042-1224` | 无快照时不改 `auth.json`。`codex.rs:1357-1366` |
| Grok | `$GROK_HOME/config.toml`、`profile_entry_config_state.json` | `[model.custom]` + `models.default=custom`。`platforms/grok.rs:649-737` | CLI 未走 `profile_off_for_platform`（该函数对 Grok 报不支持） |

Claude 认证优先级见 `.trellis/spec/ccr-cli/backend/claude-auth-runtime.md`。自动删除边界是 `CCR_MANAGED_KEYS`。

入口对照：Grok UI 已有横幅 Off（`GrokProfilesView.vue:49-75`）。Claude/Codex UI 与 TUI 无 Off。TUI `apply_selected` 只调 `apply_profile`（`tui/app.rs:921`）。Claude auth 切换会清托管 env 但不清指针（`claude_auth_service.rs:316`）。Codex auth 切换清 custom route 但不清 CCR 指针（`codex_auth_service.rs:1102`）。`ccr clear` 只清 Claude env，本期不扩平台。

## Requirements

- R1. 三家 `ccr <platform> profile off` 使用同一登录预备语义：退出 profile mode，并清掉会压制官方登录的 CCR 写入残留。
- R2. 不删除 `profiles.toml` 中的 profile 定义，不删除已保存官方账号快照，不删除 off 前已存在的 Claude `.credentials.json` 订阅凭据。
- R3. Claude：清 `CCR_MANAGED_KEYS` 与 stale 指针。无指针但 settings 仍有托管 env 时也执行清理。用户自有非托管源只进入 `remaining_suppressors`。修正 Clap `Off` 帮助。
- R4. Codex：清 `forced_login_method`、`preferred_auth_method`、`model_catalog_json`、custom bearer / 第三方路由。有入口 `auth.json` 快照则恢复。无快照且因 profile 指针或第三方 runtime 进入 off、且 `auth.json` 含 `OPENAI_API_KEY`、无 OAuth `tokens` 时，删除该 key（文件变空则删文件）。无指针、无第三方 runtime、仅官方 API key 时不改 `auth.json`。
- R5. Grok：有入口状态则恢复并退出。无入口状态且仍有激活意图或 managed shape 时失败关闭、文件不变。CLI 走 `profile_off_for_platform`。
- R6. 写盘前备份；失败回滚。日志、CLI JSON、Tauri DTO、UI 文案不含密钥。
- R7. 结果可观察：`changed`、上一 profile、Claude `remaining_suppressors`。无残留时成功且 `changed=false`。
- R8. TUI Profile Enter：先 off 再 `apply_profile`。TUI Claude/Codex Auth 切换：先 off 再 `switch_account`。Grok 仅 Profile 路径。Profile 页 `o` 只执行 off。off 失败则中止后续 apply/switch。
- R9. ccr-ui：三家 Profiles 在 Header 与 StatStrip 之间显示横幅（profile mode 或 CCR 登录残留时）；命令面板 `__off`。Claude Auth 诊断区、Codex Auth 运行时区在可 off 时显示同一动作。确认框 `type=warning`，可取消且不写盘。Grok 已有横幅保持语义，改为调用共享 `profile_off` 核心。
- R10. 不调用官方 login 进程。
- R11. 不改 `ccr clean`。不把 `ccr clear` 扩到 Codex/Grok。

## Acceptance Criteria

- [ ] AC1. 从第三方/API-key profile 执行 `ccr <platform> profile off` 后，该平台 `current_profile` / `current_config` 为空。
- [ ] AC2. Claude：托管键从 `settings.json.env` 消失；`.credentials.json` 与已保存账号仍在；用户 `ANTHROPIC_API_KEY` 仍在并出现在 `remaining_suppressors`。无指针但托管 env 仍在时，off 也清掉托管 env。
- [ ] AC3. Codex：`config.toml` 无 `forced_login_method` / `experimental_bearer_token`。有快照则 `auth.json` 等于快照。无快照 + 指针或第三方 runtime + 仅 `OPENAI_API_KEY` 时去掉该 key。无指针且无第三方 runtime 时不改官方 `auth.json`。
- [ ] AC4. Grok：有入口状态则恢复进入 profile 前的 `config.toml`。无入口状态且仍有意图/managed shape 时失败且字节不变。`profile_off_for_platform(Grok)` 不再报不支持。
- [ ] AC5. 重复 off 成功且幂等。无指针且无 CCR 残留时 `changed=false`。
- [ ] AC6. 已保存 profile 列表与内容不变。
- [ ] AC7. CLI / TUI / Tauri / UI 输出不含密钥。`ccr {claude,codex,grok} profile off --help` 说明退出 profile 并清登录残留。
- [ ] AC8. TUI：第三方 profile 下 Enter 应用到另一 profile 后，上一 profile 托管残留不在运行时文件中。第三方 profile 下切换官方账号后，指针为空，官方账号不被 CCR 托管 token 压制。`o` 只 off 不 apply。off 失败时不 apply / 不 switch。
- [ ] AC9. ccr-ui 三家 Profiles 横幅与 Claude/Codex Auth 均可完成 Off。确认取消不写盘。
- [ ] AC10. `cargo test -p ccr --test commands -- --test-threads=1`（claude/codex/grok profile 与相关 auth）；`cargo test -p ccr-cli -- --test-threads=1`；`cargo test -p ccr-codex -- --test-threads=1`；`cargo test -p ccr-tui -- --test-threads=1`；`just frontend-check-quick`。

## Out of Scope

- 删除用户手工维护的 profile 定义。
- 代理官方 login / setup-token / 刷新 token。
- Gemini / Droid / OpenCode / Qwen。
- `ccr clean`。默认删除用户自有非托管凭据。
- VS Code 接线。
- 修改上游登录优先级。
- 平台 Home 页 Off 按钮。

## Technical Notes

共享核：`crates/ccr-cli/src/application/profile_off.rs`。Codex 核：`CodexPlatform::clear_active_profile_runtime`。Grok 核：`GrokPlatform::clear_active_profile_runtime`。TUI：`tui/app.rs:921`、`claude_auth/app.rs:454`、`codex_auth/app.rs:1307`。Tauri：补 `claude_profile_off` / `codex_profile_off`；Grok 已有 `grok_profile_off`。设计见 `design.md`，执行见 `implement.md`。研究见 `research/`。
