<!-- TRELLIS:START -->
# mini-trellis

This project uses mini-trellis as a memory layer. Working knowledge lives under `.trellis/`:

- `.trellis/spec/` — durable contracts (short markdown; read `index.md` first)
- `.trellis/research/` — topic inbox (`<topic>.md`; `README.md` is not a topic)
- `.trellis/workspace/` — per-developer journals

Record a session with `/mini-trellis:remember` (Claude/OpenCode), `/mini-trellis-remember` (Pi), `$mini-trellis-remember` (Codex/ZCode), or the `mini-trellis-remember` skill (Cursor and DeepSeek Harness). When earlier decisions or relevant project rules are needed, use `mini-trellis mem recall "<topic or path>"`; expand evidence with `mini-trellis mem read "<ref>"`. No search is required for unrelated work. Promote lasting boundaries into spec with `mini-trellis-update-spec`.

Record only confirmed decisions, reusable research findings, or the session/compact handoff. Skip ordinary turns and unconfirmed guesses. New installs write notes without an automatic git commit; use `mini-trellis doctor` to check host wiring and `mini-trellis refresh --dry-run` before refreshing assets.

Keep confirmed rules short and faithful. Unconfirmed scope, exceptions, and implementation choices stay open or are labeled as inference, not added to the contract.

For a confirmed update, follow this short order: read the existing topic; edit that file; read it back and check the new rule, limits, and negations; run `python3 ./.trellis/scripts/add_session.py --title "..." --summary "..." --topic <actual-topic-path>`; read the journal and check its topic link. Journal-only success is not topic success; report partial writes accurately. New notes follow the requested or discussion language; updates preserve the document language.

When recall excerpts already answer the question with source and qualifiers, cite them and stop. Expand only missing context; do not repeat a lookup or fall back to dialogue-only `search` merely to reconfirm sufficient evidence.

ZCode conversation history is not parsed; use project knowledge or supply Markdown/text exports. For an explicit request to organize selected past sessions or supplied Markdown/text exports into project knowledge, use `mini-trellis-session-insight`; update existing documents and preserve sources.

mini-trellis does not create or drive tasks.

Managed by mini-trellis. Edits outside this block are preserved.
<!-- TRELLIS:END -->

# Repository Guidelines

## Project Structure & Module Organization
`crates/` is the Rust workspace. `crates/ccr` is the installable CLI/TUI entry point, while shared logic lives in crates such as `ccr-core`, `ccr-config`, `ccr-codex`, `ccr-db`, `ccr-usage`, and `ccr-types`. `ccr-ui/` contains the React 19 + TanStack Query + Tauri typed-IPC app (`src/`, `src-tauri/`, `tests/`). Visual direction for that app is `ccr-ui/AGENTS.md` and `ccr-ui/DESIGN.md` (airy-glass workbench). `ccr-vscode/` contains the VS Code extension (`src/providers`, `src/services`). `docs/` holds VitePress docs. `scripts/` holds repo automation and version-sync checks. Before broad repo searches, read `./code_map.md` to identify the relevant module and verification anchors. Five-tool harness entry files, load chains, and reviewer vs implementer permissions are in `docs/agents/harnesses.md` (English: `docs/en/agents/harnesses.md`). Usage analytics invoke the installed [`llmusage`](https://github.com/bahayonghang/llmuasage) CLI and read its SQLite database; the shared projection owner is `crates/ccr-usage` (all usage SQL). `ccr-ui/src-tauri/src/llmusage_adapter/` is CLI sync, NDJSON events, and DTO/error mapping only — do not link the upstream `llmusage` Rust crate.

## Build, Test, and Development Commands
- `just build` — build the Rust CLI in debug mode.
- `just test` — run Rust workspace tests.
- `just ci` — run the repo-wide CI path: version checks, fmt, clippy, tests, build, audit, and frontend checks.
- `just clean` — remove rebuildable artifacts (Cargo/Tauri `target`, frontend/docs/vscode dist, coverage, outputs). `just clean-all` also deletes `node_modules`.
- `just ui-dev` / `just ui-check` — develop or validate the Tauri UI.
- `cd ccr-ui && bun run dev` — run the web UI locally.
- `cd ccr-ui && bun run test` — run i18n and Vitest smoke tests.
- `cd ccr-vscode && npm run build && npm test` — build and test the VS Code extension.

Quick-check priority:
- General preflight: `just version-check`, then `just fmt-check`.
- Rust changes: `just lint-strict`, then `just test`.
- Frontend fast feedback: `just frontend-check-quick`.
- UI complete checks: `just ui-check` for the Tauri UI, or `just frontend-check` for frontend build/docs coverage.
- VS Code extension: `just vscode-ci`.
- Final full gate: `just ci` for cross-module changes, release readiness, or final acceptance.

## Coding Style & Naming Conventions
Rust code must stay `cargo fmt` and clippy clean. Prefer `Result`-based error handling. Do not add `unwrap` or `expect` in production paths. Rust files and modules use `snake_case`; structs, enums, and traits use `PascalCase`.

Frontend and extension code use 2-space indentation, single quotes, and no semicolons. Frontend components use `PascalCase.tsx`. Follow existing store, service, and component patterns before adding new abstractions.

## Testing Guidelines
Keep Rust integration tests under `crates/*/tests` and group them by feature area. UI smoke tests belong in `ccr-ui/tests/**/*.smoke.test.{ts,tsx}`. VS Code tests live beside source as `*.test.ts`. Start with the narrowest relevant check, then escalate by impact scope: subsystem checks for local changes, `just frontend-check` or `just vscode-ci` for full frontend/extension coverage, and `just ci` for cross-module changes or delivery-ready validation.

Shared skill sources are tracked under `.github/skills/`; several also apply to Claude Code, Codex, Grok, Kimi, and OMP, and each client installs them into its own local skill directory (`.codex/skills/` and similar are gitignored local paths that `trellis update` may rewrite). Prefer the narrowest failing gate first before escalating to full `just ci`. See `docs/agents/harnesses.md` for routing and permissions.

## Five-Harness Execution Contract
These rules apply to Claude Code, Codex, Grok Build, Kimi Code, and OMP.
- Root and Tauri Rust tests use default parallelism with `--skip export_bindings`. The separate binding-generation gate owns exports.
- A read-only reviewer inspects the approved scope. Run checks and write task evidence only when authorized. The Trellis check role may write and self-fix after implementation approval.
- Strong-model review owns root cause, security, permission boundaries, and final acceptance. Lower-cost execution follows fixed files, acceptance criteria, and commands. Model choice does not change permissions.
- Distinguish repository-delivered files, locally generated integration, and verified native loading. A fresh checkout needs the matching Trellis integration and a separate client loading/trust check.
- Keep repair commands (`just fmt`, `just version-sync`) separate from validation. Preserve required coverage thresholds, default parallelism, and hosted context names.

## Commit & Pull Request Guidelines
Recent history uses Chinese Conventional Commits with scopes and emoji, for example `feat(认证TUI): ✨ ...`, `docs(帮助文档): 📝 ...`, and `chore(release): 🔧 ...`. Keep commits atomic and scoped to one surface.

PRs should state the affected area (`CLI`, `ccr-ui`, `ccr-vscode`, docs), link the issue when available, list the verification commands you ran, and include screenshots or GIFs for UI and extension changes.

## Security & Configuration Tips
Never commit personal config, tokens, or local runtime files from home-directory toolchains. Use examples and fixtures instead. When editing config flows, preserve backup, masking, and atomic-write behavior.
