# prompts - Prompt Template Presets

Manage reusable prompt presets and apply them to platform instruction files (`CLAUDE.md`, `AGENTS.md`, `GEMINI.md`) with automatic backups.

**Version**: v3.5.0+

## Usage

```bash
ccr prompts <ACTION> [OPTIONS]
```

## Subcommands

### `list` — List Presets

List prompt presets stored for the selected platform:

```bash
ccr prompts list
ccr prompts list --platform codex
```

### `add` — Create a Preset

Create a new prompt preset from inline text or `@<file>`:

```bash
ccr prompts add <NAME> \
  --target <claude|agents|gemini> \
  --content <TEXT_OR_@FILE> \
  [--description <DESC>] \
  [--platform <PLATFORM>]
```

Examples:

```bash
# Inline prompt text
ccr prompts add code-review \
  --target claude \
  --content "Review this change for security, correctness, and idiomatic style." \
  --description "Security-focused review prompt"

# Load content from a file using @ prefix
ccr prompts add repo-rules \
  --target agents \
  --platform codex \
  --content @~/prompts/repo-rules.md \
  --description "Shared agent instructions"
```

### `apply` — Apply a Preset

Write a saved preset into its target file (automatically creating a `.backup` copy first):

```bash
ccr prompts apply code-review
ccr prompts apply repo-rules --platform codex
```

### `show` — Inspect Preset Content

Display a preset's metadata and full text:

```bash
ccr prompts show code-review
```

### `remove` — Delete a Preset

Delete a saved preset:

```bash
ccr prompts remove old-prompt
```

### `current` — Show Current Target File Content

Inspect the live contents of a target instruction file:

```bash
ccr prompts current claude
ccr prompts current agents --platform codex
ccr prompts current gemini --platform gemini
```

## Options and Targets

- `--platform <PLATFORM>`: platform namespace (`claude`, `codex`, `gemini`, `qwen`; default: `claude`).
- `-t, --target <TARGET>`:
  - `claude`: writes `CLAUDE.md` under the selected platform root
  - `agents`: writes `AGENTS.md` under the selected platform root
  - `gemini`: writes `GEMINI.md` under the selected platform root

## Storage Locations

Presets are stored in `~/.ccr/platforms/<platform>/prompts.toml`. When applied, CCR resolves the destination root by platform:

- Claude root: `~/.claude/`
- Codex root: `~/.codex/`
- Gemini root: `~/.gemini/`
- Other platforms: `~/.ccr/platforms/<platform>/`

For example, `ccr prompts apply repo-rules --platform codex` with `target = "agents"` writes `~/.codex/AGENTS.md`.

## Notes

- Applying a preset overwrites the destination file after writing a `.backup` snapshot.
- Preset names must be unique within each platform's `prompts.toml`.
- Ensure `@file` inputs are UTF-8 encoded and readable.

## Related Commands

- [`skills`](./skills) - manage installed skills
- [`version`](./version) - inspect CCR version details
- [Command Reference](./index) - browse all commands
