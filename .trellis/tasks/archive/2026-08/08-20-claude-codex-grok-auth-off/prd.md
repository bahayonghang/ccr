# Claude / Codex / Grok auth off

## Goal

用户能用 `ccr claude auth off`、`ccr codex auth off`、`ccr grok auth off` 登出当前官方运行时登录，使本机 native CLI 回到未登录（或回退到用户自有环境变量），而不删除 CCR 已保存的账号快照和 profile 定义。

该命令与 `ccr <platform> profile off` 对偶：`profile off` 退出 profile mode 并清 CCR 路由；`auth off` 退出官方登录态。

同期删除整个 `ccr opencode` 命令组与 OpenCode Auth TUI，ccr-ui OpenCode 配置页保留。

## User Value

共享机器、换账号或官方 token 损坏时，不必手工删凭据文件，也不必分别记住 `claude auth logout` / `codex logout` / `grok logout`。已保存的 CCR 账号仍可用 `auth switch` 恢复。主 TUI 不再露出已停用的 OpenCode Auth 页。

## Decisions

| ID | 决定 |
| --- | --- |
| D1 | 混合执行：CCR 能安全写的文件存储走自有写核（backup + 删除 + 回滚）；CCR 写不了的存储 spawn 官方 logout。Grok `auth.json` 仅允许本命令删除，仍不碰 `mcp_credentials.json`。不调用官方 login。 |
| D2 | `auth off` 与 `profile off` 独立。不调用 `profile_off_for_platform`，不改 profile 指针和 CCR 路由。 |
| D3 | Codex 无论是否处于第三方 profile，都清除整个运行期凭据。指针与 `config.toml` 路由不变。运行时 key 消失后需再次 `profile switch` 写回。 |
| D4 | 表面：CLI 三家；TUI Claude/Codex/Grok Auth；ccr-ui Claude/Codex/Grok Auth；VS Code 仅 CLI 包装（与 `execProfileOff` 同层，无新 webview / 无新贡献命令）。 |
| D5 | 新建 Grok Auth 面：ccr-ui `/grok/auth` + TUI Grok Auth 标签。只展示当前官方会话状态和 auth off。 |
| D6 | 删除 OpenCode Auth TUI（主 TUI 页签、无子命令启动器、`TuiLaunchers.opencode_auth`）。 |
| D7 | 删除整个 `ccr opencode` 命令组（含 `auth import-codex`）以及仅被该面使用的 Auth/Quota/Usage 服务。 |
| D8 | Windows/Linux 文件路径只删 `.credentials.json`，不重置 Claude first-launch / onboarding。macOS spawn 保持官方 `claude auth logout` 副作用。 |
| D9 | ccr-ui 确认分级：Claude/Codex auth off 为 `warning`（可用已保存快照 `auth switch` 恢复）；Grok auth off 为 `danger`（CCR 无账号快照，只能官方 `grok login`）。 |
| D10 | 文件路径成功提交后删除 `$CCR_ROOT/backups/auth-off/` 本次快照目录，不把官方凭据明文长期留在 CCR_ROOT。失败未 commit 时 Drop 回滚。native 路径不建 backup。 |

D1 路径：

| 平台 | CCR 写核 | spawn 官方 logout |
| --- | --- | --- |
| Claude | Windows / Linux 删除 `ClaudeRuntimePaths.credentials_file` | macOS：`claude auth logout` |
| Codex | `cli_auth_credentials_store = file` 时删除运行期 `auth.json` | `keyring` / `auto`：`codex logout` |
| Grok | 删除 `$GROK_HOME/auth.json` | 不需要 |

spawn 失败（官方 CLI 不在 PATH、超时、非零退出）则中止，不改 CCR 快照。文件路径失败回滚。JSON 报告 `file` 或 `native_logout`，不含密钥。

## Confirmed Facts

来源：`research/official-logout.md`、`research/codebase-auth-off-gap.md`、`research/opencode-auth-surface.md`、`.trellis/spec/ccr-cli/backend/profile-off-login-prep.md`。

| 主题 | 事实 |
| --- | --- |
| 官方登出 | Claude：`claude auth logout`；`/logout` 另重置 first-launch。Codex：`codex logout` 清 ChatGPT 与 API key。Grok：`grok logout` 清会话，之后回退 `XAI_API_KEY` |
| 默认文件 | Claude Win/Linux：`.credentials.json`。Codex file：`auth.json`。Grok：`$GROK_HOME/auth.json` |
| CCR 不能写 | Claude macOS Keychain（`claude_auth_service.rs` macOS 分支）。Codex `keyring`/`auto` 现有错误提示用户跑 `codex logout` |
| profile off | Claude 不碰 `.credentials.json`。Codex 仅 login-prep 时删 `auth.json`。Grok 文档约定不读写 `auth.json` |
| 现有 Auth CLI | Claude/Codex 有 save/list/switch/delete/current。Grok 只有 profile 组。`ccr grok` 无子命令打印帮助 |
| OpenCode | TUI 页签 + `ccr opencode` + `import-codex`；Tauri `opencode_*` 只读写配置，不调用 `OpenCodeAuthService` |
| VS Code | 已有 `execProfileOff` 服务包装，无独立贡献命令；可写 profile 动作限于 Claude/Codex（`extension-surface-contracts.md`） |
| 确认框 | ccr-ui 禁止原生 `confirm()`。`danger` = 删除/不可逆；`warning` = 可重试（`confirm-interaction-contracts.md`） |
| TUI 页签 | 顺序由 `crates/ccr-config/src/managers/tui_config.rs` 的 `TuiTabId` / `DEFAULT_TAB_ORDER` 决定。`Usage` 已 deprecated，`load()` 过滤以保住自定义排序 |
| Codex 目录 | `CodexPlatform::login_prep_codex_dirs()` 在 `CODEX_HOME` 重定向且未设 `CCR_CODEX_DIR` 时返回沙箱目录 + 默认 `~/.codex` |
| 审阅 | `research/review-disposition.md` |

## Requirements

- R1：三家 CLI 提供 `ccr {claude,codex,grok} auth off [--json]`，语义为登出当前官方运行时登录。Grok 另提供 `ccr grok auth current [--json]` 供页面读取。`ccr grok auth` 无嵌套动作时，有 TUI launcher 则进入 Grok Auth 标签，否则打印帮助。
- R2：不删除 CCR 已保存账号快照，不删除 profile 定义，不修改 profile 指针，不调用 `profile off`。
- R3：不删除用户自有环境变量和 Claude 非托管源。不删除 Grok `mcp_credentials.json`。不改 Windows/Linux 的 Claude onboarding 字段（D8）。
- R4：file 路径在无凭据文件时成功、`changed=false`、不创建 backup、不 spawn。native 路径因不可观察，每次成功 spawn 官方 logout 后 `changed=true`；官方 logout 可安全重复，退出码 0 即成功。
- R5：日志、CLI JSON、Tauri DTO、UI 文案不含密钥。结果含 `path`: `file` 或 `native_logout`。
- R6：file 路径写盘前备份；写失败则回滚且不 commit；成功 commit 后按 D10 删除本次快照目录。钥匙串路径 spawn 官方 logout；官方 CLI 缺失、超时或非零退出则失败。不调用官方 login。
- R7：Claude / Grok 在第三方 profile 下只清官方会话文件，profile 路由继续有效（Claude 托管 env 仍在；Grok `[model.custom]` 仍在）。
- R8：Codex 在第三方 profile 下仍清除整个运行期凭据（D3）。`--json` 可报告仍存在的 profile 指针，提示需 `profile switch` 恢复 key。该提示不算失败。
- R9：CLI、TUI Auth、ccr-ui Auth、VS Code 服务包装均调用同一写核。UI 确认分级见 D9；取消不写盘、不 spawn。能力位字段名为 `can_auth_off`，处理函数名为 `handleAuthOff`，不复用 profile 的 `can_off` / `handleOff`。VS Code 不加新 webview、不加新 `package.json` 贡献命令。
- R10：Grok Auth 页/标签只展示非密钥会话状态和 auth off，无 save/list/switch。
- R11：删除 OpenCode Auth TUI、`Commands::OpenCode`、`import-codex`，以及仅被该面使用的 OpenCode auth/quota/usage 服务。`--help`、`ccr help`、`ccr version`、文档、VitePress 侧边栏、命令面板不再出现 `ccr opencode`。ccr-ui `/opencode` 配置页保留。

## Acceptance Criteria

- [ ] AC1（R1）：file 存储且官方已登录时，`ccr <platform> auth off` 后 current 报告未登录（Grok 无会话，可回退 `XAI_API_KEY`）。native 路径（Claude macOS、Codex keyring/auto）以官方 logout 退出码 0 为通过，不要求 CCR current 能观察钥匙串。
- [ ] AC2（R2）：已保存账号列表与内容不变；profile 列表与 `current_config` 不变。
- [ ] AC3（R4）：file 路径重复执行成功，第二次 `changed=false` 且不 spawn。native 路径重复执行仍成功（退出码 0），`changed` 可为 `true`。
- [ ] AC4（R5）：输出与 `--help` 不含密钥；help 说明登出官方运行时登录。
- [ ] AC5（R8）：Codex 第三方 profile 下 off 后：file 则 `login_prep_codex_dirs()` 内 `auth.json` 均不存在，或 native logout 成功；指针与 `config.toml` 第三方路由仍在。
- [ ] AC6（R9, R10）：TUI 与 ccr-ui 的 Claude/Codex/Grok Auth 可完成 off；确认取消不写盘、不 spawn。Claude/Codex 确认 `warning`，Grok 确认 `danger`。Grok Auth 页无 save/list/switch。
- [ ] AC7（R9）：VS Code `execAuthOff(platform)`（或三家显式包装）发出 `ccr <platform> auth off --json`。无新贡献命令。不要求扩展内用户入口。
- [ ] AC8（R11）：主 TUI 无 OpenCode Auth 页签。`ccr opencode` 与 `ccr opencode auth import-codex` 不再合法。`--help`、`ccr help`、`ccr version`、文档、VitePress 侧边栏、命令面板无该入口。`/opencode` 配置页仍可用。
- [ ] AC9（R3）：Windows/Linux 上 Claude auth off 不修改 `~/.claude.json` 的 onboarding 字段。
- [ ] AC10（R3）：Grok auth off 后 `mcp_credentials.json` 字节不变（若原先存在）。
- [ ] AC11（R6）：钥匙串路径下官方 CLI 不在 PATH 时命令失败，CCR 账号快照不变。
- [ ] AC12（R6, D10）：file 路径写入失败时原凭据文件恢复且无残留半写；成功后 `$CCR_ROOT/backups/auth-off/` 下本次快照目录不存在。
- [ ] AC13（R7）：Claude 或 Grok 处于第三方 profile 时执行 auth off 后，profile 指针不变，且 Claude `settings.json` 托管 env / Grok `[model.custom]` 仍在。

## Out of Scope

- Gemini / Droid / Qwen / OpenCode 的 `auth off`。
- 代理官方 login / setup-token / 刷新 token。
- 删除 CCR 已保存账号快照（现有 `auth delete`）。
- 删除 Grok `mcp_credentials.json`。
- 修改上游登录优先级。
- VS Code 新 webview、新编辑器面板、新贡献命令。
- Grok Auth save/list/switch。
- 删除 ccr-ui OpenCode providers / MCP / settings。
- 复现 Claude `/logout` 的 first-launch 重置（文件路径）。

## Technical Notes

写核、DTO、spawn 与表面接线见 `design.md`。执行顺序与验证命令见 `implement.md`。研究见 `research/`。
