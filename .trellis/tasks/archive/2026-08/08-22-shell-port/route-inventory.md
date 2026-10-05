# 路由清单比对（AC2）

> 生成：2026-08-24。源：`ccr-ui/src/shell/routeCatalog.ts` `flattenCatalog()`，对照原 `src/router/index.ts`。记录数 **75**。

| # | 路径 | id | redirect | cache | hideGlobalBackground |
| --- | --- | --- | --- | --- | --- |
| 1 | `/tray/codex` | `codex-tray-panel` | | | yes |
| 2 | `/` | | | | |
| 3 | `/` | `dashboard` | | yes | |
| 4 | `/settings` | `settings` | | | |
| 5 | `/claude-code` | `claude-code` | | | yes |
| 6 | `/claude-code/settings` | `claude-code-settings` | | | |
| 7 | `/claude-code/system-prompts` | `claude-system-prompts` | | | |
| 8 | `/claude-code/profiles` | `claude-code-profiles` | | | |
| 9 | `/claude-code/auth` | `claude-code-auth` | | | |
| 10 | `/codex` | `codex` | | | |
| 11 | `/grok` | `grok` | | yes | |
| 12 | `/grok/auth` | `grok-auth` | | | |
| 13 | `/grok/profiles` | `grok-profiles` | | | |
| 14 | `/grok/settings` | `grok-settings` | | | |
| 15 | `/antigravity` | `antigravity` | | | yes |
| 16 | `/gemini-cli` | `gemini-cli` | `/antigravity` | | yes |
| 17 | `/ccr-control` | `ccr-control` | `/commands/ccr` | | |
| 18 | `/commands/:client?` | `commands` | | yes | |
| 19 | `/converter` | `converter` | | | |
| 20 | `/sync` | `sync` | | | yes |
| 21 | `/configs` | `configs` | | yes | yes |
| 22 | `/stats` | | `/usage` | | |
| 23 | `/budget` | `budget` | | | |
| 24 | `/pricing` | `pricing` | | | |
| 25 | `/usage` | `usage` | | yes | yes |
| 26 | `/monitoring` | `monitoring` | | | |
| 27 | `/sessions` | `sessions` | `/monitoring` | | |
| 28 | `/mcp` | `mcp` | `/mcp-manager` | | |
| 29 | `/mcp/unified` | `mcp-unified` | `/mcp-manager` | | |
| 30 | `/mcp-manager` | `mcp-manager` | | | |
| 31 | `/slash-commands` | `slash-commands` | | | |
| 32 | `/agents` | `agents` | | | |
| 33 | `/agents/:name` | `agent-detail` | | | |
| 34 | `/skills` | `skills` | | | |
| 35 | `/skills-manager` | `skills-manager` | `/skills` | | |
| 36 | `/skillport-manager` | `skillport-manager` | `/skills` | | |
| 37 | `/skills/add` | `skills-add` | `/skills` | | |
| 38 | `/skills/hub` | | `/skills` | | |
| 39 | `/skills/:platform/:name` | | `/skills` | | |
| 40 | `/market` | `market` | `/skills` | | |
| 41 | `/plugins` | `plugins` | | | |
| 42 | `/hooks` | `hooks` | | | |
| 43 | `/output-styles` | `output-styles` | | | |
| 44 | `/statusline` | `statusline` | | | |
| 45 | `/checkin/manage/:accountId` | `checkin-account-dashboard` | | | |
| 46 | `/checkin` | `checkin` | | | |
| 47 | `/codex/mcp` | `codex-mcp` | | | |
| 48 | `/codex/profiles` | `codex-profiles` | | | |
| 49 | `/codex/agents` | `codex-agents` | | | |
| 50 | `/codex/sessions` | `codex-sessions` | | | |
| 51 | `/codex/slash-commands` | `codex-slash-commands` | | | |
| 52 | `/codex/auth` | `codex-auth` | | | |
| 53 | `/codex/settings` | `codex-settings` | | | |
| 54 | `/codex/system-prompts` | `codex-system-prompts` | | | |
| 55 | `/antigravity/slash-commands` | `gemini-slash-commands` | | | |
| 56 | `/gemini-cli/slash-commands` | | `/antigravity/slash-commands` | | |
| 57 | `/gemini-cli/mcp` | | `/antigravity/mcp` | | |
| 58 | `/gemini-cli/agents` | | `/antigravity/agents` | | |
| 59 | `/gemini-cli/plugins` | | `/antigravity/plugins` | | |
| 60 | `/antigravity/system-prompts` | `gemini-system-prompts` | | | |
| 61 | `/gemini-cli/system-prompts` | | `/antigravity/system-prompts` | | |
| 62 | `/opencode` | `opencode` | | | yes |
| 63 | `/opencode/providers` | `opencode-providers` | | | yes |
| 64 | `/opencode/mcp` | `opencode-mcp` | | | yes |
| 65 | `/opencode/agents` | `opencode-agents` | | | yes |
| 66 | `/opencode/commands` | `opencode-commands` | | | yes |
| 67 | `/opencode/skills` | `opencode-skills` | `/skills` | | yes |
| 68 | `/opencode/plugins` | `opencode-plugins` | | | yes |
| 69 | `/opencode/settings` | `opencode-settings` | | | yes |
| 70 | `/opencode/system-prompts` | `opencode-system-prompts` | | | yes |
| 71 | `/wsl` | `wsl-management` | | | |
| 72 | `/ssh` | `ssh-management` | | | |
| 73 | `/antigravity/mcp` | `gemini-mcp` | | | |
| 74 | `/antigravity/agents` | `gemini-agents` | | | |
| 75 | `/antigravity/plugins` | `gemini-plugins` | | | |

结论：75 条路径与 Vue 表一致。第 2 行是布局父级 `/`，第 3 行是 dashboard 子级 `/`。mcp/agents/plugins 仍由 `genericPlatformDescriptorList` 生成。
