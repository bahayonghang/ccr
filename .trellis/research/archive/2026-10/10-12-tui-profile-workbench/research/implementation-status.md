# Implementation status

The user explicitly approved revision-2 candidate A for all three platforms and requested implementation. Shared code and all three platform presentations are implemented and independently reviewed. All 16 CI recipes have passed across the aggregate run and the narrowly resumed frontend suffix; the user has now authorized committing and archiving the parent and children. Earlier planning validation records describe the pre-approval state.

## Baseline

- Git base: `8115cd116632ae2d897401b1de4e8385d9b19d6b`, branch `dev`.
- Toolchain: rustc 1.98.0 (88d9e12ae 2026-08-18), cargo 1.98.0.
- Pre-change `just version-check` and `just fmt-check`: passed.
- Preserve unrelated build-cache spec and journal changes; do not commit automatically.
- All mutation/history/performance fixtures must own their temporary files and avoid the user's real profiles, credentials and databases.

## Ownership and integration

- `profile_history_impl`: ccr-store narrow read-only activity projection, ccr-cli facade, necessary ccr-types boundary contracts and tests. Supplies types to the async adapter.
- `profile_async_impl`: ccr-tui App/snapshot/mutation runtime, event pumping, safe exit, initialized Auth runtime invalidation, existing UsageApp Grok wiring. Owns app.rs/runtime.rs/event.rs/mod.rs.
- `profile_visual_impl`: ui.rs/theme.rs, shared table geometry, runtime strip, candidate-A details, sequential platform presentation and visual evidence. Coordinates body Rect/accessors with the App owner.
- Main session: integration, broad quality gates, full-scope Trellis check dispatch, specs and final acceptance. No concurrent writers to the same product files.

## Implementation and verification

- Completed: read-only last-use/history projection and zero rendering/selection I/O regressions.
- Completed: asynchronous Profile snapshots/mutations, captured operation identity, stale-result suppression and safe deferred exit.
- Completed: 34 actual Ratatui buffer exports for three platforms, both languages/themes and edge states; semantic color, geometry and scroll-reach regressions.
- Completed: fixture measurements for reload acknowledgement/data readiness, delayed mutations, history query and render cost. Native first-frame/flush and cold-disk timing remain unmeasured.
- Completed: full-scope independent review and focused suites, including exact strict Clippy. Review findings were repaired and rechecked.
- Completed: all 16 standard CI recipes. The aggregate passed its first 12 gates, then found an existing Tailwind lint compatibility issue; the reviewed config repair passed its direct controls and all four remaining recipes. See `integration-validation.md`, `ci-final.log` and `ci-frontend-recovery.log` for the exact command outcomes.

Per-owner evidence is recorded in `history-implementation.md`, `async-implementation.md` and `visual-implementation.md`. `check-final.md` records independent review; `integration-validation.md` maps acceptance to the evidence and states its limits. The user subsequently requested committing all changes and archiving the tasks. Work commits are recorded in task.json; task.py owns the final completed status and archive move. No push was requested.
