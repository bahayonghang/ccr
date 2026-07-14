# CCR UI Documentation Archive

Archived documents preserve decisions and review evidence. They do not override current code, tests, scoped agent guidance, or the maintained documents linked from the [engineering documentation index](../README.md).

## 2026

| Document | Status | Current authority |
|---|---|---|
| [Claude Profiles dashboard optimization](2026/claude-profiles-dashboard-optimization.md) | Proposed, not implemented | `src/components/claude/ClaudeProfilesOverview.vue` and `src/views/ClaudeCodeProfilesView.vue` |
| [Sync page redesign design](2026/sync-page-redesign-design.html) | Implemented; historical design artifact | [`src/views/SyncView.vue`](../../src/views/SyncView.vue) and [`src-tauri/src/commands/sync.rs`](../../src-tauri/src/commands/sync.rs) |
| [Sync page redesign implementation](2026/sync-page-redesign-implementation.md) | Implemented; historical execution plan | [`src/api/domains/sync.ts`](../../src/api/domains/sync.ts), [`src/types/syncSelection.ts`](../../src/types/syncSelection.ts), and the sync command module |
| [VibeDeck versus CCR UI analysis](2026/vibedeck-vs-ccr-ui-analysis.html) | Point-in-time comparative analysis from 2026-05-25 | Current usage behavior and UI source under `src/components/usage/`, `src/views/UsageDashboardView.vue`, and `src-tauri/src/llmusage_adapter/` |

The HTML files are retained byte-for-byte as historical artifacts. Their status is maintained here rather than injected into the original report body.
