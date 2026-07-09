# ccr-tui Backend Guidelines

> Terminal UI crate built on ccr-cli services.

## Scope

`crates/ccr-tui` owns Ratatui/Crossterm terminal interaction, app state, tab rendering, key/mouse handling, and embedded auth views. It reuses `ccr-cli` models/platforms/services rather than owning config persistence.

Reference files:

- `crates/ccr-tui/src/lib.rs`
- `crates/ccr-tui/src/tui/app.rs`
- `crates/ccr-tui/src/tui/runtime.rs`
- `crates/ccr-tui/src/tui/ui.rs`

## Structure

Keep UI state and rendering separate:

- `app.rs` owns app state, selected tab/profile, and action handling.
- `ui.rs` and tab modules render Ratatui widgets.
- `runtime.rs` owns terminal runtime and async task execution.
- Platform auth subdirectories own embedded auth flows.

Do not add config file parsing or database writes here. Call `ccr-cli`, `ccr-codex`, or `ccr-config` services.

## Error Handling

Surface recoverable profile/auth loading failures inside the UI as issue strings (`Where`/`What`) instead of panicking. `format_issue` in `app.rs` is the local pattern.

Do not let logs corrupt the terminal. The root binary selects file-only logging for TUI mode.

## Interaction Rules

Maintain stable tab/profile selection across refreshes where possible. Use explicit cached `Rect` fields for mouse hit-testing, as `App` does with `header_area`, `list_area`, and `detail_area`.

Preserve pagination helpers and page-size behavior when changing list rendering.

### Text truncation, padding, and shortcut hints

Truncate and pad cell text by terminal display width via `unicode-width`, never by `chars().count()` — CJK characters render 2 columns wide, so char-counted cells overflow their column and ratatui hard-clips them, losing the `…` marker. Follow the shared helper shape (`truncate_text`/`pad_text` in `tui/ui.rs`, same-named helpers in auth sub-apps and the usage view): accumulate per-char width, reserve 1 column for `…`, and prefer ending 1 column short over overflowing.

Keyboard shortcut hints live only in the global Keys footer. Panels and status strips carry state (selection, apply/toast feedback), not key legends — do not reintroduce per-panel shortcut lines.

### Per-tab profile selection

Each profile tab owns its selection snapshot (`PlatformTab::saved_selection`); `selected_index` / `current_page` / `selected_profile_name` on `App` are the working copy for the active tab only (`page_size` stays global). On tab switch, `save_active_tab_selection` stores the leaving tab's snapshot and `restore_active_tab_selection` loads the entering tab's — restoring a saved snapshot via `align_selection_by_name` (name-first), or focusing the enabled (`is_current`) profile via `focus_current_profile` on first visit.

Do not re-add a `sync_selection_to_profile_name()` call into `notify_tab_activated` for profile tabs, and do not use `sync_*` to realign a restored snapshot. Its Codex branch prefers `is_current` and will clobber the per-tab snapshot, making Codex always jump back to the enabled profile. Realigning a restored snapshot must stay name-first (`align_selection_by_name`).

### TUI Tab Startup Contract

The main `run_tui()` entry should construct `App::with_task_executor(...)` and leave `active_tab = 0`, so the configured tab order controls the first visible tab. Do not chain auth-tab preselection helpers from the main entry.

Auth shortcut entries may still use explicit preselection helpers:

```rust
App::with_task_executor(task_executor)?.with_claude_auth_tab();
```

When changing tab ordering, add or keep regression tests that assert:

- default ordering places `Codex Profile` first
- `active_tab = 0` selects the first configured tab
- auth shortcut helpers still select their matching auth variant after reordering

### Synthetic read-only tabs

Use a synthetic `PlatformTab` only when a tab is not a profile/auth surface but still needs to live in the configured tab bar. There is currently **no** synthetic tab in the tree: the standalone Usage tab was retired in 2026-07 (usage now renders inside profile details, see the next section). Keep this contract for any future synthetic tab:

- Add a matching `TuiTabId` in `crates/ccr-config` and include it in the complete-list default order.
- Give the synthetic tab an empty `profiles` list and route it before profile selection/apply behavior in `handle_key`, mouse handlers, activation, ticks, and `ui::draw`.
- Lazily initialize the embedded app from `App::with_task_executor(...)`; load external data with `AsyncTaskExecutor::spawn_blocking()` and a message channel so the terminal render loop never blocks on filesystem or SQLite work.
- Keep the tab read-only unless the PRD explicitly asks for mutations. For usage/statistics views, show unsupported, missing-data, empty, and query-error states inside the view rather than panicking or falling back to profile UI.
- When retiring a synthetic tab, keep its `TuiTabId` variant parse-tolerant (`#[doc(hidden)]`, filtered on load with a warn) so existing `tui.toml` custom orders survive; see the ccr-config guidelines.

Wrong:

```rust
// Synthetic tab reaches profile apply/select handling.
let action = self.map_key(key);
self.handle_profile_action(action)
```

Correct:

```rust
if self.is_my_synthetic_tab() {
    if let Some(embedded) = self.my_synthetic_app_mut() {
        return embedded.handle_key(key);
    }
    return Ok(false);
}
```

### Embedded usage engine (profile-detail Usage section)

Provider usage lives inside the Claude/Codex profile detail panel, powered by an App-level data engine (`tui/usage/app.rs::UsageApp`), not a tab:

- `UsageApp` does **not** implement `TuiApp` (no key/render duties). `App::on_tick`'s profile-tab branch drives it: `ensure_usage_engine()` + `on_activated()` (idempotent, only arms the 1-tick delay while `Idle`) + `tick()` (pumps the mpsc channel). This covers both startup landing on a profile tab and later tab switches.
- Data loads once per session via the injectable `UsageLoader` seam on `spawn_blocking` (`provider_breakdown_by_source([Claude, Codex], default filter)`); selection changes are pure in-memory lookups. `Action::Reload` (`r`) calls `engine.refresh()` alongside the profile reload; the `task_active` guard prevents task storms.
- Rendering: `ui.rs::usage_section_lines(platform, provider, state, compact)` appends the section after Activity in the codex/claude detail builders. All six states (engine-not-initialized/`Idle`/`Loading`, no provider label, no matching row, `Unsupported`, `Error`, hit) render as in-section lines — never panic, never replace the page.
- Attribution is provider-level. A profile without `profile.provider` must render `no provider label — usage unattributed` and must **not** fall back to the `provider = null` bucket (it mixes all historical unattributed usage).
- Do not re-add per-selection SQL or a second load-state machine; the engine's dataset is the only source. Tests inject loaders via `UsageApp::with_loader` — production tests must not touch `~/.llmusage`. Beware: `App::on_tick` on a profile tab lazily creates the engine with the **real** loader, so tests that tick profile tabs must pre-inject an engine.

## Logging

Use `tracing::warn!` for recoverable loading failures and diagnostics. Do not print directly from TUI code during active terminal rendering.

## Testing

Prefer unit tests for state transitions, formatting, and helpers. Use temp dirs and fixture data for auth/config state; do not read real home-directory auth files.

## Verification

For TUI changes, run:

- `just fmt-check`
- `cargo test -p ccr-tui -- --test-threads=1`
- `cargo test -p ccr -- --test-threads=1` when the binary/TUI feature surface changes
- `just lint-strict`
- For usage/statistics surfaces, also run `cargo test -p ccr-usage`
