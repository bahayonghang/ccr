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

## 3. Contracts

ConfigRow preserves the existing Tauri list fields and defaults: snapshot version, profile order, empty description/base URL, enabled/usage metadata, default/current markers and optional provider/model/account/tags. Credentials pass through the existing Secret Display mask before entering this projection. Tauri keeps its DTO derives, wire shape, export location and error mapping.

Current selection uses the ccr-config-owned file/registry resolution helper, also consumed by CLI. Missing or corrupt catalogs remain errors; reads do not initialize, migrate, repair or acquire repository write locks. Existing SQLite opening can create WAL/SHM sidecars; this is distinct from a business write and must be reported accurately.

Runtime admission is 128 operations, blocking concurrency 4, notification capacity 128, and latest progress is coalesced at 100ms intervals. Terminal state is retained separately and cannot be lost behind progress. Only the runner publishes its immutable completion, after cleanup. `request_cancel` is a request, not a terminal outcome.

Dropping a read subscription requests cancellation and closes its receiver. Dropping a job subscription only detaches the view; the app retains its runner. Shutdown stops admission, requests cancellation and awaits tracked work. RuntimeOwner must outlive the native event loop; its synchronous final join runs after that loop returns. GPUI's 200ms quit-hook budget does not prove cleanup completed.

GPUI render contains no file/DB/network/process calls, runtime construction or block_on. Path resolution and list reads run on the blocking worker. Retain selected records by domain key and reject old environment/generation/request results. The current configuration adapter supports Local/Claude only; unsupported requests must not silently read Local data.

Native fixtures use explicit profile/registry paths and domain API-generated synthetic data. Artificial read delay is accepted only with paired fixture paths, at most 30000ms, and visibly labeled. Never use real accounts or mutate global HOME/USERPROFILE to isolate tests.

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

## 5. Good / Base / Bad Cases

- Good: call ConfigReadService on a blocking worker and map ConfigRow into the existing Tauri DTO.
- Base: keep a domain API directly consumed when no shared orchestration is needed.
- Bad: copy profile SQL, expose Secret inner values, instantiate a runtime per click, or equate page drop with job completion.

## 6. Tests Required

Assert all projection fields, masking, order, version, current fallback and description defaults against domain-created fixtures. Verify business bytes/mtime/revision unchanged, with precisely identified SQLite sidecar exceptions. Test missing/empty/corrupt/legacy sources independently and retain Tauri adapter parity/bindings checks.

Use barriers or virtual time for full admission, progress coalescing, full notification queue, cancellation, late responses, view drop and shutdown. Exercise actual rendered native controls for responsiveness; headless tests cannot prove OS IME, native accessibility, DPI or performance budgets. Preserve default Rust test parallelism and `--skip export_bindings` for behavior suites.

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
