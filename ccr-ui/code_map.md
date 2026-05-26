# ccr-ui/ code map

Navigation map for `ccr-ui/**`. Behavioral rules stay in `./AGENTS.md`; use this file to route Vue, Tauri, tests, and scripts before broad grep.

## Start here

- Scoped agent rules and visual direction: `./AGENTS.md`.
- Frontend manifest and scripts: `package.json`.
- UI just recipes: `justfile` (`dev-web`, `check`, `test`, `build`, `tauri-*`).
- Vite/Vitest/ESLint entry points: `vite.config.ts`, `vitest.smoke.config.ts`, `eslint.config.js`.
- Tauri backend manifest: `src-tauri/Cargo.toml`; desktop config: `src-tauri/tauri.conf.json`.

## Runtime layers

| Layer | Paths | Use when |
|---|---|---|
| Vue app shell | `src/main.ts`, `src/App.vue`, `src/router/`, `src/layouts/` | App bootstrap, global layout, route registration, navigation behavior. |
| Views | `src/views/` | Page-level behavior for Claude, Codex, OpenCode, Gemini, MCP, skills, sync, usage, monitoring, settings, and dashboard screens. |
| Components | `src/components/` | Shared UI widgets and domain component groups; check the matching domain folder before adding a new component. |
| State and data | `src/stores/`, `src/api/`, `src/composables/` | Pinia state, Tauri/backend API wrappers, streaming hooks, reusable view logic. |
| Shared frontend contracts | `src/types/`, `src/utils/`, `src/config/`, `src/configs/`, `src/i18n/`, `src/styles/` | DTOs, presentation helpers, static config, localization, theme/style primitives. |
| Tauri backend | `src-tauri/src/` | Rust commands, app state, monitoring/events, background jobs, platform integrations, and desktop shell behavior. |
| Tests | `tests/` | Vitest smoke/i18n tests and helpers. Prefer focused `*.smoke.test.ts` updates beside the changed UI behavior. |
| Tooling/scripts | `scripts/` | Dev-server warm start, route snapshots, bundle checks, icon generation, release-window verification. |

## Frontend route map

- Route registration: `src/router/index.ts`.
- High-level pages: `src/views/*View.vue`.
- Domain components:
  - Claude: `src/components/claude/`, `src/views/Claude*View.vue`.
  - Codex: `src/components/codex/`, `src/views/Codex*View.vue`.
  - OpenCode: `src/components/opencode/`, `src/views/OpenCode*View.vue`.
  - MCP/config/sync/skills: `src/components/mcp/`, `src/components/configs/`, `src/components/sync/`, `src/components/skills/`.
  - Usage/dashboard/monitoring: `src/components/usage/`, `src/components/dashboard/`, `src/components/platform-usage/`, matching `src/views/*Usage*`, `DashboardView.vue`, `MonitoringView.vue`.
- Shared UI primitives live under `src/components/ui/`, `src/components/common/`, and `src/components/layout/`.

## Tauri backend map

| Path | Purpose |
|---|---|
| `src-tauri/src/main.rs` | Tauri app bootstrap and command registration. |
| `src-tauri/src/state.rs` | Shared app state and service wiring. |
| `src-tauri/src/commands/` | Tauri invoke command handlers by domain: Claude, Codex, OpenCode, MCP, sync, skills, usage, check-in, WSL, SSH, settings. |
| `src-tauri/src/llmusage_adapter/` | Adapter boundary around the pinned external `llmusage` git dependency; preserve compatibility wrappers. |
| `src-tauri/src/platform/`, `src-tauri/src/process/`, `src-tauri/src/ssh/` | Local platform detection, process helpers, SSH/WSL integration. |
| `src-tauri/src/*_jobs.rs`, `src-tauri/src/monitoring.rs`, `src-tauri/src/events.rs` | Background jobs, monitoring feed, and event emission. |
| `src-tauri/tests/` | Rust-side integration tests and guards. |

## Test and verification routing

- Web preview for visual/browser work: `bun run dev:web -- --host 127.0.0.1 --strictPort`, then open `http://127.0.0.1:5173/`.
- Frontend unit/smoke tests: `bun run test`; i18n only: `bun run test:i18n`; smoke only: `bun run test:smoke`.
- Frontend type/build checks: `bun run type-check`, `bun run build`, `bun run check:i18n`, `bun run check:bundle-budget`.
- Tauri backend checks: `bun run tauri:check`, `bun run tauri:test`, `bun run tauri:clippy`.
- Just gates from this directory: `just check`, `just test`, `just build`; root aliases include `just ui-check`, `just ui-test`, `just ui-build`, and `just tauri-check`. `just check` reaches frontend linting, which may auto-fix files through the package `lint` script.

## Safety and generated-output boundaries

- Do not commit generated or local runtime output from `dist/`, `src-tauri/target/`, `storybook-static/`, `test-results/`, `logs/`, `.tmp/`, `.omx/`, `.omc/`, or `node_modules/`.
- Tauri commands may read or write real user tool config under `.ccr`, `.claude`, `.codex`, `.gemini`, OpenCode, WSL, SSH, or sync paths; preserve masking, backup, atomic-write, and confirmation behavior.
- Web-mode browser validation cannot exercise every Tauri `invoke()` path; distinguish browser-runtime limitations from product regressions.
- `src-tauri/src/llmusage_adapter/` shields the app from upstream schema drift; do not bypass it for direct transcript or usage parsing.

## Design navigation

- Current visual direction is documented in `./AGENTS.md`; keep new UI aligned with the calm, precise, editorial surface direction.
- Design assets and Storybook live in `.storybook/`, `design-system/`, and Storybook scripts; generated Storybook output belongs in `storybook-static/` and should not be treated as source guidance.
