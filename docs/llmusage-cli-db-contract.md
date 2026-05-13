# llmusage CLI + Read-only DB Contract for ccr-ui

ccr-ui does not link the upstream `llmusage` Rust crate. The desktop app treats
`llmusage` as an installed runtime:

- sync/import is delegated to `llmusage sync --json-events`;
- rendering reads the active SQLite DB at `LLMUSAGE_HOME/llmusage.db` or
  `~/.llmusage/llmusage.db`;
- ccr-ui opens that DB read-only and never bootstraps, migrates, repairs, or
  writes it;
- ccr-ui never parses raw Claude/Codex/Gemini/OpenCode logs.

## Runtime path resolution

Current resolution order:

1. `LLMUSAGE_HOME` when set;
2. platform home directory + `.llmusage`.

The DB path is always `<root>/llmusage.db`. A future llmusage metadata command
should replace this inference when it becomes stable.

## Schema gate

ccr-ui currently requires `meta.schema_version >= 10` for full Usage rendering.
Each feature also checks required tables/columns before querying. Missing
schema/capability must surface as an unsupported/waiting state; ccr-ui must not
silently return fabricated zeroes for unsupported features.

## Query surfaces

The current projection uses:

- `usage_bucket_30m` for overview, daily trends, model/project breakdowns,
  heatmap, and home usage totals;
- `usage_event` plus optional `usage_event_raw` for logs;
- `source_file` and `source_sync_status` for archive/source diagnostics;
- `run_log` for last sync/export timestamps when present.

## Sync/import bridge

ccr-ui invokes:

```text
llmusage --home <root> sync --json-events [--source <id>] [--recent-days <n>] [--rebuild]
```

The NDJSON stream is parsed as typed lifecycle events and mapped to the existing
Tauri event channels:

- `usage:job-progress`
- `usage:job-recent-ready`
- `usage:job-finished`
- `usage:job-failed`

Cancellation is currently a local UI/job state only because llmusage does not
expose a documented external graceful cancel contract.

## Upstream requirements backlog

P0:

1. Stable machine-readable metadata command exposing version, root, DB path,
   schema version, and capability flags.
2. Stable SQLite read contract or views for overview, daily trends,
   model/project breakdowns, heatmap, cursor logs, and diagnostics.
3. Stable `sync --json-events` NDJSON schema with typed source id,
   optional-source-absent, terminal events, and failure details.
4. Documented CLI flags equivalent to ccr-ui import options: source selection,
   recent-days, rebuild/reset, and home/root.
5. Documented graceful cancellation command/API, including Windows process-tree
   and lock cleanup behavior.
6. Schema compatibility/deprecation policy for consumers targeting schema v10+.

P1:

7. JSON diagnostics command for source status and file state counts.
8. Stable project display fields and precedence.
9. Stable pricing/cost fields and meanings.
10. Stable token fields, including cache creation/read and reasoning output.
11. Raw event JSON availability or a documented replacement for logs.

P2:

12. Local-date bucket/view to remove timezone ambiguity.
13. Report JSON commands matching dashboard DTOs for cross-checking SQL readers.
14. Published fixture DB and NDJSON event samples for downstream tests.