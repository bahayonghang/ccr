# Product Documentation Gap Audit

## Sources Of Truth

- Workspace membership: `Cargo.toml:2-16`.
- CLI command tree: `crates/ccr-cli/src/cli/definitions.rs:83-454` plus nested subcommand modules.
- UI routes: `ccr-ui/src/router/index.ts` and platform route metadata.
- Documentation navigation: `docs/.vitepress/config.mjs`.
- Documentation gate: `docs/scripts/audit-docs.mjs`.

## Confirmed Findings

### Workspace coverage

The workspace has 13 crates: `ccr`, `ccr-core`, `ccr-config`, `ccr-sync`, `ccr-skills`, `ccr-store`, `ccr-codex`, `ccr-db`, `ccr-checkin`, `ccr-cli`, `ccr-tui`, `ccr-usage`, and `ccr-types`.

The current Chinese crate map names only a subset and the English page groups several crates without a complete one-to-one inventory. Both need to be rebuilt from current crate ownership.

### CLI coverage

The top-level command enum currently exposes 34 commands, including newer `doctor`, `claude`, `sessions`, and `provider` entries (`definitions.rs:403`, `:434`, `:445`, `:453`).

The command reference has no `claude.md` mirror. `help` is also a real command but can be covered either by a dedicated page or an explicit contract in the command overview. The conceptual `tui.md` page is valid even though TUI is launched through no-subcommand/platform entrypoints rather than a `Tui` enum variant.

### UI coverage

The current router contains a much broader module set than the existing high-level UI module documentation: Claude Code, Codex, OpenCode, Antigravity, Gemini CLI, sync, configs, usage/monitoring, sessions, MCP, agents, skills, plugins, hooks, output styles, statusline, check-in, WSL, and SSH.

### Gate weakness

`docs/scripts/audit-docs.mjs` currently verifies config links, locale file parity, placeholder translations, a small removed-page list, two UI port defaults, and banned stale phrases. It does not compare stable CLI commands with command reference coverage.

The baseline `bun run audit` currently fails because `AGENTS.md`, `TODO.md`, and `reports/ccr_code_audit_canvas.md` are treated as active localized product pages. These are maintenance/internal materials and need an explicit audit exclusion or relocation policy.

### Maintenance drift

- `docs/TODO.md` contains completed historical items and an empty checkbox rather than an actionable documentation backlog.
- `docs/README.md` describes the documentation layout but contains stale operational wording and must be aligned with the actual Bun-based scripts and ownership rules.
- Several large command pages include roadmap-like or generic content that should be reduced to current behavior, verified examples, constraints, and related commands.

## Proposed Direction

- Reorganize navigation around Start, Workflows, Interfaces, Platforms, Reference, and Troubleshooting rather than mirroring implementation history.
- Preserve bilingual parity and add missing stable command coverage.
- Rebuild architecture/crate/UI module pages from current source boundaries.
- Strengthen `audit-docs.mjs` with explicit published-page ownership and command coverage assertions that allow documented conceptual pages such as `tui`.
- Remove or convert stale TODO/maintenance material instead of translating it as product content.

## Baseline Verification

`bun run audit` fails before implementation with:

```text
Missing English mirror pages: AGENTS.md, reports/ccr_code_audit_canvas.md, TODO.md
```

This baseline failure belongs to the task and is not caused by the new Trellis files.
