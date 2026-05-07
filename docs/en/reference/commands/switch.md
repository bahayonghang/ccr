# switch - Retired Legacy Profile Switch Entrypoint

`ccr switch <config_name>` used to infer a target platform from global `current_platform` state. That entrypoint is now retired.

## Current behavior

```bash
ccr switch <config_name>
```

now returns a migration error that points to explicit platform-scoped commands.

## Use these instead

```bash
ccr claude profile switch <config_name>
ccr codex profile switch <config_name>
```

## Why it was retired

- the global `current_platform` / `default_platform` model was misleading
- Claude and Codex now need explicit runtime state
- VS Code, doctor, validate, and `ccr current` already moved to per-platform state
