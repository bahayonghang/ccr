# UI Engineering Documentation Gap Audit

## Current Inventory

`ccr-ui/docs/` currently has five files:

- `design-system/page-templates-and-surfaces.md`
- `plans/claude-profiles-dashboard-optimization.md`
- `superpowers/plans/2026-05-29-sync-page-redesign-implementation.md`
- `spark/2026-05-29-sync-page-redesign-design.html`
- `artifacts/vibedeck-vs-ccr-ui-analysis.html`

There is no root README or index explaining ownership, lifecycle, or status.

## Confirmed Findings

### Long-lived contract with drift

`design-system/page-templates-and-surfaces.md` is the only document written as a durable contributor contract. Its component and surface concepts still exist, including `MainLayout`, `ModuleSubnav`, `PageHeaderCard`, `AsyncStatePanel`, `surface-shell`, and `surface-workspace`.

However, its canonical command `bun run test:playwright:snapshots` is absent from `ccr-ui/package.json`, even though old snapshot artifacts still exist under `ccr-ui/tests/artifacts/route-snapshots/`. The verification section is therefore stale.

### Unimplemented proposal

`plans/claude-profiles-dashboard-optimization.md` is a detailed proposal, not a current contract. Its proposed `cpd-identity` and `cpd-metrics` structures are absent from `ccr-ui/src/`. It must not remain beside current guidance without an explicit proposed/superseded status.

### Completed historical implementation material

The May 29 sync design and implementation plan describe `list_sync_assets`, `SyncAssetInfo`, grouped assets, and localized asset actions. Those elements now exist in:

- `ccr-ui/src-tauri/src/commands/sync.rs`
- `ccr-ui/src/api/domains/sync.ts`
- `ccr-ui/src/types/syncSelection.ts`
- `ccr-ui/src/views/SyncView.vue`
- both locale dictionaries

These files are useful as historical decision records but are no longer active implementation plans.

### Point-in-time analysis artifact

`artifacts/vibedeck-vs-ccr-ui-analysis.html` identifies itself as an offline report generated on 2026-05-25. It is not referenced by current code or docs and should be treated as historical analysis, not current architecture.

## Proposed Direction

- Add `ccr-ui/docs/README.md` as the lifecycle and navigation entrypoint.
- Keep durable docs in explicit areas such as `architecture/`, `design-system/`, and `development/`.
- Move point-in-time plans and HTML reports under a dated `archive/` hierarchy with status metadata and a short index, or delete them if history is intentionally not retained.
- Refresh the design-system document against current tokens, components, routes, and actual verification commands.
- Add a current UI architecture/module map based on router, API facade, stores/composables, Tauri commands, and generated bindings.
- Document the boundary with the published `docs/` site to prevent duplication.

## Confirmed Product Decision

Keep point-in-time plans and reports in a dated in-repository archive. The archive index must distinguish implemented work, unimplemented proposals, and point-in-time analysis so historical material cannot be mistaken for current guidance.
