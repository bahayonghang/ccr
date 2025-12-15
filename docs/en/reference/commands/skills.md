# skills - Skills Management

Manage CCR skills: install/uninstall, scan repositories, and manage repos.

**Version**: v3.5.0+

## Usage

```bash
ccr skills <ACTION> [OPTIONS]
```

## Subcommands

- `list` — List installed skills (supports `--platform <claude|codex|gemini|qwen>`).
- `install <name>` — Install a skill (optionally `--platform`).
- `uninstall <name>` — Remove a skill (optionally `--platform`).
- `repo add <name> <url>` — Add a skills repository.
- `repo list` / `repo remove <name>` — Manage repositories.
- `scan <repo>` — Discover available skills in a repository.

## Examples

```bash
ccr skills repo add official https://github.com/ccr-skills/official
ccr skills scan official
ccr skills install code-review --platform claude
ccr skills list --platform codex
ccr skills uninstall code-review
```
