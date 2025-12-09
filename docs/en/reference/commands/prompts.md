# prompts - Prompt Presets

Manage prompt presets for different platforms (create, list, apply).

**Version**: v3.5.0+

## Usage

```bash
ccr prompts <ACTION> [OPTIONS]
```

## Subcommands

- `list` — List presets (optionally filter by `--target <platform>`).
- `add <name>` — Create a preset (`--target <platform>` + `--content <file|text>`).
- `remove <name>` — Delete a preset.
- `apply <name>` — Apply preset to current config.

## Examples

```bash
ccr prompts list
ccr prompts add pair-program --target claude --content @prompt.md
ccr prompts apply pair-program
ccr prompts remove pair-program
```
