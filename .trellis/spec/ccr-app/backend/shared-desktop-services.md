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
- Monitoring shows tracked tasks and actual optional current-process RSS. Unmeasured latency percentiles are unavailable. Refresh is explicit, late generations are rejected, and suspend drops the subscription with no hidden timer. Tests use captured main/archive fixtures, malformed storage rows/time, secret redaction, bounds and actual view transitions.

- Claude Observer keeps its databases distinct: usage reports come from the read-only `ccr-usage` dashboard; tools/subscription settings come from the captured CCR main UI database, never the session archive. Native and Tauri share state-free projection/subscription policy; framework DTOs and binding paths remain Tauri-owned.
- Opening Observer does not create or migrate a database. An explicit confirmed first subscription Save may initialize the captured main database with its existing migration owner. An unreadable or incompatible existing file is an error, not a missing-database setup case. Subscription writes compare the reviewed values inside the same write transaction; they remain app-owned after the view detaches and work independently of usage-database availability.
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
