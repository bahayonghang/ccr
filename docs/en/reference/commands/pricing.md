# `ccr pricing`

Model pricing management (web feature). Configure per-model pricing used by `ccr stats` cost calculation and budget evaluation.

## Subcommands

- `list` — list all pricing; `--verbose` shows cache pricing
- `set` — set pricing for a model
- `remove` — remove pricing for a model
- `reset` — reset to default pricing

## Usage

```bash
ccr pricing list [--verbose]
ccr pricing set <model> --input <price> --output <price> [--cache-read <price>] [--cache-write <price>]
ccr pricing remove <model> [--force]
ccr pricing reset [--force]
```

### set options

- `<model>`: model name
- `--input` / `--output`: input/output price (USD per million tokens)
- `--cache-read` / `--cache-write`: cache read/write price (optional)

Examples:
```bash
ccr pricing set claude-3-5-sonnet-20241022 --input 3.0 --output 15.0 --cache-read 0.3 --cache-write 3.75
ccr pricing set my-model --input 2.0 --output 10.0
```

### list options

- `--verbose`: include cache read/write prices

```bash
ccr pricing list
ccr pricing list --verbose
```

### remove/reset options

- `--force`: skip confirmation

```bash
ccr pricing remove my-model
ccr pricing reset --force
```

## Sample output (`list --verbose`)

```
💰 Pricing

Model                         Input     Output    Cache Read  Cache Write
claude-3-5-sonnet-20241022    $3.00/M   $15.00/M  $0.30/M     $3.75/M
my-model                      $2.00/M   $10.00/M  -           -

🔧 Default pricing (fallback)
  Input: $3.00/M
  Output: $15.00/M
```

## Notes

- Prices must be non-negative; removed models fall back to default pricing.
- Default pricing can be restored via `reset`; configs are stored locally.
- Stats and budget rely on pricing for cost computation.

## See also

- [`stats`](./stats.md) — cost & usage statistics
- [`budget`](./budget.md) — budgeting
