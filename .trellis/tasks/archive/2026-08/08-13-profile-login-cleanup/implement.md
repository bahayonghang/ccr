# Implement: profile 登录预备清理

## Order

1. **共享核**  
   `crates/ccr-cli/src/application/profile_off.rs`：`needs_login_prep`；Claude 无指针也清托管 env；`profile_off_for_platform` 纳入 Grok。修正 `cli/subcommands/{claude,codex,grok}.rs` 的 `Off` Clap 文档。

2. **Codex 无快照 auth**  
   `crates/ccr-codex/src/platforms/codex.rs` `clear_active_profile_runtime`：按 `design.md` 删除 switch 留下的 `OPENAI_API_KEY`；无指针且无第三方 runtime 时不改 `auth.json`。补单测。

3. **CLI 接线**  
   `commands/grok/profile.rs` `off_command` 改调 `profile_off_for_platform`。统一三家 JSON。集成测试：`crates/ccr/tests/commands/{claude,codex,grok}_profile.rs`。

4. **TUI**  
   `tui/app.rs` `apply_selected`：先 off 再 apply。`claude_auth/app.rs`、`codex_auth/app.rs` 切账号先 off。Profile 页 `o` + footer。off 失败则中止。

5. **Tauri**  
   `claude_profiles.rs` 加 `claude_profile_off`；`codex_profiles.rs` 加 `codex_profile_off`。`grok_profile_off` 改调共享核。`handler_registry` 注册并更新冻结计数。domain API：`ccr-ui/src/api/domains/{claude,codex}.ts`。生成绑定，勿手改 `generated/`。

6. **UI**  
   Claude/Codex Profiles：Header 与 StatStrip 之间横幅 + 命令面板 `__off`。Claude Auth 诊断区、Codex Auth 运行时区按钮。Grok 横幅保持，确认仍走 `useConfirmAction` / `type=warning`。`can_off` 来自后端，不猜文件。

7. **文案**  
   中英 i18n：退出 Profile / 清理说明。无密钥。

## Validation

```text
cargo test -p ccr-cli profile_off -- --test-threads=1
cargo test -p ccr-codex -- --test-threads=1
cargo test -p ccr --test commands -- claude_profile -- --test-threads=1
cargo test -p ccr --test commands -- codex_profile -- --test-threads=1
cargo test -p ccr --test commands -- grok_profile -- --test-threads=1
cargo test -p ccr-tui -- --test-threads=1
just fmt-check
just tauri-bindings-check
just frontend-check-quick
```

跨层完成后再跑 `just lint-strict`。密钥扫描：`python scripts/check-secret-writes.py`（若改含密写盘路径）。

## Risky files

| 路径 | 风险 |
| --- | --- |
| `ccr-codex/src/platforms/codex.rs` | 误删官方 `auth.json` |
| `application/profile_off.rs` | Grok 失败关闭被改成猜测删除 |
| `handler_registry.rs` | 漏注册或计数未更新 |
| `ClaudeCodeProfilesView.vue` / `CodexProfilesView.vue` | 破坏三页骨架同构 |
| `tui/app.rs` | off 失败仍 apply |

回滚点：核与 CLI 可单独保留；UI/TUI 可先藏按钮与 `o`。

## Before start

- 已评审 `prd.md` / `design.md` / 本文件。
- 用户批准最新规划摘要后才 `task.py start`。
- 实施前读 `trellis-before-dev` 与 jsonl 中的 spec。
- 不改 VS Code，不扩 `ccr clear`，不调官方 login。
