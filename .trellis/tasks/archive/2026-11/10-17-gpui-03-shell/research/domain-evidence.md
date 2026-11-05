# 03 command favorites/history domain implementation

Date: 2026-10-18. Windows, repository Rust 1.98.0, dev base 1dfe776b.
Implementer ownership: ccr-db UiStateManager, desktop runtime/ui_state.rs,
runtime/mod.rs factory cache and desktop manifest/lock/dependency declaration.
No UI/product shell files, root/Tauri manifests/locks, native operations,
real user configuration, global HOME, commits or publishing were changed here.

## Change boundary and behavior

The actual missing seam was global-only UiStateManager with empty-on-query-error
reads. The existing database owner now accepts DbAccess, and additive
try_get_favorites / try_get_history preserve DbError for native reads. Old
new/get_favorites/get_history return values and Tauri behavior remain compatible.
All SQL, schema/migrations, UUIDs, duplicate lookup, date/order/default history
limit, command construction and persistence remain in existing domain methods.

UiStateRequests::new(&Services) only captures source/cache/runtime. Actual
initialization, migrations and queries execute under task 02's bounded blocking
runtime. Services clones share one lazy manager/pool; production reuses the
registered pool or initializes the existing domain app pool once. A fixture
uses registry.parent()/ui-state.db and an explicit empty
ui-state-migration-home child; nonempty migration homes fail safely. Fixture
tests do not initialize GLOBAL_POOL, resolve production home, or modify HOME.
Failed initialization is retryable. Remote requests are rejected before I/O;
stored records are desktop-local/global and are not attributed to platform or
remote environment IDs.

The native projection omits raw args and full_command entirely. Other visible
text passes existing ccr_core log_redact; source error payloads do not enter
UiStateError (Storage / UnsupportedEnvironment). No new regex/security owner.
These command/display/module labels are user-provided visible metadata, not
credential inputs. Application code must never copy actual credential payloads
into them. The existing redactor recognizes sensitive JSON keys, Bearer, sk/JWT
and cookie formats; it does not promise removal of arbitrary free-text secrets
or `token=value` labels. This inherited limit is explicit, not waived by a test.
Raw args/full_command are excluded independently of pattern recognition.
Two existing mutation log payloads now use stable record IDs rather than raw
favorite.command / history.full_command, preserving stored content and results.
No existing tracing subscriber capture seam/dependency exists in ccr-db;
this narrow log change was source reviewed, with no invented capture result.
Typed add/remove/clear operations use app-owned jobs. Cancellation is checked
before invoking mutation; a committed success remains Success if cancellation
arrives before the runner returns. Reads use existing cancellation semantics.
History insertion remains the existing domain API and is reserved for actual
completed jobs in 08; this task adds no fake execution/history action.

## Actual verification

All Rust behavior tests use default parallelism and --skip export_bindings.

| Command | Exit | Result |
| --- | --- | --- |
| `cargo test -p ccr-db --locked -- --skip export_bindings` (sandbox attempt) | 1 | mbx external cache access denied before build; no product failure inferred |
| Same command with authorized existing cache access | 0 | 127 passed, doc-tests 0; private file migrations/reopen/isolation, duplicate/order/defaults/args/removal/history limit/clear and error-vs-empty covered |
| `cargo clippy -p ccr-db --locked --all-targets -- -D warnings` | 0 | Strict domain lint passed |
| `cargo test -p ccr-db --locked managers::ui_state -- --skip export_bindings` after strengthening history defaults | 0 | 2 passed; explicit default 50 / requested 75 / zero limit assertions |
| `python scripts/desktop.py cargo check --offline --all-targets` (cwd ccr-desktop; first additive lock resolution) | 0 | Domain/native adapter API compiled before shell owner's concurrent rewrite |
| `python scripts/desktop.py cargo metadata --locked --no-deps --format-version 1` | 0 | Locked workspace metadata emitted to outputs/desktop/shell-metadata.json |
| `python scripts/desktop.py cargo check --locked --all-targets` during shell rewrite | 1 | Owner in-flight app.rs lacked locales/shell lib wiring, Focusable import and exact a11y callback signature; diagnostics delivered to shell owner, no edits to their files |
| `python scripts/desktop.py cargo test --offline runtime::ui_state -- --skip export_bindings` during same rewrite | 1 | Same concurrent shell compilation failures; adapter tests did not run, not counted pass |
| `rustfmt --edition 2024 --check` scoped to owned Rust files | 0 | Validation only; explicit rustfmt repair was separate |
| `git diff --check` | 0 | Existing CRLF notices only |
| `cargo clippy -p ccr-db --locked --all-targets -- -D warnings` after ID-only logging | 0 | Strict domain lint passed again |
| `python scripts/desktop.py cargo test --locked runtime::ui_state -- --skip export_bindings` after shell compile handoff | 1 | 1 passed / 1 failed: test expected unsupported `token=value` free-text redaction; initial sentinel assumption was incorrect, actual shared behavior recorded above |
| Same locked focused test after supported Bearer sentinel correction | 0 | 2 passed, 3 other lib tests filtered; bin 0, native_ui 2 filtered; constructor/no-global-pool, storage retry, safe supported redaction, remote rejection, actual commit then latecancel all passed |

Root test/clippy required authorized external mbx cache access. A helper command
initially referenced scripts/desktop.py from root (exit 1 missing path), then used
the real ccr-desktop/scripts/desktop.py from the correct workspace. A rustfmt
attempt from the child cwd similarly used root-relative paths and reported
missing files; the subsequent correct root command succeeded. Neither failure
modified unrelated sources.

Lock resolution initially downgraded seven existing optional platform packages;
precise restores preserved their prior versions. Two update attempts reported
ambiguous package IDs (font-types and redox_users, exit 101); version-qualified
IDs fixed them. Updating redox_users re-downgraded libredox, which was restored
last. Final versions are documented in dependencies.md. No permanent resolver
override; no new external runtime dependency declared directly. New direct local
ccr-db/ccr-core are 7.5.0/MSRV1.95, defaults; dev-only tempfile =3.27.0 reuses
today's verified official registry release/MSRV1.63. Root/Tauri Cargo locks were
not changed by this owner.

## Handoff and pending integration verification

Focused adapter tests passed after shell owner's stable compile handoff. Final
full desktop tests/lint/build are coordinated by shell owner/main and independent
check; the earlier in-flight failures above are not erased. Owned source files
and manifest/lock are frozen for review. Final Cargo.lock SHA256:
`9c8f5b816b01a2db2285d0092fa223e791ffe3f48f02657ec4edad1708c7216f`.
Stable-ID-only logging is at ui_state.rs lines 85 and 143 at this handoff.
No native claim is made by this implementer. Main owns Computer Use and independent review.
User deferred resize/DPI validation; none was run here. No other OS/performance
or full migration acceptance is implied.
