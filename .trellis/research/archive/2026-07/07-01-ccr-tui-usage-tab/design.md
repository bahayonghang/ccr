# Design — ccr-tui usage/statistics tab

Parent: `07-01-provider-usage-stats` · Parent design: §6

## Data Path

C4 uses the same llmusage provider breakdown semantics as C2/C3, but the TUI
must not depend on the Tauri desktop crate. The shared boundary is a small
workspace crate:

```
~/.llmusage/llmusage.db
        │ read-only, schema-gated SQLite projection
        ▼
crates/ccr-usage
        ├── ccr-ui/src-tauri llmusage_adapter delegates provider_breakdown()
        └── crates/ccr-tui usage tab loads provider_breakdown_async()
```

`ccr-usage` owns only portable, read-only query code: path discovery,
source/filter types, schema/capability checks, DTOs, and provider breakdown SQL.
It does not run `llmusage`, migrate/create the DB, parse raw provider logs, or
write CCR state.

## TUI Integration

- Add `TuiTabId::Usage` to the complete tab-order contract. Default position is
  after the two profile tabs and before auth tabs so normal profile switching
  remains first, while usage is easy to reach.
- Add `TabVariant::Usage` and a single synthetic `PlatformTab` labeled `Usage`.
  It uses `Platform::Codex` only for the existing tab accent/icon machinery.
- Add `crates/ccr-tui/src/tui/usage/{app.rs,ui.rs}`.
- `UsageApp` loads on activation with `AsyncTaskExecutor::spawn_blocking()` and
  a message channel. Refresh uses `r`.
- `App::handle_key`, `on_tick`, and `ui::draw` route the Usage tab before profile
  handling so profile navigation and apply keys do not run on the synthetic tab.

## Rendering Contract

Render a compact table grouped by platform with rows:

`provider | requests | input | output | cache | total | approx cost`

Providers include Claude and Codex rows when present. Empty `provider_label`
renders as `unattributed`. The cost title and footer use
`approx official-equivalent price` so the surface never implies real relay
billing.

Graceful states:

- DB missing/unreadable: show where llmusage DB was expected and suggest running
  usage import/sync outside the TUI.
- Schema or missing-column gate: show an update-llmusage hint.
- Empty provider rows: show an empty-state message, not an error.
- Background task failure: keep the TUI responsive and show the error in the tab.

## Compatibility

Existing `tui.toml` files without `usage` become incomplete under the new
complete-list contract and will fall back to the new default order. This matches
the existing validation rule for newly added tab ids and avoids silently hiding
the new tab.

## Rollback

Hide the Usage tab by removing `TuiTabId::Usage` and the synthetic tab build.
The shared `ccr-usage` crate is read-only and has no persisted state to undo.
