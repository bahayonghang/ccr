# AG02 operation surface audit (2026-10-26)

Preparation only; product edits wait for AG01_READY. The reviewed parent design remains authoritative over the old market-terminal text.

## Route ownership

| Routes | Concrete owner / material path | Validation |
|---|---|---|
| /settings, /configs, /converter | features/configs/{AppSettingsView,ConfigsView,ConverterView}; ui primitives + config-card/filter CSS | configs smoke; browser real page markers |
| /claude-code/profiles, /codex/profiles, /grok/profiles | shared components/profiles + styles/components/profiles-page.css | profiles smoke; create/edit dialog fixture |
| /claude-code/settings, /codex/settings, /grok/settings, /opencode/settings | platform Settings shared base and domain wrappers | settings session/source/draft smoke; no wrapper changes |
| /claude-code/auth, /codex/auth, /grok/auth | existing auth views and shared solid surface aliases | auth smoke; browser unavailable state |
| /claude-code/system-prompts, /codex/system-prompts, /antigravity/system-prompts, /opencode/system-prompts | shared platform source/editor primitives | source/CSP guards; no source data changes |
| /mcp-manager, /codex/mcp, /antigravity/mcp, /opencode/mcp | platform/mcp shared panels, mcp manager CSS | mcp panels fixture + create/detail input surfaces |
| /agents, /agents/:name, /codex/agents, /antigravity/agents, /opencode/agents | shared platform agents | fixture name ag02-fixture; not a redirect; no real mutation |
| /slash-commands, /codex/slash-commands, /antigravity/slash-commands, /opencode/commands | shared slash command resources | route loader smoke and shared primitives |
| /plugins, /antigravity/plugins, /opencode/plugins, /hooks, /output-styles, /statusline | shared Base/platform resources + Claude resource components | operation route browser state and domain smoke |
| /skills | SkillsMigrationView | retain Skillport migration only; no manager recreation |
| /opencode/providers | OpenCode provider shared panel | existing route/namespace/provider guards |
| /sync, /wsl, /ssh | sync operation views + shared controls | sync gating/encryption/passphrase guards; SSH input display only |
| legacy redirects | original routeCatalog redirect targets | no route changes; AG04 export verifies targets |

AG03 owns platform home pages, /commands/:client?, /codex/sessions, /agent-sessions, observation routes and tray. Tokens/shell/first paint belong to AG01.

## Minimal planned product files

1. `src/features/configs/styles/config-card.css`: replace liquid-glass row, hover and chip surfaces with opaque semantic card/elevated roles; remove row blur.
2. `src/features/configs/styles/config-filters.css`: replace remaining legacy glass button backgrounds/borders with solid aliases.
3. `src/components/profiles/profile-editor-shell.css`: consume shared semantic modal aliases; full-color focus/invalid field boundary without a second blur.
4. `src/styles/components/profiles-page.css`: make cp-bg-4 opaque; accent contrast role follows primary contrast.
5. `src/ui/primitives.css`: opaque sticky scroll action, semantic destructive ink, correct contrast ink for filled primary/danger buttons, solid focus outline.
6. `src/ui/select.tsx` and `src/ui/combobox.tsx`: full strong field border and solid accent focus boundary; retain all Radix/cmdk behavior.
7. `src/ui/checkbox.tsx`: use dedicated accent contrast on checked accent fill.
8. `src/ui/popover.tsx` and `src/ui/tooltip.tsx`: update stale glass material comments only, semantic modal remains single implementation.
9. `src/features/mcp/styles/mcp-manager.css`: opaque secondary operation button and semantic danger tint.
10. `src/features/platform/mcp/{McpCreatePanel,McpDetailPanel}.tsx`: opaque protocol/list surfaces.
11. `src/features/platform/mcp/mcp-classes.ts`: opaque kv row; clear field boundaries and accent primary button; no payload changes.
12. `src/features/sync/SshManagementView.tsx`: existing input/textarea class strings gain solid field fill, strong border and focus outline; do not touch register keys/actions.

Other operation views inherit new AG01 tokens through existing solid aliases. Alpha state washes over solid parent cards are not glass or standalone reading surfaces and are retained unless a rendered contrast issue is observed. No exhaustive cosmetic rewriting is planned.

## Checks

Read actual existing files before choosing smoke names. Run configs/profiles/ui/mcp/sync and platform settings/auth guards, then type-check/lint. Screenshots/browser state proof and four theme/flavor combinations remain pending until AG01 is ready. Native, real IPC and config writes are outside this worker scope.
