# Design: 产品文档站重构

## Information Architecture

Keep existing page URLs while regrouping navigation into user tasks:

1. Start: overview, quick start, entrypoint selection, configuration.
2. Workflows: CLI workflows, multi-platform setup, sync, troubleshooting.
3. Interfaces: CLI/TUI/CCR UI overview and current UI module map.
4. Platforms: Claude, Codex, OpenCode, Gemini, Droid/migration status as implemented.
5. Reference: commands, architecture, crate map, runtime flows, migration, changelog.

The active Chinese and English page sets remain mirrors. Historical changelog pages are exempt from current-fact rewrites but not from link validity.

## Content Method

- Derive workspace ownership from `Cargo.toml` and each crate manifest/source boundary.
- Derive top-level commands from `crates/ccr-cli/src/cli/definitions.rs` and nested subcommand modules.
- Derive UI modules from `ccr-ui/src/router/index.ts`, platform capability metadata, and stable Tauri/API boundaries.
- Rewrite each active page to retain only current behavior, constraints, verified examples, and useful cross-links.
- Preserve stable paths even when a page is substantially rewritten.

## Command Coverage Audit

Extend `docs/scripts/audit-docs.mjs` to parse the top-level `Commands` enum and resolve explicit Clap `#[command(name = "...")]` overrides before PascalCase-to-kebab conversion. Require matching Chinese and English command pages for all stable commands except:

- `help`, which must be explicitly documented by the command overview;
- conceptual pages such as `tui`, which are allowed extras.

The audit also defines internal-only files/directories (`README.md`, `AGENTS.md`, `TODO.md`, `reports/`) so they are not treated as localized product pages.

## Tooling Alignment

- `docs/justfile audit` becomes the documentation audit required by `docs/AGENTS.md`.
- Dependency security audit moves to an explicit `security-audit` recipe.
- Root `just docs-check` uses Bun and runs VitePress build, product docs audit, and `ccr-ui` docs audit.
- No new JavaScript dependency is needed.

## Compatibility And Rollback

- Existing public URLs remain stable; only the missing `claude` command page is added.
- Navigation regrouping is reversible without content loss.
- Source-based audit changes land with the pages they protect.
- Generated VitePress output remains ignored and unmodified.
