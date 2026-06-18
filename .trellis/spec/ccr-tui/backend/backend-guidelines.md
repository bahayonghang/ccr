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
