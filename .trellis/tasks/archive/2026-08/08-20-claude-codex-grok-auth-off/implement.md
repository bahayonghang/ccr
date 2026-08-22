# Implement: Claude / Codex / Grok auth off

## Order

1. **共享核**  
   `crates/ccr-cli/src/application/auth_off.rs`：`AuthOffResult`、`needs_auth_off`、`auth_off_for_platform`、backup RAII、native spawn、成功后删除快照目录。Codex 探测与删除都走 `login_prep_codex_dirs()`。

2. **平台适配**  
   Claude：macOS native / 其它 OS 删 credentials，不改 onboarding。  
   Codex：file 删各 login-prep dir 的 `auth.json`；keyring/auto spawn。  
   Grok：`GrokAuthService` current/off；存在性读取 + 备份 + 删除 `auth.json`。

3. **CLI**  
   Claude/Codex `Auth` 加 `Off { json }`。Grok 加 `auth` 组。`show_version()` 去掉 OpenCode 行。

4. **TUI + tui.toml**  
   `crates/ccr-config/src/managers/tui_config.rs`：`GrokAuth`；`OpencodeAuth` deprecated 过滤。  
   删 OpenCode Auth 模块与运行时页签。加 `tui/grok_auth/`。Auth 页 `o` → auth off。`TuiLaunchers.grok_auth` 替换 `opencode_auth`。更新 `dispatch_routing.rs`。

5. **删除 `ccr opencode`**  
   Clap、dispatch、help、`check_secret_writes.py` 两条路径、仅被该面使用的 OpenCode 服务。文档与 VitePress 侧边栏全部 opencode 命令链接。用户 OpenCode 磁盘数据不删。

6. **Tauri / UI**  
   `claude_auth_off` / `codex_auth_off` / `grok_auth_off` / `grok_auth_current`，守卫用 `ensure_local_env`。生成绑定。`just tauri-command-inventory`。  
   DTO `can_auth_off`；`handleAuthOff`。GrokAuthView。Claude/Codex `warning`，Grok `danger`。

7. **VS Code**  
   `buildPlatformAuthOffArgs` / `execAuthOff` + argv 测试。

8. **文档与 spec**  
   平台命令页。Grok spec/文档改为允许 auth.json 存在性读取、备份、删除。新建 `.trellis/spec/ccr-cli/backend/auth-off.md`。

## Validation

```text
just version-check
just fmt-check
cargo test -p ccr-cli auth_off -- --test-threads=1
cargo test -p ccr-config -- --test-threads=1
cargo test -p ccr --test commands -- claude_auth -- --test-threads=1
cargo test -p ccr --test commands -- codex_auth -- --test-threads=1
cargo test -p ccr --test commands -- grok -- --test-threads=1
cargo test -p ccr --test commands -- help -- --test-threads=1
cargo test -p ccr-tui -- --test-threads=1
just tauri-bindings-check
just tauri-command-inventory
just tauri-command-inventory-check
just frontend-check-quick
cd ccr-vscode && npm test
```

跨层完成后再 `just lint-strict`（含 `secret-write-check`）。密钥：JSON/DTO/UI 不得含 fixture token。

## Risky files

| 路径 | 风险 |
| --- | --- |
| `application/auth_off.rs` | 误 spawn login；超时挂起；成功后未删 backup |
| Codex `login_prep_codex_dirs` | 探测与删除范围不一致 |
| `tui_config.rs` | 删除 `OpencodeAuth` 变体导致旧 toml 整文件回落默认 |
| `check_secret_writes.py` | 漏删 OpenCode 路径后 `lint-strict` 崩溃 |
| `dispatch.rs` `show_version` | 残留 opencode 宣传 |
| `docs/.vitepress/config.mjs` | 死链导致 docs build 失败 |
| `ClaudeAuthView.vue` | `can_off` 与 `can_auth_off` 串台 |
| `handler_registry.rs` | 漏注册；未跑 command inventory |

回滚点：核与 CLI 可单独保留；UI/TUI 可先藏按钮。OpenCode 删除与 auth off 核可分提交。

## Before start

- 已根据 `research/review-disposition.md` 修订 `prd.md` / `design.md` / 本文件。
- 实施前读 `trellis-before-dev` 与 jsonl 中的 spec。
- 不调用官方 login。不删用户 OpenCode 磁盘数据。
