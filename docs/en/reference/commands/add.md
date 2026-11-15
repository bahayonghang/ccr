# `add` - Deprecated Top-Level Interactive Add Command

> [!WARNING]
> **Deprecated & Hidden (7.x Compatibility)**: Top-level `ccr add` is **Deprecated** and **Hidden** from `--help` in CCR `7.x`. Invoking it emits a deprecation warning to `stderr` (never polluting `stdout`), and it is scheduled for removal in `8.0`.

## Canonical Replacements

| Scenario | Canonical replacement |
|---|---|
| Create a Claude profile non-interactively | `printf '%s' "$ANTHROPIC_AUTH_TOKEN" \| ccr claude profile create <name> --base-url <url> --model <model> --auth-mode api_key --auth-token-stdin` |
| Create a Codex profile non-interactively | `printf '%s' "$OPENAI_API_KEY" \| ccr codex profile create <name> --base-url <url> --model <model> --auth-mode openai_api_key --auth-token-stdin` |
| Create a Grok profile non-interactively | `printf '%s' "$GROK_API_KEY" \| ccr grok profile create <name> --base-url <url> --model <model> --api-key-stdin` (or `--env-key <ENV_VAR>`) |
| Interactive TUI profile creation | `ccr` (see [`tui`](./tui)) |
| Edit catalog projection in `$VISUAL` / `$EDITOR` | `ccr <claude\|codex\|grok> profile open` |

## Key Behavioral Differences

- **Multi-platform scope & scriptability**: Legacy `ccr add` is an interactive-only wizard hardcoded to the legacy Claude catalog; `ccr <claude|codex|grok> profile create <name>` supports all three profile platforms with `--expected-revision` and `--json` receipts.
- **Stdin-only secret entry & inactive creation**: Canonical `profile create` reads secrets safely via `--auth-token-stdin` / `--api-key-stdin` (or Grok `--env-key`) instead of exposing secrets in shell history or process arguments, and leaves `current_config` unchanged until `profile switch`.

## Related Pages

- [`claude`](./claude)
- [`codex`](./codex)
- [`grok`](./grok)
- [`tui`](./tui)
