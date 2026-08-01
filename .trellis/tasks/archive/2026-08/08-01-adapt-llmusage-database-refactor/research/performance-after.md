# llmusage Database Refactor Performance After Implementation

## Environment And Method

- Date: 2026-08-01
- Build: `ccr-usage` release profile
- Installed CLI: `llmusage 1.1.1`
- Database: `C:/Users/lyh/.llmusage/llmusage.db`
- Database size: 1,160,073,216 bytes
- Database schema: 18
- Access mode: SQLite read-only plus `PRAGMA query_only = ON`; no database writes
- Filter: local dates `2026-07-01` through `2026-07-31`, inclusive
- Asia/Shanghai UTC bounds: `[2026-06-30T16:00:00Z, 2026-07-31T16:00:00Z)`
- Sampling: 5 warm-up iterations, then 21 timed iterations per process; three independent
  process runs; the table reports the median of the three process medians
- Logs shape: first cursor page, 100 rows, `include_total = false`,
  `include_raw_json = false`

The temporary benchmark executable opened a single `Dashboard` and invoked the public
projection methods. It was deleted after sampling and is not part of the delivered code.

The principal-section comparison uses the same five production projections on both sides:
daily trends, model, provider, project and source. The old side uses the exact legacy selected
columns, grouping and ordering, restores the legacy `date(hour_start, 'localtime')` predicates,
and reproduces the five per-section read-only capability connections/column probes plus the
obsolete bucket `project_path` probe. The new side uses the public methods on one Dashboard
connection and its immutable capability snapshot. Dashboard initial-open time is excluded from
both sides.

## Results

| Probe | Before median | After median | Reduction | R7 target | Result |
| --- | ---: | ---: | ---: | ---: | --- |
| Logs date range | 11.240 ms | 0.239 ms | 97.9% | 80% | PASS |
| Overview | 7.888 ms / 7 bucket queries | 1.352 ms / 1 bucket query | 82.9% | 50% | PASS |
| Home overview | 3.123 ms / 2 bucket queries | 1.559 ms / 1 bucket query | 50.1% | 30% | PASS |
| Principal bucket sections | 14.346 ms | 8.096 ms | 43.6% | 20% | PASS |

The pre-implementation research also recorded `6.144 ms -> 4.246 ms` for a simplified
five-query SQL probe. That probe selected fewer columns and did not include the repeated
capability connections, so it is retained as planning evidence but is not mixed with the
production-equivalent comparison above.

## Individual After Medians

| Public projection | Median |
| --- | ---: |
| Daily trends | 1.649 ms |
| Model breakdown | 2.024 ms |
| Provider breakdown | 1.333 ms |
| Project breakdown | 1.038 ms |
| Source breakdown | 1.533 ms |
| Heatmap (30 days) | 1.188 ms |
| All six sections together | 8.684 ms |

The IANA local-date scalar function keeps a bounded 4,096-entry `(timezone, timestamp)` cache
on the short-lived Dashboard connection. Repeated sections therefore reuse DST-correct local
date conversions without an unbounded process cache or cross-timezone reuse.

## Deterministic Gates

- Bucket range predicates are `hour_start >= ? AND hour_start < ?` and the fixture
  `EXPLAIN QUERY PLAN` test requires `idx_usage_bucket_30m_hour_start`.
- Event range predicates are `event_at >= ? AND event_at < ?` and the fixture
  `EXPLAIN QUERY PLAN` test requires `idx_usage_event_event_at`.
- Overview performs one bucket conditional aggregate and one run-log conditional aggregate.
- Home overview performs one date/source bucket aggregate.
- Dashboard section gates use the capability snapshot built from the already-open connection;
  the connection-open instrumentation test requires zero capability reopens.
- CCR adds no index and does not migrate or write the llmusage database.

## Conclusion

All four R7 latency targets pass under the recorded representative-data method. Query-plan,
query-count and connection-reuse assertions remain the non-flaky CI gates.
