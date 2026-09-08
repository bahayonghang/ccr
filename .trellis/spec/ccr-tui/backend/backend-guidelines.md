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

## Scenario: Profile Load Issue Presentation

### 1. Scope / Trigger

- Trigger: displaying profile registry or current-profile load failures in the
  TUI.
- Applies to `format_issue`, `profile_source_path`,
  `current_profile_source_path`, and profile-tab error rendering.

### 2. Signatures

- `format_issue(location: String, error: &dyn Display) -> String`
- Shared profile parsing remains owned by `ccr-config`; the TUI receives an
  already classified `CcrError`.

### 3. Contracts

- Render stable blocks as `Where:\n  <location>\n\nWhat:\n  <reason>`.
- Indent every continuation line by two spaces so a long Windows path or a
  fallback path cannot visually merge with `What`.
- When the lower-layer error already contains the exact `Where` location,
  remove one leading duplicate while preserving the error category and reason.
- Do not parse TOML, infer fields, or expose source text in `ccr-tui`.

### 4. Validation & Error Matrix

- Single profile path + classified parse error -> one path and one reason.
- Registry path plus fallback profile path -> both remain under `Where`, each
  indented, with `What` starting after a blank line.
- Error with one leading duplicate path -> strip only that matching prefix.
- Non-matching path text inside the reason -> preserve it; do not perform broad
  replacement.

### 5. Good/Base/Bad Cases

- Good: a long Grok `profiles.toml` path wraps inside `Where` while the profile
  structure error remains separately scannable under `What`.
- Base: a short config error uses the same block layout.
- Bad: `Where: <path>\nWhat: <path>: <nested error>` duplicates the path and
  lets terminal wrapping join the two labels visually.

### 6. Tests Required

- Assert the exact `Where` / blank-line / `What` block boundaries.
- Assert a duplicated long Windows path occurs once and the actionable parser
  reason survives.
- Assert multiline fallback locations indent every continuation line.
- Run `cargo test -p ccr-tui -- --test-threads=1` and strict Clippy.

### 7. Wrong vs Correct

#### Wrong

```rust
format!("Where: {location}\nWhat: {error}")
```

#### Correct

```rust
let reason = strip_duplicate_issue_location(&error.to_string(), &location);
format!(
    "Where:\n  {}\n\nWhat:\n  {}",
    indent_issue_value(&location),
    indent_issue_value(&reason),
)
```

## Interaction Rules

Maintain stable tab/profile selection across refreshes where possible. Use explicit cached `Rect` fields for mouse hit-testing, as `App` does with `header_area`, `list_area`, and `detail_area`.

Preserve pagination helpers and page-size behavior when changing list rendering.

### Text truncation, padding, and shortcut hints

Truncate and pad cell text by terminal display width via `unicode-width`, never by `chars().count()` — CJK characters render 2 columns wide, so char-counted cells overflow their column and ratatui hard-clips them, losing the `…` marker. Follow the shared helper shape (`truncate_text`/`pad_text` in `tui/ui.rs`, same-named helpers in auth sub-apps and the usage view): accumulate per-char width, reserve 1 column for `…`, and prefer ending 1 column short over overflowing.

Keyboard shortcut hints live only in the global Keys footer. Panels and status strips carry state (selection, apply/toast feedback), not key legends — do not reintroduce per-panel shortcut lines.

### Keybindings must not rely on bare modifier combos

TUI keybindings must not depend on the terminal reporting bare modifier
combinations (e.g. Shift+Enter). `runtime.rs::setup_terminal` does not push
keyboard-enhancement flags, so on Unix terminals without the kitty keyboard
protocol such a combo arrives as the unmodified key and is indistinguishable
from it. When a modifier binding is needed, a Unix escape-sequence path must
exist too — `Shift+Tab`'s `BackTab` (Unix) + `Tab` with the `SHIFT` modifier
(Windows) dual path in `app.rs` is the reference implementation.

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
  Grok Auth surfaces. CLI-only output and raw lower-layer errors are outside
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
- Only the retired Usage identifier retains parse-tolerant compatibility. Removed `opencode_auth` is unknown and causes the existing whole-config default fallback; initial loading does not rewrite the file. Do not add a migration layer.

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
- Focus is the sole name/current/enabled summary and also carries the
  last-apply result ("Switched to X" on success, error detail on failure) as a
  persistent field. Detail groups do not repeat those fields. Wide profile
  layout is list 46% / detail 54%, and the 3-row Status strip is toast-only
  (apply results no longer repeat there).
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

## Scenario: Codex Auth Quota And Local Usage

The main Codex Auth tab and `run_codex_auth_tui()` both reach the main `App`
and `codex_auth::ui::draw_embedded`. Exercise that composed path when testing
CLI auth entry behavior; the legacy `codex_auth::ui::draw` is not its substitute.

### Quota presentation

- Read selected-account quota through `CodexAuthApp::selected_quota()`. A valid
  preview snapshot can exist while `QuotaState` is `Idle`; render the snapshot
  before adding refresh/error status, rather than replacing it with idle help.
- `hourly_percentage` and `weekly_percentage` are **remaining** percentages.
  Render a numeric value and a colored bar from the same value, using the active
  theme's quota palette. Keep unfilled cells muted and clamp bar geometry at
  the presentation boundary.
- `window_present = Some(true)` permits a bar, `Some(false)` means not provided,
  and `None` means unknown. Missing/unknown windows must not appear full or empty.
  Account-list previews and selected-account details must agree.
- Keep cached values visible during refresh, and retain a visible failure when
  refresh fails with cached data. Display `fetched_at` as quota acquisition time;
  auth `last_refresh` is a separate timestamp. Do not invent another cache TTL.
- Enforce that retention at the existing background-message cache write boundary,
  including batch preview errors, failed quota snapshots carried by `Ok`, and
  outer task errors. A later valid batch result clears a stale error only for
  the same account. A manually constructed `QuotaState::Error` render fixture
  alone does not test this message path.

### Local usage scope and severity

`CodexUsageAttributionState` owns scope-note severity. An
`AccountAttributed` result is a successful filtered aggregate, so its ordinary
coverage note is muted/informational. `VirtualAccount` and
`UnattributedFallback` retain a warning and explicitly identify the displayed
numbers as global local usage. Load failures retain error styling.

Do not infer missing or corrupt history merely because the global count exceeds
the selected account's count: other accounts also explain that difference.
Do not change `records_for_account` or mix global records into account totals to
remove a warning. CCR activation-window attribution is local history, not an
official account bill or a conversion from tokens to server-side quota.

### Space budget and regression evidence

- Size auth content from the actual `content_area`, including the space already
  consumed by the main header, runtime banner, and footer. `ViewportMode::Wide`
  alone does not imply enough height for two right-side panels.
- Prefer quota, scope, core statistics, and real errors over secondary metadata.
  Never show global fallback numbers without their scope. Omitted/truncated
  content must be apparent; labels, numbers, and ellipses use terminal width.
- Cover English/Chinese at 80×24, 100×22, 100×30, 120×22, 140×40, and 180×50;
  additionally verify graceful degradation below those sizes. Assert buffer
  cells and colors, not only that rendering succeeds.
- Include cached `Idle`, cached refresh failure, all three window-presence
  states, low/full remaining percentages, successful attribution with another
  account's records, true global fallback, and load-error fixtures. Use isolated
  directories and synthetic records; do not read personal auth or usage data.
- TestBackend evidence proves buffer composition; native terminal appearance and
  private runtime correctness require separate direct evidence.

## Logging

Use `tracing::warn!` for recoverable loading failures and diagnostics. Do not print directly from TUI code during active terminal rendering.

## Scenario: Grok Profile Tab

### 1. Scope / Trigger

- Trigger: changing Grok profile discovery, tab ordering, profile details, or
  apply behavior in the Ratatui application.
- Applies to `tui/app.rs`, `tui/ui.rs`, the `TuiTabId::GrokProfile` preference,
  and the Grok helpers exposed by `ccr-cli`.

### 2. Signatures

- `TuiTabId::GrokProfile` serializes as `grok_profile`.
- `GrokPlatform::safe_base_url_for_display(&str) -> String` owns display URL
  sanitization.
- `GrokPlatform::profile_auth_mode(&ProfileConfig) -> Result<GrokProfileAuthMode>`
  owns credential-source classification.
- `PlatformConfig::apply_profile(&str)` remains the shared apply entry point.

### 3. Contracts

- Grok contributes exactly one `TabVariant::Profile` alongside its independent `GrokAuth` tab. The profile view has no
  Claude runtime summary, Codex runtime summary, or embedded usage section.
- Its full and compact labels are `Grok Profile` / `Grok 配置` and `Grok`.
- Details show description, sanitized base URL, model, API backend, auth mode,
  env-key name when applicable, context window, backend-search support, switch
  count, and tags.
- Details never render an inline token, including a masked token or any token
  length signal. URL and auth semantics must call the Grok helpers above rather
  than being reimplemented in the TUI.
- Enter/Space use the existing profile apply path (apply and stay; quit via
  `q` / `Esc`), toast reporting, reload, current marker, and per-tab selection
  behavior without a Grok-only mutation path.

### 4. Validation & Error Matrix

- No Grok profiles -> render the existing profile empty state.
- Profile or current-marker load failure -> render the existing `Where` / `What`
  issue surface and keep the TUI running.
- Conflicting or invalid auth fields -> render a localized invalid value without
  exposing the underlying credential.
- Apply validation, CAS, or I/O failure -> show the existing localized error
  toast; do not change the current marker silently.

### 5. Good/Base/Bad Cases

- Good: an env-key profile shows `env_key (XAI_API_KEY)` and a URL without
  userinfo, query, or fragment.
- Base: a session profile with no explicit backend shows
  `responses (default)` and no token row.
- Bad: rendering `config.auth_token` through `mask_sensitive`; even the masked
  shape leaks information and violates the Grok detail contract.

### 6. Tests Required

- Assert the Grok tab's config id, English/Chinese labels, default placement,
  custom ordering, and empty-profile construction.
- Assert env-key, inline-key, and session detail modes; URL sanitization; typed
  Grok fields; and absence of plaintext and masked token output.
- Run `cargo test -p ccr-config tui_config -- --test-threads=1`,
  `cargo test -p ccr-tui -- --test-threads=1`, `just fmt-check`, and
  `just lint-strict`.

### 7. Wrong vs Correct

#### Wrong

```rust
let url = config.base_url.clone();
let token = config.auth_token.as_ref().map(mask_sensitive);
```

#### Correct

```rust
let url = config
    .base_url
    .as_deref()
    .map(GrokPlatform::safe_base_url_for_display);
let auth_mode = GrokPlatform::profile_auth_mode(config);
// No token field is constructed for Grok details.
```

## Testing

### Grok Auth accounts

- Consume only the secret-free `GrokAuthService` snapshot. Selection is independent
  of each scope's local match; local metadata never proves effective authentication.
- Save copies a selected OAuth source to CCR while Grok may continue running.
  Multiple sources require selection; an existing alias requires explicit overwrite.
- Switch confirmation requires stopping Grok first and states that local credentials
  serve new sessions; it preserves the profile route and MCP. Read activation only
  through `GrokPlatform::inspect_activation_state`.
- Delete removes only a saved item; logout explicitly removes all runtime auth.json
  credentials, preserving the account library. Confirmations default to cancellation;
  Enter/n/Esc cancel and only y submits. Do not reuse a delete-only modal title.
- Freeze the revision, scope and alias in the request. Use the shared executor's
  blocking task path and one result channel; disable duplicate operations and normal
  navigation/exit until the result is collected. Ctrl+L and resize remain responsive.
  Cancel unsubmitted dialogs on tab navigation. Never describe an in-flight action as
  canceled. A disabled executor submits nothing; a disconnected channel reports an
  unknown outcome and rereads once without replaying the mutation.
- Keep the previous snapshot on read errors and show stale state. Keep mutation
  success/failure separate from subsequent read failure. Store semantic actions for
  summaries and translate on the TUI thread. No raw credential JSON enters UI state.
- Render list/details side by side on wide screens, stacked on standard screens,
  and prioritize selection and feedback on compact screens. Keep necessary action
  hints and exit visible at 40×12; truncate identifiers by display width.
- Tests use an injected backend and TestBackend, never personal credentials. The
  September 8 implementation delivery explicitly skips execution at user request;
  static review and maintained test sources do not establish runtime acceptance.

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
