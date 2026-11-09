# Profile 切换运行时写入与清理

调研日期：2026-08-13。只记录仓库事实，不含产品决策。

## 1. 调用链

```
CLI  ccr <platform> profile switch <name>
  → switch_profile_for_platform
    crates/ccr-cli/src/application/profile_switch.rs:24
  → PlatformConfig::apply_profile

CLI  ccr claude|codex profile off
  → profile_off_for_platform
    crates/ccr-cli/src/application/profile_off.rs:125
  → Claude: clear_claude_profile_settings_overrides + 清指针
  → Codex: CodexPlatform::clear_active_profile_runtime + 清指针

CLI  ccr grok profile off
  → GrokPlatform::clear_active_profile_runtime
    crates/ccr-cli/src/commands/grok/profile.rs:331
  不经过 profile_off_for_platform（该函数对 Grok 返回「暂不支持」）
```

Tauri：`claude_apply_profile` / `codex_apply_profile` / `grok_apply_profile` 直接调 `apply_profile`。只有 `grok_profile_off`。

## 2. Claude

写入：`ClaudePlatform::apply_profile`（`platforms/claude.rs:279`）。

- `ApiKey`：`settings.apply_managed_env(pairs)`。
- `Subscription`：`settings.clear_ccr_managed_vars()`，不写 token。
- 键表：`ccr-types/src/claude_settings.rs` `env_keys::CCR_MANAGED_KEYS`。

`profile off`（`profile_off.rs:136`）：

- 有 profile 指针或 profiles 文件 `current_config` 才写盘。
- 清托管 env；清 registry `current_profile`；清 `profiles.toml` `current_config`。
- 快照到 `~/.ccr/backups/profile-off/claude-<ts>/`。
- 通过 `ClaudeAuthService::action_outcome` 报告仍会压制订阅的源。
- 不改 `.credentials.json`、不改已保存账号。

`ccr clear`（`commands/lifecycle/clear.rs`）只清托管 env，不清指针。提示仍写 `ccr switch`。

`auth switch` 里 `clear_profile_api_key_overrides_if_needed`（`claude_auth_service.rs:316`）仅在当前 profile 的 **effective** `auth_mode == ApiKey` 时清托管键。

契约：`.trellis/spec/ccr-cli/backend/claude-auth-runtime.md`。自动删除边界是 `CCR_MANAGED_KEYS`。用户 `ANTHROPIC_API_KEY` 只警告。

Clap：`ClaudeProfileAction` 中「退出当前 profile 路由…」注释贴在 `Create` 上，`Off` 无文档。`cli/subcommands/claude.rs:121-137`。

## 3. Codex

写入：`apply_profile` → official / third-party → `apply_switch_spec`（`ccr-codex/src/platforms/codex.rs:2233`、`1042`）。

`SwitchSpec` 会改：

- `config.toml`：`model`、`model_provider=custom`、`[model_providers.custom]`、`forced_login_method`、`preferred_auth_method`、`model_catalog_json`、`cli_auth_credentials_store`。
- `auth.json`：`WriteOpenAiApiKey` 写入 key 并去掉 `tokens`；`ClearOpenAi` 删除 tokens / key；bearer 把 token 写到 `experimental_bearer_token` 而不是 `auth.json`。

首次 apply 调用 `capture_profile_entry_auth_state`（create-if-absent）。路径：平台目录下 `profile_entry_auth_state.json`。

`clear_active_profile_runtime`（`codex.rs:1357`）：

1. `apply_runtime_route_without_auth(Official, File)`：删一批托管根字段，再写入官方 `custom` provider（不是隐式缺省 openai）。
2. `restore_profile_entry_auth_state`：有快照则覆盖或删除 `auth.json`，并删快照文件。
3. 清 registry `current_profile`。共享 `profile_off` 再清 `profiles.toml` 指针。

测试锚点：`crates/ccr/tests/commands/codex_profile.rs`

- `codex_profile_switch_and_off_are_consistent_and_off_keeps_auth_json`：API-key switch 清 OAuth；off 恢复入口 `auth.json`。
- `codex_profile_switches_deepseek_bearer_and_clears_runtime_on_off`：off 去掉 `forced_login_method` 与 bearer。
- 无快照 + 仅 stale 指针：off 不改 `auth.json`。

缺口：指针还在、快照缺失时，`auth.json` 里的 `OPENAI_API_KEY` 会留下。`codex login` 会看到 API key。

契约：`.trellis/spec/ccr-codex/backend/codex-provider-bearer-runtime.md`。

## 4. Grok

写入：`apply_profile`（`platforms/grok.rs:1005`）。

- 第三方：`[model.custom]` + `models.default=custom`。
- 官方 profile：恢复入口 `[model.custom]`，可选改 `models.default`。
- 首次 apply 写 `profile_entry_config_state.json`（CAS，不覆盖）。

`clear_active_profile_runtime`（`grok.rs:200`）：

- 有入口状态：恢复 `config.toml`，清指针，删入口状态。
- 无入口状态且仍有激活意图或 managed shape：`ConfigError`，文件不变。

UI：`GrokProfilesView.vue` `canOff` / `handleOff` → `grok_profile_off`。Claude / Codex 配置页无对应按钮。

契约：`.trellis/spec/ccr-cli/backend/grok-profile-runtime.md`。

## 5. 入口覆盖

| 面 | Claude | Codex | Grok |
| --- | --- | --- | --- |
| CLI `profile off` | 有 | 有 | 有（独立实现） |
| 共享 `profile_off_for_platform` | 有 | 有 | 无 |
| Tauri | 无 | 无 | `grok_profile_off` |
| UI Profiles 页 | 无 | 无 | Off 按钮 |
| TUI | 无 | 无 | 无 |
| VS Code | `execProfileOff` 未调用 | 同左 | 未接线 |

## 6. 对「方便后续登录」的含义

残留压制官方登录的机制：

- Claude：`settings.json.env.ANTHROPIC_AUTH_TOKEN` / `ANTHROPIC_BASE_URL` 优先于订阅 OAuth。`profile off` 已清托管键。用户 `ANTHROPIC_API_KEY` 仍可能压制。
- Codex：`forced_login_method=api` 限制登录方式；`auth.json` 的 `OPENAI_API_KEY` 让 Codex 走 API 而不是 ChatGPT。off 在有快照时能恢复；无快照则 key 留下。
- Grok：`[model.custom].api_key` + `models.default=custom` 覆盖原生会话。off 依赖入口状态。

现有 `off` 语义是「恢复进入 profile 前的官方状态」，不是「抹掉官方凭据以便重新登录」。

## 7. 相关历史

- 2026-07-29 会话：Claude 托管键清理不一致（`clear_anthropic_vars` vs `clear_ccr_managed_vars`）。当前 `profile_off.rs:229` 已用 `clear_ccr_managed_vars`。
- 归档任务 `06-26-claude-third-party-profile-switch-analysis`：把 `profile off` 当作故障恢复口。
- 归档任务 `07-31-codex-deepseek-switch`：bearer 的 `forced_login_method=api` 必须在 off 时清掉。
