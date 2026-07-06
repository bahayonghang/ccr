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

Use a synthetic `PlatformTab` only when a tab is not a profile/auth surface but still needs to live in the configured tab bar, such as `TabVariant::Usage`.

Contract:

- Add a matching `TuiTabId` in `crates/ccr-config` and include it in the complete-list default order.
- Give the synthetic tab an empty `profiles` list and route it before profile selection/apply behavior in `handle_key`, mouse handlers, activation, ticks, and `ui::draw`.
- Lazily initialize the embedded app from `App::with_task_executor(...)`; load external data with `AsyncTaskExecutor::spawn_blocking()` and a message channel so the terminal render loop never blocks on filesystem or SQLite work.
- Keep the tab read-only unless the PRD explicitly asks for mutations. For usage/statistics views, show unsupported, missing-data, empty, and query-error states inside the tab rather than panicking or falling back to profile UI.

Wrong:

```rust
// Synthetic tab reaches profile apply/select handling.
let action = self.map_key(key);
self.handle_profile_action(action)
```

Correct:

```rust
if self.is_usage_tab() {
    if let Some(usage_app) = self.usage_app_mut() {
        return usage_app.handle_key(key);
    }
    return Ok(false);
}
```

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
- For usage/statistics tabs, also run `cargo test -p ccr-usage`
