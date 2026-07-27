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

## Scenario: TUI Bilingual Localization

### 1. Scope / Trigger

- Trigger: adding or changing TUI-owned labels, status/error/loading text,
  overlays, toasts, shortcut footers, or post-exit summaries.
- Applies to the main profile surface and embedded Claude Auth, Codex Auth, and
  OpenCode Auth surfaces. CLI-only output and raw lower-layer errors are outside
  the translation catalog.

### 2. Signatures

- `i18n::initialize_from_config()`
- `i18n::{active_language, set_language, toggle_language}`
- `tui_text!(english_literal, chinese_literal)`
- `tui_format!(english_format, chinese_format, args...)`
- Global key contract: `Ctrl+L` toggles `TuiLanguage` before tab-specific key
  dispatch.

### 3. Contracts

- English is the deterministic default. Simplified Chinese is the only second
  language; persisted values come from `ccr-config::TuiConfig.language`.
- Initialize language before `TerminalGuard::new()` so terminal capability
  errors use the saved language, and initialize again while constructing `App`
  before the first visible frame is built.
- `Ctrl+L` updates the active catalog immediately and saves the full loaded
  `TuiConfig`; it must not reconstruct `App` or reset tab, selection,
  pagination, auth, overlay, toast, or background-task state.
- The catalog covers all TUI-owned visible text: profile and auth views, usage
  and quota sections, overlays, loading/empty/error states, toasts, footers,
  and post-exit summaries. Dynamic identifiers, paths, provider/model names,
  and raw lower-layer errors stay unchanged inside localized context.
- Completed auth operations store a semantic `CompletedAction`, not a translated
  verb or count phrase. Post-exit summaries translate that action and format raw
  values using the language active when the terminal closes.
- Active language is thread-local to keep parallel render tests isolated.
  Background tasks must return typed data or raw lower-layer errors; they must
  not call `tui_text!` / `tui_format!` on executor threads. Localize when the
  main TUI thread renders or consumes the result.
- CJK labels use `unicode-width` padding/truncation. The compact profile footer
  must retain `PgUp/PgDn details`, `Ctrl+L language`, and the existing primary
  actions in both languages.

### 4. Validation & Error Matrix

- Missing/invalid saved language -> render English and continue startup.
- Config save succeeds -> show a success toast in the newly selected language.
- Config save fails -> keep the newly selected session language, show a
  localized non-fatal error toast, and leave the previous disk file intact.
- Missing translation -> treat as a code/test defect; English is the required
  baseline for every typed catalog message.
- Background service fails -> retain its raw error and add localized TUI context
  on the render thread.

### 5. Good/Base/Bad Cases

- Good: `Ctrl+L` on an auth tab changes tab labels, panels, footer, and new
  feedback without changing the selected account or active work.
- Good: a Chinese CJK label is truncated by display width and preserves the
  ellipsis within its column.
- Base: no `tui.toml` exists, so the first frame and terminal errors are English.
- Bad: formatting a localized quota-service error inside `spawn(async move {`;
  the executor thread does not own the TUI thread's locale.
- Bad: saving a language-only TOML document and thereby replacing custom
  `tab_order`.

### 6. Tests Required

- Catalog completeness and English/Chinese selection tests.
- `Ctrl+L` detection, persistence, save-failure, and state-preservation tests.
- Ratatui `TestBackend` assertions for Chinese compact, standard, and wide
  profile layouts plus representative auth/loading/error surfaces.
- Display-width tests for CJK truncation/padding and compact footer regression.
- Terminal capability error assertion under the Chinese active language.
- Post-exit action-label assertions in both languages; changing language after
  an action must not leave an old-language verb in the summary state.

### 7. Wrong vs Correct

#### Wrong

```rust
executor.spawn(async move {
    tx.send(crate::tui_format!("Load failed: {error}", "加载失败：{error}"))
});
```

#### Correct

```rust
executor.spawn(async move {
    tx.send(error.to_string())
});

// On the TUI thread:
let message = crate::tui_format!("Load failed: {}", "加载失败：{}", error);
```

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

## Scenario: Profile Detail Semantics And Startup Theme

### 1. Scope / Trigger

- Trigger: changing profile detail fields, their visual hierarchy, responsive
  layout, startup construction order, or the global theme toggle.
- Applies to `tui/{ui,theme,app,runtime,mod}.rs` and the persisted
  `ccr_config::TuiConfig` consumed by the TUI.

### 2. Signatures

- `theme::init_theme(configured: TuiTheme)`
- `theme::toggle_theme_and_persist() -> Result<ThemeVariant>`
- `App::with_task_executor_and_config(executor, config) -> Result<App>`
- `DetailKey`, `DetailTone`, and `DetailField` are the profile-detail
  presentation model in `ui.rs`.
- Environment override: `CCR_TUI_THEME=mocha|latte|auto`.

### 3. Contracts

- Startup loads `tui.toml` once, applies its language and theme to App
  construction, constructs the App before entering the alternate screen, then
  draws the first frame immediately.
- Mocha is the deterministic default. `mocha` and `latte` environment values
  override the persisted theme. Terminal background detection runs only for
  explicit `CCR_TUI_THEME=auto`; an unset or invalid value must not call
  `termbg`.
- `Ctrl+T` changes the active palette immediately and saves the full loaded
  config so language and custom tab order survive the theme change.
- Profile builders assign every important value an explicit `DetailTone`.
  Renderers must not infer business meaning from label/value substrings.
- Codex Engine renders `model_reasoning_effort` directly after `model`.
  Missing or blank values render `-`; known values are normalized to lowercase;
  unknown strings remain visible with warning tone; non-strings render a
  localized invalid marker. The TUI does not invent Codex's effective default.
- Focus is the sole name/current/enabled summary. Detail groups do not repeat
  those fields. Wide profile layout is list 46% / detail 54%, and the 3-row
  Status strip exists only while apply/toast feedback is visible.
- Detail label widths are derived from localized display width and clamped per
  viewport. Token values pass through existing masking before they become a
  `DetailField`.

### 4. Validation & Error Matrix

- Missing/invalid TUI config -> continue with default preferences.
- Invalid `CCR_TUI_THEME` -> warn and use the persisted theme without probing.
- `CCR_TUI_THEME=auto` probe failure -> use the persisted theme.
- Theme save failure -> keep the new palette for the session, log a warning,
  and leave the previous guarded config file intact.
- Missing/blank reasoning effort -> muted `-`; unknown string -> raw normalized
  value with warning; non-string -> localized invalid marker with warning.
- Profile/runtime loading failure -> keep the recoverable in-TUI issue state;
  startup reordering must not turn it into a panic.

### 5. Good/Base/Bad Cases

- Good: a Codex profile with `model_reasoning_effort = "HIGH"` shows `high`
  beside the model with an emphasized Codex tone.
- Good: a 140x30 wide page gives the detail rail more width and omits an empty
  Status strip; 80x20 and 100x30 retain compact/standard behavior.
- Base: no theme env/config exists, so Mocha is selected without terminal I/O.
- Bad: calling `termbg` whenever `CCR_TUI_THEME` is unset adds a fixed
  approximately 100ms wait before the first frame.
- Bad: styling fields with `label.contains("model")` makes new keys and
  localized labels silently lose semantic hierarchy.

### 6. Tests Required

- Theme resolution tests assert that persisted Mocha/Latte avoids the detector,
  explicit overrides win, and only `auto` invokes the detector.
- Persistence tests assert `Ctrl+T`-equivalent saving preserves language and
  custom tab order.
- Reasoning tests cover missing, blank, uppercase known values, every supported
  level, unknown strings, and non-string values without exposing secrets.
- Style tests assert model/effort/provider/auth/token/cost tones explicitly.
- Ratatui `TestBackend` tests cover English and Chinese at 80x20, 100x30, and
  140x30, including Focus de-duplication, dynamic Status, and 46/54 wide layout.
- Run `cargo test -p ccr-config`, `cargo test -p ccr-tui`, `cargo test -p ccr`,
  `just fmt-check`, and `just lint-strict`.

### 7. Wrong vs Correct

#### Wrong

```rust
let variant = detect_terminal_variant().unwrap_or(ThemeVariant::Mocha);
let style = detail_value_style(label, value);
```

#### Correct

```rust
let variant = resolve_startup_variant(env_value, config.theme, detect_terminal_variant);
let field = DetailField::new(DetailKey::Model, value, DetailTone::Accent {
    platform: Platform::Codex,
    strong: true,
});
```

## Logging

Use `tracing::warn!` for recoverable loading failures and diagnostics. Do not print directly from TUI code during active terminal rendering.

## Testing

Prefer unit tests for state transitions, formatting, and helpers. Use temp dirs and fixture data for auth/config state; do not read real home-directory auth files.

Theme style tests must not call `set_theme` or `toggle_theme`: `ACTIVE` is
process-global, so one parallel test can change the palette between another
test's style construction and assertion. Keep palette-dependent construction
behind pure helpers and pass `&MOCHA` / `&LATTE` explicitly in tests.

```rust
// Wrong: races with every test that reads the active palette.
set_theme(ThemeVariant::Mocha);
assert_eq!(background_style().bg, Some(MOCHA.bg));

// Correct: the production wrapper still reads palette(), while the test is pure.
assert_eq!(background_style_for_palette(&MOCHA).bg, Some(MOCHA.bg));
```

## Verification

For TUI changes, run:

- `just fmt-check`
- `cargo test -p ccr-tui -- --test-threads=1`
- `cargo test -p ccr -- --test-threads=1` when the binary/TUI feature surface changes
- `just lint-strict`
- For usage/statistics surfaces, also run `cargo test -p ccr-usage`
