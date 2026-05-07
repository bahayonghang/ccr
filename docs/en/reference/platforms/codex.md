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

## History sync note

`ccr codex sync-history ...` still repairs history visibility after provider-namespace changes.
