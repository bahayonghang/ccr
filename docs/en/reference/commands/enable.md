# `enable` - Deprecated Top-Level Enable Command

> [!WARNING]
> **Deprecated & Hidden (7.x Compatibility)**: Top-level `ccr enable <config_name>` is **Deprecated** and **Hidden** from `--help` in CCR `7.x`. Invoking it emits a deprecation warning to `stderr` (never polluting `stdout`), and it is scheduled for removal in `8.0`.

## Canonical Replacements

| Scenario | Canonical replacement |
|---|---|
| Enable a Claude profile | `ccr claude profile enable <name> [--json]` |
| Enable a Codex profile | `ccr codex profile enable <name> [--json]` |
| Enable a Grok profile | `ccr grok profile enable <name> [--json]` |
| Populate a missing secret and enable in one step | `printf '%s' "$TOKEN" \| ccr <claude\|codex> profile edit <name> --auth-token-stdin --enable` (use `--api-key-stdin` on Grok) |

## Key Behavioral Differences

- **Explicit platform scope**: Legacy `ccr enable` only targets the legacy Claude profile scope; canonical commands explicitly target `claude`, `codex`, or `grok` and support `--json`.
- **Pre-enable domain validation**: `ccr <platform> profile enable <name>` validates profile completeness under the platform operation lock before setting `enabled = true`, rejecting entries that still lack required secrets (such as `--no-secrets` imports) with guidance to run `profile edit` first. Run `ccr <platform> profile switch <name>` afterward to activate the enabled profile.

## Related Pages

- [`claude`](./claude)
- [`codex`](./codex)
- [`grok`](./grok)
