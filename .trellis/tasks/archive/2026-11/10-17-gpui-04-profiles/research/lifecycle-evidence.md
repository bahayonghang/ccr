# 04 lifecycle owner implementation evidence

Date: 2026-10-18. Role: trellis-implement, domain/core ownership only. Shared workspace changes from 01–03 and other 04 workers retained. No GUI launch, user configuration/account mutation, HOME/USERPROFILE mutation, commit, push, packaging, signature or task archival.

## Boundary and APIs

- Existing `ccr-cli/application/profile_lifecycle.rs` remains the synchronous lifecycle owner. `ProfileLifecycleContext::{new,for_claude,for_codex,for_grok}` binds concrete catalog/registry, runtime, locks and history paths. Constructors do not initialize files. `instance`, `manager`, `paths`, `platform`, `history_file`, typed `codex_platform`/`grok_platform`, `operation_lock` and `run` expose the existing owners for actual adapter consumption.
- `update(..., expected_version: Option<&str>, patch)` and additive `update_profile_versioned` accept the original editor revision. Comparison remains inside the existing catalog transaction, in addition to early locked rejection. Legacy `update_profile` omits the expected version and retains current-at-operation behavior. Rename keeps exact unknown TOML values/datetimes, pointers and journal compensation; same-name edits do not activate.
- `create` uses the platform's validation/normalization/secret-storage owner, operation/resource guard and existing-catalog transaction. Invalid identifiers are checked with the existing now-public config repository validator before secret persistence. Existing secret writes participate in the same WriteJournal. `export_profile_env` delegates to Codex's actual env-map/shell-script owner; output is explicit plaintext and must stay scoped, warned and out of logs.
- Existing `profile_off.rs` cleanup/backup semantics now accept the same explicit context. Production default entrypoints remain. Claude settings/auth diagnosis and registry helpers use bound paths; Codex runtime directory lists preserve default production login-prep selection and injected fixtures select only their supplied runtime. No second cleanup list or auth/logout implementation was created.
- `LockManager::with_scoped_directory` is a narrow synchronous thread-local RAII directory selector for the existing default resolver. It covers nested catalog, leaf, journal rollback, credential and ancillary locks. Same normalized nested directory is allowed; a different nested root fails before its body. Unscoped `with_default_path` is unchanged. No process env override, new lock identity, async handoff or global setup.
- Grok public save/apply/delete/off retain their private operation lock. Their already-locked inner helpers and private borrowed PlatformConfig adapter allow generic lifecycle calls to take profile_application → grok_profile_operation → catalog → leaf, rather than invert catalog/private-operation order. No TLS semantic-lock reentrancy was introduced.
- Grok `update_profile_versioned` owns the existing save-destination/apply-if-Active/delete-source staged policy, with `GrokProfileUpdateOutcome::{Updated,Renamed,RenameApplyFailed,RenameCleanupFailed}`. It binds expected revision under the reentrant catalog guard while holding the private operation lock first. `finish_grok_profile_rename` is the same pure decision owner consumed by adapter callback tests; payload-bearing outcomes have no Debug.
- Grok `delete_profile_guarded` and pure `grok_profile_delete_plan` retain Active/Drifted/UnsafeMissingEntryState, explicit force and unrelated-target rules. Unsafe state never authorizes automatic cleanup, even with force. Tauri and native consumers must refresh real state after partial outcomes.
- Codex apply's pre-clear OAuth synchronization now uses existing `CodexOAuthTokenService::from_dirs` for the platform's actual paths, avoiding ambient-root resolution from an otherwise injected instance. Lifecycle post-commit auth synchronization captures the existing default service for legacy contexts and an explicit-path service for injected contexts. History parent preparation matches `Database::init_default` while using the bound file; failure remains a committed warning and replay does not repeat it.

## Files owned and changed

- `crates/ccr-core/src/core/lock.rs`
- `crates/ccr-config/src/managers/config/repository.rs` (pure identifier validator visibility only)
- `crates/ccr-cli/src/application/profile_lifecycle.rs`
- `crates/ccr-cli/src/application/profile_lifecycle_tests.rs` (new private fixture suite)
- `crates/ccr-cli/src/application/profile_off.rs`
- `crates/ccr-cli/src/platforms/claude.rs`
- `crates/ccr-cli/src/platforms/grok.rs`
- `crates/ccr-cli/src/services/claude_auth_service.rs` (explicit runtime constructor only)
- `crates/ccr-codex/src/platforms/codex.rs` (explicit OAuth sync paths only)

Some listed tracked files already contained 02 changes; those were preserved. No manifests/locks, native UI/runtime, Tauri commands, task status or inventory were edited by this owner. Main owns spec edits; their factual review was sent separately.

## Actual verification

Environment: Windows, PowerShell, repository pinned Rust 1.98.0. Early invocations used the existing Cargo/MBX path; final checks use `rustup which cargo` and prepend that exact toolchain's bin directory to PATH in the verification child only, so Cargo/rustc/rustdoc match. No global PATH, toolchain, HOME or cache change. Root target remains the existing managed cache. Commands below keep `--locked`, default test parallelism and `--skip export_bindings`.

| Command | Actual result | Raw output |
| --- | --- | --- |
| `cargo check -p ccr-cli --locked --lib` initial | lock refused while manifest owner was synchronizing the new shared dependencies; no lock repair by this owner | tool output; later source check in lifecycle-check.log |
| same check after shared lock synchronization | 101: existing factory returns Arc, context initially declared Box; corrected to Arc | `outputs/desktop/profiles/lifecycle-check.log` |
| injected suite first | 4 pass / 1 fail: overbroad test assumed generic active Grok rename succeeds; retained actual baseline Unchanged instead of changing semantics | `lifecycle-injected-tests.log` |
| corrected initial injected suite | 6 pass, exit 0 | `lifecycle-injected-tests-final.log` |
| extended suite initial | 8 pass / 2 fail: fixture assumed NoAuth env export contains a secret and assumed an unapplied description edit remains Active | `lifecycle-injected-complete.log`, `lifecycle-injected-corrected.log` |
| extended suite corrected | 12 pass, exit 0 | `lifecycle-injected-final.log` |
| `test -p ccr-core --locked scoped_lock_director -- --skip export_bindings` | 2 pass, exit 0; real guarded writes on independent threads, nested/error/panic restore, no directory creation on lookup | `core-lock-scope-tests.log` |
| `test -p ccr-cli -p ccr-config -p ccr-codex --all-features --locked -- --skip export_bindings` | exit 0: CLI 402 pass/1 ignored, dispatch 12 pass, Codex 392 pass/2 ignored, config 123 pass/1 ignored; doctests 1 pass/2 ignored; 24 binding exports filtered | `lifecycle-domain-regression.log` |
| `clippy -p ccr-core -p ccr-config -p ccr-codex -p ccr-cli --all-features --all-targets --locked -- -D warnings` initial | 101, one collapsible_if in new ancillary code; fixed without allow suppression | `lifecycle-domain-clippy.log` |
| same strict Clippy final | exit 0 | `lifecycle-domain-clippy-final.log` |
| final `test -p ccr-cli --all-features --locked application::profile_lifecycle::injected_tests -- --skip export_bindings` | **13 pass, exit 0** after the final history parent/replay adjustment | `lifecycle-injected-frozen.log` |
| `rustfmt --edition 2024 --check` on owned Rust files and `git diff --check -- <owned files>` | exit 0, no repair hidden in validation; earlier rustfmt repair was separate and explicit | tool output |

The broad three-package regression preceded the final narrow history-parent compatibility fix and its extra test. Final strict Clippy compiled the changed code/all tests, and the 13-test suite re-executed that code. The existing legacy ancillary contract is being rechecked separately; its final result is appended below. Full root `just ci`, Tauri adapter regression, native screenshots and final 04 acceptance are owned by the coordinator/check role, not claimed here.

## What the new tests actually prove

- Three typed constructors and missing list/current reads create no fixture files; mismatched platform rejects before file access.
- Real platform create/edit/read/apply/replay/off, typed Codex auth normalization/export, exact catalog revision, unknown extension values/TOML datetime retention.
- Two independently constructed clients and one shared revision produce exactly one successful save and one conflict for Claude/Codex/Grok. Separate staged Grok clients prove the same under its own owner.
- Invalid patches/identifiers, duplicate creation and stale creation do not overwrite; deterministic catalog commit failure restores existing secret-store/catalog state; guarded apply failure restores runtime/markers/revision; blocked backup directory leaves prior state intact.
- Actual staged Grok apply failure retains destination/source and old Active runtime. Actual cleanup failure induced by removing the fixture entry state after runtime publication retains both names and reports the real unsafe/new-marker state. A genuinely Active rename succeeds. Legacy generic active Grok rename remains Unchanged; native must use the staged owner.
- Direct Grok apply held at a real runtime writer can run concurrently with context apply and staged edit without catalog/private-operation inversion.
- Active/Drifted deletion blocks without force; force clears and deletes only the target. Unsafe entry state blocks both ordinary and force deletion with unchanged catalog/runtime.
- History directory creation and blocked-parent outcomes preserve committed status, warning, current profile and same-ID replay.

## Limits and integration notes

- Partial Grok cleanup failures do not universally guarantee both names: an existing lower-level delete can commit its catalog removal before a later registry failure. Always retain/report the actual remaining names and refresh runtime truth; do not auto-repeat activation.
- The synchronous owner itself does not cancel a committed write. Native scheduling/cancel terminal behavior must be verified in the app-owned runtime, without splitting the journal across workers.
- No native UI, IME, accessibility transport, DPI/scale matrix, macOS/Linux execution, release performance, packaging or complete migration acceptance is claimed by these domain tests. User deferred scale/DPI validation.

## Final freeze append

- Final legacy `test -p ccr-cli --all-features --locked profile_ancillary_contract -- --skip export_bindings`: **1 pass, exit 0**, `outputs/desktop/profiles/lifecycle-legacy-ancillary-final.log` (31.07s test duration; this is not a performance sample).
- Independently captured final owned-file `rustfmt --edition 2024 --check` and `git diff --check` exit codes: both **0**. Fixed-toolchain path/version and exact source SHA256 are in `outputs/desktop/profiles/lifecycle-frozen-receipt.json`.
- The first un-escalated root Cargo probe could not access the existing managed target cache (Access denied); it was an environment failure, not a product test result. Required cache access was then granted and actual outcomes above were recorded. No global cache cleanup/configuration was performed by this agent. MBX savings messages are historical reports and are not evidence of a new deletion.
- Domain/core product files are frozen for independent review. Main/native/shared owners continue adapter/native integration; this report does not close 04.
