# 跨平台功能面差异矩阵（AC1 / AC5）

普查日期：2026-08-24。源：仍在树内的 Vue 实现（`census-raw.json`）。项的原子化遵循 `design.md` §5.1。无未确认格：每个「平台 × 功能面」格均已抽取；该平台无此面则整列为空（本表只列有源文件的平台）。

方法：字段（模板绑定 / 表单键）、操作（按钮与菜单动作）、校验、IPC wrapper、文案 key、分支状态。共有项归属 `base`，单平台项归属 `config.<platform>`，多平台非全有项并列 `config.*`。

Grok Settings 的权威字段是契约中的 11 个 dotted key（`models.default` 等）。普查脚本从 i18n 路径抽出的 `formKeys` 可能混入文案段名，以契约字段为准。

## Auth 判定（AC2 / R6）

### 三项输入

1. **比值（全统一假设）**  
   Claude OAuth 账号快照 + Codex OAuth/配额/Provider 向导 + Grok token 会话若塞进同一个 base，实际条件位置将覆盖账号列表、OAuth 向导、配额、过滤器、重命名、session save、credentials 文件、profile off 等。理论分支点按 §5.1 拆出后 ≥40；实际分支点同量级。比值无法压到已验证的 `BaseSlashCommands` 密度之下。

2. **保留重复的成本**  
   三平台都要改的共性行为：refresh、auth-off、session status、loading、confirm-off。条目约 5，平台 3，编辑点约 15。auth-off 已有独立契约与 domain wrapper。

3. **统一后的条件密度**  
   全统一 base 预估 800+ 行 / 40+ 分支 ≈ 0.05，高于 `BaseSlashCommands`（hideChrome / features 少量开关）。OAuth 向导无法用可选字段表达，会变成平台名分支。

### 结论：部分统一

子集（覆盖 3 个平台、≥3 项）：session status、refresh、auth-off、local-only、confirm-off。实现：`src/configs/auth.ts` + `BaseAuth`。Grok Auth 收敛为薄壳。Claude Auth 与 Codex Auth 的 OAuth 账号管理保留 Vue，不计入 AC3 基线（1,179 + 958 = 2,137 行）。

无 `src/configs/auth.ts` 的全量 OAuth 模型；`auth.ts` 只承载 session 子集。

## 行数对比（AC3）

| 口径 | 行数 |
| --- | --- |
| PRD 基线（20 个重复实现） | 15,672 |
| 排除保留的 Auth Vue（Claude 1,179 + Codex 958） | 2,137 |
| 调整后基线 | 13,535 |
| 新建 base + per-surface config + 薄壳（不含搬迁的 MCP 面板） | 约 3,400 |
| 预估目标区间 | 6,000–7,500 |

低于区间的原因：统一层按 config 字段驱动表单/列表/CRUD，不把每个平台编辑器的全部 JSX 再写一遍。Profiles 共享层与 MCP 管理面板是复用/搬迁，不计入新建。视图子任务的薄壳可继续补平台独有编辑器。

## 验证矩阵（AC6）

未迁 Vue 壳的格：用 config/props 映射 + base smoke，不做完整 UI。

| 平台 | settings | profiles | auth | mcp | agents | plugins | commands |
| --- | --- | --- | --- | --- | --- | --- | --- |
| claude | config+`BaseSettings` smoke | config+`BaseProfiles` | session 子集 `BaseAuth`；OAuth 保留 Vue | 无独立重复文件（走 mcp-manager） | config+`BaseAgents` | config+`BasePlugins` | config+`BaseCommands` |
| codex | config+`BaseSettings` smoke | config+`BaseProfiles` | session 子集；OAuth 保留 Vue | 补齐 STDIO/HTTP/stats/auth/toolScope 后接入 `BaseMcp` | 补齐 projectContext 后接入 `BaseAgents` | 无独立重复文件 | 无 |
| grok | config+`BaseSettings` smoke | config+`BaseProfiles` | `BaseAuth` 薄壳 | 无此面 | 无此面 | 无此面 | 无此面 |
| opencode | config+`BaseSettings` smoke | 无此面 | 无此面 | 接入 `BaseMcp` | 接入 `BaseAgents` | 接入 `BasePlugins` | 接入 `BaseCommands` |
| gemini | 无此面 | 无此面 | 无此面 | generic 补齐后 `BaseMcp` | generic 补齐后 `BaseAgents` | generic 补齐后 `BasePlugins` | 无（slash-commands 已统一） |

无未验证格：无此面标「无此面」；有此面均有 config 映射或保留判定。

## 追溯说明（AC5）

下表「统一后位置」列与差异项同行。映射规则：

- `base` → 对应 `features/platform/**/Base*.tsx` 与 `visibleSettingsFields` / 列表 CRUD
- `config.<p>` → `src/configs/<surface>.ts` 的该平台导出
- Auth 非 session 项 → 保留的 Vue 文件

---

下列为普查正文。

## settings

| 功能面 | 维度 | 项 | claude | grok | codex | opencode | 归属 | 统一后位置 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| settings | 字段 | `alwaysThinkingEnabled` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 字段 | `analytics` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `approval_policy` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `autoUpdates` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 字段 | `autoUpdatesChannel` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 字段 | `autoupdate` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 字段 | `availableModels` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 字段 | `back` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 字段 | `check_for_update_on_startup` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `cleanupPeriodDays` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 字段 | `cli` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 字段 | `conflict` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 字段 | `customModels` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 字段 | `custom_models` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 字段 | `defaultAgent` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 字段 | `developer_instructions` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `disable_response_storage` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `effortLevel` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 字段 | `experimental_use_rmcp_client` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `eyebrow` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 字段 | `features` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `feedback` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `fields` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 字段 | `file_opener` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `footer` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 字段 | `helpers` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 字段 | `hide_agent_reasoning` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `history` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `includeCoAuthoredBy` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 字段 | `instructions` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `instructionsText` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 字段 | `keybindsJson` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 字段 | `language` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 字段 | `loading` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 字段 | `localOnly` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 字段 | `managed` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 字段 | `maxOutputTokens` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 字段 | `maxThinkingTokens` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 字段 | `messages` | — | ✓ | ✓ | — | config.grok,config.codex | src/configs/settings.ts |
| settings | 字段 | `model` | ✓ | ✓ | ✓ | ✓ | base | `BaseSettings` / `visibleSettingsFields` |
| settings | 字段 | `model_auto_compact_token_limit` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `model_context_window` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `model_provider` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `model_reasoning_effort` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `model_reasoning_summary` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `model_verbosity` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `mouse` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 字段 | `options` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 字段 | `permissionJson` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 字段 | `personality` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `placeholders` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 字段 | `prefersReducedMotion` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 字段 | `reload` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 字段 | `respectGitignore` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 字段 | `sandbox_mode` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `sandbox_workspace_write` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `save` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 字段 | `saving` | — | ✓ | ✓ | — | config.grok,config.codex | src/configs/settings.ts |
| settings | 字段 | `security` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `serverHostname` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 字段 | `serverMdns` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 字段 | `serverPort` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 字段 | `sessionUi` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 字段 | `share` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 字段 | `shell_environment_policy` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `showSpinnerTree` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 字段 | `showTurnDuration` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 字段 | `show_raw_agent_reasoning` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `skipDangerousModePermissionPrompt` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 字段 | `smallModel` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 字段 | `snapshot` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 字段 | `source` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 字段 | `spinnerTipsEnabled` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 字段 | `status` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 字段 | `subtitle` | — | ✓ | ✓ | — | config.grok,config.codex | src/configs/settings.ts |
| settings | 字段 | `suppress_unstable_features_warning` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `tabs` | — | ✓ | ✓ | — | config.grok,config.codex | src/configs/settings.ts |
| settings | 字段 | `terminalProgressBarEnabled` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 字段 | `theme` | ✓ | — | — | ✓ | config.claude,config.opencode | src/configs/settings.ts |
| settings | 字段 | `themeOptions` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 字段 | `title` | — | ✓ | ✓ | — | config.grok,config.codex | src/configs/settings.ts |
| settings | 字段 | `tools` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `toolsJson` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 字段 | `tui` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `ui` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `validation` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 字段 | `value` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 字段 | `web_search` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 字段 | `worktreeOptions` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 字段 | `worktrees` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 操作 | `addEnvVar` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 操作 | `addTag('availableModels', form.availableModels)` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 操作 | `addTag('permAdditionalDirs', permAdditionalDirs)` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 操作 | `addTag('permAllow', permAllow)` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 操作 | `addTag('permDeny', permDeny)` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 操作 | `addTag('sandboxAllowedDomains', sandboxAllowedDomains)` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 操作 | `addTag('sandboxExcludedCmds', sandboxExcludedCmds)` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 操作 | `changeTab('source')` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 操作 | `changeTab(tab.key)` | ✓ | ✓ | ✓ | — | config.claude,config.grok,config.codex | src/configs/settings.ts |
| settings | 操作 | `envEntries.splice(idx, 1)` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 操作 | `form.availableModels.splice(i, 1)` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 操作 | `handleSave` | ✓ | ✓ | ✓ | — | config.claude,config.grok,config.codex | src/configs/settings.ts |
| settings | 操作 | `loadSettings` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 操作 | `permAdditionalDirs.splice(i, 1)` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 操作 | `permAllow.splice(i, 1)` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 操作 | `permDeny.splice(i, 1)` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 操作 | `reloadLatest` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 操作 | `sandboxAllowedDomains.splice(i, 1)` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 操作 | `sandboxExcludedCmds.splice(i, 1)` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 操作 | `saveAll` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 操作 | `setBooleanValue('cli.auto_update', option.value)` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 操作 | `setBooleanValue('cli.show_tips', option.value)` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 操作 | `setBooleanValue('session.load_envrc', option.value)` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | IPC 命令 | `getClaudeSettings` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | IPC 命令 | `getClaudeSettingsRaw` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | IPC 命令 | `getCodexConfig` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | IPC 命令 | `getCodexConfigRaw` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | IPC 命令 | `getCurrentEnvironment` | ✓ | ✓ | ✓ | — | config.claude,config.grok,config.codex | src/configs/settings.ts |
| settings | IPC 命令 | `getOpenCodeConfig` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | IPC 命令 | `getOpenCodeTuiSettings` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | IPC 命令 | `grokApi` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | IPC 命令 | `listClaudeSettingsLayers` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | IPC 命令 | `listCodexConfigLayers` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | IPC 命令 | `listOpenCodeThemes` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | IPC 命令 | `saveClaudeSettingsRaw` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | IPC 命令 | `saveCodexConfigRaw` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | IPC 命令 | `updateClaudeSettings` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | IPC 命令 | `updateCodexConfig` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | IPC 命令 | `updateOpenCodeConfig` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | IPC 命令 | `updateOpenCodeTuiSettings` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 校验规则 | `pattern` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 校验规则 | `required:` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 校验规则 | `validate` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `,` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `Instructions` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 文案 key | `OpenCode 设置已保存` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 文案 key | `Runtime config · opencode.json` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 文案 key | `Server / tools / permissions` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 文案 key | `TUI config · tui.json` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.back` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.env.add` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.env.empty` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.git.commitAttribution` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.git.includeCoAuthored` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.git.prAttribution` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.model.addModel` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.model.alwaysThinking` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.model.availableModels` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.model.defaultModel` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.model.effortLevel` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.model.maxOutputTokens` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.model.maxThinkingTokens` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.model.noOverride` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.permissions.additionalDirs` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.permissions.allow` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.permissions.defaultMode` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.permissions.deny` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.permissions.skipDangerous` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.sandbox.allowLocalBinding` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.sandbox.allowedDomains` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.sandbox.autoAllowBash` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.sandbox.enabled` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.sandbox.excludedCommands` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.save` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.saveSuccess` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.saving` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.tabs.env` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.tabs.git` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.tabs.model` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.tabs.permissions` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.tabs.sandbox` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.tabs.ui` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.title` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.ui.autoUpdates` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.ui.cleanupDays` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.ui.language` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.ui.misc` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.ui.progressBar` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.ui.reducedMotion` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.ui.respectGitignore` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.ui.showTurnDuration` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.ui.spinnerTips` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.ui.spinnerTree` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.ui.theme` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `claudeSettings.ui.updateChannel` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.features.analytics` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.features.experimentalRmcp` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.features.featureFlags` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.features.feedback` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.features.historyMaxBytes` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.features.historyPersistence` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.messages.loadFailed` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.messages.saveFailed` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.messages.saveSuccess` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.model.autoCompactLimit` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.model.contextWindow` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.model.model` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.model.modelPlaceholder` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.model.modelProvider` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.model.modelProviderPlaceholder` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.model.personality` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.model.reasoningEffort` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.model.reasoningSummary` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.model.verbosity` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.saving` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.security.approvalPolicy` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.security.disableResponseStorage` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.security.networkAccess` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.security.sandboxMode` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.security.shellIncludeOnly` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.security.shellIncludeOnlyHint` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.security.shellIncludeOnlyPlaceholder` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.security.writableRoots` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.security.writableRootsHint` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.security.writableRootsPlaceholder` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.subtitle` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.tabs.features` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.tabs.model` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.tabs.security` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.tabs.tools` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.tabs.ui` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.title` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.tools.developerInstructions` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.tools.developerInstructionsPlaceholder` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.tools.fileOpener` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.tools.instructions` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.tools.instructionsPlaceholder` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.tools.toolWebSearch` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.tools.viewImage` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.tools.webSearch` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.ui.alternateScreen` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.ui.animations` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.ui.checkForUpdate` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.ui.hideAgentReasoning` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.ui.notifications` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.ui.showRawAgentReasoning` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.ui.showTooltips` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `codex.settings.ui.suppressUnstableWarning` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `common.back` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `common.cancel` | ✓ | ✓ | ✓ | — | config.claude,config.grok,config.codex | src/configs/settings.ts |
| settings | 文案 key | `common.loading` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `common.save` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `default_agent` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.back` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.cli.description` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.cli.eyebrow` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.cli.title` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.conflict.description` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.conflict.reload` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.conflict.title` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.customModels.baseUrl` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.customModels.empty` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.customModels.eyebrow` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.customModels.model` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.customModels.sourceAction` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.customModels.title` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.eyebrow` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.fields.autoCompact` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.fields.autoUpdate` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.fields.channel` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.fields.defaultModel` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.fields.forkWorktree` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.fields.loadEnvrc` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.fields.newSessionWorktree` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.fields.reasoningEffort` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.fields.showTips` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.fields.theme` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.footer.formatting` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.footer.moreConfig` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.footer.openSource` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.helpers.autoCompact` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.helpers.autoUpdate` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.helpers.channel` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.helpers.defaultModel` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.helpers.forkWorktree` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.helpers.loadEnvrc` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.helpers.newSessionWorktree` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.helpers.reasoningEffort` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.helpers.showTips` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.helpers.theme` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.loading` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.localOnly.description` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.localOnly.environment` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.localOnly.title` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.managed.action` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.managed.description` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.managed.rejectedTitle` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.managed.title` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.messages.loadFailed` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.messages.saveFailed` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.messages.saveSuccess` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.model.description` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.model.eyebrow` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.model.title` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.options.currentValue` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.options.disabled` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.options.enabled` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.options.unset` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.placeholders.defaultModel` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.reload` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.save` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.saving` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.sessionUi.sessionDescription` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.sessionUi.sessionEyebrow` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.sessionUi.sessionTitle` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.sessionUi.uiDescription` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.sessionUi.uiEyebrow` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.sessionUi.uiTitle` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.source.noBackup` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.source.policyNotice` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.status.activation` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.status.exists` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.status.file` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.status.missing` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.status.pending` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.status.pendingCount` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.subtitle` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.tabs.cli` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.tabs.label` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.tabs.model` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.tabs.sessionUi` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.tabs.source` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.themeOptions.${option}` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.title` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.validation.autoCompact` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.worktreeOptions.${option}` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.worktrees.description` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.worktrees.eyebrow` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.settings.worktrees.title` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.states.activation.${key}` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `grok.states.unknown` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 文案 key | `keybinds JSON` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 文案 key | `mDNS` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 文案 key | `permission JSON` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 文案 key | `settingsRaw.discard` | ✓ | ✓ | ✓ | — | config.claude,config.grok,config.codex | src/configs/settings.ts |
| settings | 文案 key | `settingsRaw.discardMessage` | ✓ | ✓ | ✓ | — | config.claude,config.grok,config.codex | src/configs/settings.ts |
| settings | 文案 key | `settingsRaw.discardTitle` | ✓ | ✓ | ✓ | — | config.claude,config.grok,config.codex | src/configs/settings.ts |
| settings | 文案 key | `settingsRaw.sourceTab` | ✓ | — | ✓ | — | config.claude,config.codex | src/configs/settings.ts |
| settings | 文案 key | `settingsRaw.unsupportedEnvironment` | ✓ | — | ✓ | — | config.claude,config.codex | src/configs/settings.ts |
| settings | 文案 key | `share` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 文案 key | `small_model` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 文案 key | `tools JSON` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 文案 key | `update:modelValue` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 文案 key | `主机名` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 文案 key | `主题` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 文案 key | `保存全部` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 文案 key | `启用 autoupdate` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 文案 key | `启用 mouse` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 文案 key | `启用 snapshot` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 文案 key | `手动` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 文案 key | `拆分管理 ` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 文案 key | `模型` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 文案 key | `每行一个路径或 glob，会进入 ` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 文案 key | `禁用` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 文案 key | `端口` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 文案 key | `自动` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 文案 key | `设置` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 分支状态 | `!isTuiNotificationEventsConfig` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 分支状态 | `activeTab !== 'source'` | ✓ | ✓ | ✓ | — | config.claude,config.grok,config.codex | src/configs/settings.ts |
| settings | 分支状态 | `activeTab !== 'source' && !localOnly` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 分支状态 | `activeTab === 'source'` | ✓ | ✓ | ✓ | — | config.claude,config.grok,config.codex | src/configs/settings.ts |
| settings | 分支状态 | `envEntries.length === 0` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 分支状态 | `form.availableModels.length > 0` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 分支状态 | `form.features && Object.keys(form.features).length > 0` | — | — | ✓ | — | config.codex | src/configs/settings.ts |
| settings | 分支状态 | `hasUnknownOption('cli.channel', channels)` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 分支状态 | `hasUnknownOption('hints.fork_worktree_mode', worktreeModes)` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 分支状态 | `hasUnknownOption('hints.new_session_worktree_mode', worktreeModes)` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 分支状态 | `hasUnknownOption('models.default_reasoning_effort', reasoningEfforts)` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 分支状态 | `hasUnknownOption('ui.theme', themes)` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 分支状态 | `loading` | ✓ | — | ✓ | — | config.claude,config.codex | src/configs/settings.ts |
| settings | 分支状态 | `localOnly` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 分支状态 | `permAdditionalDirs.length > 0` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 分支状态 | `permAllow.length > 0` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 分支状态 | `permDeny.length > 0` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 分支状态 | `sandboxAllowedDomains.length > 0` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 分支状态 | `sandboxExcludedCmds.length > 0` | ✓ | — | — | — | config.claude | src/configs/settings.ts |
| settings | 分支状态 | `saveState === 'conflict'` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 分支状态 | `saveState === 'managed_locked'` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 分支状态 | `saving` | — | — | — | ✓ | config.opencode | src/configs/settings.ts |
| settings | 分支状态 | `settings?.custom_models.length` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 分支状态 | `settings?.managed_keys_locked` | — | ✓ | — | — | config.grok | src/configs/settings.ts |
| settings | 分支状态 | `toast` | — | — | ✓ | — | config.codex | src/configs/settings.ts |

## profiles

| 功能面 | 维度 | 项 | claude | grok | codex | 归属 | 统一后位置 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| profiles | 字段 | `auth_mode` | ✓ | — | ✓ | config.claude,config.codex | src/configs/profiles.ts |
| profiles | 字段 | `base_url` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 字段 | `description` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 字段 | `enabled` | ✓ | — | ✓ | config.claude,config.codex | src/configs/profiles.ts |
| profiles | 字段 | `name` | ✓ | — | ✓ | config.claude,config.codex | src/configs/profiles.ts |
| profiles | 字段 | `provider` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 字段 | `provider_type` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 字段 | `tagsInput` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 操作 | `handleAdd` | — | ✓ | ✓ | config.grok,config.codex | src/configs/profiles.ts |
| profiles | 操作 | `handleOff` | ✓ | ✓ | ✓ | base | `BaseProfiles` |
| profiles | 操作 | `openAddForm()` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 操作 | `refreshProfiles` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 操作 | `refreshProfiles()` | ✓ | — | ✓ | config.claude,config.codex | src/configs/profiles.ts |
| profiles | 操作 | `resetFilters` | — | ✓ | ✓ | config.grok,config.codex | src/configs/profiles.ts |
| profiles | 操作 | `resetFilters()` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 操作 | `runRecovery` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | IPC 命令 | `addClaudeProfile` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | IPC 命令 | `addCodexProfile` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | IPC 命令 | `applyClaudeProfile` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | IPC 命令 | `applyCodexProfile` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | IPC 命令 | `claudeProfileOff` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | IPC 命令 | `codexProfileOff` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | IPC 命令 | `deleteClaudeProfile` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | IPC 命令 | `deleteCodexProfile` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | IPC 命令 | `exportClaudeProfiles` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | IPC 命令 | `exportCodexProfiles` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | IPC 命令 | `getClaudeProfilesRaw` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | IPC 命令 | `getCodexProfile` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | IPC 命令 | `getCodexProfilesRaw` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | IPC 命令 | `getCurrentEnvironment` | ✓ | ✓ | ✓ | base | `BaseProfiles` |
| profiles | IPC 命令 | `grokApi` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | IPC 命令 | `listClaudeProfiles` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | IPC 命令 | `listCodexModels` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | IPC 命令 | `listCodexProfiles` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | IPC 命令 | `saveClaudeProfilesRaw` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | IPC 命令 | `saveCodexProfilesRaw` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | IPC 命令 | `updateClaudeProfile` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | IPC 命令 | `updateCodexProfile` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.actions.off` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.addProfile` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.applyFailed` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.applyProfile` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.back` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.cancel` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.clearSearch` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.commandPaletteButton` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.commandPaletteShortcut` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.confirm.offMessage` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.confirm.offTitle` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.confirmApply` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.confirmDeleteBackupFootnote` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.createProfile` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.currentProfile` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.deleteConfirm` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.deleteFailed` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.deleteTooltip` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.descLabel` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.emptyDesc` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.emptyTitle` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.exportFailed` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.exportSuccess` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.fields.authMode` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.fields.baseUrl` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.fields.model` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.fields.name` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.fields.tags` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.groups.disabled` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.groups.enabled` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.loadFailed` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.loadFailedTitle` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.messages.offFailed` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.messages.offSuccess` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.notSet` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.operationFailed` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.overflowMenu` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.pinLimitReached` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.providerUnset` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.refreshFailedHint` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.refreshFailedTitle` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.reloadAction` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.renameConfirmBody` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.renameConfirmCta` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.renameConfirmTitle` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.renameConflict` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.retry` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.runtimeBanner.description` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.runtimeBanner.title` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.searchEmptyDesc` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.searchEmptyTitle` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.statStrip.authSplit` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.statStrip.authTitle` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.statStrip.healthHintIssues` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.statStrip.healthHintOk` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.statStrip.healthTitle` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.statStrip.profileSubtitle` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.statStrip.totalHint` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.subtitle` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.templateAuthModeSwitched` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.title` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.toolbar.actionsLabel` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `claudeProfiles.totalCount` | ✓ | — | — | config.claude | src/configs/profiles.ts |
| profiles | 文案 key | `codex.actions.delete` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.actions.off` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.addProfile` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.apply` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.authModes.${authMode \|\| ` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.authTokenHints.${form.auth_mode}` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.backToCodex` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.commandPaletteButton` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.commandPaletteShortcut` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.confirm.offMessage` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.confirm.offTitle` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.confirmApply` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.confirmDeleteBackupFootnote` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.customRelay` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.deleteConfirm` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.description` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.empty.clearFilters` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.empty.noResults` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.emptyHint` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.emptyState` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.exportFailed` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.exportSuccess` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.fields.authMode` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.fields.baseUrl` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.fields.model` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.fields.name` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.fields.tags` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.groups.disabled` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.groups.enabled` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.loadFailedTitle` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.messages.deleteSuccess` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.messages.envExportCopied` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.messages.envExportCopyFailed` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.messages.offFailed` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.messages.offSuccess` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.notAvailable` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.officialConfig` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.overflowMenu` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.pinLimitReached` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.refreshFailedHint` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.refreshFailedTitle` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.reloadAction` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.retry` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.runtimeBanner.description` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.runtimeBanner.title` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.statStrip.configModeHint` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.statStrip.healthHintIssues` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.statStrip.healthHintOk` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.statStrip.healthTitle` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.statStrip.profileSubtitle` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.statStrip.totalHint` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.subtitle` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.title` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.toolbar.actionsLabel` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.profiles.updateProfile` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.states.deleteFailed` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.states.loadFailed` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.states.saveFailed` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.status.configMode` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.status.currentConfig` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.status.notSet` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `codex.status.totalProfiles` | — | — | ✓ | config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `common.cancel` | ✓ | ✓ | ✓ | base | `BaseProfiles` |
| profiles | 文案 key | `common.export` | ✓ | — | ✓ | config.claude,config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `grok.dashboard.localOnly.description` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.dashboard.localOnly.environment` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.dashboard.localOnly.title` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.actions.add` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.actions.apply` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.actions.delete` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.actions.exportSummary` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.actions.off` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.actions.reload` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.actions.retry` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.back` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.commandPaletteButton` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.commandPaletteShortcut` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.confirm.applyMessage` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.confirm.applyTitle` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.confirm.deleteMessage` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.confirm.deleteTitle` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.confirm.forceDeleteAction` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.confirm.forceDeleteMessage` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.confirm.forceDeleteTitle` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.confirm.offDriftedMessage` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.confirm.offMessage` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.confirm.offTitle` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.driftBanner.${activation.value}.description` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.driftBanner.${activation.value}.title` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.empty.clearFilters` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.empty.hint` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.empty.noResults` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.empty.title` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.fields.authMode` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.fields.baseUrl` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.fields.description` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.fields.model` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.fields.name` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.fields.tags` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.groups.disabled` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.groups.enabled` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.loadFailedTitle` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.messages.applyFailed` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.messages.applySuccess` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.messages.createSuccess` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.messages.deleteFailed` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.messages.deleteSuccess` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.messages.exportSuccess` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.messages.loadFailed` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.messages.offFailed` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.messages.offSuccess` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.messages.recoveryFailed` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.messages.recoverySuccess` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.messages.saveFailed` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.messages.unexpectedResponse` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.messages.updateSuccess` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.overflowMenu` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.pinLimitReached` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.refreshFailedTitle` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.renameRecovery.${recovery.status}.action` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.renameRecovery.${recovery.status}.title` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.statStrip.authMode` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.statStrip.authModeHint` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.statStrip.current` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.statStrip.currentHint` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.statStrip.healthSummary` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.statStrip.healthTitle` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.statStrip.total` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.statStrip.totalHint` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.subtitle` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.title` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.toolbar.actionsLabel` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.unsafeDelete.manualRecovery` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.profiles.unsafeDelete.title` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.states.notSet` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `grok.states.unknown` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 文案 key | `profilesRaw.continue` | ✓ | — | ✓ | config.claude,config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `profilesRaw.edit` | ✓ | — | ✓ | config.claude,config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `profilesRaw.openWarningMessage` | ✓ | — | ✓ | config.claude,config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `profilesRaw.openWarningTitle` | ✓ | — | ✓ | config.claude,config.codex | src/configs/profiles.ts |
| profiles | 文案 key | `settingsRaw.unsupportedEnvironment` | ✓ | — | ✓ | config.claude,config.codex | src/configs/profiles.ts |
| profiles | 分支状态 | `!localOnly` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 分支状态 | `activation !== 'inactive'` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 分支状态 | `canOff` | ✓ | ✓ | ✓ | base | `BaseProfiles` |
| profiles | 分支状态 | `confirmDiffRows.length > 0` | ✓ | ✓ | ✓ | base | `BaseProfiles` |
| profiles | 分支状态 | `loading` | ✓ | ✓ | ✓ | base | `BaseProfiles` |
| profiles | 分支状态 | `localOnly` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 分支状态 | `recovery` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 分支状态 | `showRawEditor` | ✓ | — | ✓ | config.claude,config.codex | src/configs/profiles.ts |
| profiles | 分支状态 | `unsafeDeleteRecovery` | — | ✓ | — | config.grok | src/configs/profiles.ts |
| profiles | 分支状态 | `viewMode === 'list'` | ✓ | ✓ | ✓ | base | `BaseProfiles` |

## auth

| 功能面 | 维度 | 项 | claude | grok | codex | 归属 | 统一后位置 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| auth | 字段 | `json` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 操作 | `activeManagerTab = 'accounts'` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 操作 | `activeManagerTab = 'providers'` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 操作 | `handleAuthOff` | ✓ | ✓ | ✓ | base | `BaseAuth` session subset |
| auth | 操作 | `handleDelete(account.name)` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 操作 | `handleOff` | ✓ | — | ✓ | config.claude,config.codex | retain Claude/Codex Auth Vue; session subset in `BaseAuth` |
| auth | 操作 | `handleRefresh` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 操作 | `handleSave` | ✓ | — | ✓ | config.claude,config.codex | retain Claude/Codex Auth Vue; session subset in `BaseAuth` |
| auth | 操作 | `handleSwitch(account.name)` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 操作 | `openAddAccountModal()` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 操作 | `refresh` | — | ✓ | — | config.grok | src/configs/auth.ts |
| auth | 操作 | `refreshAll` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 操作 | `showSaveForm = false` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 操作 | `showSaveForm = true` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | IPC 命令 | `claudeAuthOff` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | IPC 命令 | `claudeProfileOff` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | IPC 命令 | `codexAuthOff` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | IPC 命令 | `codexProfileOff` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | IPC 命令 | `deleteClaudeAuth` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | IPC 命令 | `deleteCodexAuth` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | IPC 命令 | `getClaudeAuthCurrent` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | IPC 命令 | `getCodexAllQuotas` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | IPC 命令 | `getCodexAuthCurrent` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | IPC 命令 | `grokAuthCurrent` | — | ✓ | — | config.grok | src/configs/auth.ts |
| auth | IPC 命令 | `grokAuthOff` | — | ✓ | — | config.grok | src/configs/auth.ts |
| auth | IPC 命令 | `listClaudeAuthAccounts` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | IPC 命令 | `listClaudeProfiles` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | IPC 命令 | `listCodexAuthAccounts` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | IPC 命令 | `listCodexProfiles` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | IPC 命令 | `saveClaudeAuth` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | IPC 命令 | `switchClaudeAuth` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | IPC 命令 | `switchCodexAuth` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 校验规则 | `required` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `${unobservableLabels.length} 个不可观测层` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `${visibleSuppressors.length} 个可见竞争来源` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `API Key 批准记录` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `Access Token 到期` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `Claude 官方订阅` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `Claude 官方账号已保存` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `Issue 报告行为` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `Profile + 官方订阅` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `Profile 等待官方订阅` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `Profile 驱动（API key）` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `auth.confirmOffClaude` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `auth.confirmOffCodex` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `auth.confirmOffGrok` | — | ✓ | — | config.grok | src/configs/auth.ts |
| auth | 文案 key | `auth.confirmOffTitle` | ✓ | ✓ | ✓ | base | `BaseAuth` session subset |
| auth | 文案 key | `auth.off` | ✓ | ✓ | ✓ | base | `BaseAuth` session subset |
| auth | 文案 key | `auth.offDescription` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `auth.offFailed` | ✓ | ✓ | ✓ | base | `BaseAuth` session subset |
| auth | 文案 key | `auth.offSuccess` | ✓ | ✓ | ✓ | base | `BaseAuth` session subset |
| auth | 文案 key | `auth.offUnchanged` | ✓ | ✓ | ✓ | base | `BaseAuth` session subset |
| auth | 文案 key | `codex.actions.delete` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.accountOverview` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.backToCodex` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.featureComingSoon` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.filters.planOptions.all` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.filters.planOptions.plus` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.filters.planOptions.pro` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.filters.planOptions.team` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.filters.planOptions.unknown` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.filters.sortOptions.nameAsc` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.filters.sortOptions.savedDesc` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.filters.sortOptions.usedDesc` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.filters.statusOptions.all` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.filters.statusOptions.attention` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.filters.statusOptions.current` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.filters.statusOptions.virtual` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.loginState.apiKeyActive` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.loginState.loggedInUnsaved` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.loginState.notLoggedIn` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.off.action` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.off.confirm` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.off.description` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.off.failed` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.off.success` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.off.title` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.profileGuard.noCurrentProfile` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.refresh` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.status.currentAccount` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.status.loginState` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.status.noAccount` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.status.totalAccounts` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.switch` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.auth.title` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.states.deleteFailed` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.states.loadFailed` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `codex.states.saveFailed` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 文案 key | `common.cancel` | ✓ | ✓ | ✓ | base | `BaseAuth` session subset |
| auth | 文案 key | `common.refresh` | — | ✓ | — | config.grok | src/configs/auth.ts |
| auth | 文案 key | `grok.auth.sessionFile` | — | ✓ | — | config.grok | src/configs/auth.ts |
| auth | 文案 key | `grok.auth.signedIn` | — | ✓ | — | config.grok | src/configs/auth.ts |
| auth | 文案 key | `grok.auth.signedOut` | — | ✓ | — | config.grok | src/configs/auth.ts |
| auth | 文案 key | `grok.auth.subtitle` | — | ✓ | — | config.grok | src/configs/auth.ts |
| auth | 文案 key | `grok.auth.title` | — | ✓ | — | config.grok | src/configs/auth.ts |
| auth | 文案 key | `grok.dashboard.header.eyebrow` | — | ✓ | — | config.grok | src/configs/auth.ts |
| auth | 文案 key | `grok.dashboard.localOnly.description` | — | ✓ | — | config.grok | src/configs/auth.ts |
| auth | 文案 key | `grok.dashboard.localOnly.title` | — | ✓ | — | config.grok | src/configs/auth.ts |
| auth | 文案 key | `。` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `仅官方订阅运行时` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `例如 work / personal` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `例如 公司订阅 / 个人订阅` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `保存` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `保存、切换、删除 Claude Code 官方订阅账号快照；切换会更新 ${credentialsFile}，并只清理 CCR 托管的 Profile 设置。` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `保存中…` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `保存当前官方登录` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `保存当前登录` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `切换` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `切换后仍存在${authOwnershipLabel(source.ownership)}认证来源：${formatAuthSource(source)}（${authConfidenceLabel(source.confidence)}）` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `切换官方账号` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `删除` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `删除官方账号` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `到期` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `刷新` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `取消` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `名称` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `存在，仅作解释` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `官方契约` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `官方账号管理` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `尚未保存任何官方账号快照。` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `已保存` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `已保存账号` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `已保存账号快照` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `已切换到 ${name}` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `已切换到 ${name}，并清理 ${clearedCount} 个 CCR 托管设置` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `已删除 ${name}` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `已登录` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `已登录（已保存为 ${loginState.value.account_name}）` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `已登录（未保存）` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `已退出 Profile 并清理登录残留` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `当前` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `当前 Profile` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `当前必须已经通过 ` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `当前推定来源` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `当前生效` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `当前由 API key profile 控制` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `当前运行时官方登录` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `描述（可选）` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `操作` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `未发现可见竞争来源` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `未登录` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `未绑定` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `未观察到` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `未解析` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `未解析或存在同级歧义` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `正在加载账号信息…` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `每个快照都保存当前 ` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `状态` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `登录状态` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `确定切换到官方账号 "${name}" 吗？` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `确定删除官方账号 "${name}" 吗？` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `置信度` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `范围限于当前 CCR 进程和已解析的用户级文件。` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `覆盖同名账号` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `计费类型` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `订阅` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `订阅类型` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `认证来源诊断` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `请求失败` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `账号 UUID` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `账号名称` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `账号名称不能为空` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `运行时模式` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `返回 Claude Code` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `退出 Profile` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `退出 Profile 后仍存在${authOwnershipLabel(source.ownership)}认证来源：${formatAuthSource(source)}（${authConfidenceLabel(source.confidence)}）` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `退出当前 Profile 并清理会压制官方登录的 CCR 运行时残留？已保存的账号不会删除。` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `速率档位` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 文案 key | `邮箱` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 分支状态 | `account.description` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 分支状态 | `account.is_current` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 分支状态 | `activeManagerTab === 'accounts'` | — | — | ✓ | config.codex | retain `CodexAuthView.vue` |
| auth | 分支状态 | `authActionError` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 分支状态 | `canAuthOff` | ✓ | ✓ | ✓ | base | `BaseAuth` session subset |
| auth | 分支状态 | `canOff` | ✓ | — | ✓ | config.claude,config.codex | retain Claude/Codex Auth Vue; session subset in `BaseAuth` |
| auth | 分支状态 | `currentInfo` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 分支状态 | `loading` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 分支状态 | `localOnly` | — | ✓ | — | config.grok | src/configs/auth.ts |
| auth | 分支状态 | `runtimeSummary` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |
| auth | 分支状态 | `visibleSuppressors.length > 0` | ✓ | — | — | config.claude | retain `ClaudeAuthView.vue` |

## commands

| 功能面 | 维度 | 项 | claude | opencode | 归属 | 统一后位置 |
| --- | --- | --- | --- | --- | --- | --- |
| commands | 字段 | `agent` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 字段 | `description` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 字段 | `model` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 字段 | `name` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 字段 | `subtask` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 字段 | `template` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 操作 | `activeCategory = category` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 操作 | `activeCollection = collection` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 操作 | `handleCancel` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 操作 | `handleClearHistory` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 操作 | `handleClearOutput` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 操作 | `handleCopyOutput` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 操作 | `handleExecute` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 操作 | `handleToggleFavorite` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 操作 | `loadFavorite(favorite)` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 操作 | `loadHistoryItem(item)` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 操作 | `openCreate()` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 操作 | `openEdit(command)` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 操作 | `removeCommand(command)` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 操作 | `saveCommand` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 操作 | `setSelectedCommand(cmd.name)` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 操作 | `showModal = false` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | IPC 命令 | `addFavorite` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | IPC 命令 | `addOpenCodeCommand` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | IPC 命令 | `addRecentItem` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | IPC 命令 | `cancelCcrCommandJob` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | IPC 命令 | `clearRecentItems` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | IPC 命令 | `deleteOpenCodeCommand` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | IPC 命令 | `getFavorites` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | IPC 命令 | `getRecentItems` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | IPC 命令 | `listCommands` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | IPC 命令 | `listConfigs` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | IPC 命令 | `listOpenCodeCommands` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | IPC 命令 | `removeFavorite` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | IPC 命令 | `startCcrCommandJob` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | IPC 命令 | `updateOpenCodeCommand` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 校验规则 | `required` | ✓ | ✓ | base | `BaseCommands` |
| commands | 校验规则 | `required:` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | ` ` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `Built-in behavior` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 文案 key | `Command name 和 description 为必填项` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 文案 key | `Command 已创建` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 文案 key | `Command 已删除` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 文案 key | `Command 已更新` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 文案 key | `OpenCode 命令模板支持注入 shell 输出与文件内容。` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 文案 key | `agent` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 文案 key | `ccr` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `command name *` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 文案 key | `commands.addFavorite` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.args` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.argsPlaceholder` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.badgeArgs` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.badgeBlocked` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.badgeDanger` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.badgeReadOnly` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.badgeSafe` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.cancelJob` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.cardJobIdle` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.catalogTab` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.categoryAll` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.categoryBlocked` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.categoryDanger` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.categoryOther` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.categoryRead` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.categoryWrite` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.clear` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.clearHistory` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.clientPreview` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.clientPreviewCommandDescription` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.clientUnavailableDescription` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.clientUnavailableTitle` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.commandBlockedDescription` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.commandBlockedTitle` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.composerEyebrow` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.copy` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.dangerConfirmDescription` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.dangerConfirmTitle` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.description` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.duration` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.executing` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.exitCode` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.favorites` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.history` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.historyFailed` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.historySuccess` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.jobStatus` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.ledgerEyebrow` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.ledgerSubtitleActive` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.ledgerSubtitleIdle` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.ledgerTruncated` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.linesCount` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.noFavorites` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.noHistory` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.output` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.paletteSubtitle` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.paletteTitle` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.previewLabel` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.processing` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.readyDescription` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.readyTitle` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.removeFavorite` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.requiredArgsPlaceholder` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.run` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.runtimeClientPreview` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.runtimeReady` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.runtimeRunning` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.runtimeWeb` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.searchPlaceholder` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.selectCommand` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.selectCommandHint` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.selectConfig` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.stale` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.status.${status}` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.terminalOutput` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.title` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.unknownError` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.webUnavailableDetail` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `commands.whitelistBadge` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 文案 key | `description *` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 文案 key | `model` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 文案 key | `subtask` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 文案 key | `template` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 文案 key | `从测试、review、scaffold 这类高频动作开始封装。` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 文案 key | `保存` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 文案 key | `删除` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 文案 key | `取消` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 文案 key | `命令` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 文案 key | `命令模板支持 $ARGUMENTS、位置参数、shell 输出和文件引用。` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 文案 key | `强制以 subtask 方式执行` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 文案 key | `暂无自定义 Command` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 文案 key | `添加 Command` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 文案 key | `管理 markdown / JSON 形式的自定义命令模板，并展示 built-in command 覆盖语义。` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 文案 key | `编辑` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 文案 key | `编辑 Command` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 文案 key | `自定义命令可以覆盖 ` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 文案 key | `配合 ` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 分支状态 | `!canLoadPersistedCommand(favorite.command)` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 分支状态 | `!canLoadPersistedCommand(item.command)` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 分支状态 | `activeCollection === 'catalog'` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 分支状态 | `activeCollection === 'history'` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 分支状态 | `args.trim()` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 分支状态 | `command.agent` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 分支状态 | `command.subtask` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 分支状态 | `currentSnapshot` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 分支状态 | `filteredFavorites.length === 0` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 分支状态 | `filteredHistory.length === 0` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 分支状态 | `hasLedgerOutput` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 分支状态 | `isRunning` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 分支状态 | `ledgerTruncated` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 分支状态 | `loading` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 分支状态 | `runtimeUnavailable` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 分支状态 | `saving` | — | ✓ | config.opencode | src/configs/commands.ts |
| commands | 分支状态 | `selectedCommand === 'switch'` | ✓ | — | config.claude | src/configs/commands.ts |
| commands | 分支状态 | `selectedCommandInfo?.dangerous` | ✓ | — | config.claude | src/configs/commands.ts |

## mcp

| 功能面 | 维度 | 项 | codex | opencode | 归属 | 统一后位置 |
| --- | --- | --- | --- | --- | --- | --- |
| mcp | 字段 | `command` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 字段 | `enabled` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 字段 | `environmentJson` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 字段 | `headersJson` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 字段 | `id` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 字段 | `openai` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 字段 | `type` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 字段 | `url` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 操作 | `armOrDeleteSelected` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 操作 | `cancelEditor` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 操作 | `clearFilters` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 操作 | `copyCli('opencode mcp auth ${server.id}')` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 操作 | `copyCli('opencode mcp debug ${server.id}')` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 操作 | `copyCli('opencode mcp logout ${server.id}')` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 操作 | `draft.transport = 'http'` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 操作 | `draft.transport = 'stdio'` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 操作 | `focusFilter = item.value` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 操作 | `openCreate('local')` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 操作 | `openEdit(server)` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 操作 | `prefillDocsPreset` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 操作 | `removeServer(server.id)` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 操作 | `resetDraftFromBaseline` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 操作 | `saveServer` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 操作 | `selectServer(server)` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 操作 | `showModal = false` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 操作 | `startCreate('http')` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 操作 | `startCreate('stdio')` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 操作 | `stateFilter = item.value` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 操作 | `transportFilter = item.value` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 操作 | `void loadServers(true)` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 操作 | `void saveDraft()` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 操作 | `void toggleServer(server)` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | IPC 命令 | `addCodexMcpServer` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | IPC 命令 | `addOpenCodeMcpServer` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | IPC 命令 | `deleteCodexMcpServer` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | IPC 命令 | `deleteOpenCodeMcpServer` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | IPC 命令 | `listCodexMcpServers` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | IPC 命令 | `listOpenCodeMcpServers` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | IPC 命令 | `updateCodexMcpServer` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | IPC 命令 | `updateOpenCodeMcpServer` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 校验规则 | `required` | ✓ | ✓ | base | `BaseMcp` |
| mcp | 校验规则 | `required:` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 校验规则 | `validate` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `CLI handoff` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `Codex Config Reference` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `Codex MCP 控制台` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `Codex MCP 服务器已创建` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `Codex MCP 服务器已删除` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `Codex MCP 服务器已更新` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `Create` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `Docs MCP` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `Edit` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `HTTP transport` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `HTTP 模式需要 URL` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `Legacy` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `MCP 服务器` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `MCP 服务器已创建` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `MCP 服务器已删除` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `MCP 服务器已更新` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `OAuth-enabled remote server 登录。` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `OpenAI Docs MCP` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `OpenCode ` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `OpenCode 的 MCP OAuth 与调试动作本质上还是 CLI 能力，这里直接给你可执行命令。` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `STDIO 更适合本地工具链，HTTP 更适合远程或托管 MCP 服务。` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `STDIO 模式需要 command` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `Scoped tools` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `Server id 不能为空` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `Server 名称` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `Server 总数` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `Servers` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `URL` | ✓ | ✓ | base | `BaseMcp` |
| mcp | 文案 key | `bearer_token_env_var` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `bearer_token_env_var 与 env_http_headers 比明文 token 更可控。` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `cwd` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `disabled_tools` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `enabled_tools` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `enabled_tools / disabled_tools 用来缩小 Codex 可见的工具面。` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `http` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `required=true 会把 server 升级成启动时必须可用的依赖。` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `startup_timeout_ms / bearer_token` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `一手资料` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `交互式添加 local 或 remote server。` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `传输层` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `保存` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `保存变更` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `入口` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `全部` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `关注项` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `准备删除` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `创建服务器` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `删除` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `刷新` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `加载 Codex MCP 服务器失败` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `参数` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `取消` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `取消编辑` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `可以先从空白模板开始，或直接填入 OpenAI Docs MCP。` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `可添加本地命令型 server，或远程 HTTP/SSE server。` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `启用该 MCP server` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `命令` | ✓ | ✓ | base | `BaseMcp` |
| mcp | 文案 key | `在线` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `填入 OpenAI Docs MCP 模板` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `官方能力面` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `已启用` | ✓ | ✓ | base | `BaseMcp` |
| mcp | 文案 key | `已复制` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `已暂停` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `已禁用` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `已限制` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `建议一行一个参数；如果只写一行，保存时会按空格拆分。` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `必需` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `恢复表单` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `按官方 mcp_servers 配置面管理 transport、tool scope、auth 注入与启动策略。` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `换个关键词或清空筛选再试一次。` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `授权` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `排查 OAuth / transport 连接问题。` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `搜索 name、command、url 或 tool scope` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `新建 Codex MCP server` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `新建 HTTP` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `新建 STDIO` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `旧兼容` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `旧版页面只覆盖少量字段；这里已经扩到官方 Codex MCP 配置面。` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `暂无 MCP 服务器` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `服务器 ID *` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `服务器已停用` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `服务器已启用` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `本地` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `没有匹配的服务器` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `没有额外策略` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `添加 MCP 服务器` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `添加服务器` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `清单` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `清空筛选` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `状态` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `环境` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `环境变量` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `环境变量 JSON` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `现有 server 名称保持锁定；如需改名，建议新建后删除旧项。` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `登出` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `确认删除` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `管理 local / remote MCP 定义，并提供官方 CLI auth / debug / logout 动作。` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `类型` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `编辑` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `编辑 MCP 服务器` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `编辑当前 server` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `缺少 URL` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `缺少 command` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `请填写 server name` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `请求头` | ✓ | ✓ | base | `BaseMcp` |
| mcp | 文案 key | `请求头 JSON` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `调试` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `超时` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `返回概览` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `还没有 Codex MCP 服务器` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `这里直接编辑 Codex 的 transport、tool scope、headers、env 和 timeout。` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `这页按官方 Codex 与 MCP 文档重建，不再停留在旧 CRUD 壳层。` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `远程` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 文案 key | `选中一个 server，或者先创建新配置` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `选中一个 server，或者先创建新配置。` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `鉴权感知` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 文案 key | `附加项` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 分支状态 | `!editorMode` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 分支状态 | `!server.enabled` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 分支状态 | `draft.transport === 'http'` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 分支状态 | `draft.transport === 'stdio'` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 分支状态 | `editorMode === 'edit'` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 分支状态 | `error && !servers.length` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 分支状态 | `form.type === 'local'` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 分支状态 | `hasLegacyCompatibility(server)` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 分支状态 | `hasScopedTools(server)` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 分支状态 | `loading` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 分支状态 | `saving` | — | ✓ | config.opencode | src/configs/mcp.ts |
| mcp | 分支状态 | `server.required` | ✓ | — | config.codex | src/configs/mcp.ts |
| mcp | 分支状态 | `server.type === 'remote'` | — | ✓ | config.opencode | src/configs/mcp.ts |

## agents

| 功能面 | 维度 | 项 | codex | opencode | 归属 | 统一后位置 |
| --- | --- | --- | --- | --- | --- | --- |
| agents | 字段 | `body` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 字段 | `description` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 字段 | `disable` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 字段 | `hidden` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 字段 | `mode` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 字段 | `model` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 字段 | `name` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 字段 | `permissionJson` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 字段 | `steps` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 字段 | `temperature` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 操作 | `activePanel = 'installed'` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 操作 | `activePanel = 'sources'` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 操作 | `bulkRenameModalOpen = false` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 操作 | `copyModalOpen = false` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 操作 | `exportAgent(agent)` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 操作 | `handleBackToGlobal` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 操作 | `handleBulkDelete` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 操作 | `handleBulkRename` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 操作 | `handleBulkValidate` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 操作 | `handleChooseProject` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 操作 | `handleCopyAgent` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 操作 | `handleDeleteAgent(agent)` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 操作 | `handleExportSelected` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 操作 | `handleRefresh` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 操作 | `handleRenameAgent` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 操作 | `handleSwitchToSavedProject` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 操作 | `handleValidateAgent(agent)` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 操作 | `openBulkCopyModal` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 操作 | `openBulkRenameModal` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 操作 | `openCopyModal(agent)` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 操作 | `openCreate()` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 操作 | `openCreateModal` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 操作 | `openEdit(agent)` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 操作 | `openEditModal(agent)` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 操作 | `openRenameModal(agent)` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 操作 | `removeAgent(agent)` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 操作 | `renameModalOpen = false` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 操作 | `saveAgent` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 操作 | `showModal = false` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 操作 | `triggerImport` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | IPC 命令 | `addOpenCodeAgent` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | IPC 命令 | `deleteOpenCodeAgent` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | IPC 命令 | `listOpenCodeAgents` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | IPC 命令 | `updateOpenCodeAgent` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 校验规则 | `required` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 校验规则 | `validate` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `${agent.nicknameCandidates.length} 个昵称` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `${diagnostics.length} 条诊断` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `Agent name 和 description 为必填项` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `Agent 已创建` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `Agent 已删除` | ✓ | ✓ | base | `BaseAgents` |
| agents | 文案 key | `Agent 已更新` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `Agent 已重命名` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `Agent 数量` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `Agents` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `Built-in layout` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `Codex Agent 已创建` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `Codex Agent 已更新` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `OpenCode 内置两个 primary agent 和两个 subagent，页面重点是展示自定义 agent 如何挂在这个体系上。` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `TOML 无效` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `a` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `all` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `body prompt` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `codex.agents.addAgent` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `codex.agents.deleteConfirm` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `codex.agents.emptyHint` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `codex.agents.emptyState` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `codex.agents.noResults` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `codex.agents.noResultsHint` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `codex.agents.pageTitle` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `codex.agents.searchPlaceholder` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `common.cancel` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `common.delete` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `common.loading` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `common.refresh` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `description *` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `mode` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `name *` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `permission JSON` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `primary` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `steps` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `subagent` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `temperature` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `上下文控制` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `上次项目` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `不限` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `会话数量` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `例如 -review` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `例如 feature-` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `保存` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `全局` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `内置 Agent` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `写 frontmatter，写 body prompt，然后交给 OpenCode 的 Task / agent runtime 使用。` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `删除` | ✓ | ✓ | base | `BaseAgents` |
| agents | 文案 key | `删除所选 Agent` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `前缀` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `取消` | ✓ | ✓ | base | `BaseAgents` |
| agents | 文案 key | `只读` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `同一时间只会激活一个管理上下文。` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `后缀` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `复制` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `复制 Agent` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `导入` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `导入完成` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `导出` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `已切换到项目上下文` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `已删除所选 Agent` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `已复制 ${queue.length} 个 Agent` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `已安装` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `已校验 ${agent.name}` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `已校验 ${selectedAgents.value.length} 个 Agent` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `已禁用` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `已返回全局 Agent 视图` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `已重命名 ${selectedAgents.value.length} 个 Agent` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `当前` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `当前上下文没有诊断信息。` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `当前作用域` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `打开上次项目` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `批量重命名 Agent` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `把 OpenCode 的 built-in agent 模式和自定义 agents 放在同一张操作面板里。` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `数据来自 Codex 总览页会话统计。` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `新名称` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `新建 primary / subagent，用于计划、评审、文档或其它专项工作流。` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `暂无描述` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `暂无自定义 Agent` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `未配置 body prompt。` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `来源` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `校验` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `模型` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `步数` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `添加 Agent` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `留空则保持当前名称` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `目标上下文：` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `目标名称` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `确认从当前上下文删除 ${selectedAgents.value.length} 个所选 Agent 吗？` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `继承` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `编辑` | ✓ | ✓ | base | `BaseAgents` |
| agents | 文案 key | `编辑 Agent` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `诊断` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `请先选择项目` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `请先选择项目上下文` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `请至少提供前缀、后缀中的一项` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `路径` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `返回全局` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `选择当前可见 Agent` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `选择项目` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `重命名` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `重命名 Agent` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `隐藏` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `隐藏 subagent` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 文案 key | `项目` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 文案 key | `项目：${lastProjectRoot.value}` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 分支状态 | `activePanel === 'installed'` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 分支状态 | `activePanel === 'installed' && hasProjectShortcut && !isProjectMode` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 分支状态 | `activePanel === 'installed' && isProjectMode` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 分支状态 | `agent.hidden` | — | ✓ | config.opencode | src/configs/agents.ts |
| agents | 分支状态 | `agent.model` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 分支状态 | `agent.nicknameCandidates?.length` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 分支状态 | `agent.parseError` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 分支状态 | `agent.sandboxMode` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 分支状态 | `diagnostics.length === 0` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 分支状态 | `lastProjectRoot` | ✓ | — | config.codex | src/configs/agents.ts |
| agents | 分支状态 | `loading` | ✓ | ✓ | base | `BaseAgents` |
| agents | 分支状态 | `saving` | — | ✓ | config.opencode | src/configs/agents.ts |

## plugins

| 功能面 | 维度 | 项 | claude | opencode | 归属 | 统一后位置 |
| --- | --- | --- | --- | --- | --- | --- |
| plugins | 字段 | `add` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 字段 | `cancel` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 字段 | `config` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 字段 | `configHint` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 字段 | `configPlaceholder` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 字段 | `enablePlugin` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 字段 | `enabled` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 字段 | `id` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 字段 | `idPlaceholder` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 字段 | `name` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 字段 | `namePlaceholder` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 字段 | `update` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 字段 | `value` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 字段 | `version` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 字段 | `versionPlaceholder` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 操作 | `handleAdd` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 操作 | `handleDelete(plugin.id)` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 操作 | `handleEdit(plugin)` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 操作 | `handleSubmit` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 操作 | `handleToggle(plugin.id)` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 操作 | `removePackage(item.name)` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | 操作 | `savePackage` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | 操作 | `showAddForm = false` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 操作 | `showModal = false` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | 操作 | `showModal = true` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | IPC 命令 | `addOpenCodePlugin` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | IPC 命令 | `addPlugin` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | IPC 命令 | `deleteOpenCodePlugin` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | IPC 命令 | `deletePlugin` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | IPC 命令 | `listOpenCodeLocalPlugins` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | IPC 命令 | `listOpenCodePlugins` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | IPC 命令 | `listPlugins` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | IPC 命令 | `togglePlugin` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | IPC 命令 | `updatePlugin` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 校验规则 | `required` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | 文案 key | `1. 全局配置` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | 文案 key | `2. 项目配置` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | 文案 key | `3. 全局插件目录` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | 文案 key | `4. 项目插件目录` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | 文案 key | `Load order` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | 文案 key | `Local plugin files` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | 文案 key | `commands.unknownError` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `common.cancel` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `common.delete` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `npm plugin packages` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | 文案 key | `npm 插件已删除` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | 文案 key | `npm 插件已添加` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | 文案 key | `package name` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | 文案 key | `package name 不能为空` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.addPlugin` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.addSuccess` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.configJsonError` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.delete` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.deleteFailed` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.deleteSuccess` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.disable` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.disabled` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.edit` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.editPlugin` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.enable` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.fillRequired` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.form.add` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.form.cancel` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.form.config` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.form.configHint` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.form.configPlaceholder` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.form.enablePlugin` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.form.id` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.form.idPlaceholder` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.form.name` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.form.namePlaceholder` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.form.update` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.form.version` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.form.versionPlaceholder` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.loadFailed` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.noPlugins` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.operationFailed` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.subtitle` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.title` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.toggleFailed` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.updateSuccess` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `plugins.version` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 文案 key | `保存` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | 文案 key | `删除` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | 文案 key | `取消` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | 文案 key | `向 ` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | 文案 key | `将 npm 插件配置与本地插件文件分开展示，并补上官方 load order 语义。` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | 文案 key | `插件` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | 文案 key | `插件会按以下顺序加载，适合在排查覆盖关系时直接对照。` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | 文案 key | `暂无 npm 插件配置。` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | 文案 key | `未发现本地插件文件。` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | 文案 key | `来自 ` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | 文案 key | `添加 npm 插件` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | 文案 key | `这些条目来自 ` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | 分支状态 | `!plugin.enabled` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 分支状态 | `loading` | ✓ | ✓ | base | `BasePlugins` |
| plugins | 分支状态 | `localPlugins.length === 0` | — | ✓ | config.opencode | src/configs/plugins.ts |
| plugins | 分支状态 | `plugin.config` | ✓ | — | config.claude | src/configs/plugins.ts |
| plugins | 分支状态 | `saving` | — | ✓ | config.opencode | src/configs/plugins.ts |

