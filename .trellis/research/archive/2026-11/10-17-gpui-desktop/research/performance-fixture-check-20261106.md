# Explicit Tauri fixture — independent check

Scope: the opt-in fixture boot, captured core graph and shared explicit-path seams in the current worktree. This is source/isolated-test evidence. No native window, WebView storage inspection, benchmark sampling, WSL/SSH/OAuth or real account action was run by this reviewer.

## Findings (fixed)

### Captured file paths still used ambient lock directories

- `commands/claude_profiles.rs`: the allowed direct Claude add/delete handlers built a captured `ClaudePlatform` but invoked its catalog mutation outside the fixture's thread-local lock scope. `base::mutate_profiles` reaches `catalog::acquire_catalog_lock`, which calls `LockManager::with_default_path`.
- `commands/claude.rs`: a captured `SettingsManager` was insufficient for Settings/Agents/Plugins/Commands writes. Its `update_atomic_async` creates a new blocking task; the underlying guarded writer resolves the default lock directory on that task, rather than using the manager field as an ambient override.
- Added `commands/claude_fixture_tests.rs`. Each parent test spawns only its exact test in a fresh process, passes explicit synthetic roots, and sets that child's `CCR_LOCK_DIR` to an ordinary fixture file. Both tests **failed before the fix** with OS error 183, demonstrating actual fallback. No parallel parent HOME/USERPROFILE mutation was used.
- Direct add/delete now enter `fixture::with_locks` inside their blocking closure. The fixture Settings branch enters the scope in its own blocking closure and calls the same synchronous `update_atomic` owner. The Normal branch keeps its existing `update_atomic_async` path. Both tests then **passed 2/2**, including profile add/delete/readback and settings readback with the poisoned default untouched.

### Authentication summary recreated an ambient platform

- Independent call-chain review found `ClaudeAuthService::get_runtime_summary` constructed `ClaudePlatform::new()` even when the service's authentication files and process-environment policy were captured. Both allowed auth-read commands could therefore read host profile/settings facts.
- Reported the public-owner boundary to the coordinator and implementer before changing it. The implementer added captured `with_profile_paths`, bound it from `PlatformRoots::claude_auth` and profile-off reconstruction, and rejects an environment-isolated service lacking profile paths before any default platform construction. Normal service construction retains its existing behavior.
- The reviewer inspected the resulting branch and binding sites. Implementer-supplied verification: a new synthetic ambient-root-as-file trap retains the selected profile/provider, unbound isolated summary fails closed, and inherited process credentials are absent; root `captured_` **5/5** and the auth owner **26/26** pass. This is not represented as a reviewer-run pre-fix auth probe against the real host.

### Default preference-save compatibility

- `state.rs`: newly caching an optional preference path accidentally made Normal updates synchronously fail when the cached path was absent, although the previous implementation attempted the default save asynchronously.
- Only fixture saves now require/use the captured path. Normal saves keep the old detached `to_save.save()` call. This local compatibility correction was source checked and covered by the latest strict type/lint check; no global HOME mutation was added to manufacture this corner case.

## Reviewed data flow

- Fixture parsing/marker/tree validation occurs before logger initialization and Tauri context creation. Missing/invalid/linked roots fail without a Normal fallback. Runtime identity is immutable.
- Configured windows are disabled before Tauri's automatic construction. The original main composition is built after captured State initialization, and initial/main-rebuild/tray builders share the selected WebView data directory. macOS explicitly refuses this contract before side effects.
- Explicit catalog migration keeps the existing migration owner. Main/global managers share the same installed pool; archive and legacy migrations receive captured paths/home and remain separate databases. Default wrappers remain available.
- Allowed profile, Claude settings/MCP/collections, usage/session, UI-state/preferences and monitoring paths were traced to captured owners. Session registries disable environment overrides. Install/trust service construction itself does not run their stored host-probe closures.
- Startup skips pending OAuth recovery and WSL/SSH discovery for fixtures; Local refresh keeps the captured environment. Tray reads captured Codex files and does not query remote quota. Maintenance/monitoring bridge keep their original lifecycle on captured storage.
- Both registry dispatch and the command macro's Rust runtime wrapper reject unreviewed IDs before the handler future body. The implementer retained normal manifest/confirmation audit before fixture rejection, with payload-free rejection logging; risk labels alone do not grant fixture capability.

## Verification

- Reviewer-run red: pinned Cargo/Rust 1.98, locked/offline Tauri `commands::claude::fixture_tests` — **0 passed, 2 failed**, both caused by the poisoned ambient lock directory.
- Reviewer-run green: same command after the local fixes — **2 passed**, default parallelism, `--skip export_bindings`; test subprocesses themselves each require `1 passed`.
- Reviewer-run `cargo --config .cargo/tauri-ci.toml clippy --manifest-path ccr-ui/src-tauri/Cargo.toml --locked --offline --all-targets --all-features -- -D warnings` — **exit 0**. Includes the lock fixes, preference compatibility branch, captured auth-summary API and registry audit ordering. The existing command-macro linker informational warning explicitly ignores `-D warnings`; no lint suppression was added.
- Scoped Rust formatting and diff checks on reviewer-owned files pass.
- Implementer-supplied checks: explicit logging **3/3**, registry **21/21**, runtime policy **11/11**, and root auth/captured checks above. Final fresh fixture **3/3** plus its actually executed ignored child also passes after the summary/lock fixes: the child points CCR_ROOT/DATA/CONFIG/CLAUDE_CONFIG_DIR at synthetic ordinary-file traps and still exercises selected-root auth, MCP and generic profile CAS. These counts overlap; they are not summed as unique coverage. The reviewer did not redundantly rerun the unchanged complete graph.

## Findings (not fixed) / qualification limits

- This mode admits a reviewed local core graph, not every Tauri capability. Other platform, external-process and network actions remain closed; they need their own captured call-chain implementation before inclusion in a benchmark scenario.
- Same logical S0/S1/S2 generation/loading across both clients, real process-create/first-present/action-ready clocks and sample collection are unfinished engineering outside this review slice. Performance remains **not_run / not_comparable**.
- Root traversal/critical path checks and builder configuration are source/headless evidence. Actual WebView cookies/localStorage/IndexedDB/cache containment, native loading and OS behavior remain **user-deferred / not_run**. Windows/Linux configuration does not prove macOS or Linux native acceptance.
- Parent final integrated GPUI gates, full `just ci`, packaging and task acceptance are coordinator-owned. No task was archived, no code was committed, and no full-migration or visual PASS is claimed here.
