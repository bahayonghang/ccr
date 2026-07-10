# Add TUI bilingual language switching

## Goal

Add a persistent Chinese/English language switching system to the CCR TUI so
users can choose the interface language while retaining English as the default
experience.

## Requirements

- The TUI supports English and Simplified Chinese interface text.
- English is used when no language preference has been saved.
- A user can switch the TUI language through an in-product TUI interaction.
- The selected language is persisted in `<CCR_ROOT>/tui.toml` (normally
  `~/.ccr/tui.toml`) and restored on later TUI launches.
- Existing CCR configuration safety behavior must be preserved, including any
  established guarded/atomic-write and compatibility conventions.
- The feature is limited to the Rust TUI unless repository evidence shows that
  shared configuration contracts require a narrowly scoped cross-crate change.
- The MVP covers the complete TUI-owned surface: profile screens, Claude Auth,
  Codex Auth, OpenCode Auth, shared overlays, usage/status content, footer and
  loading/error text, and post-exit action summaries.
- `Ctrl+L` switches languages immediately from every TUI tab and persists the
  selection; the MVP does not add a dedicated settings screen.
- Keyboard shortcuts remain stable across languages and use Latin keys.
- Missing translations and unsupported saved language values fall back to
  English without preventing TUI startup.

## Acceptance Criteria

- [x] A fresh TUI launch with no saved language preference renders in English.
- [x] The user can switch the visible TUI interface between English and
  Simplified Chinese immediately, from any TUI tab, without manually editing
  configuration files.
- [x] The selected language survives process restart through configuration
  stored in `<CCR_ROOT>/tui.toml`.
- [x] Existing configurations that do not contain a language preference remain
  valid, preserve their configured tab order, and default to English.
- [x] An unsupported or malformed language preference falls back to English
  without discarding an otherwise valid tab order.
- [x] All TUI-owned user-visible strings across profile screens, Claude Auth,
  Codex Auth, OpenCode Auth, shared overlays, usage/status content, footers,
  placeholders, toasts, and post-exit summaries are available in both
  languages, with English fallback for missing translations.
- [x] Switching language does not alter selected tabs/profiles, pagination,
  active background work, or authentication state.
- [x] Relevant configuration and TUI behavior is covered by automated tests.
- [x] The narrow TUI/configuration verification gates pass.

## Out Of Scope

- Localization of `ccr-ui`, `ccr-vscode`, documentation, or CLI-only command
  output unless a shared string is also rendered inside the TUI.
- Languages other than English and Simplified Chinese.

## Open Questions

- None.

## Product Decisions

- The MVP must cover the complete TUI surface rather than shipping a
  profile-only partial translation.
- `Ctrl+L` is the global immediate language switch and writes the selected
  language to `tui.toml`; no dedicated settings screen is included in the MVP.

## Confirmed Repository Facts

- `ccr-config::TuiConfigManager` already resolves `<CCR_ROOT>/tui.toml`, loads
  missing/invalid files without blocking startup, and is the required owner of
  TUI preference persistence.
- `ccr-core::fileio::write_toml` provides the established lock, temporary-file,
  fsync, and atomic-replacement write path.
- The TUI has no settings screen. Its runtime already reserves `Ctrl+T` as a
  global immediate theme switch, so `Ctrl+L` is the proposed parallel language
  interaction.
- No reusable Rust i18n layer exists in the workspace. User-visible text is
  spread across the main profile UI, three embedded authentication apps,
  shared overlays, usage/status rendering, and post-exit output.
