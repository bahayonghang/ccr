# Design: Grok Profile Parse Diagnostics

## Scope And Ownership

- `ccr-config` remains the single owner of profile TOML parsing and file-load diagnostics.
- `ccr-cli` owns Grok field validation, CLI CRUD/JSON exposure, runtime mapping, drift comparison, and entry-state restoration.
- `ccr-tui` only lays out the shared error and removes a path already represented by `Where`; it does not parse TOML.
- `ccr-tui` also displays the Grok `reasoning_effort` beside the model using its existing bilingual detail label.
- The live system `profiles.toml` is diagnostic input only and is not modified.

## Parser Flow

1. Parse the document into a generic TOML table. Failure here is a TOML syntax error.
2. Select the documented profile shape from top-level metadata: scalar `default_config` or `current_config` marks the full `CcsConfig` format; otherwise preserve the legacy simplified map path.
3. Deserialize exactly the selected typed shape. Failure here is a profile structure/type error; do not retry the other shape and hide the original location.
4. Convert successful `ConfigSection` values to `ProfileConfig` exactly as today.

This preserves empty-document and simplified-map compatibility while preventing a typed error in a clearly full document from being replaced by an unrelated fallback error.

## Safe Diagnostic Contract

Add a small private formatter in `crates/ccr-config/src/platforms/base.rs`:

- Use `toml::de::Error::span()` plus the original string to compute 1-based line and Unicode-aware column.
- Use `Error::message()`, never `Display`, so the TOML source line is not included.
- Distinguish `TOML 语法错误` from `profile 结构错误`.
- Preserve safe field/expected-type information. Redact the rejected literal from semantic messages such as `unknown variant` or `invalid value`.
- When a message cannot be safely reduced, report the category and location without the raw value.
- When no span exists, omit the position rather than inventing one.

`load_profiles_from_toml` unwraps the inner `ConfigFormatInvalid` message and prefixes the file path once. It must not stringify an already formatted `CcrError` into another instance.

## TUI Presentation

Change `format_issue` to render stable blocks:

```text
Where:
  <path>

What:
  <reason>
```

Before rendering `What`, remove one leading `<path>: ` fragment from the lower-layer message when it matches the `Where` path. Preserve the error category once. This prevents long Windows paths from visually joining the next label and avoids showing the same path twice.

## Reasoning-Effort Contract

### Profile Representation

- Store `reasoning_effort` as a string in `ProfileConfig.platform_data`, matching other Grok-specific fields.
- Add it to the Grok editable-field allowlist, `create --reasoning-effort`, generic `set-field` handling, safe `current/list --json` summaries, and TUI details.
- Trim the value and reject empty/non-string data. Accept only Grok Build's `ReasoningEffort` enum values (`none`, `minimal`, `low`, `medium`, `high`, `xhigh`, `max`) and normalize them to lowercase. Model-menu option ids are presentation/input labels whose separate canonical `value` remains one of these enum values, so an id must not be written as `default_reasoning_effort`.

### Runtime Mapping

For a third-party profile with a value:

1. Set `model.custom.supports_reasoning_effort = true`.
2. Set `model.custom.reasoning_effort` to the profile value.
3. Set `models.default_reasoning_effort` to the same value.

For an official profile, only step 3 applies; built-in model capability remains owned by Grok.

If the profile omits the field, restore the entry `models.default_reasoning_effort` before completing the apply. Replacing/restoring the complete `model.custom` table already prevents custom-model reasoning fields from leaking across profiles.

### Entry-State Compatibility

- Extend `ProfileEntryConfigState` with `original_default_reasoning_effort: Option<toml::Value>` and `#[serde(default)]` so entry-state files created by CCR 7.0.1 still deserialize. When that field is absent, recover the original value from the legacy state's captured `content` before restoring.
- Capture and restore the exact TOML value. This avoids destroying a pre-existing value even if a future Grok release broadens its representation.
- Add a focused helper that restores or removes only `models.default_reasoning_effort`, preserving all unrelated `[models]` keys.
- `runtime_matches_profile` already rebuilds expected runtime through `apply_profile_to_config`; the new fields therefore become part of drift detection without a parallel comparison path.

### Examples And Documentation

- Add `reasoning_effort = "high"` to the canonical and mirrored Grok relay examples.
- Add `--reasoning-effort high` to CLI examples and document `set-field`/JSON behavior.
- Update the executable Grok runtime spec to declare the new managed keys and restoration rules.

## Compatibility And Risk

- No `CcrError` variants change. The shared profile-create argument gains an optional field passed as `None` by non-Grok callers.
- Valid full and simplified profiles retain their current output.
- Ambiguous documents without scalar full-format metadata keep the legacy simplified interpretation.
- Error wording changes intentionally; tests should assert stable semantic fragments rather than the complete parser-library sentence.
- The main risk is accidental secret disclosure from error formatting. Sentinel tests cover malformed syntax and typed-field failures.
- Old entry-state JSON remains readable through a defaulted optional field. The main runtime risk is stale global effort leaking across profile switches; restoration tests cover present and absent entry defaults.

## Rollback

Reverting the implementation commit restores prior behavior; there is no eager data migration or user-file mutation. Runtime changes occur only when the user later invokes a profile switch, and `off` restores the captured entry state.
