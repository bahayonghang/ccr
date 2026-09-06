# ccr-ui/ code map

Navigation map for `ccr-ui/**`. Behavioral and visual rules stay in `./AGENTS.md` and `DESIGN.md`; this file only routes the React shell, Tauri backend, tests, and scripts before broad grep. Do not redefine visual tokens here.

## Start here

- Scoped agent rules and market-terminal visual direction: `./AGENTS.md`, `DESIGN.md`.
- Frontend manifest and scripts: `package.json` (`packageManager: bun@1.4.0`).
- UI just recipes: `justfile` (`dev-web`, `check`, `test`, `build`, `tauri-*`).
- Vite/Vitest/ESLint entry points: `vite.config.ts`, `vitest.smoke.config.ts`, `eslint.config.js`.
- Tauri backend manifest: `src-tauri/Cargo.toml`; desktop config: `src-tauri/tauri.conf.json`.

## Runtime layers

| Layer | Paths | Use when |
|---|---|---|
| React app shell | `src/main.tsx`, `src/shell/` (`App.tsx`, `router.tsx`, `routeCatalog.ts`, `MainLayout.tsx`, Query client) | App bootstrap, layout, route registration, navigation. |
| Features | `src/features/` | Page-level and domain UI: Claude, Codex, Grok, Gemini/Antigravity, OpenCode, MCP, skills, sync, usage, monitoring, configs, commands, check-in, tray, agent-sessions. |
| API | `src/api/domains/`, `src/api/generated/`, `src/api/index.ts` | Domain wrappers and generated typed clients. `src/api/tauri.ts` is a compatibility facade; new wrappers belong in `src/api/domains/*`. |
| Shared frontend contracts | `src/types/` (including `src/types/generated/`), `src/utils/`, `src/config/`, `src/configs/`, `src/i18n/`, `src/styles/`, `src/ui/` | DTOs (ts-rs generated types from Rust), helpers, localization, tokens. |
| Tauri backend | `src-tauri/src/` | Rust commands, app state, monitoring/events, background jobs, platform integrations, and desktop shell behavior. |
| Tests | `tests/` | Vitest smoke tests grouped by domain (`api/`, `profiles/`, `usage/`, `shell/`, …), plus `setup/`, `helpers/`, `fixtures/`, and `i18n.test.cjs`. Prefer focused `*.smoke.test.{ts,tsx}` updates in the matching domain folder. |
| Tooling/scripts | `scripts/` | Dev-server warm start, route snapshots, bundle checks, icon generation, release-window verification. |

Legacy `src/views/` and `src/composables/` paths may still exist; new page work belongs under `src/features/` and `src/shell/`. Do not add Vue SFCs or Pinia stores.

## Frontend route map

- Route registration: `src/shell/router.tsx` + `src/shell/routeCatalog.ts`.
- High-level pages: `src/features/<domain>/*View.tsx` (and related screens).
- Domain folders (non-exhaustive): `src/features/claude/`, `codex/`, `grok/`, `gemini/`, `opencode/`, `mcp/`, `configs/`, `sync/`, `usage/`, `monitoring/`, `commands/`, `checkin/`, `agent-sessions/`, `tray/`, `platform/`.
- Shared UI primitives live under `src/ui/` and remaining `src/components/` (for example profiles).

## Tauri backend map

| Path | Purpose |
|---|---|
| `src-tauri/src/main.rs` | Tauri app bootstrap and command registration. |
| `src-tauri/src/state.rs` | Shared app state and service wiring. |
| `src-tauri/src/commands/` | Tauri invoke command handlers by domain. |
| `src-tauri/src/llmusage_adapter/` | CLI sync, NDJSON events, and DTO/error mapping only. Usage SQL belongs in `crates/ccr-usage`. Do not link the upstream `llmusage` crate. |
| `src-tauri/src/platform/`, `src-tauri/src/process/`, `src-tauri/src/ssh/` | Local platform detection, process helpers, SSH/WSL integration. |
| `src-tauri/src/*_jobs.rs`, `src-tauri/src/monitoring.rs`, `src-tauri/src/events.rs` | Background jobs, monitoring feed, and event emission. |
| `src-tauri/tests/` | Rust-side integration tests and guards. |

## Test and verification routing

- Web preview for visual/browser work: `bun run dev:web -- --host 127.0.0.1 --strictPort`, then open `http://127.0.0.1:5173/`. Browser or Playwright tools being available is not authorization to operate the UI.
- Frontend unit/smoke tests: `bun run test`; i18n only: `bun run test:i18n`; smoke only: `bun run test:smoke`.
- Frontend type/build checks: `bun run type-check`, `bun run build`, `bun run check:i18n`, `bun run check:bundle-budget`.
- Tauri backend checks: `bun run tauri:check`, `bun run tauri:test`, `bun run tauri:clippy`.
- Just gates from this directory: `just check`, `just test`, `just build`; root aliases include `just ui-check`, `just ui-test`, `just ui-build`, and `just tauri-check`. `just check` reaches frontend linting, which may auto-fix files through the package `lint` script.

## Safety and generated-output boundaries

- Do not commit generated or local runtime output from `dist/`, `src-tauri/target/`, `storybook-static/`, `test-results/`, `tests/__screenshots__/`, `logs/`, `.tmp/`, `.omx/`, `.omc/`, or `node_modules/`.
- Tauri commands may read or write real user tool config under `.ccr`, `.claude`, `.codex`, `.gemini`, OpenCode, WSL, SSH, or sync paths; preserve masking, backup, atomic-write, and confirmation behavior.
- Web-mode browser validation cannot exercise every Tauri `invoke()` path; distinguish browser-runtime limitations from product regressions.
- `src-tauri/src/llmusage_adapter/` does not own usage SQL. Do not bypass `crates/ccr-usage` for transcript or usage parsing.

## Design navigation

- Visual world, tokens, and rejected ruts: `./AGENTS.md` and `DESIGN.md`. Tokens mirror `src/styles/tokens.css`.
- Design assets and Storybook live in `.storybook/`, `design-system/`, and Storybook scripts; generated Storybook output belongs in `storybook-static/` and should not be treated as source guidance.
