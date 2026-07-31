# Platform Migration Guide

This page describes the migration patterns still supported by the current CCR command surface.

## Current Rules

- there is no documented `ccr migrate`
- there is no `ccr platform migrate`
- the retired `ccr platform init` command is replaced by each platform's `profile init`; see the [platform migration map](../commands/platform)
- Gemini and Droid templates are copied manually until those platforms expose their own init command

## Pattern 1: Keep platforms side by side

This is the safest path for most users.

```bash
ccr claude profile init
ccr codex profile init
ccr grok profile init

mkdir -p ~/.ccr/platforms/gemini
cp examples/gemini/profiles.toml ~/.ccr/platforms/gemini/profiles.toml

ccr platform switch claude
ccr platform switch codex
ccr platform switch gemini
```

Use this when you want separate profile sets per platform with fast switching and easy rollback.

## Pattern 2: Move profiles between platforms

Use export/import for structure, then update platform-specific credentials manually.

```bash
# Export from the source platform
ccr platform switch claude
ccr export -o claude-profiles.toml --no-secrets

# Prepare the Gemini target manually
mkdir -p ~/.ccr/platforms/gemini
cp examples/gemini/profiles.toml ~/.ccr/platforms/gemini/profiles.toml
ccr platform switch gemini

# Import the structure
ccr import claude-profiles.toml --merge --backup

# Then replace model names and API keys manually
ccr validate
```

Notes:

- profile names and descriptions can move across platforms
- tokens, model names, and platform-specific fields must be reviewed manually
- use `--no-secrets` for transfer files you want to inspect or share

## Pattern 3: Legacy single-file config to the current layout

If you still keep `~/.ccs_config.toml`, use it as reference input instead of expecting a dedicated migration command.

Recommended order:

```bash
ccr init
ccr claude profile init
ccr platform switch claude
ccr add
ccr list
```

If you already have an exportable bundle, prefer `ccr import <file> --merge --backup`.

## Verification Checklist

After any migration step:

```bash
ccr platform current
ccr list
ccr current
ccr validate
```

Check these points:

- the target platform is initialized
- imported or recreated profiles show up in `ccr list`
- settings write to the expected target file for that platform
- credentials and model names were updated to platform-native values

## Rollback

The current rollback primitives are:

- switch back to the original platform with `ccr platform switch`
- re-import a known-good export file with `ccr import`
- rely on the automatically created backup from `ccr import --backup` or destructive settings writes

## See Also

- [Platform Support](./index)
- [Claude Platform](./claude.md)
- [Gemini Platform](./gemini.md)
- [Architecture](/en/reference/architecture)
