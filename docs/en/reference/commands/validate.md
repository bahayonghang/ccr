# `validate` - Cross-Platform Read-Only Configuration Validation

`ccr validate` performs a strictly read-only inspection of the global registry and the SQLite profile catalogs (`profiles.db`), authentication modes, and existing host runtime files for Claude, Codex, and Grok. The command never initializes files, repairs `current` markers, creates backups, or creates log directories.

## Usage

```bash
ccr validate
ccr validate --json
```

## Scope and Deterministic Order

Diagnostics always evaluate targets in **`registry -> Claude -> Codex -> Grok`** order:

- **Global registry (`registry`)**: `<CCR_ROOT>/config.toml` (default `~/.ccr/config.toml`).
- **Platform profile catalogs**: `<CCR_ROOT>/platforms/<claude|codex|grok>/profiles.db` (SQLite-backed; unmigrated or malformed legacy files are also classified read-only without mutating disk state).
- **Claude runtime**: `settings.json` resolved via `ClaudeRuntimePaths` environment precedence.
- **Codex runtime**: `config.toml` and credential state resolved under the Codex runtime directory.
- **Grok runtime**: `$GROK_HOME/config.toml` (default `~/.grok/config.toml`).

Each platform invokes the same domain validator used during `profile switch`:
- Claude `subscription` profiles do not require API-key fields; `api_key` profiles enforce Claude endpoint and credential rules.
- Codex validates the configured `openai_chatgpt`, `openai_api_key`, `provider_env_key`, `provider_bearer`, or `no_auth` mode.
- Grok validates official model-selector profiles separately from third-party `api_key` / `env_key` profiles.

Inactive disabled profiles are allowed to remain in the catalog and emit a `disabled` warning; a disabled profile marked as `current` emits a `disabled` error (exit code `90`). Claude resolves an existing catalog marker before an existing registry marker, reporting `warning/invalid` without repairing either marker when one is stale or conflicting; explicit markers with no valid fallback return `62`. Runtime file checks run even when no profiles exist, reporting corrupt or unreadable runtime files immediately.

## `--json` Output Contract (`DiagnosticReport`)

Passing `--json` prints the structured `DiagnosticReport` JSON to `stdout` while preserving identical process exit codes:

```json
{
  "checks": [
    {
      "platform": null,
      "target": "registry",
      "severity": "info",
      "category": "valid",
      "message": "registry: valid.",
      "exit_code": 0
    },
    {
      "platform": "claude",
      "target": "profiles",
      "severity": "info",
      "category": "valid",
      "message": "profiles: valid.",
      "exit_code": 0
    }
  ]
}
```

`DiagnosticCheck` fields:
- `platform`: `"claude" | "codex" | "grok" | null` (`null` for global `registry` checks).
- `target`: inspected target identifier (such as `"registry"`, `"profiles"`, `"runtime"`, or a profile name).
- `severity`: `"info" | "warning" | "error"`.
- `category`: `"valid" | "missing" | "corrupt" | "unreadable" | "invalid" | "disabled" | "inactive" | "runtime"`.
- `message`: sanitized diagnostic summary (never includes tokens, raw source documents, or rejected secret values).
- `exit_code`: numeric `CcrError` exit code (`0` for `info` and `warning`).

## Categories and Exit Codes

| Condition | Category (`category`) | Exit code |
|---|---|---|
| Valid configuration, or warnings only | `valid`, `missing`, `inactive`, `disabled` | `0` |
| Profile domain validation failed or current profile is disabled | `invalid`, `disabled` | `90` |
| Malformed profile catalog syntax or field type | `corrupt` | `14` |
| Profile or runtime read failure (permission denied, sharing violation, directory at file path) | `unreadable` | `51` |
| Malformed existing runtime JSON or TOML | `corrupt` | `40` or `41` |
| Explicit `current` target is missing with no permitted fallback | `invalid` | `62` |

When multiple errors occur, the first `error` in `registry -> Claude -> Codex -> Grok` order determines the process exit code; reports containing only `info` or `warning` checks exit with `0`. `missing` strictly indicates a nonexistent path, whereas access or I/O failures are classified as `unreadable` (`51`).

## Related Pages

- [`doctor`](./doctor) — environment checks, runtime auth status, conflict detection, and optional `--online` probes
- [`current`](./current) — side-by-side Claude and Codex runtime overview
- [`claude`](./claude) / [`codex`](./codex) / [`grok`](./grok) — platform profile repair and switching commands (Gemini and Droid remain compatibility adapters without a profile CLI surface)
