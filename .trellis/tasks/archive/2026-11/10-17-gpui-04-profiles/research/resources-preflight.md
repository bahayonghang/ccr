# GPUI 06 prerequisite findings (limited source research)

Date: 2026-10-18. Active task remains04; this is read-only source research while
04 implementers run their checks. It does not activate, implement or accept06.

## Inventory and a route-count blind spot

The retained parent migration inventory assigns **80 commands and38 route
records (24 surfaces,14 redirects)** to06. Do not regenerate the capture over
live evidence. Command module counts: system_prompts4, claude24, codex20,
gemini9, opencode9, unified_mcp4, builtin_prompts3, mcp_presets7.

There are no active Skills CRUD command IDs in that baseline. Current
`ccr-ui/src/features/claude/SkillsMigrationView.tsx` explicitly says embedded
Skills install/market/source management was removed; it detects/opens the
external skillport app and links its instructions. `/skills` is a real route,
and seven aliases redirect there. `shell_detect_skillport_app`,
`shell_open_skillport_app`, and skills-manage aliases belong to09 in the
inventory. Do not infer installed native Skills CRUD merely from `/skills` or
80 command mappings.

The approved06 PRD explicitly names ccr-skills and existing install/uninstall
capability. `crates/ccr-skills/src/services/skills_service.rs` still owns real
inventory/detail/content/files/source/install/sync/remove/marketplace methods.
Its default `new` resolves ambient home plus OS cache; no public injected
constructor appeared in the inspected signatures. If this approved domain
surface is consumed natively, add a narrow constructor to that owner, inject
cache/platform/target paths too, and retain its filesystem/install policies.
Do not resurrect removed Tauri endpoints or assume external-app launch is an
equivalent native install action. Record the explicit native route disposition
and09 integration dependency when06 is implemented.

## Inspected owners and extraction candidates

- `commands/claude_mcp_config.rs` already owns actual scoped MCP behavior:
  context, user/project/local paths, precedence/effective/approval diagnostics,
  versioned object mutation, masked previews and merge-with-secret-retention.
  `unified_mcp.rs` consumes it. Extract actual shared framework-free policy
  rather than writing another JSON MCP parser. Preserve scope and approval.
- `commands/unified_mcp.rs` also has separate Codex/Antigravity readers rooted
  at ambient home, with raw env/headers on DTOs and parser Display errors.
  These DTOs cannot become native list/Debug state. Typed masked list and
  scoped editor values must be distinct; fixture paths must include every
  platform, not fall back from an unavailable environment to Local.
- `ccr-skills/managers/mcp_preset_manager.rs` owns preset catalog/install and
  `McpSyncManager` source/sync operations. Tauri `mcp_presets.rs` schedules it
  in blocking workers and maps per-platform outcomes. Preserve partial result
  semantics and existing default requested targets (Codex/Gemini), without
  claiming all-platform transactionality or copying its install code.
- `commands/system_prompts.rs` restricts Local prompt access through a closed
  file-spec list: Claude CLAUDE.md, Codex AGENTS.md, Antigravity GEMINI.md,
  OpenCode AGENTS.md. Its aliases normalize existing platform IDs; save uses
  the existing raw versioned writer. Consume05's extracted raw owner rather
  than duplicate CAS/secret file persistence. Size warning64KiB and Codex32KiB
  limit hint are observed metadata, not blanket new file-size limits.
- Codex source-agent install/sync/accept/untrack are real06 commands, not08
  generic CLI jobs; actual runtime scheduling still uses02's app-owned jobs.

## Follow-up before implementation

Read06's complete curated contexts and affected owner specs. This limited pass
has not traced all80 commands, archive/path guards, all install subprocesses,
agent source policy or platform resource editors. It has run no product tests,
network requests, install/uninstall, native UI or real user configuration read.
Expand the source map before assigning file ownership; giant shared Tauri
claude/codex/grok/opencode modules need one current writer each. Keep default
test parallelism and `--skip export_bindings`, overriding obsolete serial-test
examples in older package guidelines as the user's current instruction requires.
