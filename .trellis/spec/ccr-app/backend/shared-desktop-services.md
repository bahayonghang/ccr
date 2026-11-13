# Shared Desktop Services

## 1. Scope / Trigger

Applies to `crates/ccr-app`, the Tauri configuration-list adapter and GPUI service/runtime consumers. This contract accompanies GPUI task 02; actual verification and remaining gaps belong in its evidence, not in this spec.

Extract only orchestration that both consumers use. Do not move domain persistence, SQL, credential handling, command policy or process cleanup into a second implementation. ccr-app must not depend on the ccr umbrella, Tauri, GPUI or WebView. ccr-usage remains the sole usage SQL owner.

Incremental session archive refresh can skip an unchanged container/group/member only while its corresponding archive rows are live. Restored source bytes and mtime may exactly match the previously deleted source; missing/deleted archive state must still be parsed and upserted to live. Preserve absent deleted entries and exact path/member identities. Live unchanged sources retain their fast path; do not invalidate the whole fingerprint cache to restore one session.

## 2. Signatures

- `config::ConfigReadService::for_claude() -> ccr_core::Result<Self>` constructs paths.
- `ConfigReadService::from_paths(profiles_path: PathBuf, registry_path: PathBuf) -> Self` injects fixture/embedding paths.
- `ConfigReadService::list(&self) -> ccr_core::Result<Vec<ConfigRow>>` performs blocking domain reads.
- `runtime::RuntimeOwner::{new, handle, shutdown}` owns one long-lived Tokio runtime.
- `RuntimeHandle::read(key, work)` takes a blocking read returning `Result<T, E>`.
- `RuntimeHandle::job(key, work)` takes a runner returning `Completion<T, E>` after its cleanup.
- `RuntimeHandle::job_with_finalizer(key, work, finalize)` additionally joins adapter-owned terminal persistence, including when queued work never starts.
- Both return `Result<TaskSubscription<T, E>, AdmissionError>`.
- `RequestKey` contains environment, platform, generation and request_id. `ResponseGate::accepts` compares all four.
- `platform_settings::Document::{read, save_text, save_validated}` uses a non-Debug `Snapshot` carrying original content, token, source identity and existence; saves return `SaveOutcome::{Saved, Conflict}`.
- `usage::read_dashboard(&Dashboard, &QueryFilter, Option<u32>)` assembles owner projections without adding SQL. `sessions::{list_sessions, get_detail_with_registry}` consumes the existing archive repository and provider registry.
- `usage::capabilities::Report::detect(&AppPaths, probe_cli)` merges the existing `ccr-usage` database capabilities with the closed installed-CLI version probe; Tauri retains its wire DTO and native displays structured support/reason rows.
- `resources::system_prompts::{platform_specs, prompt_spec, file_description, augment_save_result}` owns the fixed instruction-file registry and legacy result policy. Native writes use `save_prompt_file_with_locks` with the original snapshot token and captured lock directory; Tauri retains its raw-file error adapter.
- `jobs::policy::{command_catalog, validate_command_request}` and `jobs::commands::CommandJobs` are the common command policy and retained runner. `jobs::process::SidecarContext` supplies the consuming executable, repository root, debug policy, expected SHA256 and package version.
- `CommandJobs::observe(self: &Arc<Self>, id) -> Result<(Snapshot, Observation), Error>` attaches notification and captures retained state under the same lock. `Observation::next` treats deltas as wakeups and reloads authoritative snapshots, including after bounded-channel lag.

## 3. Contracts

ConfigRow preserves the existing Tauri list fields and defaults: snapshot version, profile order, empty description/base URL, enabled/usage metadata, default/current markers and optional provider/model/account/tags. Credentials pass through the existing Secret Display mask before entering this projection. Tauri keeps its DTO derives, wire shape, export location and error mapping.

Current selection uses the ccr-config-owned file/registry resolution helper, also consumed by CLI. Missing or corrupt catalogs remain errors; reads do not initialize, migrate, repair or acquire repository write locks. Existing SQLite opening can create WAL/SHM sidecars; this is distinct from a business write and must be reported accurately.

Runtime admission is 128 operations, blocking concurrency 4, notification capacity 128, and latest progress is coalesced at 100ms intervals. Terminal state is retained separately and cannot be lost behind progress. Only the runner publishes its immutable completion, after cleanup. `request_cancel` is a request, not a terminal outcome.

Dropping a read subscription requests cancellation and closes its receiver. Dropping a job subscription only detaches the view; the app retains its runner. Shutdown stops admission, requests cancellation and awaits tracked work. RuntimeOwner must outlive the native event loop; its synchronous final join runs after that loop returns. GPUI's 200ms quit-hook budget does not prove cleanup completed.

Command Recent persistence belongs to the retained admitted ticket, not a visible view observer. The finalizer reacquires the same bounded blocking permit without skipping on cancellation and is awaited before executor terminal publication/shutdown completion. It holds the runner's owned terminal snapshot; queued/unavailable admission uses `CommandJobs::abandon_snapshot` to capture the terminal under the registry lock. Never re-read an evictable job ID after waiting for finalizer admission: capacity/TTL pruning may already have removed it. A persistence panic or write failure cannot replace the command's immutable process outcome. Replayed admission and repeated observation do not create another finalizer. Native history deliberately omits arbitrary argument values from both args and full_command at rest; UI-only redaction is insufficient. An admitted command that never launches keeps its actual unavailable/non-success outcome.

The GPUI main shell installs a per-window `on_window_should_close` guard. The default policy opens at most one confirmation in the existing dialog stack and vetoes immediate removal; Cancel/Escape preserve the underlying editor, dialog and app-owned work. Until dirty state is reliably aggregated across domains, the default also confirms idle exits. Settings, application-menu Quit and Ctrl/Cmd+Q route through the same guard. An explicit persisted “Quit directly” preference bypasses confirmation with a visible unsaved-change warning. Both choices retain the last-window quit observer and final runtime join. Probe, startup-failure exits and platform/application-level `App::quit` that directly invokes shutdown remain outside this guard: locked GPUI 0.3.9 exposes no public quit-veto hook. Do not request shutdown on opening or cancelling a dialog. Wording promises a cancellation request and waiting for work, not guaranteed cancellation or successful cleanup. Deferred initial focus belongs to that opening and must not steal focus after dismissal, reopening or user input.

GPUI theme, language and close policy persist only in the independent `ccr/desktop-gpui/preferences.json` configuration namespace. Reads and atomic writes use bounded workers. Generation ordering prevents an older queued save from overwriting a newer choice. Malformed, newer-format or unreadable files remain intact with explicit error feedback; a failed save is not reported as persisted. Fixture registry paths must be absolute and preferences remain under that fixture parent, with no HOME fallback. “Show preferences file” reveals the captured path only after successful persistence; requesting an OS reveal is not proof of delivery.

Window restore uses the same optional preference document and the same native window stack. Read it on the bounded worker before creating the window, validate stored bounds against current displays, and retain visible load errors instead of overwriting invalid files. Observe actual window bounds; debounce persistence without inventing a file watcher. A confirmed close waits for the final save's terminal before removing the window. Bound queue waiting so four long-running workers cannot deadlock exit: cancel only the queued final save on expiry, await its real terminal, then follow normal shutdown. An already-started atomic write still finishes under its owner. Failure may lose the latest window placement but must not trap an authorized quit or claim successful persistence. Theme/language/close edits retain the window state.

Claude rules and output styles retain their actual file sources; agents, slash commands and plugins retain the existing typed settings collections. These are different storage contracts, not interchangeable Markdown files. Native edits capture roots and original versions, preserve unknown siblings and omitted credentials, and use guarded writes/backups. Resource source switching must not rebind an active draft. Codex agents and source installation share one extracted owner with Tauri; source reads use captured roots/client, cache reads do not fetch the network, and filenames/tracked targets cannot escape the agent root or follow links. Installation and tracking changes reuse the existing write journal. App-owned results retain safe outcomes, while leaving the resource view clears editor plaintext, previews and source snapshots.

Builtin/user prompt templates fill the existing instruction editor draft; applying a template is not an implicit file write. The original target revision survives until explicit Save. User preset storage reuses the domain parser/schema under a captured root and guarded document write. Only supported domain targets get a preset library; unsupported OpenCode preset requests must not map to Claude.

MCP presets reuse the domain catalog/spec projection and the existing source-bound writer. Cross-source copy keeps source plaintext in an opaque backend session, captures both source and target versions, and confirms the selected target before mutation. Translate only supported transport/header/enabled representations; reject unsupported or unknown options instead of silently dropping them. Existing destination siblings survive, and copied credentials never populate UI draft controls or retained operation snapshots.

Native multi-target MCP presets and whole-source synchronization capture an opaque source session and each original destination revision. Plan Add/Replace per entry; replacement defaults off and requires explicit confirmation. Commit each destination through the original writer's one `save_batch`, preserving unrelated servers. Unsupported entry conversions and target write conflicts remain individual failures; prior successful targets are not described as rolled back. Source changes reject the reviewed batch instead of silently rereading newer credentials.

Codex agent copy validates the original source token under the store lock and parses that same snapshot; a renderer-side precheck followed by a fresh unversioned read is insufficient. Switching a resource scope/catalog clears the previous projection before the new read, including failure paths, and an active editor remains bound to its original scope. New resource and automatic preference I/O reject linked/reparse ancestors as well as linked leaf paths; resource backup and lock locations follow the same boundary. Test actual scope-button behavior when Kit does not export a disabled observation value.

GPUI render contains no file/DB/network/process calls, runtime construction or block_on. Path resolution and list reads run on the blocking worker. Retain selected records by domain key and reject old environment/generation/request results. The current configuration adapter supports Local/Claude only; unsupported requests must not silently read Local data.

Native fixtures use explicit profile/registry paths and domain API-generated synthetic data. Artificial read delay is accepted only with paired fixture paths, at most 30000ms, and visibly labeled. Never use real accounts or mutate global HOME/USERPROFILE to isolate tests.

Native platform operations retain captured roots through commit; fixture roots never fall back to HOME. JSON dirty-leaf updates preserve unknown siblings, Codex typed patches stay in their shared owner, and source writes retain the originally read token. `runtime::Platform` is request identity, not the profile-domain enum: OpenCode requests must be rejected by Claude/profile adapters rather than mapped to Claude.

Codex auth process warnings run only on explicit Save review and use the existing captured-root auth owner without reading authentication just to enumerate processes. Its cached best-effort empty PID result is an observation; failed/unperformed diagnostics remain unknown. Callback checks bind only 127.0.0.1:1455 and distinguish unavailable checks from address-in-use; OAuth startup retains its final bind. Host diagnostics reject fixture/benchmark, remote and wrong-platform requests before roots or probes. Confirmed cleanup consumes an opaque captured pending plan, joins commit-aware controller cleanup, then uses the closed port-discovery descriptor. Validate status, timeout, both stream truncation flags, stderr warnings and UTF-8 before parsing exact local listener endpoints. Request cancellation only on registry-owned matching handles; preserve unknown PIDs for manual action, and distinguish cancellation requests from completed exits or port release. Confirmation focus enters safe Cancel after mount with generation checks and returns to its original trigger on dismissal.

Usage/session consumers require all three absolute `--insights-usage-root`, `--insights-archive-db`, `--insights-provider-home` paths together. Missing usage storage is an explicit error, not zero usage. Preserve provider-owned validation, opaque archive IDs, keyset cursors and conservative fidelity; leave/suspend clears transcript plaintext. Tauri keeps its wire/TS DTOs and explicitly maps shared results.

Native fixture launchers must use the exact three path strings in `fixture.json`. On Windows, replacing a saved `\\?\D:\...` provider path with `D:\...` changes source identity even when both resolve to the same file. Verify fixture ownership using filesystem identity, preserve the original launch strings, and create a fresh fixture after a mismatched launch. Do not normalize domain IDs to hide a fixture error.

Command runners retain 64 jobs with a 30-minute terminal TTL, 500 lines and 512KiB per output stream. Admission and cancellation capability share one lock; terminal publication follows process-tree cleanup and bounded reader joins. The native fixture child is a closed synthetic mode under an ownership-marked outputs directory, never a renderer-selected executable. Tauri keeps its wire DTOs/events; shared `TimedOut` maps to existing wire `Failed` with timeout feedback. Native leave detaches its observer; return atomically restores the snapshot and notification attachment. The view applies snapshots at most every 100ms, rejects stale generation/job identity and never replaces a terminal snapshot with a late running state. Terminal revisit attaches no waiting task. Hidden pages do not poll.

Native foreground and background actions both use the retained streaming command owner and shared request validator. Foreground runs responsively in the current workbench; this does not assert wire/result equivalence with Tauri's one-shot capped foreground output. Help validates its target against the closed command policy and constructs only `help` plus that target; arbitrary argument drafts cannot enter this action.

Native portable packaging must include the command owner’s matching CLI sidecar. `desktop-package` builds the root `ccr` binary with `--locked` into the validated desktop `target/sidecar`, checks its exact package version, hashes it, and injects `CCR_SIDECAR_SHA256` only into the GPUI release build subprocess. The ZIP contains both sibling executables and their hashes; archive hashes are checked before atomic promotion. A failed build, version mismatch, path escape, or changed executable must leave the previous package intact. A GPUI-only executable is not a complete command-workbench package. `desktop-dev` forwards argv directly through both recipe layers, including fixture paths containing spaces. Python command regressions cover these boundaries; a mock build does not prove release packaging or native operation succeeded.

Budget patches validate the complete object before any setter runs: booleans stay typed, limits are finite nonnegative numbers or null, warning percent is an integer0..100. Valid multi-field saves retain the existing sequential I/O semantics. Budget/pricing managers privately preserve unknown TOML siblings and surviving row extensions; explicit removal/reset must not resurrect deleted models. Archive refresh is app-owned, but a terminal callback may reload only the same Sessions query generation with no detail selected.

## 4. Validation & Error Matrix

Native MCP writes share the extracted `ccr-app::claude_mcp` policy and `mcp_document` codecs with Tauri. An editor captures source, scope and original revision; save/delete must not follow later selection changes. Preserve unknown fields and omitted secrets, and do not prefill secret values into the patch editor. Legacy Claude settings are read-only; OpenCode uses its platform source editor. Tauri command signatures and payloads remain unchanged.

Ordinary MCP env/header replacements with an empty value are omitted even when the user entered a name; explicit clearing remains the Advanced patch protocol. Switching HTTP to a stdio command removes the known mutually exclusive URL/type fields while retaining unrelated extension siblings. Confirmations follow the same generation-guarded post-mount safe-button focus contract as other native editors.

Native WebDAV settings and folder CRUD preserve unknown TOML fields, credentials and unrelated folders through the original document revision. Transfers use the existing `ccr-sync` owner and its backup/transaction behavior. Ordinary folder transfer retains the existing plaintext contract with visible content disclosure; it is not the sensitive-assets encrypted protocol. A transfer without safe cancellation must not expose a fictitious Cancel action. Retain app-owned terminal results across navigation and invalidate a confirmation when its target/settings change.

Native check-in operations capture one database access and crypto root through provider/account/record/balance/WAF managers and nested batch services. Existing global constructors remain compatible for other consumers. Fixture initialization uses explicit isolated storage; schema migrations receive only an empty fixture-owned home, never the real user home. Cookies remain encrypted and a blank edit preserves existing credentials; provider deletion with accounts is rejected. Default fixtures deny network. Loopback tests require an explicit test-only client with proxy and redirect disabled; this is not evidence for external providers or browser WAF login.

Opening an existing check-in account editor may read only that account's original credentials into its masked inputs. Keep the plaintext DTO out of retained management snapshots/logs; bind each response to the editor request and account, and discard it on replacement, cancellation or leave. Missing crypto keys fail without creating replacements. Fixture/Benchmark crypto creation and synchronous account/import/CDK writes use captured lock scopes; never carry thread-local lock scopes across await/spawn. An exporter's internal file-write scope does not cover earlier crypto initialization. Revalidate retained import source paths against the current fixture before the domain owner reads them.

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
| Queued command cancelled or view detached before runner starts | Retained finalizer still persists its actual terminal; final shutdown joins the write |
| Recent persistence fails or panics | Preserve process outcome; do not replay an ambiguous write |
| Invalid later budget field | Reject the entire supplied patch before the first setter; original bytes unchanged |

## 5. Good / Base / Bad Cases

- Good: call ConfigReadService on a blocking worker and map ConfigRow into the existing Tauri DTO.
- Base: keep a domain API directly consumed when no shared orchestration is needed.
- Bad: copy profile SQL, expose Secret inner values, instantiate a runtime per click, or equate page drop with job completion.

## 6. Tests Required

Assert all projection fields, masking, order, version, current fallback and description defaults against domain-created fixtures. Verify business bytes/mtime/revision unchanged, with precisely identified SQLite sidecar exceptions. Test missing/empty/corrupt/legacy sources independently and retain Tauri adapter parity/bindings checks.

Use barriers or virtual time for full admission, progress coalescing, full notification queue, cancellation, late responses, view drop and shutdown. Exercise actual rendered native controls for responsiveness; headless tests cannot prove OS IME, native accessibility, DPI or performance budgets. Preserve default Rust test parallelism and `--skip export_bindings` for behavior suites.

Finalizer regression tests hold a queued cancellation through finalization and prove both terminal publication and shutdown wait for its release; a finalizer panic preserves the runner's failure. Native Recent regressions inspect raw persisted rows after detach/shutdown and repeated admission, plus managed synthetic success/failure/cancel outcomes. Arbitrary positional arguments must be absent before persistence, rather than merely masked by a later read projection.

Quit fallback tests must hold real work across an expired asynchronous shutdown observer, then prove the final owner join waits for that work. Process-tree shutdown fixtures must confirm readiness and retain OS process handles through exit verification. If a synthetic descendant exits naturally after a fixed duration, bound the entire test interval from before job admission through shutdown and exit confirmation below that duration; timing only the final handle wait can falsely pass after shutdown waited for natural exit. This bound excludes natural completion and is not a performance receipt.

Main-window close regressions dispatch the installed close callback and exercise rendered Cancel/default focus, Tab/Shift+Tab containment, Escape, repeated requests, cancel-then-reopen and explicit confirmation. Default Cancel focus alone is insufficient: dispatch Enter before deferred focus entry and on both decision buttons; the dialog's generic Enter binding must not confirm from its root or while Cancel is focused. Verify that cancelling retains drafts, underlying confirmations and active work without requesting runtime cancellation; only confirming removes the window. Wait for rendered controls to become enabled rather than treating zero runtime tasks as proof that UI completion has rendered. These headless interactions do not establish native OS-close or cross-platform acceptance.

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


### Platform file resources and merged Claude settings

- OpenCode agents/commands use the existing `.config/opencode` user and `.opencode` project Markdown roots, not the raw settings document or an invented home directory. Antigravity commands retain the Gemini-compatible `.gemini/commands` TOML contract. Extension manifests are read-only projections of the existing extension list; no install/remove/edit capability is inferred.
- Dynamic resource edits capture kind, scope, path and original version. Preserve unknown YAML/TOML fields; reject traversal and linked ancestors. Desktop and Tauri consume the same app owner, while Tauri command signatures remain stable.
- Claude statusline/hooks remain reachable through the same captured settings-source editor. Validate a candidate with `ccr_types::ClaudeSettings` without serializing that typed projection over the original JSON; otherwise unknown fields would be lost. A page/worker existing in isolation is insufficient: policy editing must be reachable from the actual Shell Usage route.

- Platform file save/delete must validate backup and lock directories, including every ancestor, as well as the destination. A safe document path does not make sibling backup/lock paths safe.


### Native debug render stack

GPUI/Kit value builders can reserve hundreds of KiB of debug stack per large render function, even before a conditional branch executes. Windows native main uses the executable's 1 MiB stack while ordinary Rust test workers may have more; passing headless tests does not prove a native menu can render. For a reproduced overflow, inspect compiler frame allocation and split the specific render into natural sections without changing tree order or handlers. Do not add a larger product stack to mask an unknown cause, or add independent function frames as if they were a measured call stack. Preserve the Windows Settings subprocess regression at 1 MiB and verify the actual native action after rebuilding the executable.


### Fixture locks and filesystem containment

- A captured fixture database and known_hosts file do not isolate hidden lock writes. SSH trust confirmation, connect-time known_hosts refresh and upload staging must also use the fixture lock directory. Enter the thread-local scope inside the actual blocking write closure; it does not propagate across `spawn_blocking` or an await boundary. Production defaults/public command signatures remain unchanged.
- Validate fixture containment using actual canonical filesystem paths (or the canonical existing parent for a new file), not the lowercased normalized lock identity. On Windows those are different representations and case-sensitive directories may exist. Keep traversal and linked-path rejection; missing fixture parents are refused rather than falling back to Local.
- Shared-writer tests must capture their own thread-local lock directory, instead of depending on another parallel test's temporary `CCR_LOCK_DIR`. A child-only poisoned default lock path can prove the complete fixture operation never falls back to ambient user locks, without changing HOME/USERPROFILE or serializing the suite.

### Native usage import and WebDAV management

- Native usage import consumes the same installed `llmusage` CLI executor as Tauri through `ccr-app::usage::import`; `ccr-usage` remains the usage SQL owner. Opening or refreshing a report must not start import. Missing databases must not prevent an explicit first import.
- Capability reads do not create/migrate storage or start import. Explicit fixture policy disables CLI lookup/execution before any ambient resolver, rather than claiming the host binary is absent. Reject unsuccessful, timed-out or truncated version probes. Database feature reasons distinguish missing/unreadable storage, schema, table and column support. Do not display raw error/detail strings in native capability rows; native CLI version display is redacted and bounded. Upstream graceful-cancel capability remains separate from the desktop's local tracked process cancellation.
- Native imports expose the installed CLI protocol's source filter and date window. A platform-scoped view binds that source through admission and retained completion. Non-Claude imports do not ingest Claude tool files. Never invent a custom-home CLI flag; an unsupported provider home is rejected before spawn.
- An import is app-owned, singleton and retained across view detachment. Cancel requests remain nonterminal until process cleanup returns; cleanup failure is distinct from cancellation. Progress publication is coalesced at the existing 100ms budget and final state is retained independently of observers. Hidden views detach instead of polling.
- The CLI cannot select an arbitrary provider home: reject custom provider roots before admission rather than scanning ambient local data. Explicit fixture launches reject production import; synthetic test subprocesses require captured fixture roots. Environment rejection applies before process launch.
- After an explicit successful CLI import, ingest Claude tool facts from the same captured provider home through the shared scanner. Changed JSONL files are reread with stable sequence identities; file/line limits are 64 MiB/1 MiB, including cumulative bytes if a file grows after stat. Commit each file's tool rows and cursor in one repository-owned transaction. Cancellation/error must retain truthful already-imported usage and per-file committed state; report tool failures separately without exposing raw paths/errors in UI snapshots.
- WebDAV disconnect binds both canonical and legacy configuration revisions. Clear canonical credentials using the existing write/backup owner, then remove the legacy source; report legacy cleanup failure as partial completion. Keep folders, unknown fields and protected backups, and disclose that backups may retain credentials.
- Bulk folder transfers select only enabled canonical folders and delegate to the existing single-folder transfer owner. Preserve each result and download backup; a failed folder must not be described as full-batch success or imply rollback of previously completed folders.

### Native observer and session operations

- Global monitoring is separate from Claude Observer. Its app-owned feed captures the main UI DB (fixture `observer.db`), never the usage archive or a fallback global pool. Connection-bound repository methods own SQL and conversion. Native strict decoding rejects malformed rows and timestamps; established Tauri tolerant decoding is preserved independently.
- Lifecycle owners emit only fixed event labels after actual outcomes. The native memory feed is bounded to 500 entries and 512 KiB of UTF-8 payload; sanitize and bound display fields before memory/persistence, omit arbitrary metadata and correlation payloads, and cap merged visible output independently. Failed persistence retains memory events and explicit error state. Opening monitoring does not create/migrate storage.
- Monitoring shows tracked tasks, actual optional current-process RSS, a bounded successful Usage-page report-read p95 with retained sample count, and retained SSH session authority0/1. Other latency cohorts remain unavailable. Refresh is explicit, late generations are rejected, and suspend drops the subscription with no hidden timer. Tests use captured main/archive fixtures, malformed storage rows/time, secret redaction, bounds and actual view transitions.
- Native host system information is a separate explicit read in the existing Monitoring page. The bounded-worker adapter uses the pinned sysinfo kernel owner, refreshing only CPU and memory; do not use new_all/refresh_all or enumerate processes for this request. CPU usage is the mean of reported logical CPU usages after two refreshes separated by at least MINIMUM_CPU_UPDATE_INTERVAL, matching the Tauri calculation. Preserve raw memory/swap bytes and optional missing fields; display GiB accurately and keep host RAM distinct from process RSS. Missing/zero capacity has no invented used percentage; the logical CPU count is not a physical-core count. Show the UTC observation time and actual CPU observation interval, never label that interval as command latency/p95. Reject non-Local requests and Services::is_fixture() (including Benchmark) before any host probe. Construction, feed refresh and hidden views do not sample the host; suspend cancels the read subscription, clears facts and rejects late generations. Tests use synthetic DTOs only for readable units/busy/stale/suspend controls, and verify actual fixture/benchmark/remote refusal without inspecting real hardware.

- Claude Observer keeps its databases distinct: usage reports come from the read-only `ccr-usage` dashboard; tools/subscription settings come from the captured CCR main UI database, never the session archive. Native and Tauri share state-free projection/subscription policy; framework DTOs and binding paths remain Tauri-owned.
- Tool/session cost aggregates are nullable: all unpriced or partially priced groups remain unknown; an explicitly recorded zero stays zero. Session token/model/start metadata stays absent when tool events cannot supply it. Default session ranking uses call count, applied before the limit; explicit cost ranking puts unknown costs last. No session-cost SQL is added to llmusage or the Claude Workbench.
- Observer tool reports use one captured `ReportClock.as_of` for live and fixed inclusive instant windows, excluding future events. Heatmap weekday/hour grouping converts each timestamp in its historical report timezone (`local` or `utc`), which both UIs label. Missing/invalid tool-event timestamps are skipped and counted per changed-file scan, never replaced with scan time. Skipped tool uses retain their sequence positions; rescanning a changed file removes those known-invalid positions together with valid upserts and its cursor in one transaction. Unchanged-file cursors still skip parsing; this does not claim a historical rebuild of previously cached files.
- Opening Observer does not create or migrate a database. An explicit confirmed first subscription Save may initialize the captured main database with its existing migration owner. An unreadable or incompatible existing file is an error, not a missing-database setup case. Subscription writes compare the reviewed values inside the same write transaction; they remain app-owned after the view detaches and work independently of usage-database availability.
- Native subscription reads preserve all three states: missing keys, explicitly saved values (including defaults), and storage/invalid-value failure. `read_saved`/`save_saved_if_current` compare presence and values in the transaction; legacy Tauri methods retain their existing defaults contract.
- A subscription save conflict keeps the draft, marks the stored revision stale and refuses further submissions until the stored values are re-read; a fresh save then compares against the re-read revision. A clean rollback of a profile save keeps the draft and its original revision and never reports the save as successful.
- Claude Workbench captures one environment, platform, report clock/timezone and generation for today's, month-to-date, imported-history and 30-day detail projections. SQL stays in `ccr-usage`; rendering performs no IO. Missing rows differ from measured zero; output already includes reasoning, and zero cache denominator is unavailable.
- Platform MCP authority is captured by scoped `Services`. Editor, mutation, copy source/target and batch commit validate actual resources independently of `RequestKey.platform`. Global MCP keeps cross-platform capability. Non-Codex session trash is rejected at event and command admission; global/Codex retain existing local-source and revision checks.
- Native Claude resources resolve the captured Claude config root, including `CLAUDE_CONFIG_DIR`, for both fixed instruction files and Markdown stores. Extensions discover bounded local agent/skill/legacy-command metadata and the plugin installation registry read-only; skills metadata uses `ccr-skills`. Each source reports missing/empty/error independently, and unsafe paths are reported separately from unreadable ones. A legacy settings array that is absent is reported as absent, never as zero entries. Legacy settings arrays are preserved and labelled unread by Claude. No discovery performs installation, refresh, network or execution, and discovery never proves client loading.
- Tauri output-style storage also resolves `ClaudeRuntimePaths.config_dir`; its backups stay in the CCR Claude backup directory (`output-styles` under `PlatformPaths(Claude).backups_dir`, not the Claude config root) and locks keep their captured root. Installed fixtures override ambient roots/locks inside storage; their existing command capability refusal is retained. Batch upsert preserves omitted styles, guarded backups and atomic writes. Process-isolated tests exercise the real commands with a custom config directory and verify the default home remains byte-identical.
- Codex session actions start with an opaque archive identity, restore and validate the provider source, then bind the captured Codex home, source path and bounded content-version token. Filesystem preparation runs on the bounded worker. Confirmation must not silently target a newer source or a different selected session.
- Native destructive session removal uses the existing manifest-backed Codex trash service, preserving its index repair/rollback and CLI recovery contract. Do not add a second trash format or replace it with direct deletion. Archive deletion markers and any failure after the file operation must report the actual committed/partial state.
- Export uses the existing Markdown projection and exposes its suggested filename/truncation. Transcript preview is view-scoped, explicit clipboard copy is user-triggered, and leaving the view clears the temporary preview; do not retain transcript plaintext in app-owned mutation snapshots.
- Explicit Markdown file save captures the reviewed preview, uses a private absent-target guarded write, rejects linked destinations and confines fixture output to the captured root. A cancelled or stale native dialog admits no writer. Trash metadata UI renders a bounded page with stable identity and persistent Previous/Next controls; do not construct every row for a large collection.

### Native platform management

- Grok workbench activation comes from `inspect_activation_state`, including Drifted and UnsafeMissingEntryState; do not infer it from a Current badge, credential presence, or default profile. Credential mode describes the recorded profile's source, not verified remote login. Read-only overview must not reconcile pointers, repair runtime files, or activate defaults. Keep enabled/total profile counts distinct and reject remote/wrong-platform requests before resolving local paths.

- Captured file-based authentication off passes the reviewed credential token to guarded compare-delete. Its rollback restores only into an absent target; never overwrite a concurrent credential rotation. Legacy APIs remain compatible and official macOS logout retains its own session semantics.
- Grok account operations consume captured runtime/store/lock/backup roots. Authentication off must preserve the domain service's rotated-credential save and rollback policy; direct deletion of the runtime auth file is not an equivalent implementation. Enter scoped locks inside the actual blocking owner call.
- Codex provider-library updates compare the reviewed file version under the existing domain lock and bind the guarded final write to the same source revision. Omitted/blank credential input preserves existing keys; updates preserve creation metadata. The existing typed store does not promise raw unknown-field persistence.
- Codex quota refresh is an explicit app-owned operation: the existing service may refresh credentials and persist observations. Detaching a view is not cancellation. Reject fixture network work before constructing the live service, and keep failures/credential payloads out of display snapshots. Both quota and token-refresh HTTP requests have a 30-second request deadline. A multi-account batch can exceed five seconds overall; final runtime join waits for actual completion, so this is not a blanket five-second exit receipt. The model catalog uses the shared existing builtin/custom merge policy without inventing remote availability.

### Common Benchmark admission and default report calendar

- GPUI and Tauri explicitly select the same `benchmark::Seed` owner and their own
  paired endpoint. It validates files before either client creates effect owners;
  a legacy Fixture marker never silently selects Benchmark. Main-file digests
  cannot attest committed WAL, so reject nonempty WAL/journal for every DB role
  without repair/checkpoint; sidecar links, nonfiles and metadata errors also
  fail closed, including S0's absent catalog. SHM data is not hashed but its path
  and regular-file type remain validated.
- ReportClock is report/calendar context, not elapsed/process time. Default
  Benchmark workbench, Home/Observer and frontend presets consume its UTC date;
  preserve live Local/legacy wrappers and explicit request/store dates/timezone.
  Heatmap keeps its existing independent rolling-days semantics. Do not claim it
  obeys other panels' manual ranges. Do not override Date, timers or OS clocks.
- Keep SQL in ccr-usage/ccr-db owners. Fixed timestamp upper bounds must preserve
  subsecond distinctions supported by SQLite: `datetime()` truncates seconds and
  can incorrectly include an event 1ms after as_of. Use the established numeric
  time comparisons and test inclusive as_of plus excluded +1ms; do not rewrite
  legacy live predicates as part of adding a fixed clock.
- A Rust-accepted UTC leap second is not representable by frontend Date. Reject
  it in shared Seed before startup, instead of letting the frontend fall back to
  a host date. Canonical report JSON is read-only and never contains source paths
  or executable payloads. Readback success is not native/performance acceptance.


## Fixed S2 runtime source

### 1. Scope / Trigger

Changes to `benchmark/s2.rs`, its retained desktop adapters, source admission or stage accounting. This source is an engineering mechanism, not a performance result.

### 2. Signatures

- `Owner::admit(&Arc<Self>, &Seed) -> Result<Run, &'static str>`; `snapshot() -> Snapshot`; `observe() -> watch::Receiver<Snapshot>`.
- `request_cancel(run_id) -> Result<Snapshot, &'static str>`; `request_shutdown()`; `Run::drive(cancel, deliver, flush) -> Option<&'static str>`.
- `complete_joined(error) -> Result<Snapshot, &'static str>` is called only after the actual driver joins and adapter final IO/scoped tracking finishes. `Ack` separates ring, eviction, DB attempted/committed/failed/pending and per-input emit fields.

### 3. Contracts

- Only explicit validated Benchmark/S2 reaches admission. Local, old Fixture and S0/S1 reject before owner allocation/effects. Process-wide once is consumed on admission and never reset, including a later executor admission failure.
- Fixed production:1000 intended millisecond slots/s,60000 slots/60s,queue128,persist=true. Shortened profile exists only in this crate's cfg(test); adapter tests use production and actively cancel. No editable rate/duration, autostart or prefilled array.
- Real Instant controls skip-to-current-slot deadlines, elapsed and cleanup. Never catch up in a burst. Attempted slots generate deterministic MonitoringEntry IDs `benchmark-s2-v1-` + five-digit slot and data timestamp captured as_of + slot ms. Report clock never paces work.
- ACK only actual own IDs/tickets; ordinary unit flush of a mixed batch must settle S2 tickets too. Keep pending tickets bounded128. Unit call, queue acceptance or empty buffer is not a durable write. Native preserves its existing per-entry writes; Tauri retains threshold20/final batches.
- Every active projection obtains a fresh monotonic revision under the state lock, including equal millisecond readings. Coalesced watch reports use the existing100ms cadence plus phase transitions. Actual join stores one immutable terminal DTO/revision.
- Attempt rate=attempted/real producer seconds. Delivered and committed rates=actual cumulative ACK counts/total seconds through drain,flush,join. Source time stops with producer; Cleaning total time grows; terminal freezes. Valid zero stays zero; None requires no positive denominator.
- Stop producer, drain within500ms or count unstarted queue entries abandoned, await already-started IO, checked final flush/settle every ticket, atomically detach ring tracking, then terminal. Never extend pool30s/SQLite5s/native2s/exit500ms budgets or shut down a DB still used by S2.
- Each ring owner has one lazy eviction scope. Count matching removed S2 IDs even on ordinary pushes. Final count/detach share the pop lock; later logs cannot change the run. No unbounded receipt journal/broadcast/full-ring polling.

### 4. Validation & Error Matrix

| Condition | Result / retained fact |
| --- | --- |
| Local/oldFixture/S0/S1 | `benchmark_s2_unavailable`, no owner/tasks/effects |
| Previously claimed process | `benchmark_s2_process_used`, no second queue |
| Wrong run ID | `benchmark_s2_wrong_run`, original run unchanged |
| Queued/Running or pending producer/tickets at complete | `benchmark_s2_cleanup_incomplete`, no terminal |
| Executor admission failure | joined failure/tombstone, no retry reset |
| DB checkout / transaction error | failed ticket with actual attempted=false/true; never committed |
| Cleanup over500ms | Cleaning until actual join, then failure/budget fact retained |
| Worker/final-cleanup cannot join | never fabricate terminal success |

### 5. Good / Base / Bad Cases

- Good: explicit S2, real paced attempts, cancelled source, checked mixed-batch tickets settled, joined terminal readback.
- Base: normal Local and old Fixture retain existing clocks, batching and exit behavior.
- Bad: count requested60000 as delivered, use report time for pacing, reset once after cancel, or count every mixed-batch row as S2.

### 6. Tests Required

- `source_identity_once_and_deterministic_events`: identity, once, wrong ID, no queued terminal.
- `each_active_projection_has_its_own_revision_and_terminal_is_frozen`: fresh revision without ms resolution; retained terminal unchanged.
- `real_short_source_counts_actual_stages_and_joins`, `slow_sink_never_catches_up_and_budget_abandons_unstarted_queue`: real monotonic slots, actual ACKs, skipped/drop/abandoned and overbudget.
- `cancellation_and_final_io_remain_nonterminal_until_ack`: closed final-flush barrier cannot terminal before driver join.
- Native `captured_native_run_cancels_and_joins_actual_persistence` and Tauri `production_profile_cancel_joins_real_batched_storage_and_scoped_evictions`: production profile active cancel, actual SQLite counts, no pending tickets, owned evictions and frozen terminal.

### 7. Wrong vs Correct

Wrong: force_flush returned remaining_buffered=0, so mark committed and shutdown DB. Correct: harvest this run's terminal tickets, join inflight work/final IO, detach scope under pop lock, then publish immutable terminal and allow DB shutdown.


## Scenario: native Usage read latency and retained SSH metrics

### 1. Scope / Trigger

`monitoring::LatencyWindow`, native Services/Insights/Monitoring observation, and `ssh::Workspace::retained_session_count`. Tauri's legacy command/db-query p95, Moka cache and remembered SSH/password maps retain their distinct meanings and wire shape.

### 2. Signatures

- `LatencyWindow::{record(Duration), snapshot() -> LatencySnapshot}`; fixed `CAPACITY=2048`.
- `LatencySnapshot {sample_count:usize,p95_ms:Option<f64>}`: count is current retained window size, never total lifetime successes.
- `Workspace::retained_session_count() -> usize`: lock-only0/1 optional-session projection.
- Native Metrics adds usage_report_latency and retained_ssh_sessions; no IPC or new SQL API.

### 3. Contracts

- Observe only native `InsightsQuery::Usage`, with real Instant immediately around existing `usage::read::read_dashboard_at` after `open_dashboard` succeeds. Record only Ok results. Freeze elapsed before acquiring the statistics lock. Exclude opening DB, queue/admission, worker join, UI/render and failed reads; Home/Workbench/Logs/Sessions are not this cohort.
- Keep at most2048 completed duration samples, evict oldest, calculate nearest rank ceil(n*.95)-1. Empty ->None/count0; real zero ->Some(0)/count1. This is neither all-application/statement latency nor qualified performance/budget evidence.
- One window belongs to captured Services. Clones share that owner. `with_insights_paths` compares effective paths before/after override and resets only when actual captured paths change; unchanged paths or an override ignored by Benchmark cannot silently discard history. No dynamic registry/global metric singleton.
- Monitoring only snapshots these owners; it cannot trigger Usage reads to obtain samples. Existing explicit refresh/generation/suspend behavior stays. Hidden pages do not poll or start host observations.
- SSH metric reads only already captured Services.ssh.get(), then the Workspace session lock. Uninitialized ->actual0 retained sessions without owner construction, host lookup, filesystem, key scan, identity read, process or socket. A held1 is app authority, not TCP/reachability or Tauri's remembered-state-map size.

### 4. Validation & Error Matrix

| Condition | Required observation |
| --- | --- |
| No successful Usage-page report / DB open failure / invalid report / other query | No new sample; empty remainsNone |
| Completed successful report later discarded/cancelled by UI | Record once: service actually completed |
| Same effective captured paths / Services clone | Preserve/share window |
| Changed effective captured paths | New empty window; old owner's samples unchanged |
| No SSH Workspace / cleared/disconnected session |0 without creating/probing |
| Existing retained session with lookup disabled/poisoned transport |1 without IO; disconnect ->0 |

### 5. Good / Base / Bad Cases

Good: manual Usage read adds one real duration; later Monitoring displays the retained sample count and p95. Base: opening Monitoring with a real readable Usage fixture leaves the window empty; hidden suspension adds no samples. Bad: recording failed reads, including stats-lock delay, counting Workbench as this cohort, mapping no samples to0, or probing SSH to make a count.

### 6. Tests Required

- `latency_empty_zero_and_nearest_rank_are_distinct`, `latency_window_evicts_oldest_and_counts_only_retained_samples`: None/0, nearest-rank and rolling2048 retention.
- `usage_latency_samples_only_successful_opened_report_body_and_keeps_captured_owners_isolated`: actual owned fixture read, invalid/missing/Logs exclusion, clone sharing and effective root reset.
- `retained_session_metrics_observe_only_memory_and_follow_real_owner_transitions`: synthetic real connect, database authority removed, panic-on-transport read guard, then disconnect0.
- `monitoring_projects_latency_and_retained_ssh_without_initializing_or_probing`: same native captured snapshot path for empty/zero and injected synthetic session0/1.
- Existing `monitoring_filters_real_main_events_and_suspension_detaches_without_polling` now also verifies accessible None/0/sample count with a readable Usage fixture and hidden no-poll; run its existing central Windows1MiB child.

### 7. Wrong vs Correct

Wrong: time from request submission and call it SQL p95, or create/connect Workspace to count sessions. Correct: freeze the completed successful Usage service-body duration into its captured2048 window; snapshot existing memory authority only and label both scopes precisely.
