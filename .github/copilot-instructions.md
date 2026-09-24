# GitHub Copilot Workspace Instructions

This repository keeps GitHub Copilot for VS Code workspace assets under `.github/`.

## Boundaries

- GitHub Copilot workspace customization lives in `.github/copilot-instructions.md`, `.github/instructions/`, `.github/prompts/`, and `.github/agents/`.
- Tracked shared project skills stay in `.codex/skills/`. `.claude/` is git-ignored, so `.claude/skills/` exists only in local checkouts that create it.
- Codex CLI runtime/config lives in the user home directory under `~/.codex/`.
- CCR Unified Codex profiles live under `~/.ccr/platforms/codex/profiles.toml`.

Do not use GitHub Copilot naming for Codex CLI. They are different products and use different customization surfaces.

## Working Rules

- Follow repository `AGENTS.md` and nearby project docs before editing.
- Keep diffs small and reversible; reuse existing abstractions before adding new ones.
- Do not add dependencies without an explicit request.
- Prefer `just` recipes for verification when they exist.
- Preserve masking, backup, lock, and atomic-write behavior in config flows.
- Keep internal implementation comments in Chinese and public API docs in English.

## Scoped Instructions

- Rust workspace: [`./instructions/rust.instructions.md`](./instructions/rust.instructions.md)
- UI and extension work: [`./instructions/ui.instructions.md`](./instructions/ui.instructions.md)
- Docs and terminology: [`./instructions/docs.instructions.md`](./instructions/docs.instructions.md)

## Shared Skills

This repository keeps `.codex/skills/` as the single tracked source of shared skills and does not duplicate them into `.github/skills/`.
