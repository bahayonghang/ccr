# Claude Platform Guide

Claude is a first-class platform in the current explicit runtime/profile model.

## Main current path

```bash
ccr claude auth current
ccr claude profile list
ccr claude profile switch <name>
ccr claude profile current
ccr claude profile off
```

## Model

- `ccr claude auth ...`: official-auth account management
- `ccr claude profile ...`: apply a saved profile into `~/.claude/settings.json`
- `ccr claude profile off`: leave profile mode and return to the official-auth runtime

## Key paths

- Runtime settings: `~/.claude/settings.json`
- Profiles: `~/.ccr/platforms/claude/profiles.toml`
- Registry pointer: `[claude].current_profile` in `~/.ccr/config.toml`
