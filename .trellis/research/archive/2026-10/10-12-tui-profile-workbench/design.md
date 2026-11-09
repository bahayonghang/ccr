# TUI Profile workbench design

Status: revision 2 candidate A approved by the user; implementation and integrated verification complete (all standard CI recipes; see the aggregate/recovery record). Source of requirements: [prd.md](prd.md). Candidate A in [revision 2](research/right-panel-v2.md) supersedes the original side-by-side details arrangement. Measurements and actual Ratatui evidence are recorded in `research/{history,async,visual}-implementation.md`; earlier research records the pre-implementation findings.

## 1. Architecture and ownership

```text
ccr-store: read-only history projection
        -> ccr-cli: platform/profile activity snapshot
ccr-config / platform services: profile and runtime snapshot
ccr-usage: existing provider/source SQL projection
        -> ccr-tui background tasks -> versioned App state -> Ratatui view
ccr-tui Apply/Off intent -> one blocking worker -> existing lifecycle owner
        -> typed committed/warning/recovery outcome -> affected snapshot refresh
```

Do not copy SQL into TUI, add usage SQL to `ccr-db`, or link upstream llmusage Rust. CCR operation history belongs to `ccr-store`; llmusage projections belong to `ccr-usage`. New last-use data is a presentation read model, not another persistent field in `ProfileConfig`.

Shared implementation lives in the parent task. Children consume the common geometry/data state and add platform-specific field assembly. `ui.rs` and `app.rs` are shared hotspots: implement child changes sequentially. Extract a narrowly named module only when the new activity or table responsibility warrants it, not as an unrelated file split.

## 2. Visual system

### Theme and atmosphere

Dense, quiet terminal workbench. Existing Mocha/Latte surfaces, hairline grid, strong model/profile values, neutral labels. No animation on keyboard navigation. This is a Ratatui cell layout; font size/family are terminal-owned.

### Palette and roles

Use `theme.rs` existing colors rather than hardcoded colors in widgets. Mocha background #1e1e2e, surface #313244, rules #585b70, primary #cdd6f4, secondary #a6adc8, good #a6e3a1, warning/cost #f9e2af, error #f38ba8. Codex accent #89b4fa, Claude #fab387, Grok paired neutral foreground (#cdd6f4 / Latte #4c4f69) as a distinct identity token. Keep current selected-row mauve to preserve product continuity. Semantic status never relies only on color.

### Typography

One terminal font; three roles: bold identity/value, ordinary body/label, muted annotation. Right-panel headings use sentence case. Remove italic styling from routine table descriptions. Left-align identifiers; right-align counts and amounts; align timestamps consistently. CJK width uses `unicode-width` and existing truncation helpers. Dates/costs use complete compact forms, not trailing ellipses.

### Components and state

- Runtime strip: fixed labels and styled values for mode, bound profile/model and auth. Unknown is neutral; required-but-missing auth warns; errors are red. Unbound runtime-only is a valid separate state, not automatically an error. Preserve typed status decisions.
- Profile table: header, thin header separator, vertical column guides, full-row selection; distinct current/disabled/pending markers. No per-row boxes and no extra separator row between each record.
- Focus summary: selected name and current/enabled/pending status exactly once, plus persistent last-apply result when present.
- Details follow one reading path: compact model/connection summary, provider usage ledger, then recorded activations. Tags stay with identity. Hide unset optional rows in the normal view; expanded mode retains inspection of all fields. Invalid required data is always visible. Do not fill the normal view with unset counts or hypothetical error-policy prose.
- Usage: one requests/tokens/estimated-cost strip followed by five aligned token-category bars and exact compact counts. Each bar uses the largest displayed category as the common scale; it is not a composition, quota, trend or cache hit rate. Unknown stays unknown, zero has no filled bar. Numeric source totals are independent of this comparison.
- Activity: switch count next to its heading; the newest of up to three vertically aligned timestamps is Last used. Embed the timezone in each timestamp, without separate Recorded at or UTC-offset fields. Retain one concise activation-scope line.
- Pending/error/empty state stays inside its panel; previous data may remain visible with refresh status. No fake zeros.

### Layout

Retain outer 46/54 wide split. Runtime uses 4 rows in normal/wide, 3 in compact, with explicit width-aware shortening/reflow; all important status semantics remain visible or reachable in detail. The right panel stays in one vertical flow at every width. Its connections use a short label column (11 cells in the English preview), with values wrapping in the remaining width. Localize and measure the label column; never clip a translated label into a value. Long model names move reasoning to its own row. The usage ledger uses the width for a meaningful category comparison rather than distant key/value pairs.

Show category bars when the ledger has at least 58 content cells, after padding; below that retain aligned labels and counts. Use a common bar track and a value column just after it, with partial terminal-block cells for small nonzero counts. Totals may wrap to two rows without losing estimate/scope labels. Fit the detail frame to its content when space permits, and cap it at the available viewport. Compute actual wrapped line count for scrolling, including the final wrapped line. A short panel shows an explicit scroll affordance; content must remain reachable with the existing detail scrolling keys. Empty datasets shrink to their honest empty-state messages.

Table layout uses its actual inner width, independently of viewport name:

| Inner cells | Columns in order |
|---|---|
| >=88 | Status 3, Profile 24, Description flexible, Model 20, Last used 14 |
| 62..87 | Status 3, Profile >=22, Description flexible, Last used 14 |
| 40..61 | Status 3, Profile flexible, Last used 14 |
| <40 | Status + Profile; last use remains in selected detail |

Guides/gaps are subtracted first. A shared layout object determines header, cells, clipping, body Rect, page size and hit testing. Header is one row plus one separator. Body rows stay height one. This avoids separator-to-record mapping complexity.

### Depth

Neutral borders and background steps. Accent indicates platform/focus, not every enclosing border. No nested decorative cards, thick side markers or simulated shadows.

### Do / don't

Do retain complete status meaning, scope/units on numbers, current-vs-selected distinction and one global shortcut footer. Don't duplicate selected identity, decorate absent data, fabricate trends or fees, or change authentication semantics to fit the layout.

### Responsive

Keep the current 90/120 width and 22-row mode boundaries. Verify 60×16, 80×20, 89×22, 90×22, 100×30, 119×30, 120×22, 140×30 and 200×50. At very short height, table remains primary and current status remains succinct; no panel may generate negative or overflowing dimensions.

### Implementation prompts

- Runtime: existing theme semantic styles, label normal/value bold, 4-row outer box, explicit 3-row compact form, no text-derived business status.
- Table: one row header + one row rule, row height 1, status/Profile/time priority, whole-row existing selection style, all widths in terminal cells.
- Details: follow revision-2 candidate A; compact connection rows, one usage ledger and vertical activation history. Section rules use `theme::border`; use typed value tones, measured wrapping, common category scale and adjacent counts. Do not restore the rejected Configuration | Activity split or repeat timestamps. Keep all graph preparation in memory.

## 3. Last-use and activity contract

The UI label requested is `Last used` / `上次使用`; selected activity detail retains a short visible disclosure, `Last used = successful CCR activation` / `上次使用按 CCR 成功启用记录统计`. Expanded detail documents that this is the last successful activation recorded by CCR, not the last API call. This semantic compromise is part of final user review.

Use successful `Switch` operation history keyed by explicit `(platform, to_config)` and strict RFC3339 timestamp parsing to UTC. Do not use file mtime, provider's latest API request, auth account `last_used`, usage_count, or deduplicated provider activation log as profile last use. Historical rows without reliable platform attribution remain unattributed. Missing records show `No record` / `无记录`; unavailable/corrupt storage shows its load state; invalid timestamp never becomes now. A future valid timestamp displays an absolute time with a clock warning, never a negative relative duration.

Add a narrow read-only projection in ccr-store, surfaced by ccr-cli. Open existing database read-only, do not create directories/database, migrate, checkpoint or change pragmas on a display read. Keep narrow SELECT fields; do not load `env_changes`/credentials. Filter known operation/status and normalize times explicitly; RFC3339 offsets mean lexical MAX is not a correct latest-instant comparison. Prefer one batched pass for requested platform/profile keys, bounded result state per key. Recent successful activation timestamps may be retained as the newest three entries per key in this same pass; no history transcript or arbitrary metadata is displayed.

Keep `usage_count()` labeled `Switches`/`切换次数` from the existing committed metadata; it is not number of requests and need not equal retained history length. No persisted last-used migration is needed. History retention, older unattributed rows, profile rename/reuse and ancillary write failures limit completeness, so say 'recorded'. Same-name deleted/recreated profiles cannot be distinguished without a stable profile ID; show named-profile history and document this limitation, do not invent identity continuity.

Load once in the background for profile state, cache per session, refresh on explicit reload and successful mutation. If activation commits but history write fails, show the committed-warning outcome and retain the last recorded timestamp with warning. Do not fabricate a successful persistent history entry.

## 4. Right-panel data by platform

| Region | Codex | Claude | Grok |
|---|---|---|---|
| Runtime | Existing typed Codex summary, bound profile/model/auth independent | Existing Claude runtime/auth summary | Existing platform current/auth evidence; no extra probe per render |
| Engine | model, reasoning, base URL | model + main/Opus/Sonnet/Haiku/subagent mappings when set | model, reasoning, api_backend, supports_backend_search |
| Routing/Auth | provider, auth mode/source, env key name, required auth | provider, API-key/official mode, safe auth source, effective priority | provider, inline/env/session mode, env variable name, masked endpoint |
| Activity | last recorded activation, switches, tags, up to 3 recorded activation times | same | same |
| Usage | source=Codex + exact provider | source=Claude + exact provider | source=Grok + exact provider using existing projection |

Usage section: requests, input/output/reasoning/cache-read/cache-write or supported combined cache, total, approximate official-equivalent USD. Display supported independent token counts in the shared ledger without assuming `total_tokens` equals their sum. Scope explicitly all-time provider-level, not profile/account exact billing. Implementation review confirmed the existing UsageDataset/UsageApp exposes no query observation timestamp, so no loaded-at value is fabricated or added as new state. No quota claims derived from token totals. No provider label means unattributed, no matching rows means no recorded usage, missing/unsupported schema means unavailable, query failure stays visible. Verify field availability before committing individual metric labels.

Adding Grok means extending the existing UsageApp source filter and platform mapping and reusing its single in-flight guard and cache. No extra SQL/state engine or new collection process.

## 5. Asynchronous scheduling and consistency

Construct a lightweight App shell using existing theme/config behavior. Schedule active Profile snapshot first, load other Profile tabs lazily. Configuration/path parsing needed for a platform snapshot belongs in its blocking worker. Do not let a slow hidden platform postpone the first frame.

Use existing AsyncTaskExecutor/spawn_blocking + channel pattern, typed messages with platform ID and request generation. Mutations additionally carry operation ID and intended profile. One in-flight profile mutation at a time; repeated Apply/Off is acknowledged as busy without starting another writer. Other browsing remains responsive. Reads returning before/after a mutation must not overwrite a newer committed state. Mutation outcomes are always consumed; generation filtering drops obsolete read snapshots only.

The complete existing lifecycle call remains inside one synchronous worker, including lock, journal, commits and ancillary work. No await within that call. Preserve outcome distinctions and localized feedback on the TUI thread. Refresh the affected platform/runtime and activity snapshot after commit; leave unrelated platform snapshots intact. Explicit `r` refreshes current platform/activity and the existing shared usage engine with single-flight guards.

While a profile mutation runs, keep Profile navigation available but reject keyboard/mouse navigation to Auth tabs with busy feedback. This bounded policy avoids adding a new cross-embedded-app mutation gate. Enter/Space currently apply and keep the TUI open; preserve that behavior. q/Esc/Ctrl+C set a pending exit intent and must not cut off a write. Drain the outcome and restore the terminal before exit. Worker disconnection is an unknown outcome, never an automatic retry. Read jobs may be abandoned on shutdown; writes may not.

`TuiApp::on_tick()` currently returns only whether to redraw, not whether to quit. Add a narrow exit-state accessor with a false default for unaffected apps, and have the runtime inspect it after pumping completions and before its next wait. App exposes ready-to-exit only when quit was requested and no mutation outcome remains pending. Test quit with no further input: completion alone must cause exit. Ctrl+T and other early event branches must run the same completion/exit tail rather than bypassing it with `continue`.

After Profile mutation, invalidate or update the matching initialized Auth view's runtime summary as well as Profile state. Codex Auth keeps a separate summary and its current activation hook does not reload it; schedule the existing safe read on next activation, or propagate the corresponding fresh typed snapshot. Do not display the old current profile after entering Auth, and do not reinitialize the whole Auth app/reset account selection to achieve this.

Pump Profile worker messages at App level even if the user changes tabs. Drain completions and due timers after each event, not only timeout ticks; cap waits to 50 ms while a job is pending, returning to the existing idle interval afterward. Continuous keyboard input must not starve a result. Relative timestamps schedule only their next display boundary. Existing idle redraw suppression is already present and must be preserved.

## 6. Performance strategy

The original structural diagnosis is in research/profile-performance.md. Controlled measurements now appear in the implementation reports; the priorities below explain the approved scope:

1. Main-thread synchronous profile load/apply/off/reload -> background snapshot and mutation jobs.
2. Successful switch reloading all platforms -> only affected snapshot; hidden tabs lazy.
3. Reuse current usage batching; add activity batching with zero render-time I/O.
4. Avoid gratuitous detail Vec clones and repeated width derivation. Add keyed display cache only if measured CPU benefit justifies invalidation complexity (selection, size, language, theme, expansion, data generation, relative-time boundary).
5. Fix background-completion latency/starvation from timeout-only pumping; preserve existing state-change-driven tick redraw, toast expiry, Auth/quota progress and low idle CPU.
6. JSONL full-scan provider activation dedup and repeated history pool initialization are measured follow-ups inside lifecycle. Do not alter their contract merely to reach a speed target.

Proposed controlled benchmark targets: action acknowledgement/completion propagation p95 <=100 ms under a 1 s injected load/apply; warm 140×40 TestBackend render p95 <=16 ms at 1000 fixture profiles, also report 200×50; zero unchanged-state idle redraw/load calls. These are targets, not results or universal hardware promises. First snapshot and apply completion wall time include actual I/O/safe writes and must be reported separately from responsiveness. Query plans/timing and read counts come from owned fixtures; no indexing/migration in the upstream llmusage database.

## 7. Compatibility and rollback

No Profile TOML schema, tab-order identity, public command or auth contract changes. `Claude Profile` is presentation only; `TuiTabId` remains stable. TUI-local brand labels avoid changing CLI/platform-wide emoji consumers. Plain product names are the portable fallback; official graphic marks are conditional on proven support, not fabricated or mandatory new dependencies.

Roll out in commits by responsibility when authorized: read model, async shell, shared visual layout, platform adaptations. Each can be reverted without a data migration. On any snapshot failure, retain recoverable panel state, never write to repair a read. Existing unrelated worktree changes remain untouched.

## 8. Verification / review gate

See implement.md for ordered commands. Require read-only DB proof, timestamp/cross-platform cases, async delayed-worker tests, lifecycle outcome regressions, geometry assertions and actual rendered views. No personal credentials or real profile mutations are needed for tests. Update the relevant TUI/lifecycle spec statements only when the implementation is verified; current synchronous-TUI wording must not be left stale.
