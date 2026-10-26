# Profile Application Lifecycle

## Scope and owner

- Shared owner: crates/ccr-cli/src/application/profile_lifecycle.rs.
- CLI switch, TUI apply, and desktop apply call the same synchronous owner.
- Claude and Codex update/rename call update_profile. The patch closure changes a ProfileConfig in memory. The application owns persistence, pointers, activation, and compensation.
- CLI, Tauri, and TUI schedule the complete synchronous lifecycle inside one blocking worker. TUI scheduling belongs to `ccr-tui/src/tui/profile_tasks.rs`; the thread-local journal must not cross worker boundaries. Do not await, spawn, read stdin, print, or exit while the operation journal is active.
- Keep public CLI exports and existing IPC command IDs. Do not add CcrError variants for operation outcomes.

## Prepare and execute

1. Validate the operation ID, profile existence, enabled state, and platform rules before runtime writes.
2. Acquire the platform application lock. Re-read the target and bind the repository snapshot token.
3. Declare the complete file set: platform profiles, registry, runtime, and platform-owned auth/entry/secret files. Include the operation record.
4. Write an interrupted/recovery marker, start the journal, and verify source versions before mutation.
5. Apply runtime and pointer changes. Verify declared files. Save the committed outcome before committing the journal.
6. Increment usage and record history only after activation commits. History uses the operation ID. Auth-registry and provider analytics failures are ancillary warnings.

The application lock serializes cooperating lifecycle callers. The leaf writer still owns backup, CAS, atomic replacement, and permissions. A noncooperating external writer can cause compensation to fail; retain its version and return recovery_required.

## Outcomes and replay

- unchanged plus activation_committed=false: activation failed and committed target writes were restored.
- applied: the mutation committed. activation_committed distinguishes an active rename/apply from an inactive update.
- applied_with_warning: the mutation committed and ancillary work failed or remains pending. Do not convert the result into a generic activation failure.
- recovery_required: the application cannot verify safe restoration. Return only identifiers, warning codes, and recovery paths. Do not include credentials or raw I/O errors in the outcome.
- Replaying the same apply operation ID returns its stored outcome without activation, count, history, or ancillary retries. The source profile may have been deleted after commit.
- A persisted interrupted marker requires recovery. It does not authorize automatic activation after restart.
- CLI prints the warning and retains successful activation status. TUI localizes warning/error severity. Desktop returns the structured outcome; React refreshes state and does not show a success-only toast or repeat an already committed rename activation.
- Mutation responses contain status and identifiers. Do not return the raw profile credential payload after update.

## Rename

- Reject blank, reserved, and existing destination names before mutation.
- Seed the destination with the exact repository TOML section before applying the ProfileConfig delta. Preserve unknown fields and TOML datetime values.
- Save the destination, update current/default references, delete the source, and apply the destination when the source was active.
- The same journal covers every stage. A failed stage restores files and pointers or reports recovery_required.
- Same-name edits retain existing behavior and do not activate implicitly.

## Required verification

- Share fixtures across the actual CLI switch adapter, TUI backend adapter, and desktop service adapter.
- Exercise invalid/disabled/missing targets, deletion after preparation, successful replay after source deletion, count/history failure, every guarded write, post-publication failure, and external versions that prevent compensation.
- Run Claude/Codex rename fixtures through both the application and actual desktop helpers. Cover every write stage, current/default references, credentials, unknown values, reserved names, and inactive edit warnings.
- Capture tracing with a secret sentinel. Check safe DTO and Debug output, source boundary guards, and localized TUI presentation.
- Exercise React hooks for committed warnings and recovery, one activation call, state refresh, and editor suppression of duplicate activation.

## Scenario: enable policy and direct repository mutation

### 1. Scope / Trigger

- Trigger: enabling a disabled profile, or changing a desktop repository mutation that checks the active profile.
- Owner: `crates/ccr-cli/src/application/profile_lifecycle.rs`. The generic desktop configuration adapter supports Claude; the shared lifecycle supports its existing Claude, Codex, and Grok fixtures.

### 2. Signatures

```rust
pub fn enable_profile(request: ApplyProfileRequest) -> Result<ProfileOutcome>
pub fn operation_lock(platform: Platform) -> Result<FileLock>
```

`enable_profile` uses the same request, journal, outcome, and replay contract as `apply_profile`. `operation_lock` exposes the existing application lock to synchronous adapters.

### 3. Contracts

- Enable allows a disabled target through preparation. All other target, authentication, and platform validation still applies.
- Persist `enabled = true` and activate the target inside one application execution and one `WriteJournal`. Do not save policy first and start a separate apply operation.
- The journal covers enabled policy, runtime files, current/default pointers, and the operation record. Failure restores the declared files or returns `recovery_required`.
- Same-operation-ID replay does not reapply, increment usage, or retry ancillary work. Post-commit history or usage failure remains an `applied_with_warning` outcome.
- Direct update-disable and delete adapters acquire `operation_lock`, resolve the platform's actual current profile, then call the repository mutation in the same synchronous worker. The current-profile check must happen after acquiring the guard.
- The Claude resolver can fall back from an invalid file marker to the registry. Use that resolver when protecting the active profile. Preserve the repository's existing current/default protections.
- Drop the guard before calling `apply_profile`, `enable_profile`, or `update_profile`; those functions acquire the application lock themselves. Never hold the guard across an await.
- The application guard does not replace the T01 resource lock, version check, atomic writer, backup, or permissions.

### 4. Validation & Error Matrix

| Condition | Required result |
| --- | --- |
| Disabled valid target passed to enable | Persist enabled policy and activate once |
| Disabled target passed to ordinary apply | Reject under the existing apply validation |
| Guarded write fails | Restore enabled policy and prior files, or return `recovery_required` |
| Usage/history fails after commit | Return committed outcome with warning; do not repeat activation |
| File current is invalid but registry current resolves | Reject disable/delete of the resolved active profile |
| Another lifecycle operation owns the application lock | Wait, then re-read active profile before repository mutation |
| Adapter holds a guard and calls a lifecycle owner | Invalid lock composition; release the outer guard |

### 5. Good/Base/Bad Cases

- Good: a disabled target becomes enabled and active; replay returns the recorded outcome.
- Base: an ordinary switch retains disabled-target rejection.
- Bad: the adapter saves enabled policy, apply fails, and the policy remains changed.
- Bad: an adapter reads current before waiting for the lock and later deletes the newly active target.

### 6. Tests Required

- `profile_contract::tests::enable_policy_activation_and_compensation`: exercise Claude/Codex/Grok success, same-ID replay, and every guarded write failure; compare the complete fixture files and disabled policy after compensation.
- `commands::config::contract_tests::actual_handlers_switch_and_enable_commit_runtime_and_policy`: call actual handlers and assert runtime, current profile, and enabled policy agree.
- `actual_handlers_protect_registry_fallback_active_profile`: set an invalid file marker and valid registry current; list identifies the active target, and disable/delete reject without writes.
- `direct_mutation_waits_for_application_lock_and_rechecks_activation`: change current while the worker waits; assert the worker checks the new current after acquiring the guard.
- Preserve default test parallelism. Serialize generated-file export tests only under the generation workflow.

### 7. Wrong vs Correct

Wrong: `save_enabled(true)` followed by a separate `apply_profile(request)`, or a current-profile read before acquiring the application guard.

Correct:

```rust
// One lifecycle owner manages policy, activation, and compensation.
let outcome = enable_profile(request)?;

// A separate direct repository mutation checks current under the guard.
let _guard = operation_lock(platform)?;
let current = create_platform(platform)?.get_current_profile()?;
// Validate current, then call the T01 mutation. Do not call a lifecycle owner here.
```

## Cross-crate behavioral fixtures

Legacy cross-crate `profile_contract` fixtures use the explicit, ownership-marked `CCR_PROFILE_CONTRACT_ROOT` child-process seam. Tauri contract parents configure CCR keys on `Command`, preserve HOME/USERPROFILE, require the named child to pass and keep default parent test parallelism. See [Test Fixtures](./test-fixtures.md). Do not restore the old practice of mutating global HOME or serializing the whole suite.

New injected lifecycle fixtures construct `ProfileLifecycleContext` directly. A profile path alone does not isolate nested default lock resolution; context `run` binds the existing writer chain to its synchronous [scoped lock directory](../../ccr-core/backend/scoped-lock-directory.md). It must remain on one worker without await/spawn.

## Scenario: injected lifecycle and original draft revision

### 1. Scope / Trigger

Applies when native/embedding consumers need explicit catalog, registry, runtime, lock and history roots, or save a draft read before another client changed the catalog. Legacy public apply/enable/update/off APIs keep their default resolver behavior.

### 2. Signatures

- `ProfileLifecycleContext::{new, for_claude, for_codex, for_grok}` select the production resolver or explicit platform paths and runtime/lock/history paths.
- `context.update(name, target, expected_version: Option<&str>, patch)` accepts the original catalog revision; `None` retains legacy current-at-operation behavior.
- `context.create(name, profile, expected_version)` delegates normalization/secret storage to the platform owner and guards the original revision when supplied.
- `update_profile_versioned(platform, name, target, expected_version, patch)` is the production-path convenience entry.
- `context.{apply, enable, off, needs_login_prep, export_profile_env}` retain the existing domain owners.

### 3. Contracts

- Constructors and missing-source reads are path-only: no catalog migration, directory creation or ambient-home fallback from an explicitly injected context.
- Original revision comparison occurs inside the locked catalog mutation/transaction. Reading a fresh token immediately before saving cannot protect an older editor draft.
- Application, journal, backup, secret store, runtime markers, history and nested leaf locks all use the same context. No global HOME/USERPROFILE mutation or environment trick supplies missing paths.
- Claude/Codex journaled rename preserves unknown values and rollback/recovery. Same-name edits do not implicitly activate.
- Grok's existing staged rename is a distinct policy: save destination, apply if active, delete source. Its typed partial apply/cleanup failures preserve actual remaining names and runtime state, which adapters must re-read. Apply failure retains both names; cleanup may fail after catalog deletion but before registry cleanup. Do not promise both names in every cleanup failure, replace these outcomes with a generic successful rename, or silently fold them into Claude semantics.
- Context operations acquire application lock, then Grok's private operation lock, then catalog, then leaf locks. The standalone Grok staged domain API acquires private operation then catalog; callers composing application policy supply its outer application lock. The domain's already-locked adapter avoids acquiring the private operation lock from inside a catalog transaction. Direct Grok apply/off joins the same private lock; do not use reentrant TLS to hide inversions.
- Grok guarded delete retains active/drifted/unsafe and explicit force rules in the Grok owner. A generic repository deletion is not a replacement.

### 4. Validation & Error Matrix

| Condition | Required result |
| --- | --- |
| Two native clients save the same revision | One commits, the other conflicts; unchanged fields survive |
| Wrong platform request on an injected context | Reject before accessing files |
| Invalid field, failed backup or guarded apply write | Preserve prior state or return existing recovery outcome |
| Failure after secret-store update/catalog commit attempt | Restore both under the existing journal, or report recovery |
| Grok staged apply/cleanup fails | Typed partial outcome with names/runtime truth retained |
| Native view closes during work | App runtime owns cleanup; no detached synchronous journal |

### 5. Good / Base / Bad Cases

- Good: scoped context plus original revision plus typed patch, all on one worker.
- Base: legacy caller omits expected revision and preserves its former behavior.
- Bad: compare token outside the lock, then call unversioned update; or use default auth/off helpers from a path-injected fixture.

### 6. Tests Required

`profile_lifecycle_tests` covers missing-read purity, original-draft CAS, apply/rename/replay/off, typed normalization, invalid/backup/write/commit failures, secret retention, Grok staged failure states, guarded delete and direct-apply concurrency. Run affected ccr-cli/config/codex/core suites plus real Tauri and native adapters with default parallelism and `--skip export_bindings`. Evidence must distinguish actual tests from source review/native screenshots.

### 7. Wrong vs Correct

Wrong: hold a catalog guard, then call public Grok apply which acquires its private operation lock; or save a draft with a newly read revision.

Correct: enter the platform application owner, acquire Grok's private operation lock before catalog work, use its already-locked adapter, and compare the editor's original revision inside the catalog transaction.
