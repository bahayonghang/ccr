---
description: "Implement a CCR UI or extension change while preserving existing patterns"
agent: implementer
---

Implement the requested UI or extension change in this repository.

Checklist:
- Read `.github/copilot-instructions.md` and `.github/instructions/ui.instructions.md`.
- Preserve the current React 19 + TanStack Query, Tauri, and VS Code extension structure. Do not add Vue SFCs or Pinia stores.
- Keep the diff focused and avoid introducing new dependencies unless explicitly required.
- Reuse existing components, styles, and stores before creating new ones.
- Run the narrowest relevant UI verification and summarize the evidence.
