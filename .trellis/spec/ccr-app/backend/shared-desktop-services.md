# Shared Desktop Services

## 1. Scope / Trigger

Applies to `crates/ccr-app`, the Tauri configuration-list adapter and GPUI service/runtime consumers. This contract accompanies GPUI task 02; actual verification and remaining gaps belong in its evidence, not in this spec.

Extract only orchestration that both consumers use. Do not move domain persistence, SQL, credential handling, command policy or process cleanup into a second implementation. ccr-app must not depend on the ccr umbrella, Tauri, GPUI or WebView. ccr-usage remains the sole usage SQL owner.

## 2. Signatures

- `config::ConfigReadService::for_claude() -> ccr_core::Result<Self>` constructs paths.
- `ConfigReadService::from_paths(profiles_path: PathBuf, registry_path: PathBuf) -> Self` injects fixture/embedding paths.
- `ConfigReadService::list(&self) -> ccr_core::Result<Vec<ConfigRow>>` performs blocking domain reads.
- `runtime::RuntimeOwner::{new, handle, shutdown}` owns one long-lived Tokio runtime.
- `RuntimeHandle::read(key, work)` takes a blocking read returning `Result<T, E>`.
- `RuntimeHandle::job(key, work)` takes a runner returning `Completion<T, E>` after its cleanup.
- Both return `Result<TaskSubscription<T, E>, AdmissionError>`.
- `RequestKey` contains environment, platform, generation and request_id. `ResponseGate::accepts` compares all four.
- `platform_settings::Document::{read, save_text, save_validated}` uses a non-Debug `Snapshot` carrying original content, token, source identity and existence; saves return `SaveOutcome::{Saved, Conflict}`.
- `usage::read_dashboard(&Dashboard, &QueryFilter, Option<u32>)` assembles owner projections without adding SQL. `sessions::{list_sessions, get_detail_with_registry}` consumes the existing archive repository and provider registry.
- `resources::system_prompts::{platform_specs, prompt_spec, file_description, augment_save_result}` owns the fixed instruction-file registry and legacy result policy. Native writes use `save_prompt_file_with_locks` with the original snapshot token and captured lock directory; Tauri retains its raw-file error adapter.
- `jobs::policy::{command_catalog, validate_command_request}` and `jobs::commands::CommandJobs` are the common command policy and retained runner. `jobs::process::SidecarContext` supplies the consuming executable, repository root, debug policy, expected SHA256 and package version.
- `CommandJobs::observe(self: &Arc<Self>, id) -> Result<(Snapshot, Observation), Error>` attaches notification and captures retained state under the same lock. `Observation::next` treats deltas as wakeups and reloads authoritative snapshots, including after bounded-channel lag.

## 3. Contracts

ConfigRow preserves the existing Tauri list fields and defaults: snapshot version, profile order, empty description/base URL, enabled/usage metadata, default/current markers and optional provider/model/account/tags. Credentials pass through the existing Secret Display mask before entering this projection. Tauri keeps its DTO derives, wire shape, export location and error mapping.

Current selection uses the ccr-config-owned file/registry resolution helper, also consumed by CLI. Missing or corrupt catalogs remain errors; reads do not initialize, migrate, repair or acquire repository write locks. Existing SQLite opening can create WAL/SHM sidecars; this is distinct from a business write and must be reported accurately.

Runtime admission is 128 operations, blocking concurrency 4, notification capacity 128, and latest progress is coalesced at 100ms intervals. Terminal state is retained separately and cannot be lost behind progress. Only the runner publishes its immutable completion, after cleanup. `request_cancel` is a request, not a terminal outcome.

Dropping a read subscription requests cancellation and closes its receiver. Dropping a job subscription only detaches the view; the app retains its runner. Shutdown stops admission, requests cancellation and awaits tracked work. RuntimeOwner must outlive the native event loop; its synchronous final join runs after that loop returns. GPUI's 200ms quit-hook budget does not prove cleanup completed.

GPUI render contains no file/DB/network/process calls, runtime construction or block_on. Path resolution and list reads run on the blocking worker. Retain selected records by domain key and reject old environment/generation/request results. The current configuration adapter supports Local/Claude only; unsupported requests must not silently read Local data.

Native fixtures use explicit profile/registry paths and domain API-generated synthetic data. Artificial read delay is accepted only with paired fixture paths, at most 30000ms, and visibly labeled. Never use real accounts or mutate global HOME/USERPROFILE to isolate tests.

Native platform operations retain captured roots through commit; fixture roots never fall back to HOME. JSON dirty-leaf updates preserve unknown siblings, Codex typed patches stay in their shared owner, and source writes retain the originally read token. `runtime::Platform` is request identity, not the profile-domain enum: OpenCode requests must be rejected by Claude/profile adapters rather than mapped to Claude.

Usage/session consumers require all three absolute `--insights-usage-root`, `--insights-archive-db`, `--insights-provider-home` paths together. Missing usage storage is an explicit error, not zero usage. Preserve provider-owned validation, opaque archive IDs, keyset cursors and conservative fidelity; leave/suspend clears transcript plaintext. Tauri keeps its wire/TS DTOs and explicitly maps shared results.

Native fixture launchers must use the exact three path strings in `fixture.json`. On Windows, replacing a saved `\\?\D:\...` provider path with `D:\...` changes source identity even when both resolve to the same file. Verify fixture ownership using filesystem identity, preserve the original launch strings, and create a fresh fixture after a mismatched launch. Do not normalize domain IDs to hide a fixture error.

Command runners retain 64 jobs with a 30-minute terminal TTL, 500 lines and 512KiB per output stream. Admission and cancellation capability share one lock; terminal publication follows process-tree cleanup and bounded reader joins. The native fixture child is a closed synthetic mode under an ownership-marked outputs directory, never a renderer-selected executable. Tauri keeps its wire DTOs/events; shared `TimedOut` maps to existing wire `Failed` with timeout feedback. Native leave detaches its observer; return atomically restores the snapshot and notification attachment. The view applies snapshots at most every 100ms, rejects stale generation/job identity and never replaces a terminal snapshot with a late running state. Terminal revisit attaches no waiting task. Hidden pages do not poll.

Native foreground and background actions both use the retained streaming command owner and shared request validator. Foreground runs responsively in the current workbench; this does not assert wire/result equivalence with Tauri's one-shot capped foreground output. Help validates its target against the closed command policy and constructs only `help` plus that target; arbitrary argument drafts cannot enter this action.

Budget patches validate the complete object before any setter runs: booleans stay typed, limits are finite nonnegative numbers or null, warning percent is an integer0..100. Valid multi-field saves retain the existing sequential I/O semantics. Budget/pricing managers privately preserve unknown TOML siblings and surviving row extensions; explicit removal/reset must not resurrect deleted models. Archive refresh is app-owned, but a terminal callback may reload only the same Sessions query generation with no detail selected.

## 4. Validation & Error Matrix

| Condition | Required behavior |
| --- | --- |
| Missing catalog / corrupt or legacy-unmigrated source | Existing domain error; no automatic repair |
| Empty valid catalog | Successful empty list |
| Invalid stored current marker | Existing file/registry fallback precedence |
| Admission full / stopped | Explicit Full / Stopped; no unbounded queue |
| Slow worker / late completion | UI remains responsive; stale key rejected |
| Read view dropped / job view dropped | Cancel read / detach job, respectively |
| Progress queue full on completion | Retained terminal still delivered |
| Cleanup still running | No completed/cancelled claim before runner returns |
| Parser or path error displayed natively | Safe static explanation; no raw secret-bearing error text |
| Settings revision/source/existence changed | Conflict; retain draft and never silently overwrite |
| OpenCode request routed to profile-domain reader | Unsupported; never dispatch Claude |
| Usage storage missing / transcript provider unavailable | Typed safe error / truthful provider status; no fabricated data or default-home fallback |
| Prompt target/session environment or platform changed | Reject before write; retain original-token draft on conflict |
| Consumer sidecar context missing or release hash invalid | Fail closed before process spawn; never search PATH |
| Return to a command that completes during subscription attachment | Atomic observe returns terminal or delivers subsequent terminal; no gap between status and subscribe |
| Command progress receiver lags | Recover latest bounded snapshot, preserving terminal and explicit truncation counters |
| Late command event from another generation/job or after terminal | Ignore without changing current status |
| Invalid later budget field | Reject the entire supplied patch before the first setter; original bytes unchanged |

## 5. Good / Base / Bad Cases

- Good: call ConfigReadService on a blocking worker and map ConfigRow into the existing Tauri DTO.
- Base: keep a domain API directly consumed when no shared orchestration is needed.
- Bad: copy profile SQL, expose Secret inner values, instantiate a runtime per click, or equate page drop with job completion.

## 6. Tests Required

Assert all projection fields, masking, order, version, current fallback and description defaults against domain-created fixtures. Verify business bytes/mtime/revision unchanged, with precisely identified SQLite sidecar exceptions. Test missing/empty/corrupt/legacy sources independently and retain Tauri adapter parity/bindings checks.

Use barriers or virtual time for full admission, progress coalescing, full notification queue, cancellation, late responses, view drop and shutdown. Exercise actual rendered native controls for responsiveness; headless tests cannot prove OS IME, native accessibility, DPI or performance budgets. Preserve default Rust test parallelism and `--skip export_bindings` for behavior suites.

Quit fallback tests must hold real work across an expired asynchronous shutdown observer, then prove the final owner join waits for that work. Process-tree shutdown fixtures must confirm readiness and retain OS process handles through exit verification. If a synthetic descendant exits naturally after a fixed duration, bound the entire test interval from before job admission through shutdown and exit confirmation below that duration; timing only the final handle wait can falsely pass after shutdown waited for natural exit. This bound excludes natural completion and is not a performance receipt.

Settings fixtures must prove dirty-only retention, missing-versus-empty identity and two-client CAS. Usage fixtures compare exact values and filters against ccr-usage; session fixtures verify real provider pagination, stale request rejection and plaintext teardown. Native confirmation regressions must model callback-before-draw ordering: the locked GPUI platform drains frame callbacks before drawing, unlike Kit's click helper. Schedule one-shot entry from the mounted confirmation render, guard its generation, and verify Cancel entry, Tab containment, Escape, cancellation and navigation without seeding inside focus.

Run `cargo test -p ccr-app --locked -- --skip export_bindings` for resource policy/parity and command admission, cancellation-before-spawn, timeout, held-reader cleanup, output bounds and four-byte UTF8 truncation. Run desktop rendered resources/jobs/shell tests and Tauri `system_prompts`, `command_exec`, `process::` consumer tests. Binding checks retain transactional generation. Native prompt save/readback and synthetic process actions need their own exact candidate receipts; they do not prove real sidecar, Claude rules, install/sync/checkin or full resource parity.

Command observation regressions cover running detach/reconnect, terminal reconnect, notification lag, cancellation remaining nonterminal until the owner finishes, stale generation/job rejection and immutable terminal. Native foreground/help controls retain safe argument validation and never treat a malformed free-text draft as help argv.

## 7. Wrong vs Correct

Wrong:

```text
render -> ConfigManager.snapshot()
cancel click -> write terminal Cancelled
filter change -> accept whichever response arrives last
```

Correct:

```text
typed action -> bounded blocking read -> masked projection -> matching RequestKey -> local notify
cancel click -> request_cancel -> runner cleanup -> immutable completion
view drop -> close subscription; app-owned job continues under its owner
```

Wrong: schedule confirmation focus from the click action and assume next-frame callbacks see the newly mounted tab stops. Correct: record pending entry, schedule once from confirmation render, then enter on a later callback guarded by the same confirmation generation. Keep actual native retry separate from headless proof.

Editor and nested-decision return handles have different lifetimes. `ProfilesView::act` captures the outer editor trigger only from `Mode::Closed`; `Confirmation` captures its immediate trigger independently. A nested Advanced/Restore/Force decision must not replace the outer handle. Regressions must dismiss the decision and then close the editor, asserting both focus returns.

Recovery actions and committed warnings belong to their original platform. `ProfilesView::set_platform` clears `grok_recovery`, `grok_force_delete`, message and failed state on an actual identity change, before a new platform can dispatch. Same-platform status reads retain durable warnings. Test a typed `rename_apply_failed` outcome, same-platform refresh, then platform change: old retry/force controls disappear before any mutation; never send a wrong-platform request to demonstrate the defect.
