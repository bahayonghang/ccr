# Resources checkpoint — 2026-10-29

Status remains in_progress. First fixed-instruction-file slice is integrated through shared policy, GPUI workers/editor and Tauri delegation. Claude rules/list parity and the remaining76 commands remain open; no complete06 acceptance.

Main executed current app56 library+15 integration, desktop44 library+4 shell+2 UI, Tauri system_prompts5, strict desktop/shared/Tauri Clippy, root/Tauri/desktop formatting, transactional bindings, registry inventory, version/governance and sensitive-write checks: exit0. Raw receipts, retained failures and exact scope are in parent [continuation report](../10-17-gpui-desktop/research/continuation-20261029.md). Independent source review: [check report](../10-17-gpui-08-jobs/research/check-20261029.md).

Actual immutable candidate8a0149 showed missing fixed file, Cancel initial acknowledgement focus, Tab to Confirm, editor entry and successful synthetic English/Chinese save. Bytes were read from the explicit fixture and matched. Screenshot/tree: `outputs/desktop/profiles/native/resumed29-native/resource-save.*`. Native reread/CAS/leave and other platforms are NOT_RUN on this candidate because minimized-window/user-input protection stopped further input. Headless CAS/teardown/focus tests passed separately. No real credentials/config, install/network, commit or archive.

## MCP manager page — 2026-11-05

`/mcp-manager` (`page-mcp-manager`) is now a local read page. It is not a writer and does not report process status. The sheet stays the existing solid paper; this pass does not claim backdrop blur. The configuration converter remains the in-page `tool-converter` control.

### What a loaded page shows

The page reads four existing local documents through captured `PlatformRoots` and `platform_settings::read_text`. Fixture mode uses `desktop-mcp-fixture/home` and does not create that directory when the files are absent. A non-local environment is rejected before any read.

| Source | Entries shown | When the file is absent |
| --- | --- | --- |
| Claude state file (`state_file`, normally `~/.claude.json`) | user `mcpServers`, plus `projects.<key>.mcpServers` | "Claude state file: not found." |
| Claude `settings.json` | top-level `mcpServers` only | "Claude settings.json: not found." |
| Codex `config.toml` | `[mcp_servers]` | "Codex config.toml: not found." |
| Antigravity `mcp_config.json` | top-level `mcpServers` | "Antigravity mcp_config.json: not found." |

A present file with no entries says that source has no MCP entries. Invalid JSON/TOML, or an MCP map of the wrong shape, drops only that file and says no entries were taken from it. Other files still appear. Counts are the number of parsed entries from that file.

Each row is the platform name and the entry name. `disabled: true` adds "disabled in the file". The detail shows the command when the file has one, whether a URL or arguments are present, and the raw `disabled` / `enabled` booleans when those fields exist. It does not show environment values, headers, argument text, URL text, or bearer tokens. A Claude project entry shows the stored project key, which may be a path.

With no readable entries, the page says "No MCP entries were found in the files this page reads." It does not invent a server, a running state, or a zero count.

These sources stay explicit gaps, with no file read:

- Claude project `.mcp.json` — this window has no captured project root, so approval and effective-server precedence are not calculated.
- OpenCode MCP — no reader is connected.
- Grok — `mcp_credentials.json` is not listed as servers.

Instruction files on Claude, Codex, Antigravity, and OpenCode stay the existing fixed-file reader and scoped editor. Skills, agents, plugins, hooks, and slash commands are not newly mounted. `SkillsManager::list_skills` creates directories and uses the real home, so it is not a safe captured-root read. No add, remove, or install action was added.

A name filter hides the detail of any entry that is not in the filtered list. A present file with no MCP map is `Empty` (“no MCP entries”), not a zero count. `Entries(0)` uses that same empty sentence. A server map of the wrong shape invalidates only that file and does not stringify environment values, headers, arguments, URLs, or bearer tokens into the catalog.

### Commands

Check re-run from the repository root, 2026-11-05, after the filter and empty/invalid fixes:

| Command | Exit |
| --- | --- |
| `just desktop-fmt-check` | 0 |
| `just desktop-lint` | 0 |
| `just desktop-test` | 0 |

`just desktop-test` reported Python unittest 12 OK, AccessKit harness 3 OK, library 68 OK, `native_shell` 6 OK, `native_ui` 2 OK. Cargo printed an existing `ccr-cli` warning summary during the desktop build; this pass did not edit `ccr-cli`.

The rendered page test opens the command, URL, environment, bearer-token, and header rows and asserts those secret strings are absent from the accessibility labels. It does not read the developer home. PRD 6.1–6.6 stay open: no install/uninstall proof, no project `.mcp.json` precedence, no skills/agents/plugins/hooks/slash manager, and no DPI, three-OS, native blur, or full Tauri parity evidence. The sheet remains the existing solid paper. This slice does not claim backdrop blur. The configuration converter stays the in-page `tool-converter` control, and the sidebar remains the 13 destinations.
