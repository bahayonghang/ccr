# `ccr stats`

Cost & usage statistics (web feature). Supports summary, import, export, and clear.

## Subcommands

- `summary` — stats summary (grouping/top/trend; preferred)
- `import` — import cost CSV
- `export` — export stats (JSON/CSV)
- `clear` — delete historical data
- `cost` — deprecated alias of `summary`

## Usage

```bash
ccr stats summary [OPTIONS]
ccr stats import <csv_file> [--format auto|claude-hub|custom] [--skip-validation]
ccr stats export [--format json|csv] [--output <path>] [--range today|week|month|custom] [--start YYYY-MM-DD] [--end YYYY-MM-DD]
ccr stats clear [--before YYYY-MM-DD] [--force] [--dry-run]
```

### summary options

- `--range`: `today` (default) | `week` | `month` | `custom`
- `--start` / `--end`: custom range (with `custom`)
- `--by-model` / `--by-project` / `--by-platform`: grouping
- `--top <N>`: top-N costly sessions
- `--details`: include trend & extra groups

Examples:
```bash
ccr stats summary --range week --by-model --details
ccr stats summary --range custom --start 2025-01-01 --end 2025-01-31 --top 10
```

### import options

- `<csv_file>`: CSV path
- `--format`: `auto` (default) | `claude-hub` | `custom`
- `--skip-validation`: skip format checks

```bash
ccr stats import costs.csv --format auto
ccr stats import claude_hub.csv --format claude-hub --skip-validation
```

### export options

- `--format`: `json` (default) | `csv`
- `--output <path>`: file path; omit to print
- `--range` / `--start` / `--end`: same as `summary`

```bash
ccr stats export --format json --output report.json --range month
ccr stats export --format csv --range custom --start 2025-01-01 --end 2025-01-15
```

### clear options

- `--before YYYY-MM-DD`: delete data before date (default: 30 days ago)
- `--force`: skip confirmation
- `--dry-run`: preview only

```bash
ccr stats clear --before 2025-01-01
ccr stats clear --before 2025-01-01 --force
```

## Sample output (summary)

```
📊 Cost Stats - week

ℹ 💰 Total Cost: $12.3456
ℹ 📊 Records: 156

✓ 🎫 Tokens:
  📥 Input: 1.5M
  📤 Output: 800K
  💾 Cache: 300K
  📊 Cache Efficiency: 65.23%

🤖 By Model:
  • 3-5-sonnet-20241022: $85.2000
  • 3-5-haiku-20241022: $32.1000

📁 By Project:
  • project-a: $45.0000
  • project-b: $35.2000

📈 Daily Trend:
  2025-10-27 - $12.3456 (156 calls)
  2025-10-26 - $8.9012 (123 calls)
```

## Storage

- Dir: `~/.claude/stats/`
- Files: monthly `costs_YYYYMM.csv`
- Columns: `timestamp,id,session_id,project,platform,model,input_tokens,output_tokens,cache_read_tokens,cache_write_tokens,input_cost,output_cost,cache_cost,total_cost,duration_ms,description`

## Works with budget & pricing

- `ccr budget status|set|reset`: shows usage based on stats
- `ccr pricing list|set|remove|reset`: pricing feeds cost calculation

## See also

- [`budget`](./budget.md) — budgeting
- [`pricing`](./pricing.md) — model pricing
- [`history`](./history.md) — audit log
