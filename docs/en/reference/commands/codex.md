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
ccr codex auth off
ccr codex profile list
ccr codex profile switch <name>
ccr codex profile current
ccr codex profile off
```

## `auth` vs `profile`

| Command family | Purpose |
|---|---|
| `ccr codex auth ...` | save, switch, export, and import official auth accounts, or log out the current official runtime |
| `ccr codex profile ...` | apply a CCR profile into the Codex runtime or clear its route and runtime credentials |

`ccr codex auth off` logs out the current official runtime. The command is independent from `profile off`. The command does not change the profile pointer or the `config.toml` route. In profile mode, CCR still clears runtime `auth.json` for a file store, or CCR runs `codex logout` for keyring and auto. `--json` may report a remaining `profile_pointer`. That pointer is a warning, not a failure. Run `profile switch` again to write the key back.

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
# Switch the target profile first. The bare command does not write runtime files
# and does not run upstream doctor.
ccr codex profile switch future
ccr codex fix

# Explicitly repair local drift that can be handled safely.
ccr codex fix --repair-runtime

# Preview process cleanup and profile replay without signals or file writes.
ccr codex fix --dry-run --repair-runtime

# Run upstream doctor only when you need that extra evidence.
ccr codex fix --doctor
```

The process stage only matches native Codex or the Node Codex wrapper running `app-server` under
the current owner. It does not match `codex exec`, `codex resume`, `codex login`, or unrelated tools
that merely contain `codex app-server` in their arguments. Cleanup sends TERM first, then rediscovers
matching targets every 300 ms for up to about three seconds. The grace loop stops as soon as the
target set is empty. Every matching process still present at the deadline receives KILL. After any
signal in this run, CCR still waits about one second and takes a final snapshot. Identities that
appear only in that settle snapshot go into `respawned`; they do not receive a deadline KILL, and
the command exits with code 2. Owner, PID start time, and argv are checked again before every
signal. If the current owner or command line cannot be read safely, CCR reports
`process_state = unavailable` and sends no further signals instead of claiming `clean`.

The diagnosis separates profile pointers, route, credential consistency, and provider validity. CCR's reconciliation only compares the locally saved secret with the configured credential source; it adds no third-party credential probe and never prints key values, masked fragments, lengths, or fingerprints. The default path does not run upstream `codex doctor`. Pass `--doctor` when you need that extra evidence. `provider_auth_validity = not_checked` therefore means neither success nor failure at the provider.

Process cleanup, CCR runtime inspection/repair, and optional upstream doctor report independently. A runtime
stage failure reports `runtime_consistency = unavailable`; later independent stages still run.
Raw process argv and sensitive stage-error content are never rendered.
`--repair-runtime` does not imply `--doctor`.

Exit codes:

| Exit code | Meaning |
|---|---|
| `0` | No confirmed local drift; provider validity may still be unchecked |
| `1` | CCR runtime inspection or repair failed |
| `2` | An app-server remains, or process discovery/cleanup could not be completed safely |
| `3` | Local profile/runtime drift remains, or the snapshot changed during doctor |
| `127` | `--doctor` was passed and `codex` is not available on `PATH` |

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
