# GitHub Copilot Workspace Support

CCR now ships the official GitHub Copilot for VS Code workspace assets in-repo and keeps them clearly separated from Codex CLI runtime configuration.

## Asset Map

| Path | Purpose |
|------|---------|
| `.github/copilot-instructions.md` | repository-wide default guidance |
| `.github/instructions/*.instructions.md` | scoped Rust, UI, and docs instructions |
| `.github/prompts/*.prompt.md` | reusable prompt starters |
| `.github/agents/*.agent.md` | reusable custom Copilot agents |
| `AGENTS.md` | tracked shared project rules |
| `.github/skills/*/SKILL.md` | tracked source of shared checks and workflow guidance |

## Important Boundary

### GitHub Copilot for VS Code

- reads workspace assets from `.github/*`
- can discover shared project skills
- powers VS Code Chat and Agent Mode collaboration

### Codex CLI

- keeps runtime configuration under `~/.codex/`
- is managed by CCR profiles under `~/.ccr/platforms/codex/profiles.toml`
- is not the same customization surface as GitHub Copilot workspace assets

## Shared Rules and Local Installation

Shared rules reference `AGENTS.md` and Git-tracked `.github/skills/*/SKILL.md` files. Claude Code, Codex, Grok Build, Kimi Code, and OMP can read that guidance and run the same checks. Each client installs them into its own local skill directory (such as `.codex/skills/`, excluded by Git), and each client configuration controls automatic skill discovery and loading.

`.claude/skills/` contains optional local installation files excluded by Git. The asset check validates the tracked shared sources in a fresh checkout. New GitHub Copilot-specific skills require an explicit ownership and delivery decision.

## What This Repository Adds

- one repository-wide Copilot instruction file
- three scoped instruction files for Rust, UI, and docs
- three reusable prompt files
- three reusable custom agents: `researcher`, `implementer`, and `reviewer`
- `just copilot-check` plus `scripts/quality/check-copilot-assets.mjs` to verify the asset set and catch naming drift between GitHub Copilot and Codex CLI

## Maintenance Rules

1. When you add or rename `.github/*` assets, update this page and the VitePress sidebar in the same change.
2. Maintain shared rules in `AGENTS.md` and tracked `.github/skills/*/SKILL.md` files. Manage local installation directories through each tool configuration.
3. In docs, `GitHub Copilot` means the VS Code workspace features; `Codex` means Codex CLI.
4. After changing assets or the checker, run `node --test scripts/quality/check-copilot-assets.test.mjs`, `just copilot-check`, and `just docs-check`. All five execution tools use these commands.

The checker normalizes LF and CRLF in memory and accepts one UTF-8 BOM at the start of a file. Document bytes remain unchanged. Frontmatter requires separate `---` lines as opening and closing delimiters. The closing delimiter can occur at the end of the file. Current assets use single-line scalar fields with nonempty required values. Missing fields, blank or comment-only values, unclosed delimiters, duplicate fields, and malformed quotes produce a nonzero exit code.

## Official References

- [Custom instructions](https://code.visualstudio.com/docs/copilot/customization/custom-instructions)
- [Prompt files](https://code.visualstudio.com/docs/copilot/customization/prompt-files)
- [Custom agents](https://code.visualstudio.com/docs/copilot/customization/custom-agents)
- [Agent skills](https://code.visualstudio.com/docs/copilot/customization/agent-skills)
