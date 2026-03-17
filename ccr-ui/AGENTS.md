# ccr-ui Agent Notes

These notes apply to everything under `ccr-ui/` and supplement the repository-level `AGENTS.md`.

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

- The target brand tone is `geeky / playful / distinctive`.
- Preserve a strong product identity instead of drifting into generic SaaS admin styling.
- The UI should feel like a purpose-built control console for advanced users, with personality but without losing usability.

### Aesthetic Direction

- Keep the purple-led palette as the main visual axis.
- Keep Liquid Glass / Glassmorphism layering.
- Keep anime-style background atmosphere.
- Keep catgirl / NEKO naming and light decorative branding.
- Remove `guofeng` styling, naming, tokens, and component variants over time; treat them as legacy, not a parallel design language.

### Accessibility And Motion

- Support both light and dark themes.
- Maintain high contrast in both themes.
- Use motion to improve hierarchy, feedback, and polish, but keep it restrained and non-disruptive.
- Preserve reduced-motion compatibility for core interactions and animated surfaces.

### Design Principles

- Power-first UX: optimize for speed, density, and control for expert users.
- Distinctive, not generic: keep the Neko + Liquid Glass + purple-tech identity visible.
- Style supports usability: decorative layers must not reduce readability, navigation clarity, or task completion.
- One visual language: converge on the Neko / Liquid Glass direction and eliminate conflicting `guofeng` branches.
