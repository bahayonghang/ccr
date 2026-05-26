# ccr-ui Agent Notes

These notes apply to everything under `ccr-ui/` and supplement the repository-level `AGENTS.md`. Read `./code_map.md` before broad grep or repo-wide code search inside `ccr-ui/`.

## Local Verification

- For full local UI checks from this directory, use `just check`; from the repository root, use `just ui-check`. The frontend lint path can apply ESLint/Stylelint fixes, so inspect the diff after running it.
- For narrow frontend checks, use `bun run type-check`, `bun run test`, and `bun run build`.
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

- The target brand tone is `calm / precise / editorial`.
- Preserve a strong product identity instead of drifting into generic SaaS admin styling or mascot-heavy novelty.
- The UI should feel like a purpose-built Anthropic-like workbench for advanced users: quiet, focused, and trustworthy under high information density.

### Aesthetic Direction

- Align new work to an `Anthropic-like editorial surface` direction.
- Prefer warm neutrals, charcoal text, subtle borders, restrained translucency, and low-chroma accents over purple-led gradients or loud glass effects.
- Treat heavy Liquid Glass / Glassmorphism, anime atmospherics, and catgirl / NEKO branding as legacy directions to remove over time.
- Remove `guofeng`, `neko`, anime, and purple-tech styling, naming, tokens, and component variants over time; treat them as legacy, not parallel design languages.

### Accessibility And Motion

- Support both light and dark themes.
- Maintain high contrast in both themes.
- Use motion to improve hierarchy, feedback, and polish, but keep it restrained and non-disruptive.
- Preserve reduced-motion compatibility for core interactions and animated surfaces.

### Design Principles

- Power-first UX: optimize for speed, density, and control for expert users.
- Distinctive, not generic: express identity through calm precision, editorial hierarchy, and restrained surfaces instead of mascots, neon accents, or decorative glass.
- Style supports usability: visual layers must improve readability, navigation clarity, and task completion rather than compete with them.
- One visual language: converge on the Anthropic-like editorial direction and eliminate conflicting `guofeng` / `neko` branches.
