# ccr-ui Agent Notes

These notes apply to everything under `ccr-ui/` and supplement the repository-level `AGENTS.md`. Read `./code_map.md` before broad grep or repo-wide code search inside `ccr-ui/`.

## Local Verification

- For full local UI checks from this directory, use `just check`; from the repository root, use `just ui-check`. The default frontend lint path is no-fix (`bun run lint`); use `bun run lint:fix` only when intentionally mutating files locally.
- For narrow frontend checks, use `bun run type-check`, `bun run test`, and `bun run build`. Smoke tests live in domain folders under `tests/` (`tests/profiles/`, `tests/usage/`, …); Vitest include is `tests/**/*.smoke.test.{ts,tsx}`.
- For Tauri Rust checks, use `bun run tauri:check`, `bun run tauri:test`, and `bun run tauri:clippy`.

## Playwright And Browser Automation

- For Playwright work, prefer the web version instead of the Tauri desktop shell.
- Start the frontend from `ccr-ui/` with `bun run dev:web -- --host 127.0.0.1 --strictPort`.
- Use `http://127.0.0.1:5173/` as the default target URL for browser automation.
- Do not use `bun run tauri:dev` or `just tauri-dev` for Playwright tasks unless the task explicitly requires Tauri window APIs.
- When validating UI in web mode, note that Tauri-only `invoke()` paths may not work in a plain browser runtime; treat those as runtime-environment limitations rather than Playwright failures.

## Design Context

### Users

- The primary users are AI CLI power users.
- Optimize for frequent configuration, switching, monitoring, and management of multiple AI CLI tools, MCP, Agents, plugins, sync, and runtime state.
- Favor high information density, fast scanning, and operational clarity over beginner-friendly simplification.

### Brand Personality

- The target brand tone is `calm / precise / airy`.
- Preserve a strong product identity instead of drifting into generic SaaS admin styling or mascot-heavy novelty.
- The UI should feel like a purpose-built airy workbench for advanced users: quiet, focused, and trustworthy under high information density.

### Aesthetic Direction

- The approved visual world is the airy-glass workbench (轻盈玻璃): one paper-like application sheet over a static mint-to-lavender field, with matched light/dark palettes. `DESIGN.md` is the source of truth; `.impeccable/design.json` is its machine-readable sidecar.
- Only the non-scrolling app sheet may blur (20px, saturate 1.15); cards, inputs, tables, charts, sticky surfaces and overlays stay opaque. At <960px, reduced transparency or no blur support, the sheet is opaque and blur is none.
- Green marks action/link/focus, purple only explicit output. Preserve independent danger/warning/status and platform/source identities. Preserve bounded charts, tabular figures, fixed virtual rows, honest empty states and the command status bar.
- Preserve theme/flavor/accent/font preference domains, bootstrap migration, reduced-motion ownership and existing information density. Do not add a second theme provider or native transparency.
- Avoid nested/decorative glass, floating blobs, anime/catgirl/NEKO and `guofeng` branches.

### Accessibility And Motion

- Support both light and dark themes.
- Maintain high contrast in both themes.
- Use motion to improve hierarchy, feedback, and polish, but keep it restrained and non-disruptive.
- Preserve reduced-motion compatibility for core interactions and animated surfaces.

### Design Principles

- Power-first UX: optimize for speed, density, and control for expert users.
- Distinctive, not generic: express identity through calm precision, clear hierarchy, and restrained surfaces with one restrained sheet and solid reading surfaces.
- Style supports usability: visual layers must improve readability, navigation clarity, and task completion rather than compete with them.
- One visual language: converge on the airy-glass direction defined in `DESIGN.md` and eliminate conflicting `guofeng` / `neko` branches.
