# `ccr budget`

Budget management (web feature). Configure daily/weekly/monthly limits, warning thresholds, enable/disable control, and reset limits. Uses `ccr stats` data.

## Subcommands

- `status` — show budget usage and warnings
- `set` — configure limits and warning threshold; enable/disable control
- `reset` — reset all limits

## Usage

```bash
ccr budget status
ccr budget set [--daily <amount>] [--weekly <amount>] [--monthly <amount>] [--warn-at <0-100>] [--enable|--disable]
ccr budget reset [--force]
```

### set options

- `--daily` / `--weekly` / `--monthly`: set daily/weekly/monthly budget (USD)
- `--warn-at <0-100>`: warning threshold percentage (default 90)
- `--enable` / `--disable`: turn budget control on/off (mutually exclusive)

Examples:
```bash
ccr budget set --daily 10 --weekly 50 --monthly 200 --warn-at 90 --enable
ccr budget set --monthly 150 --disable  # only disable control
```

### reset options

- `--force`: skip confirmation

```bash
ccr budget reset
ccr budget reset --force
```

## Sample output (status)

```
💰 Budget Status
✅ Budget control enabled

Period   Current   Limit    Usage   State
Daily    $2.50     $10.00   25.0%   ✅ OK
Weekly   $15.00    $50.00   30.0%   ✅ OK
Monthly  $62.00    $200.00  31.0%   ✅ OK
```

Warnings show ⚠️/❌ when near or over thresholds.

## Notes

- Relies on `ccr stats` data (web feature); ensure stats are collected.
- Warnings do not block calls; they are informational.

## See also

- [`stats`](./stats.md) — cost & usage statistics
- [`pricing`](./pricing.md) — model pricing management
