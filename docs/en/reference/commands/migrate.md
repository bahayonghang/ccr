# migrate - Legacy → Unified Migration

Move from the legacy single-file mode (`~/.ccs_config.toml`) to the default Unified mode (`~/.ccr/` registry + platform directories).

**Version**: v3.6.0+

## Usage

```bash
ccr migrate [--check] [--platform <name>]
```

- `--check`: Dry-run, only report what would be migrated.
- `--platform <name>`: Migrate a specific platform only (default: all implemented platforms).

## Flow

1) Read `~/.ccs_config.toml`  
2) Generate `~/.ccr/config.toml` (with `current_platform`)  
3) Create `platforms/<name>/profiles.toml` per platform  
4) Keep the original file with backup and history entry  

## Examples

```bash
ccr migrate --check           # inspect only
ccr migrate --platform claude # migrate Claude only
ccr migrate                   # full migration
```

> Legacy mode remains available after migration, but Unified workflow (platform + profiles) is recommended.
