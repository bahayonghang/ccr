# GitHub Copilot Workspace Support

CCR now ships the official GitHub Copilot for VS Code workspace assets in-repo and keeps them clearly separated from Codex CLI runtime configuration.

## Asset Map

| Path | Purpose |
|------|---------|
| `.github/copilot-instructions.md` | repository-wide default guidance |
| `.github/instructions/*.instructions.md` | scoped Rust, UI, and docs instructions |
| `.github/prompts/*.prompt.md` | reusable prompt starters |
| `.github/agents/*.agent.md` | reusable custom Copilot agents |
| `.claude/skills/` | canonical shared project skills |

## Important Boundary

### GitHub Copilot for VS Code

- reads workspace assets from `.github/*`
- can discover shared project skills
- powers VS Code Chat and Agent Mode collaboration

### Codex CLI

- keeps runtime configuration under `~/.codex/`
- is managed by CCR profiles under `~/.ccr/platforms/codex/profiles.toml`
- is not the same customization surface as GitHub Copilot workspace assets

## Why There Is No `.github/skills/`

GitHub Copilot can discover shared skills from `.claude/skills/`, `.github/skills/`, and `.agents/skills/`. This repository intentionally keeps `.claude/skills/` as the single source of truth so the same skills are not duplicated across multiple directories.

If the repository ever needs GitHub Copilot-specific skills, that can be introduced later with an explicit ownership decision.

## What This Repository Adds

- one repository-wide Copilot instruction file
- three scoped instruction files for Rust, UI, and docs
- three reusable prompt files
- three reusable custom agents: `researcher`, `implementer`, and `reviewer`
- `just copilot-check` plus `scripts/quality/check-copilot-assets.mjs` to verify the asset set and catch naming drift between GitHub Copilot and Codex CLI

## Maintenance Rules

1. When you add or rename `.github/*` assets, update this page and the VitePress sidebar in the same change.
2. Shared project skills stay in `.claude/skills/` by default.
3. In docs, `GitHub Copilot` means the VS Code workspace features; `Codex` means Codex CLI.
4. Run `just copilot-check` before landing related changes.

## Official References

- [Custom instructions](https://code.visualstudio.com/docs/copilot/customization/custom-instructions)
- [Prompt files](https://code.visualstudio.com/docs/copilot/customization/prompt-files)
- [Custom agents](https://code.visualstudio.com/docs/copilot/customization/custom-agents)
- [Agent skills](https://code.visualstudio.com/docs/copilot/customization/agent-skills)
