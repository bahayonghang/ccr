---
applyTo: "ccr-ui/**/*.{ts,tsx,vue,css,scss,md},ccr-vscode/**/*.{ts,tsx,js,json,md}"
description: "Frontend, Tauri UI, and VS Code extension conventions for CCR"
---

# UI Instructions

- Preserve the existing Vue 3, Tauri, and extension patterns already used in the repository.
- Keep 2-space indentation, no semicolons, and single quotes in frontend code.
- Reuse existing components, stores, and styles before adding new patterns.
- Do not conflate GitHub Copilot workspace assets under `.github/` with Codex CLI runtime files under `~/.codex/`.
- Use the narrowest relevant UI verification command, such as `just frontend-check`, `just frontend-check-quick`, or `just ui-check`.
