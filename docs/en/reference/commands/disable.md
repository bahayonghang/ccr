# `disable` - Deprecated Top-Level Disable Command

> [!WARNING]
> **Deprecated & Hidden (7.x Compatibility)**: Top-level `ccr disable <config_name> [--force]` is **Deprecated** and **Hidden** from `--help` in CCR `7.x`. Invoking it emits a deprecation warning to `stderr` (never polluting `stdout`), and it is scheduled for removal in `8.0`.

## Canonical Replacements

| Scenario | Canonical replacement |
|---|---|
| Disable a Claude profile | `ccr claude profile disable <name> [--force] [--json]` |
| Disable a Codex profile | `ccr codex profile disable <name> [--force] [--json]` |
| Disable a Grok profile | `ccr grok profile disable <name> [--force] [--json]` |
| Exit active runtime cleanly before disabling | `ccr <claude\|codex\|grok> profile off` followed by `ccr <claude\|codex\|grok> profile disable <name>` |

## Key Behavioral Differences

- **Explicit platform scope & JSON output**: Legacy `ccr disable` only targets the legacy Claude profile scope; canonical commands explicitly target `claude`, `codex`, or `grok` and support `--json` (`{ ok, platform, name, message, enabled, current_profile }`).
- **Active-profile protection**: `profile disable` rejects disabling the currently active profile unless `--force` is passed (and `profile edit <name> --disable` always rejects disabling the active profile). Switch to another profile or run `profile off` before disabling the active entry.

## Related Pages

- [`claude`](./claude)
- [`codex`](./codex)
- [`grok`](./grok)
