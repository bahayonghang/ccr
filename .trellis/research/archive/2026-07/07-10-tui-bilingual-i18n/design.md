# TUI Bilingual Language Switching Design

## Status

Implemented and verified. Complete-TUI coverage and the global `Ctrl+L`
interaction are confirmed; no product or scope questions remain open.

## Architecture And Boundaries

### Configuration ownership (`ccr-config`)

- Add a public `TuiLanguage` value type with stable serialized values for
  English and Simplified Chinese.
- Extend `TuiConfig` with a defaulted `language` field. Missing language values
  deserialize as English, preserving compatibility with existing `tui.toml`
  files.
- Parse an unsupported language independently from `tab_order` so an invalid
  language falls back to English without discarding a valid custom tab order.
- Add `TuiConfigManager::save` and route it through
  `ccr_core::fileio::write_toml`; TUI code must not write TOML directly.
- Continue resolving the file through `CCR_ROOT`, with `~/.ccr/tui.toml` as the
  normal default.

### Translation ownership (`ccr-tui`)

- Add one TUI-local i18n module as the source of truth for English and
  Simplified Chinese messages.
- Use stable message keys or typed message functions, with English as the
  baseline/fallback. Keep interpolation in typed formatting helpers so dynamic
  account names, paths, counts, and error details are not concatenated into
  duplicated sentence templates at call sites.
- Keep platform/product names, profile names, account names, paths, key names,
  and protocol terms untranslated unless the surrounding label is localized.
- Do not move UI text into `ccr-config`; that crate owns the persisted language
  value, not translation content.

### Runtime state and switching

- Load the full `TuiConfig` before constructing user-visible TUI state and set
  the active language to its saved value.
- Reserve `Ctrl+L` as a global language action before per-tab key delegation,
  matching the existing `Ctrl+T` theme behavior. It must work on profile and
  embedded authentication tabs.
- Switching updates the active catalog immediately, redraws the current frame,
  and saves the updated full `TuiConfig` through `TuiConfigManager` so custom
  tab order is preserved.
- A persistence error keeps the selected session language, presents a localized
  non-fatal error, and leaves the prior on-disk file intact through guarded
  writes.
- Language switching must not reconstruct `App` or embedded auth apps; current
  selection, overlays, login state, toasts, and background tasks stay intact.

### User-visible scope

The complete-TUI MVP localizes:

- tab labels, profile list/details, routing/auth and usage sections;
- all Claude Auth, Codex Auth, and OpenCode Auth views and status/toast text;
- shared confirmation/input overlays and loading/error placeholders;
- global and per-view footer labels/hints, including the `Ctrl+L` hint;
- post-terminal action summaries printed by `tui/mod.rs`.

Raw errors from lower layers may remain in their source language, but TUI-owned
context and recovery guidance around them must be localized.

## Data Flow

1. `TuiConfigManager::with_default()` resolves `<CCR_ROOT>/tui.toml`.
2. `load_or_default()` produces a complete tab order and language, defaulting
   the language to English independently of tab-order validity.
3. TUI startup initializes the active language before building labels and
   rendering the first frame.
4. Render and action code resolves TUI-owned text through the central catalog.
5. `Ctrl+L` toggles the active language, updates the in-memory `TuiConfig`, and
   calls `TuiConfigManager::save`.
6. The next launch restores the saved value.

## Compatibility And Failure Behavior

- Existing files containing only `tab_order` remain valid and gain English by
  default without requiring migration.
- A missing file keeps current startup behavior and does not create a file until
  the user changes language.
- Parse failures outside the recoverable language field retain the current
  `load_or_default` behavior and fall back to the complete default config.
- A save failure is recoverable and must not terminate the TUI.
- English is the deterministic fallback for missing catalog entries.

## Trade-offs

- A central in-code catalog adds no runtime file discovery or new localization
  dependency and keeps missing keys testable at compile/test time. It requires a
  deliberate extraction pass across the large TUI surface.
- A profile-only first slice is smaller, but produces mixed-language navigation
  as soon as the user opens an auth tab. That partial scope was rejected in
  favor of coherent complete-TUI coverage.
- Runtime-global locale access minimizes invasive parameter plumbing across
  nested render helpers. Tests must serialize mutations of the active locale or
  use an injectable/test-scoped locale guard to avoid parallel-test leakage.

## Rollback

- The configuration addition is backward compatible and optional. Reverting
  TUI use of the field leaves older binaries able to ignore the extra TOML key.
- Keep the config model/save changes and translation extraction in separable
  commits so catalog work can be reverted without risking unrelated profile
  configuration behavior.

## Task Shape

Keep this as one implementation task. Configuration, locale state, translation
catalog, and complete-surface extraction share one runtime contract, and a
partial child cannot meet the user-facing acceptance criteria independently.
Use ordered implementation stages and separate commit boundaries for
configuration infrastructure and catalog extraction instead of creating a
planning-only parent with tightly coupled children.
