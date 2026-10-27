# Route rendering evidence

76 catalog records = 57 pages + 18 redirects + 1 layout parent. All 75 page/redirect entries were visited in light/dark neutral (150 visits). The layout parent is not a second dashboard. Four-combination representative/control evidence is separate.

Source: AG04/evidence/web-complete/results.json and screenshots. PASS_WEB_RENDER confirms route content/identity and blur budget; it does not prove native operations, populated data, localization completeness or every interaction state. Dynamic names/account IDs use isolated synthetic parameters.

The final configs workspace min-width correction is supplemented by AG02/evidence/cards/2026-10-26T17-03-33.324Z/ at 1200/800/390 widths in all four combinations. Other final route/layout evidence is unchanged.

| Catalog path | ID | Kind | Light | Dark |
|---|---|---|---|---|---|
| /tray/codex | codex-tray-panel | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| / | — | layout | STRUCTURAL | STRUCTURAL |
| / | dashboard | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /settings | settings | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /claude-code | claude-code | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /claude-code/settings | claude-code-settings | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /claude-code/system-prompts | claude-system-prompts | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /claude-code/profiles | claude-code-profiles | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /claude-code/auth | claude-code-auth | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /codex | codex | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /grok | grok | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /grok/auth | grok-auth | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /grok/profiles | grok-profiles | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /grok/settings | grok-settings | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /antigravity | antigravity | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /gemini-cli | gemini-cli | redirect → /antigravity | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /ccr-control | ccr-control | redirect → /commands/ccr | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /commands/:client? | commands | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /converter | converter | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /sync | sync | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /configs | configs | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /stats | — | redirect → /usage | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /budget | budget | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /pricing | pricing | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /usage | usage | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /monitoring | monitoring | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /sessions | sessions | redirect → /monitoring | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /mcp | mcp | redirect → /mcp-manager | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /mcp/unified | mcp-unified | redirect → /mcp-manager | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /agent-sessions | agent-sessions | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /mcp-manager | mcp-manager | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /slash-commands | slash-commands | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /agents | agents | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /agents/:name | agent-detail | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /skills | skills | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /skills-manager | skills-manager | redirect → /skills | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /skillport-manager | skillport-manager | redirect → /skills | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /skills/add | skills-add | redirect → /skills | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /skills/hub | — | redirect → /skills | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /skills/:platform/:name | — | redirect → /skills | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /market | market | redirect → /skills | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /plugins | plugins | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /hooks | hooks | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /output-styles | output-styles | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /statusline | statusline | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /checkin/manage/:accountId | checkin-account-dashboard | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /checkin | checkin | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /codex/mcp | codex-mcp | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /codex/profiles | codex-profiles | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /codex/agents | codex-agents | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /codex/sessions | codex-sessions | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /codex/slash-commands | codex-slash-commands | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /codex/auth | codex-auth | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /codex/settings | codex-settings | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /codex/system-prompts | codex-system-prompts | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /antigravity/slash-commands | gemini-slash-commands | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /gemini-cli/slash-commands | — | redirect → /antigravity/slash-commands | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /gemini-cli/mcp | — | redirect → /antigravity/mcp | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /gemini-cli/agents | — | redirect → /antigravity/agents | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /gemini-cli/plugins | — | redirect → /antigravity/plugins | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /antigravity/system-prompts | gemini-system-prompts | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /gemini-cli/system-prompts | — | redirect → /antigravity/system-prompts | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /opencode | opencode | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /opencode/providers | opencode-providers | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /opencode/mcp | opencode-mcp | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /opencode/agents | opencode-agents | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /opencode/commands | opencode-commands | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /opencode/skills | opencode-skills | redirect → /skills | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /opencode/plugins | opencode-plugins | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /opencode/settings | opencode-settings | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /opencode/system-prompts | opencode-system-prompts | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /wsl | wsl-management | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /ssh | ssh-management | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /antigravity/mcp | gemini-mcp | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /antigravity/agents | gemini-agents | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
| /antigravity/plugins | gemini-plugins | page | PASS_WEB_RENDER | PASS_WEB_RENDER |
