# GPUI 05 local settings and accounts evidence

Date: 2026-11-05. This records the desktop routes that now show real local settings and account state. It does not mark PRD 5.1–5.6 passed. No OAuth, device login, logout, or production write was run. Paper stays the existing solid sheet. The sidebar stays 13 routes.

## What the routes show

Claude Code, Codex, and Grok still open on the profile catalog. Settings and accounts is the existing in-page tool. Antigravity and OpenCode open directly on their settings reader. The page shows only fields the reader allows, and it does not treat a missing file, an unread credential store, or file presence as login success.

The live projection below comes from `ConfigSource::Local` through `runtime::platforms::read`, with `CCR_DESKTOP_LOCAL_PLATFORM_PROBE=1`. Output contained field names, account names, and provider ids only.

| Route | This machine |
| --- | --- |
| `/claude-code` | Catalog stays the primary page. Settings and accounts: `settings.json` found. model `opus`, theme `dark-daltonized`, autoUpdatesChannel `latest`. No runtime credential detected. No saved accounts. |
| `/codex` | Catalog stays the primary page. Settings and accounts: `config.toml` found. model_reasoning_effort `high`. Credential store is not the file store, so no login state is shown. Saved account `khan` is listed and is not current. |
| `/grok` | Catalog stays the primary page. Settings and accounts: `config.toml` found. default_model `grok-4.7-build-fast`, reasoning_effort `xhigh`, channel `alpha`, auto_update `true`. Runtime auth file is present. The page says that is not a remote login check. No saved accounts. `signed_in` stays false. |
| `/opencode` | `opencode.json` found. model `magpie/cursor/grok-4.7@500k`. Provider id `magpie`. Accounts are not supported, and login state is not reported. The TUI tab finds `tui.json` and has no supported theme or mouse value. |
| `/antigravity` | Legacy `settings.json` found. Typed settings are not supported, so values stay hidden. Accounts are not supported. There is no save control. |

Fixture coverage, not this machine: a missing file stays missing and is not created; an invalid document shows no values; secret-like values and provider secrets are omitted; a Codex auto store does not claim login; a Grok demo root does not query the machine account store.

## Commands

| Command | Exit |
| --- | --- |
| `just desktop-fmt-check` | 0 |
| `just desktop-lint` | 0 |
| `just desktop-test` (first run on this tree) | 101 |
| `just desktop-test` (one rerun) | 0 |

The first `just desktop-test` failed only in `features::insights::tests::rendered_usage_controls_read_fixture_filters_and_keep_exact_costs`, on an assertion that two SQLite byte vectors were equal. The test was not changed and usage storage was not edited. The single rerun passed, including that test. Python unit tests were 12 passed, and the AccessKit harness was 3 passed, on both runs.

Focused route evidence before the full gate: `platform_routes_show_supported_local_status_without_secrets` passed. It keeps the Claude catalog on the primary page, then checks settings and account copy for all five routes without the fixture secret.

## Checker repeat

1. Run `just desktop-fmt-check`, `just desktop-lint`, and `just desktop-test` with default parallelism and `--skip export_bindings`. If the usage byte comparison fails once, record it and rerun `just desktop-test` once. Do not change that test or the usage database.
2. Open the five routes in a fixture window and confirm the catalog remains primary for Claude, Codex, and Grok, Antigravity has no save control, and no route shows a fixture secret, “Home insights”, or “Apply / refresh”.
3. Optional live read: set `CCR_DESKTOP_LOCAL_PLATFORM_PROBE=1` and run `python scripts/desktop.py cargo test --locked --lib opt_in_local_projection_prints_safe_fields_only -- --nocapture --skip export_bindings` from `ccr-desktop`. Confirm the output has no credential material. Do not start OAuth, logout, or any production write.
4. Leave PRD 5.1–5.6 open. Official logout, real OAuth success, provider mutation parity, glass, other operating systems, IME/UIA, and performance remain separate.

## Checker result (2026-11-05)

PRD 5.1–5.6 stay open. This pass did not start OAuth, logout, or a production write, and it did not set `CCR_DESKTOP_LOCAL_PLATFORM_PROBE`.

### Fixes

- File, auth, field, account, and provider facts are visible text. They had been accessibility labels only.
- Claude and Codex local reads keep `signed_in` false. Credential detection stays on the credential state.
- Antigravity patch and source writes return unsupported and leave the legacy file unchanged.
- The opt-in probe prints file state, credential state, counts, and field keys. It does not print field values, account names, or provider ids.

### Routes

Claude Code, Codex, and Grok still open on their profile catalogs. Their settings tool reads the existing settings file and auth snapshot. OpenCode reads `opencode.json`, with the legacy `config.json` fallback, and lists provider ids. Antigravity reads legacy `settings.json`, shows no values, and has no save control. Unsupported auth does not say the user is logged in. The paper stays the solid sheet. The sidebar stays 13 routes.

### Commands observed by this checker

| Command | Exit |
| --- | --- |
| `just desktop-fmt-check` | 0 |
| `just desktop-lint` | 0 |
| `just desktop-test` | 0 |

`just desktop-test` passed on the first checker run, including `rendered_usage_controls_read_fixture_filters_and_keep_exact_costs`. No rerun was required. Python unit tests were 12 passed and the AccessKit harness was 3 passed. Default parallelism and `--skip export_bindings` were unchanged.
