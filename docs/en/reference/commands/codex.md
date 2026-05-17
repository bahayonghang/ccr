# codex - Codex Runtime and Multi-Account Management

`ccr codex` is the Codex-specific command group. Its main user-facing surfaces are:

- `ccr codex auth ...`: official-auth multi-account management
- `ccr codex profile ...`: runtime/profile routing
- `ccr codex sync-history ...`: history visibility repair after provider-namespace changes

## Common commands

```bash
ccr codex auth current
ccr codex auth list
ccr codex profile list
ccr codex profile switch <name>
ccr codex profile current
ccr codex profile off
```

## `auth` vs `profile`

| Command family | Purpose |
|---|---|
| `ccr codex auth ...` | save, switch, export, and import official auth accounts |
| `ccr codex profile ...` | apply a CCR profile into the Codex runtime or exit back to official-auth runtime |

## Current `profile` surface

- `list`
- `current`
- `switch <name>`
- `off`
- `create`
- `set-field`
- `enable`
- `disable`
- `delete`

## `sync-history`

Keeps its existing role: repairing old-history visibility after `openai` / `custom` namespace changes.

Common modes:

```bash
# Existing behavior: explicitly write one provider, defaulting to the last 7 days
ccr codex sync-history --provider custom --dry-run
ccr codex sync-history --provider openai

# New bridge mode: bridge openai/custom/missing-provider history into the current runtime provider
ccr codex sync-history --bridge official-custom --dry-run
ccr codex sync-history --bridge official-custom --all-history

# Diagnose provider, SQLite, preview, cwd, Desktop first-page limits, and encrypted_content
ccr codex sync-history status
```

Additional constraints:

- `--provider` keeps the compatible behavior; when omitted, CCR still reads the current `~/.codex/config.toml`.
- `--bridge official-custom` resolves the target from the current runtime: official/implicit OpenAI targets `openai`, while third-party profiles target `custom`.
- `--all-history` disables the 7-day filter; ordinary mode still defaults to the last 7 days.
- Bridge / all-history SQLite repair only touches `openai`, `custom`, and missing-provider rows by default; pass repeatable `--include-provider <name>` for additional providers.
- Write mode backs up rollout first lines, `state_5.sqlite`, and `.codex-global-state.json`; `--dry-run` prints the plan without writing files.
- `encrypted_content` is only counted and warned about. CCR does not decrypt, re-encrypt, edit message bodies, or change file mtimes.
