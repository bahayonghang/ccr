# Coverage threshold note for `08-22-arch-quality-perf` (batch 7 / its R5)

`ccr-ui/vitest.smoke.config.ts` keeps `coverage.thresholds.lines = 70`.
Comment in that file (2026-08-23): React foundation measured 72.86% lines;
pre-migration baseline 75.4%; 70% retained.

This task rewrote remaining Vue-path smokes and restored the Grok Local-only
dashboard cases. It did not add coverage dimensions (functions / branches /
statements), per `08-22-arch-quality-perf` design §4.

**Conclusion for arch-quality-perf R5:** keep `lines: 70`. Recheck with
`just frontend-coverage` after this task's smoke rewrite. If measured lines
stay ≥70, do not raise the threshold in the same change as the rewrite.
