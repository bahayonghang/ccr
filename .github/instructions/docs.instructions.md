---
applyTo: "docs/**/*.md,docs/.vitepress/**/*.mjs,examples/**/*.md,ccr-ui/README*.md"
description: "Documentation naming and path conventions for CCR"
---

# Documentation Instructions

- Use exact product names:
  - GitHub Copilot means the VS Code extension and workspace assets under repository `.github/`.
  - Codex CLI means runtime/config files under `~/.codex/`.
  - CCR Unified Codex profiles mean `~/.ccr/platforms/codex/profiles.toml`.
- Do not use GitHub Copilot product naming when the subject is Codex CLI.
- Keep docs aligned with real implementation paths in code.
- When GitHub Copilot workspace support is relevant, point readers to `docs/guide/github-copilot-workspace.md` or `docs/en/guide/github-copilot-workspace.md`.
