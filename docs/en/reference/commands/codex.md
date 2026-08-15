# codex - Codex Runtime and Multi-Account Management

`ccr codex` is the Codex-specific command group. Its main user-facing surfaces are:

- `ccr codex auth ...`: official-auth multi-account management
- `ccr codex profile ...`: runtime/profile routing
- `ccr codex fix`: stale-process cleanup and local profile/runtime consistency diagnosis
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
| `ccr codex profile ...` | apply a CCR profile into the Codex runtime or clear its route and runtime credentials |

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
- `open`

## `fix`

```bash
# Switch the target profile first. The bare command does not write runtime files.
ccr codex profile switch future
ccr codex fix

# Explicitly repair local drift that can be handled safely.
ccr codex fix --repair-runtime

# Preview process cleanup and profile replay without signals or file writes.
ccr codex fix --dry-run --repair-runtime
```

The process stage only matches native Codex or the Node Codex wrapper running `app-server` under
the current owner. It does not match `codex exec`, `codex resume`, `codex login`, or unrelated tools
that merely contain `codex app-server` in their arguments. Cleanup sends TERM first, keeps
rediscovering replacement PIDs during an approximately three-second grace window, and sends KILL
to every matching process still present at the deadline. Owner, PID start time, and argv are checked
again before every signal. If the current owner or command line cannot be read safely, CCR reports
`process_state = unavailable` and sends no further signals instead of claiming `clean`.

The diagnosis separates profile pointers, route, credential consistency, and provider validity. CCR's reconciliation only compares the locally saved secret with the configured credential source; it adds no third-party credential probe and never prints key values, masked fragments, lengths, or fingerprints. The command still runs upstream `codex doctor` as supplemental evidence, whose checks depend on the installed Codex version. `provider_auth_validity = not_checked` therefore means neither success nor failure at the provider.

Process cleanup, CCR runtime inspection/repair, and upstream doctor report independently. A runtime
stage failure reports `runtime_consistency = unavailable`; doctor still runs when it is available.
Raw process argv and sensitive stage-error content are never rendered.

Exit codes:

| Exit code | Meaning |
|---|---|
| `0` | No confirmed local drift; provider validity may still be unchecked |
| `1` | CCR runtime inspection or repair failed |
| `2` | An app-server remains, or process discovery/cleanup could not be completed safely |
| `3` | Local profile/runtime drift remains, or the snapshot changed during doctor |
| `127` | `codex` is not available on `PATH` |

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
