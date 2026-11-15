# `init` - Deprecated Top-Level Initialization Command

> [!WARNING]
> **Deprecated & Hidden (7.x Compatibility)**: Top-level `ccr init [--force]` is **Deprecated** and **Hidden** from `--help` in CCR `7.x`. Invoking it emits a deprecation warning to `stderr` (never polluting `stdout`), and it is scheduled for removal in `8.0`.

## Canonical Replacements

| Scenario | Canonical replacement |
|---|---|
| Initialize the Claude profile catalog and register the platform | `ccr claude profile init [--json]` |
| Initialize the Codex profile catalog and register the platform | `ccr codex profile init [--json]` |
| Initialize the Grok profile catalog and register the platform | `ccr grok profile init [--json]` |
| Bootstrap Git, workflow engine (`mini-trellis` / `trellis`), and `ref/` in the current repo | `ccr project init [--workflow mini-trellis\|trellis]` |

```bash
ccr claude profile init
ccr codex profile init
ccr grok profile init
ccr project init
```

## Key Behavioral Differences

- **Idempotent per-platform SQLite initialization**: Legacy `ccr init` bootstrapped legacy user-level files (overwriting existing configuration under `--force`); `ccr <claude|codex|grok> profile init` idempotently initializes `~/.ccr/platforms/<platform>/profiles.db` from the embedded inactive template (`current_config = ""`) and registers the platform in `~/.ccr/config.toml`, leaving existing catalogs untouched.
- **No live `profiles.toml` or host runtime side effects**: `profile init` never creates a live `profiles.toml` file and never modifies host runtime settings; `--json` outputs `{ ok, platform, profiles_file, created, registered }`. Repository workflow setup lives separately under [`project init`](./project-init).

## Related Pages

- [`claude`](./claude)
- [`codex`](./codex)
- [`grok`](./grok)
- [`project init`](./project-init)
- [`platform`](./platform)
