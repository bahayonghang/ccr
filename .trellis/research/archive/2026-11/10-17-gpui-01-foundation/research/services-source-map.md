# Research: GPUI 02 configuration source map and runtime seams

- Query: Locate the real configuration-list chain, isolated fixture support, the smallest reusable application seam, compatibility gates, and bounded/stale-response patterns for approved GPUI child 02.
- Scope: internal; read-only source investigation, no implementation or task activation.
- Date: 2026-10-17
- Context: Active task remains `10-17-gpui-01-foundation`. The coordinator authorized this file under 01 research for later consumption by 02. Sources reflect the current workspace, not the planning SHA.

## Findings

### Files found and actual chain

| File and anchor | Responsibility / finding |
| --- | --- |
| `ccr-ui/src-tauri/src/commands/config.rs:23` | Tauri `ConfigInfo` wire DTO and `ts(export_to)` owner. |
| `ccr-ui/src-tauri/src/commands/config.rs:160` | Actual `list_configs(Option<String>) -> Result<Vec<ConfigInfo>, String>`; validation before `spawn_blocking`, snapshot, active-profile resolution, masked row projection. |
| `ccr-ui/src-tauri/src/commands/config/adapter.rs:23` | `required_platform`: generic configuration surface accepts explicit `claude` only; absent and unsupported platform errors are distinct. This file currently owns mutation adapters, not list orchestration. |
| `ccr-ui/src-tauri/src/services/mod.rs:1` | Existing service modules are agent_sessions, claude_observer, home_insights, usage. There is no configuration service adapter here. |
| `crates/ccr-config/src/managers/config/manager.rs:23` | `ConfigManager::new(path)` supports explicit fixture paths; accepts legacy profile path or database path through domain-owned catalog location resolution. |
| `crates/ccr-config/src/managers/config/manager.rs:39` | `for_platform` resolves named platform without selecting registry order or creating files. |
| `crates/ccr-config/src/managers/config/repository.rs:51` | `ConfigSnapshot { config, version }`; `snapshot()` at line 136 preserves the catalog revision for later CAS edits. |
| `crates/ccr-config/src/services/config_service.rs:95` | Existing `ConfigService::list_configs()` reads domain data but is **not** a drop-in equivalent of Tauri list. |
| `crates/ccr-cli/src/platforms/claude.rs:120` | Current-profile read orchestration: load domain profiles, declared marker, registry marker, invoke domain precedence resolver; no repair. |
| `crates/ccr-config/src/platforms/base.rs:458` | `load_profiles_from_toml(path)` is compatibility-named domain API reading the catalog; missing returns empty, other errors propagate. Do not implement another TOML reader. |
| `crates/ccr-config/src/platforms/base.rs:472` | `load_current_profile_marker(path)` trims declared current marker; missing/empty means absent, other read errors propagate. |
| `crates/ccr-config/src/platforms/base.rs:791` | `resolve_file_current_profile` is the existing pure Claude precedence owner; returns current and optional repair suggestion. |
| `crates/ccr-config/src/managers/platform_config.rs:285` | `PlatformConfigManager::new(path)` injects registry path. `load()` at line 341 returns default for missing files without writing. |
| `crates/ccr-core/src/core/secret.rs:81` | Secret Display/Debug/default serialization masks via the existing mask owner. |

The real chain is **Tauri command -> domain snapshot plus CLI Claude current-profile resolver -> domain profile/registry APIs**, not Tauri config adapter -> existing configuration service. The first migration must remove/move existing orchestration, rather than adding an unused parallel service.

### Compatibility that a naive ConfigService replacement loses

1. Tauri returns `description.unwrap_or_default()`; `ConfigService` uses `display_description()`. Preserve the former on the existing wire surface.
2. Every Tauri row has the snapshot revision. `ConfigService::ConfigInfo` has no version.
3. Tauri current status is `create_platform(platform)?.get_current_profile()`, including valid registry fallback when catalog marker is stale. `ConfigService` compares the stored catalog marker directly.
4. Tauri base URL and masked token are strings with empty defaults; domain service fields are `Option` and `Secret`. Tauri usage count is `u64` (`ts(as = "f64")`), domain service count is `u32`.
5. Row order follows catalog `sections` iteration. Preserve it; no incidental sorting/filtering.
6. Tauri validation preserves `platform_required` versus `config_platform_unsupported`; do not silently enable Codex/Grok through this legacy generic surface.
7. Missing catalog is currently an error, not an empty successful list. `catalog/mod.rs:439` distinguishes `Absent -> ConfigMissing`, `NotMigrated -> migration-required error`, `Ready -> read-only connection`. An existing empty catalog can yield an empty successful list.
8. Tauri list prefixes manager-construction failures with `Failed to create ConfigManager:`, uses the existing domain Display for domain failures, and uses `Task join error:` for Tokio join failures (`config.rs:163-210`). Avoid gratuitous wire error text changes.
9. Queries do not initialize, migrate, repair markers, write backups, or acquire mutation locks. Current resolution returns repair suggestions but list must not execute them.

The existing fallback regression is `commands/config/contract_tests.rs:281` (`actual_handlers_protect_registry_fallback_active_profile`): catalog marker `ghost`, registry `new`, list marks `new` current, updates/deletion still protect it. The masking assertion is at line 73.

### Recommended smallest extraction

Recommended seam is a small framework-free `ccr-app::config` module returning display-safe typed rows and carrying `ConfigSnapshot.version`. Keep the Tauri DTO and all `ts(export_to)` attributes in `commands/config.rs`; map shared rows there or in its existing adapter. The shared response need not derive TS or serialize credentials.

To avoid bringing CLI's broad dependency graph into `ccr-app`, extract **only** the already-existing `ClaudePlatform::current_profile_resolution` read orchestration into a path-injected helper in `ccr-config/src/platforms/base.rs`, then delegate from ClaudePlatform. Suggested shape (names are implementation choices):

```rust
resolve_file_current_profile_at(
    profiles_path: &Path,
    registry_path: &Path,
    platform_name: &str,
) -> ccr_core::Result<CurrentProfileResolution>
```

Its implementation must reuse `load_profiles_from_toml`, `load_current_profile_marker`, `PlatformConfigManager::new(registry_path).load()`, and `resolve_file_current_profile`; it must not copy precedence logic. `get_current_profile_from_registry` currently uses default registry resolution (`base.rs:767`), so an explicit-path equivalent or this combined helper is needed. The existing pure precedence resolver takes `IndexMap<String, ProfileConfig>`; use existing `section_to_profile`, not another model conversion. Keeping separate reads reproduces today's behavior; improving consistency is a separate concern, not permission to change precedence.

Then `ccr-app` uses explicit `ConfigManager`/registry paths for tests and `PlatformPaths::new(Platform::Claude)` for production construction. `PlatformPaths` fields are public (`models/platform.rs:423`), and `new` is the domain path owner (`:455`); avoid adding duplicate `<root>/platforms/...` production path logic in desktop code. A fixture constructor can receive the two concrete domain paths directly. `ConfigManager::new` and `PlatformConfigManager::new` already provide the necessary injection seam without process-wide HOME changes.

Alternative if the tiny domain extraction is deferred: let ccr-app depend on `ccr-cli` and reuse `create_platform`; this respects the explicit no-`ccr`-umbrella constraint but pulls CLI, network/sync/store/etc. dependencies (`crates/ccr-cli/Cargo.toml:26`). It also leaves the no-global-env fixture problem unsolved. Prefer the small domain extraction above.

Do not create wrappers for every existing `ConfigService` method or migrate mutations during 02. Profile mutation/CAS/apply belongs to 04. No new SQL/schema/persistence owner is required.

### Types, errors, masking

- Shared domain inputs can use existing `Platform`; request identity needs typed environment identity, platform identity, request ID/generation. Do not reuse a local path as environment identity.
- A display row should contain only masked `auth_token` text, or retain `Secret` without any plaintext conversion; the UI list never needs `Secret::expose()`.
- `Secret::Display`, `Debug`, and `Serialize` use `mask_sensitive` (`secret.rs:81-102`); do not add a second masking implementation.
- New application/runtime errors should be local to `ccr-app` (typed domain/source, cancelled, queue-full, unsupported environment, stopped, join/worker failures as actually needed). `CcrError` is frozen; do not add variants. Bridge existing domain `CcrError` rather than flattening away classifications inside the shared service.
- Keep Tauri's string conversion at the adapter boundary. Raw config source, rejected credential literals, or full parser Display text must not enter display diagnostics, logs, or screenshots.
- `ConfigSnapshot`/catalog transactions should be created and consumed on the blocking worker; do not send thread-bound catalog guards/transactions into GPUI entities.

### Fixture support and important existing-test conflict

`ConfigManager::new(temp_path)` + `snapshot` + explicit registry manager is sufficient for read tests with no environment mutation. A real fixture must be a domain-created `profiles.db`, not just a written `profiles.toml`: a legacy TOML alone now intentionally returns migration-required (`catalog/mod.rs:439`). Use public domain setup APIs (e.g. `replace_toml(content, "")` at `manager.rs:92`) rather than reproducing catalog SQL. That setup mutates only fixture data but uses default lock resolution; isolate setup lock paths via **child-process environment** or a supported injected lock owner, not test-runner global environment mutation.

`ccr-config::test_support::TestCcrEnv` is `pub(crate)` and `#[cfg(test)]` only (`lib.rs:35-52`), not reusable from ccr-app. It modifies CCR_ROOT/CCR_LOCK_DIR under its own lock. Public feature `test-support` exports fault hooks, not this fixture.

**Critical existing-test caveat:** Tauri config `fixture()` at `contract_tests.rs:10` calls `ccr_cli::application::profile_contract::isolated_fixture`; that calls `TestHome::new_with_home_env()` (`profile_contract.rs:34`). The latter writes global `HOME` **and** `USERPROFILE` (`ccr-cli/src/lib.rs:100-104`). This conflicts with this user's explicit no-global-HOME testing rule and the parent design prohibition on global env serialization. Do not run this test suite unchanged and claim it complies. New read parity tests should use explicit-path service/adapter seams and/or isolated child processes whose environment is set with `Command::env`, without mutating the parent test process. If existing contract tests are required, narrowly replace or route their fixture before running them; do not change production profile semantics as a workaround.

The smaller existing `TestHome::new()` (`ccr-cli/src/lib.rs:48-83`) avoids HOME but still changes several process-wide overrides. It is not the final answer to the parent's stronger explicit fixture/root-provider requirement. Avoid globally downgrading to serial tests: keep default test parallelism and `--skip export_bindings`.

### Runtime patterns to reuse and gaps to implement

| Existing source | Reusable behavior / limits |
| --- | --- |
| `ccr-ui/src-tauri/src/commands/command_exec.rs:1497` | `try_send_output_batch` handles Full by counting dropped lines; Closed stops the producer. Suitable for lossy **logs**, not required terminal delivery. |
| `ccr-ui/src-tauri/src/commands/command_exec.rs:1535` | 100ms interval + `MissedTickBehavior::Skip` demonstrates fixed 10Hz batching. Existing queue cap is **256** (`:28`), not the new GPUI notification budget 128. Do not alter Tauri constants to match. |
| `crates/ccr-core/src/core/log_bridge.rs:96` | Already shared nonblocking bounded log enqueue, drop counter and reentrancy guard. Queue capacity is 256 (`:13`); do not use it for completion events. |
| `ccr-ui/src-tauri/src/usage_jobs.rs:236` | Single owner admits jobs; cancel retains admission and reports `CancelRequested` (`:265`); progress rejects cancelled/terminal records (`:278`); only runner cleanup completion writes terminal (`:291`); duplicates cannot rewrite terminal. |
| `ccr-ui/src/features/commands/commandJobStore.ts:55` | Existing generation counter; read gate at `:146-150` checks epoch + request counter + job identity before accepting a response. Port the principle to typed Rust RequestKey, not the JS store. |
| `ccr-ui/src/shell/eventBridge.ts:121-130` | Disposed subscriptions reject callbacks; a late listener registration immediately unlistens. Cleanup at `:190-193`. Rust Drop/weak entity handling needs the same guarantee. |
| `ccr-ui/src/configs/environmentSession.ts:10` | Environment identity compares both id and env_type; readiness/fresh environment verification fails closed. |

No existing ready-made Rust `RequestKey(environment, platform, generation)` GPUI read runtime was found. It must be added for 02. Likewise the sources above are coupled to concrete jobs/Tauri DTOs; do not wholesale relocate usage/command registries before their migration tasks. A small typed runtime for the real read chain plus a small app-owned lifecycle primitive can establish 02 without migrating unrelated domains.

Required new behavior: one long-lived Tokio owner; bounded admission and finite blocking concurrency; at most one current page read; generation invalidation on environment/filter changes; stale completion rejected even if cancellation cannot stop already-started blocking I/O; dropping a view removes read subscriptions; app-owned side-effect jobs survive view closure; shutdown cancels and awaits owners without falsely marking complete. Notifications capacity **128**, progress coalesced at **100ms**, terminal state retained authoritatively and delivered/reconciled independently of lossy progress/logs. Queue saturation tests must prove terminal visibility and bounded memory, including when a view closes or changes generation.

Do not implement SSH/WSL by relabeling local config reads. 02 can establish typed environment keys and explicit unsupported errors; remote support remains 09. Legacy Tauri behavior remains unchanged.

### Minimal implementation file whitelist for coordinator

One current writer per shared file. Suggested minimal boundary:

- `crates/ccr-app/Cargo.toml`, `src/lib.rs`, `src/config.rs`, `src/runtime.rs` (or a small runtime directory), focused tests: new framework-free orchestration/types/runtime behavior.
- `Cargo.toml`, `Cargo.lock`: add ccr-app workspace member/path dependency; leave root CLI default-members unchanged.
- `crates/ccr-config/src/platforms/base.rs`: only path-injected current-resolution extraction and focused tests if selecting the recommended no-CLI-dependency seam.
- `crates/ccr-cli/src/platforms/claude.rs`: delegate existing read orchestration to that helper; preserve repair methods unchanged.
- `ccr-ui/src-tauri/src/commands/config.rs`, `commands/config/adapter.rs`: list delegation and wire-only mapping; preserve DTO definition/export path and mutation behavior.
- `ccr-ui/src-tauri/src/commands/config/contract_tests.rs`: fixture/parity regression additions only, subject to the HOME caveat.
- `ccr-ui/src-tauri/Cargo.toml`, `Cargo.lock`: shared crate dependency/lock.
- `ccr-desktop/Cargo.toml`, `Cargo.lock`, `src/runtime/*`, `src/app.rs` (existing actual 01 filenames must be checked at handoff): real async configuration read and local entity updates, no synchronous render I/O.
- Optional narrow fixture helper in `crates/ccr-cli/src/application/profile_contract.rs` / `src/lib.rs` **only if** the coordinator chooses to repair existing contract fixture usage; do not refactor all fixtures or broaden production behavior.

Do not edit ccr-core error variants, catalog SQL, generated TS, handler names/policies, CLI dispatch, usage ownership, Tauri UI components, or other domains for this first chain. The coordinator owns task state, manifests, evidence and migration inventory updates.

### Verification handoff (not executed by this researcher)

1. `cargo test -p ccr-app --locked -- --skip export_bindings` with deterministic queue/generation/cancel/shutdown tests and fixture parity covering revision, row order, current fallback, masking, missing/corrupt/not-migrated/empty states; snapshot fixture bytes/mtime/files before and after reads.
2. `cargo test -p ccr-config --locked -- --skip export_bindings` and targeted Claude current-profile tests if domain extraction occurs. Existing tests' environment fixture behavior must be considered against the user's restriction before execution.
3. Tauri targeted config command tests: `cargo --config .cargo/tauri-ci.toml test --manifest-path ccr-ui/src-tauri/Cargo.toml --locked commands::config -- --skip export_bindings`, **after** HOME fixture issue is handled. `contract_tests.rs:31`, `:81`, `:281`, `:326`, `:453` anchor masking, platform validation, fallback, CRUD/CAS and process concurrency regressions.
4. `just tauri-bindings-check` (`justfile:1724`, `ccr-ui/scripts/check-generated-bindings.mjs:8-32`) regenerates under a transaction, compares normalized structure, restores original bytes. Preserve `ConfigInfo` export location. This gate does write temporary generated content but is a validation gate, not a migration authorization.
5. `just tauri-command-inventory-check` (`justfile:1743`) ensures macro/handler metadata stays unchanged when moving body logic.
6. `just desktop-check`, `just desktop-test`, `just desktop-lint` use 01's delivered commands; verify `--locked`. A controlled slow read must leave actual GPUI UI interactive. Native evidence remains separate from Rust tests.
7. Approved 02 gates include `just version-check`, `just fmt-check`, `just lint-strict`, `just test` and final affected Tauri/desktop gates. Do not run repair commands as validation; default Rust parallelism remains intact.
8. `cargo tree -p ccr-app` (root) plus independent desktop dependency inspection: no `ccr` umbrella/tauri/gpui/WebView in ccr-app's graph. GPUI may depend on ccr-app in its own workspace.

### Related specs and context read

- Full 02 `prd.md`, `design.md`, `implement.md`; parent `research/architecture.md`; relevant parent `design.md` contracts.
- `.trellis/workflow.md`, `code_map.md`; `trellis-start` skill loaded for workflow context only; no task activation or git commands.
- `.trellis/spec/ccr-config/backend/{index,backend-guidelines,test-fixtures,profile-repository}.md`: domain owner, pure queries, CAS, marker/secret/fixture rules.
- `.trellis/spec/ccr-core/backend/{index,ccr-error-freeze}.md`: existing error freeze and Secret ownership.
- `.trellis/spec/ccr-ui/frontend/{index,api-facade-boundary,environment-scoped-dashboard-contracts,command-job-lifecycle}.md`: wire/generation/subscription/terminal compatibility.
- `.trellis/spec/guides/{index,code-reuse-thinking-guide}.md`.

## External References

None used: this is an internal source-map topic. GPUI dependency version research belongs to 01's foundation owner and was deliberately not duplicated. Workspace currently declares shared version `7.5.0` (`Cargo.toml:23`) and ccr-cli MSRV `1.95` (`crates/ccr-cli/Cargo.toml:5`); these are observed manifests, not external registry verification.

## Caveats / Not Found

- No code, Cargo, task status, evidence or migration inventory was modified; no tests, user-config operations, git operations, native UI, or performance measurements were run. This document is research, not implementation evidence.
- Research role forbids loading `implement.jsonl` / `check.jsonl`; those were intentionally not read despite the dispatch request. Implement/check roles must load them at their actual dispatch.
- `C:\Users\lyh\.codex\RTK.md` was missing at the exact requested path. Memory registry keyword search found no relevant prior GPUI/config entries; no memory-derived claims were used.
- Some initial narrow `rg` probes used nonexistent or unsupported wildcard paths and were corrected by locating the real files; these were search misses, not test results.
- Foundation concurrently owns new `ccr-desktop` files. At the initial read, `ccr-desktop/src` did not exist; filenames in the whitelist must be reconciled with actual 01 delivery before edits.
- Existing parent architecture includes old line numbers/version snapshots; citations above were checked against current source. No assertion is made that 01 or 02 acceptance is complete.
