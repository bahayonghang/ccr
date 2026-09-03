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

- The target brand tone is `calm / precise / terminal`.
- Preserve a strong product identity instead of drifting into generic SaaS admin styling or mascot-heavy novelty.
- The UI should feel like a purpose-built market terminal for advanced users: quiet, focused, and trustworthy under high information density.

### Aesthetic Direction

- The visual world is the market terminal (行情终端): a warm-dark, dark-first operating surface where amber marks command, active, and focus; green and red carry status only; and mono type with tabular figures is earned by data contexts. `DESIGN.md` in this directory is the source of truth (tokens mirror `src/styles/tokens.css`); `.impeccable/design.json` is its machine-readable sidecar.
- Align new work to the world's signatures: hairline-ruled panels, 2px identity ticks, bounded charts, honest empty states, and the bottom command status bar. The rejected rut is the metric-card row plus unbounded hero chart.
- Treat heavy Liquid Glass / Glassmorphism, anime atmospherics, and catgirl / NEKO branding as legacy directions to remove over time.
- Remove `guofeng`, `neko`, anime, and purple-tech styling, naming, tokens, and component variants over time; treat them as legacy, not parallel design languages.

### Accessibility And Motion

- Support both light and dark themes.
- Maintain high contrast in both themes.
- Use motion to improve hierarchy, feedback, and polish, but keep it restrained and non-disruptive.
- Preserve reduced-motion compatibility for core interactions and animated surfaces.

### Design Principles

- Power-first UX: optimize for speed, density, and control for expert users.
- Distinctive, not generic: express identity through calm precision, clear hierarchy, and restrained surfaces instead of mascots, neon accents, or decorative glass.
- Style supports usability: visual layers must improve readability, navigation clarity, and task completion rather than compete with them.
- One visual language: converge on the market-terminal direction defined in `DESIGN.md` and eliminate conflicting `guofeng` / `neko` branches.
