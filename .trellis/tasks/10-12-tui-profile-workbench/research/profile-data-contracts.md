# Research: Profile activity timestamps and detail-panel data contracts

- Query: Identify truthful last-used semantics and useful existing data for the Codex, Claude, and Grok TUI profile workbench, including ownership, attribution, read-only boundaries, and performance/test seams.
- Scope: internal; source and specifications only. No personal configuration, credentials, database rows, profile mutations, or product-code changes.
- Date: 2026-10-12

## Findings

### Decision summary

1. Use a **read-only, batch projection of successful profile Switch history**, owned by `ccr-store` and exposed through `ccr-cli`, for the proposed activity timestamp. Its meaning is **last recorded successful application of this platform/profile name**, not last model request. Preserve the requested “Last used / 上次使用” column only with a visible explanation of this meaning; “Last applied / 上次应用” is the more precise wording.
2. Existing `ProfileConfig::usage_count()` is a successful-activation counter. Continue presenting it as **Switch count / 切换次数**. Do not add a history row count or activation-log count as an allegedly more precise lifetime count.
3. Continue consuming the shared `ccr-usage` provider projection. Add `SourceKind::Grok` to the existing dataset and render path, with the same capability and attribution states as Claude/Codex. Grok is already a supported shared source type; missing TUI wiring is not an unsupported database source.
4. Enrich the detail panel first with fields already present in cached profile DTOs and `ProviderBreakdownDto`: cache-read/write and reasoning splits, estimated cache-aware/cache-free costs, explicitly scoped activity, and platform-specific configuration. No new per-selection SQL, credential reads, or duplicated usage aggregates are required for these improvements.

### Files found and ownership

| File | Description and source anchor |
| --- | --- |
| `crates/ccr-tui/src/tui/app.rs` | Profile rows, per-tab configs, and loading: `ProfileItem` at 31, `PlatformTab` at 51, `build_profile_tab_data` at 267. |
| `crates/ccr-tui/src/tui/ui.rs` | Grok details at 1094, Codex at 1193, Claude at 1351; provider lookup at 1489 and metric formatting at 1556. |
| `crates/ccr-tui/src/tui/usage/app.rs` | Injectable, background-loaded provider dataset; loader at 153 and load-state conversion at 137. |
| `crates/ccr-tui/src/tui/profile_backend.rs` | Thin shared-lifecycle apply adapter at 5; operation outcome presentation at 9. |
| `crates/ccr-config/src/models/platform.rs` | Shared `ProfileConfig` at 174; metadata at 261–286, count getter at 352. |
| `crates/ccr-config/src/managers/config/types.rs` | Persisted `ConfigSection`, including count and flattened extensions at 132–156. |
| `crates/ccr-config/src/managers/platform_config.rs` | Platform-level `last_used` at 37; setters at 236 and 251. |
| `crates/ccr-config/src/managers/provider_activation.rs` | Provider attribution JSONL schema at 42; deduplication at 157; host-only sanitization at 208. |
| `crates/ccr-cli/src/application/profile_lifecycle.rs` | Post-commit count/history/analytics side effects at 313–363. |
| `crates/ccr-store/src/history.rs` | History schema types at 93–129, default constructor at 180, timestamp decode at 193, writer at 225. |
| `crates/ccr-store/src/storage/database.rs` | Default path and initialization at 25–71; history SQL schema and indexes at 218–237. |
| `crates/ccr-usage/src/db.rs` | Read-only dashboard open at 202; provider SQL at 447; per-source batch adapter at 494. |
| `crates/ccr-usage/src/queries.rs` | `ProviderBreakdownDto` at 119 and `TaggedProviderBreakdown` at 159; optional richer DTOs at 33, 56, and 77. |
| `crates/ccr-usage/src/capabilities.rs` | Provider minimum schema at 14, required columns at 330; capability read-only open at 405. |
| `crates/ccr-usage/src/source.rs` | `SourceKind::Grok` at 28; canonical storage ID at 57 and aliases at 83. |
| `crates/ccr-db/src/models/claude_profile.rs` | A separate desktop settings-snapshot model, with created/updated times at 25–26; not this TUI's profile source. |

### The available timestamps are different concepts

| Candidate | Actual meaning | Suitability for the profile table |
| --- | --- | --- |
| `ProfileConfig` / `ConfigSection` | Neither has a typed per-profile `last_used`, `last_switched_at`, or `last_activated_at`. | Cannot display a field that does not exist. Avoid inventing a persisted timestamp from defaults or filesystem mtime. |
| Platform registry `last_used` | One timestamp for a platform's current-marker activity, including deactivation. | Not per-profile. Never copy it into all rows or interpret it as a last request. |
| Auth account `last_used` | Timestamp on an authentication account, changed when that account is activated. | Different entity; one account can be used with multiple profiles. Do not join by display name. |
| Provider activation JSONL `activated_at` | Beginning of a recorded provider/profile activation interval. Consecutive equivalent events are deduplicated. | Useful attribution evidence, not an exact last-apply history; exclude from primary last-used calculation. |
| CCR history `timestamp` | Time a successful Switch history entry was recorded after the profile activation committed. | Best existing source for explicitly labeled last recorded application. Missing entries remain “No record / 无记录”. |
| llmusage `event_at` | Model-usage event timestamp; current projection identifies source/provider, not this profile identity. | A future “provider last activity” may use it, but it cannot be presented as exact profile last usage. |
| Desktop `ClaudeProfile.updated_at` | Update or activation of a separate settings/MCP/output-styles/budget snapshot. | Not a join key or timestamp source for this TUI profile list. |

Evidence:

- Profile fields end with metadata/count/enabled/platform extensions at `crates/ccr-config/src/models/platform.rs:261`; persisted fields do likewise at `crates/ccr-config/src/managers/config/types.rs:132`.
- `set_current_profile(..., None)` still updates registry `last_used` at `crates/ccr-config/src/managers/platform_config.rs:251`; clearing the registry marker also updates it at `crates/ccr-config/src/platforms/base.rs:664`.
- Auth-account updates are separate at `crates/ccr-codex/src/services/codex_auth_service.rs:1399` and `crates/ccr-cli/src/services/claude_auth_service.rs:972`.
- Activation events carry `(platform, profile, provider, activated_at, event)` at `crates/ccr-config/src/managers/provider_activation.rs:42`. Deduplication ignores timestamp and compares event kind/provider/profile at 168. Thus reapplying a current profile can increment the profile count but produce no new activation-log entry.
- `ClaudeProfile` explicitly represents a complete desktop configuration snapshot at `crates/ccr-db/src/models/claude_profile.rs:6`; both update and set-current update its `updated_at` in `crates/ccr-db/src/database/repositories/claude_profile_repo.rs:95` and 129. The inspected TUI list reads `PlatformConfig::load_profiles`, not that repository (`app.rs:294`).

### Recommended history projection

**Ownership and minimal output**

Add one narrow read API to the history owner (`ccr-store`) and reuse it through `ccr-cli`. Return a map keyed by exact `(platform, profile_name)` containing the last valid recorded successful application timestamp, plus explicit load/error state. Keep the original timestamp string until strict parsing; expose a parsed UTC instant for comparison/formatting. The service need not return history actors, notes, backup paths, environment changes, or whole serialized entries.

**Selection rules**

- Select only raw `operation = 'Switch'`, raw `result = 'Success'`, a recognized explicit platform in `extra`, and a nonempty `to_config`.
- Match exact platform plus exact profile name. Same names across Claude/Codex/Grok must stay separate.
- The current lifecycle stores the platform in `extra` and the target in `to_config` (`crates/ccr-cli/src/application/profile_lifecycle.rs:334–346`). Older entries with absent, arbitrary, or ambiguous `extra` must not be guessed to mean Claude or the currently selected platform.
- Parse RFC3339 strictly and compare instants in UTC. Do not compare raw timestamp strings or use `MAX(timestamp)` blindly: history timestamps contain the local UTC offset (`crates/ccr-store/src/history.rs:140`, 248), and lexical ordering of mixed offsets can differ from chronology.
- Invalid timestamps do not become “now”. Keep a partial-data indication or diagnostic for invalid eligible records; do not invent precision.
- No matching record means “No record / 无记录”, not “Never used”, zero, the current clock, the registry timestamp, or the provider's activity time.
- Profiles have name-based identity here. An inactive rename has no successful Switch for its new name; do not infer an alias from descriptions, provider, or tokens. Deletion followed by reuse of a name is an identity limitation to document.

**Why not reuse `HistoryManager::load()` directly?**

- `HistoryManager::with_default()` calls `Database::init_default()` (`history.rs:180`), which creates the directory, opens a writable pool, and migrates (`storage/database.rs:25–71`). A table read must not trigger those writes.
- Its general row decoder silently replaces a malformed timestamp with `Local::now()` (`history.rs:193–195`), unknown result values with Success (197–201), and unknown operation strings with Switch (57–65). Those compatibility behaviors would fabricate this new display's evidence.
- `load()` selects all history fields, sorts all rows, decodes environment-change JSON, and silently drops row errors (`history.rs:274–287`). The proposed field only needs three small columns and validated filtering.

**Read and performance contract**

- Respect existing data-path precedence: `CCR_DATA_DIR`, then `CCR_ROOT`, then the default CCR directory (`storage/database.rs:25–40`). Extract/reuse path resolution without calling the initializer.
- Open the existing database read-only; never create, migrate, clear, or repair it for display. Missing database, absent history table, unreadable database, query failure, empty valid data, and malformed eligible records need distinguishable internal outcomes.
- Load the narrow projection once in a blocking worker, reducing rows into a map. Refresh after relevant committed profile actions or explicit reload; arrow-key navigation and drawing read the cached result only.
- Avoid one query per profile and avoid a UI-side scan of every history record on each frame. Existing indexes are `history(timestamp)` and `history(operation)` (`storage/database.rs:236–237`). Measure the filtered batch with a large synthetic history before proposing another index/migration.
- Display refresh/load state separately from an absent per-profile record. If retaining a previous good snapshot after a refresh error, identify it as stale.

**Counts and transaction semantics**

`usage_count` increments with saturating addition only after the activation journal commits (`profile_lifecycle.rs:313–331`). History and provider analytics are independent best-effort side effects (333–363). Therefore count, history-row count, and activation-log count can differ legitimately. A failed history write does not mean activation failed; an idempotent replay does not repeat any of these effects. Keep the existing `switch_count` display and do not add a second supposedly exact “total applications” count from history.

### Provider usage: accurate scope and Grok integration

- `UsageApp` holds the one in-memory provider dataset; current production loading requests only Claude/Codex (`crates/ccr-tui/src/tui/usage/app.rs:153–160`). It already uses `spawn_blocking` and an injected `UsageLoader` (38, 100–114).
- `Dashboard::provider_breakdown_by_source` calls the same shared provider query for each explicit source, then tags the result (`crates/ccr-usage/src/db.rs:492–514`). Add Grok to this source list instead of inventing another store or SQL query in the TUI.
- Grok's canonical source is already `grok` (`crates/ccr-usage/src/source.rs:28`, 57, 83). There is no Grok-specific unsupported gate in this provider query.
- Opening the dashboard requires readable schema metadata and schema >= 10 (`db.rs:202–220`). Provider breakdown specifically requires schema >= 14 (`capabilities.rs:14`, 276), the required token/cost columns in `usage_bucket_30m`, and `provider_label` in both `usage_bucket_30m` and `usage_event` (`capabilities.rs:330–346`). Compatible schema alone does not establish that any Grok events have been synchronized.
- The query groups by `provider_label`; it sums `event_count`, token splits, and costs (`db.rs:447–486`). It does not group by profile, API key, or auth account. Two profiles sharing a provider in the same source should show the same totals, explicitly labeled **provider-level, all-time**.
- Profile matching is by its explicit nonempty provider label plus source. Missing provider labels must not fall back to the aggregate null/unattributed bucket. Current lookup implements the important None boundary at `ui.rs:1501–1511`; ensure any new empty/whitespace handling is equally conservative.
- The activation map records only metadata and URL host, not credentials (`provider_activation.rs:42`, 125–145, 208–220). llmusage sync consumes that map when it exists; do not claim a new profile label instantly repairs historical attribution. Actual upstream source ingestion/attribution availability must remain an explicit data state.
- A loaded dataset without a matching Grok/provider row is “No usage recorded”; an old schema is Unsupported; a missing/unreadable/query-failing DB is Error. `state_from_load_result` already owns these distinctions (`usage/app.rs:137–149`). Do not display zero costs/tokens for unsupported or failed reads.

### Useful right-panel fields without another query

| Area | Existing data to expose | Accuracy rules |
| --- | --- | --- |
| Profile identity | description, provider/type, account label, tags, enabled/current status | Selected profile identity and actual runtime identity are separate. Keep one Focus summary instead of repeating it in every section. |
| Activity | existing switch count; proposed last recorded successful application | Explain the timestamp source; do not count requests as switches or fill missing records with guesses. |
| Codex engine/auth | model, reasoning effort, base URL, wire API, env-key name, OpenAI requirement/login method, auth mode/source | Reuse `CodexPlatform` semantic helpers and cached summary; never inspect credential files during rendering. |
| Claude engine | model, existing Opus/Sonnet/Haiku/Fable defaults and display names, subagent model, effort level, timeout and auto-compact settings | These typed fields already exist in `ProfileConfig` at `models/platform.rs:201–257`; show configured values only and collapse unset values. Do not pretend they are measured runtime results. |
| Grok engine/auth | model/reasoning, backend, context window, backend-search flag, sanitized base URL, auth mode, env-key name; provider/account/tags where present | Preserve typed invalid/missing states. Session auth mode alone is not proof that an official session is valid. |
| Usage | requests, total/input tokens, assistant output, reasoning output, cache read, cache creation | `ProviderBreakdownDto` already has each split (`queries.rs:119–136`). Current UI merges output+reasoning and read+creation (`ui.rs:1575–1576`, 1614, 1622); showing the separate values adds information without more SQL. |
| Estimated costs | with-cache estimate, without-cache estimate, clearly labeled difference if desired | Both costs already exist. Do not call these actual provider bills, cash saved, account balance, or subscription quota. |

The current noncompact note says “approx official-equivalent · provider-level (all-time)” (`ui.rs:1648`); compact rendering returns early without that note (`ui.rs:1557–1591`). The redesign should keep scope and estimate qualifiers visible at every supported viewport.

Further possible information—seven-day trend, top models, provider last event, freshness—is not currently in `UsageDataset`. Existing `OverviewPayload` contains `last_sync_at`/`last_export_at` (`queries.rs:43`), while `DailyTrendDto` and `ModelBreakdown` have useful metrics (56, 77). Those are extension candidates, not freely available fields. If selected, batch/provider-scope them in `ccr-usage`; avoid per-row/per-selection follow-up queries. A local “loaded at” timestamp is not an upstream “last synced at” timestamp.

### Secret and runtime boundaries

- Keep `configured_token_text` and `ccr_core::mask_sensitive` as the established Codex/Claude token presentation boundary (`ui.rs:1659–1673`). Grok's current profile details intentionally do not render even a masked inline token; regression evidence is `ui.rs:3846`.
- Grok sanitizes URL userinfo, query, and fragment with `GrokPlatform::safe_base_url_for_display` (`crates/ccr-cli/src/platforms/grok.rs:174–193`; consumed at `ui.rs:1101`). New summary/table endpoint displays should preserve this protection. Existing Codex/Claude raw base-URL rendering at `ui.rs:1240` and 1389 is a relevant review point when rebuilding those displays.
- Do not dump `platform_data`, settings snapshots, raw history notes, environment values, or bearer tokens merely to fill whitespace. Use an allowlist of typed, meaningful fields.
- Codex/Claude runtime summaries already expose mode, current profile/provider, auth mode/source, and auth/login state (`crates/ccr-codex/src/models/codex_auth.rs:474`; `crates/ccr-types/src/claude_auth.rs:300`). Use semantic states for colors. `PlatformTab` currently has no equivalent Grok summary field (`app.rs:51–64`); any new runtime data must come from Grok's existing domain diagnosis, not a UI guess from the highlighted row.

### Test seams and acceptance evidence

No tests were run in this research-only phase. Proposed implementation checks:

1. **History projection fixtures:** mixed platforms with identical profile names; successful/failed/warning/update/unknown operations; blank or unrecognized platform; absent target; invalid timestamp; mixed offsets whose lexical order is wrong; missing database/table; read-only/unreadable path; unchanged files/directories before and after queries. Assert no fallback to “now” and no initialization or migration.
2. **Count versus timestamp:** keep `usage_count` independent of returned history records. Reuse lifecycle fixtures for `HistoryFailed`, `UsageCountFailed`, analytics failure, committed warnings, and same-operation-ID replay. Do not make UI history refresh repeat activation.
3. **Usage projection:** extend `provider_breakdown_by_source_tags_rows_per_platform` (`crates/ccr-usage/src/db.rs:1430`) with Grok and cross-source equal provider names. Retain schema/missing-column cases at 1526 and 1555. Unknown/missing provider must never consume the unattributed aggregate.
4. **Injected loader UI:** use `UsageApp::with_loader`, not a real home database. Existing tests at `usage/app.rs:217` cover load-state classification; TUI provider assertions at `ui.rs:4564`, 4588, 4612, 4633, and 4752 cover attribution, status, metrics, and injected provider switching.
5. **Rendering/security:** TestBackend at compact/standard/wide widths, both languages, CJK names, relative/absolute timestamps, long provider labels, no raw or Grok-masked token, sanitized endpoints, explicit estimate/source qualifiers, and loading/stale/error versus no-record states.
6. **Performance:** synthetic large history/profile/provider fixtures with a counting loader. Confirm one batch load per refresh and zero database/filesystem activity during selection and draw. Record baseline and improved timings before claiming a bottleneck or speedup. An O(provider-count) in-memory lookup is visible in `ui.rs:1534`; index by source/provider only if measurement warrants it.

Use the repository's current higher-priority test contract: default parallelism with `--skip export_bindings`, even where an older package spec still mentions `--test-threads=1`. Expected focused packages are `ccr-store`, `ccr-cli`, `ccr-usage`, and `ccr-tui`, followed by scope-appropriate strict lint and integration gates.

## Related specs

- `.trellis/spec/ccr-tui/backend/backend-guidelines.md`: cached background usage engine; provider-level attribution; no per-selection SQL; semantic detail tones, masking, localization, terminal display-width handling.
- `.trellis/spec/ccr-config/backend/profile-repository.md:36`: pure read/current-marker rules and platform runtime resolution; no reader-side initialization or marker repair.
- `.trellis/spec/ccr-cli/backend/profile-application-lifecycle.md:11`: activation commits before count/history; ancillary warning and replay semantics; preserve the synchronous transaction within one worker.
- `.trellis/spec/ccr-store/backend/backend-guidelines.md:5`: history/storage ownership and SQLite error mapping.
- `.trellis/spec/ccr/backend/llmusage-provider-adapter.md:117`: provider grouping, canonical sources, single SQL owner, capability errors, existing provider-map requirement, and shared TUI DTOs.
- `.trellis/spec/ccr-db/backend/backend-guidelines.md:37`: upstream usage databases remain read-only; no migrations of upstream tool DBs.
- `.trellis/spec/ccr-types/backend/backend-guidelines.md`: shared serialization compatibility; avoid changing shared wire DTOs just to restyle this page.

## External references and versions

- No external sources were needed or consulted; findings describe the checked-out code, not independently verified upstream behavior.
- Repository manifest anchors: `crates/ccr-tui/Cargo.toml` pins Ratatui 0.30.2, Crossterm 0.29.0, Unicode Width 0.2.2, and Rust 1.95; `ccr-usage` provider compatibility constants are schema 10 minimum open / schema 14 provider breakdown.

## Caveats / Not Found

- Exact per-profile last API request time is not present in the inspected contracts. The proposed minimal implementation is last recorded successful profile application with explicit semantics, pending final planning review.
- No installed llmusage version, real data freshness, Grok synchronized rows, or real attribution completeness was inspected. Provider support is code/capability evidence, not proof of a user's existing dataset.
- History is ancillary and can be cleared (`crates/ccr-store/src/history.rs:418`); missing rows do not prove a profile was never used. No automatic history retention limit was found in this manager.
- Legacy platform-less history, profile rename/reuse, and incomplete recordings limit name-based attribution. Preserve unknown states rather than silently guessing.
- No performance timings were measured, so query/render costs discussed here are source-backed candidates, not measured latency claims.
- Icon asset sourcing and visual layout are owned by the sibling UI research; this file does not prescribe raster logos or terminal graphics protocols.
