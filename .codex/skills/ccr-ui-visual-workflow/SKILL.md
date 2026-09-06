---
name: ccr-ui-visual-workflow
description: CCR UI visual verification workflow. Use for ccr-ui visual design, layout, screenshot, Playwright, Browser, route preview, or frontend polish tasks. Validate the web preview; do not default to the Tauri desktop shell. Applies to Claude Code, Codex, Grok, Kimi, and OMP. UI tools available are not UI operation authorization.
---

# CCR UI Visual Workflow

Use the web preview plus Browser-based inspection for visual work in `ccr-ui/`. Do not default to the Tauri desktop shell unless the task explicitly depends on native Tauri window APIs.

This skill applies to Claude Code, Codex, Grok Build, Kimi Code, and OMP — not Codex only. A browser, Playwright, or UI tool being available is **not** authorization to operate the UI.

## Default Loop

1. Work from `ccr-ui/`.
2. Start the web preview:

   ```powershell
   bun run dev:web -- --host 127.0.0.1 --strictPort
   ```

3. Open Browser at:

   ```text
   http://127.0.0.1:5173/
   ```

4. Navigate to the relevant route, take screenshots when useful, and compare against the task's visual intent.
5. Iterate on React (`PascalCase.tsx`), CSS, i18n, and smoke tests until the route is visually coherent and functionally intact.
6. Run the narrow verification for the changed surface. These gates are independent and may run in parallel:

   ```powershell
   bun run type-check
   bun run lint
   bun run test
   ```

## Visual Direction

- Follow `ccr-ui/AGENTS.md` and `ccr-ui/DESIGN.md` (market terminal). Do not redefine tokens in this skill.
- Avoid reintroducing heavy glassmorphism, purple-led generic SaaS styling, anime/catgirl/NEKO, or `guofeng` visual language.
- Optimize for advanced AI CLI users: dense information, clear readiness states, safe next actions, and fast scanning.

## Tauri Boundary

- Plain web preview may not support Tauri-only `invoke()` paths. Treat those as environment limitations, not visual test failures.
- Use `bun run tauri:dev`, `just tauri-dev`, or desktop-window automation only when the task explicitly requires native Tauri behavior.
- Do not fake backend/native capability in UI copy; surface unsupported or unavailable states honestly.

## Evidence To Report

- route(s) inspected
- screenshot/browser evidence when collected
- commands run
- known limitations, especially any Tauri-only paths not exercised in web mode
- whether UI operation was explicitly authorized
