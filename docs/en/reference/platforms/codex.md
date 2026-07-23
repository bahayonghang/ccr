# Codex Platform Configuration Guide

Codex now uses a split model: one surface for official-auth accounts and another for runtime/profile routing.

## Main current path

```bash
ccr codex auth current
ccr codex auth list
ccr codex profile list
ccr codex profile switch <name>
ccr codex profile current
ccr codex profile off
ccr codex fix
```

## `auth` vs `profile`

- `ccr codex auth ...`: save / switch / import / export official-auth accounts
- `ccr codex profile ...`: write a CCR profile into `~/.codex/config.toml` and `~/.codex/auth.json`
- `ccr codex profile off`: leave profile mode and restore the official-auth runtime

## Key paths

- Runtime config: `~/.codex/config.toml`
- Runtime auth: `~/.codex/auth.json`
- Profiles: `~/.ccr/platforms/codex/profiles.toml`
- Registry pointer: `[codex].current_profile` in `~/.ccr/config.toml`

## Runtime diagnosis and repair

Switch to the profile you intend to diagnose before running `fix`:

```bash
ccr codex profile switch future
ccr codex fix
```

`ccr codex fix` cleans up stale app-server processes and compares the registry pointer, `profiles.toml`, `config.toml`, `auth.json`, and the current process environment at invocation time. It reports `process_state`, `runtime_consistency`, and `provider_auth_validity` separately.

Process discovery explicitly loads command lines and owners and only handles Codex `app-server`
processes owned by the current user. Cleanup identifies processes by `PID + start_time`, discovers
replacement PIDs throughout the TERM grace window, and revalidates owner and argv before every
signal. Output contains redacted summaries only. If a safe snapshot cannot be established, CCR
reports `process_state = unavailable` instead of treating the unknown state as `clean`.

The bare command is diagnostic only. To replay the saved profile through the existing atomic apply path, opt in explicitly:

```bash
ccr codex fix --repair-runtime
ccr codex fix --dry-run --repair-runtime
```

`--repair-runtime` does not change or rotate the saved secret. Combined with `--dry-run`, it neither terminates processes nor writes `config.toml` or `auth.json`.

Process cleanup, runtime inspection/repair, and doctor are independent stages. When the runtime
stage is unavailable, CCR still runs doctor when possible and exits with code `1`; an app-server
that remains or unavailable process discovery takes precedence with exit code `2`.

CCR's reconciliation adds no third-party credential probe. The command still runs upstream `codex doctor`, whose checks depend on the installed Codex version. Even when `runtime_consistency = match`, `provider_auth_validity` remains `not_checked`. If the provider still returns `INVALID_API_KEY`, verify or update the key saved in that profile instead of repeatedly cleaning app-server processes.

## History sync note

`ccr codex sync-history ...` still repairs history visibility after provider-namespace changes. When moving between official and third-party profiles, prefer:

```bash
ccr codex sync-history --bridge official-custom --dry-run
ccr codex sync-history --bridge official-custom --all-history
```

Bridge mode repairs list visibility only. If a history contains `encrypted_content`, CCR warns that it cannot re-encrypt it, so continue/compact may still be constrained by the original account/provider encryption boundary.
