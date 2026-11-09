# Research: remaining GPUI 08 owners and next implementation slice

- Query: identify the smallest substantial install/sync/checkin/balance migration after the command workbench.
- Scope: internal; source inspection only, no install, network, account or real-data operations.
- Date: 2026-11-05

## Findings

### Recommended next slice: llmusage installation

Implement the complete native install interaction using the existing `ccr_cli::services::install_service::InstallService` directly. Do not extract a second owner into ccr-app: the current Tauri adapter is already thin and the GPUI package already depends on ccr-cli.

Existing owner and adapter anchors:

- `ccr-ui/src-tauri/src/commands/install.rs:20` delegates detection; the file owns eight IPC commands, event forwarding and Tauri background admission only.
- `crates/ccr-cli/src/services/install_service.rs:160` owns the attempt slot, bounded event ring, canonical plan registry, clock, host probe and executor.
- `install_service.rs:189`, `:194`, `:199` expose detect, probe and plan. Planning checks fresh backend detection/capabilities against hints rather than trusting renderer input.
- `install_service.rs:276`, `:282` expose execute and execute-with-admission. Only an opaque PlanId is accepted; executable actions remain private. A slot rejects concurrent attempts; consumption validates a 120-second TTL, single use and host fingerprint.
- `install_service.rs:346`, `:370`, `:374`, `:378` expose cancel, current attempt, recent events and manual catalog. Cancel is only a request. The slot is released by a task awaiting executor completion, independently of the event receiver.
- `crates/ccr-cli/src/services/install_exec.rs:57` executes closed package-manager actions through ManagedProcess. `:157` races wait/cancel; `:175` drains both readers before terminal publication; `:303` records to the retained ring before best-effort delivery.
- `crates/ccr-cli/src/services/install_ring_buffer.rs:1` retains 50 recent Log events plus a separate terminal slot. Started/Progress are not retained; reconstruct running state with `current_attempt_id`, and recover logs/terminal via `recent_events`.
- `crates/ccr-cli/src/services/install_types.rs:255` defines safe InstallPlanView; `:328` defines ordered typed InstallEvent; `:466` defines RingBufferSnapshot.

Suggested file ownership and integration:

1. New `ccr-desktop/src/runtime/install.rs`: typed adapter over a Services-owned Arc<InstallService>; schedule host detection/probing/manual-catalog work off the GPUI thread, retain the owner across navigation, reject unsupported environments before probes or execution, and expose safe static error messages.
2. New `ccr-desktop/src/features/install/mod.rs` and `tests.rs`: check host, show detection and plan effects/expiry/elevation, explicit execute acknowledgement, cancel feedback, bounded logs, terminal/error/manual instructions, and revisit recovery. Display manual commands as copyable text; never pass edited text to execution.
3. Main integration owns `ccr-desktop/src/runtime/mod.rs` (module + Arc field), `src/features/mod.rs` (module export), `src/shell/mod.rs` (native navigation and lifecycle) and shell tests. Avoid modifying current command-workbench files in parallel.
4. No new domain dependency is required: ccr-desktop already depends on ccr-cli and ccr-app. Implementation additionally needs a direct Tokio dependency for entering the existing async runtime, cancellation/event draining and watch subscriptions; reuse the already desktop-locked 1.53.3 rather than introducing a new version (the root manifest's `1.53.1` is a compatible version requirement). Tests add ccr-cli's existing `test-support` feature in the desktop dev-dependency. Do not enable test-support in production.
5. Existing `InstallService::test_with_runner` at `install_service.rs:385` is available under test/test-support and supplies a canonical synthetic plan without host detection or package-manager execution. Use it for duplicate execution, detach/revisit, cancel-before-terminal, cleanup admission retention and late event tests.

Existing owner regressions are in `install_service.rs:544` (modified capabilities), `:561` (stale detection), `:581` (unknown plan), `:599` (expiry), `:620` (host mismatch), `:641` (reuse), `:666` (single winner), and `:742` (occupied attempt). `install_exec.rs:326`, `:352`, `:362` cover closed commands, ignoring renderer executable data and bounded unterminated output.

Verification should include focused desktop adapter/headless UI tests, relevant ccr-cli install owner tests using default parallelism and `--skip export_bindings`, desktop strict Clippy/fmt, and independent check. Keep OS-native pixels, real package installation and performance acceptance separate. Do not run the existing `detect_returns_result` host probe test as synthetic process evidence.

### Sync requires shared orchestration extraction first

- `ccr-ui/src-tauri/src/commands/sync.rs:190` owns a five-item fixed asset registry, while `:255` onward owns response and asset/config orchestration. ccr-sync already owns transport, encryption, bounds and atomic transactions; duplicating the Tauri orchestration in GPUI would create policy drift.
- `sync.rs:405` / `:409` load canonical WebDAV configuration with legacy fallback only when the canonical file does not exist. `:441` validates transport; `:447` resolves scoped local assets; `:561`/`:607` handle push/pull; `:647` selects sync direction; the private truth table follows at `:711`.
- `sync.rs:913`, `:924`, `:931`, `:938`, `:943` expose list/push/pull/sync/all-assets. Other entries at `:1056` onward cover generic/folder operations, status/folder CRUD and WebDAV configure/test/disconnect.
- `crates/ccr-sync/src/sync/folder_manager.rs:67` and `config.rs:56` accept explicit config paths for isolated fixtures. Default constructors must not leak into a fixture adapter. Domain service constructors/limits and transactions stay in ccr-sync.
- Extract fixed registry, canonical config and asset decisions to `crates/ccr-app/src/sync/` with explicit roots; Tauri keeps TS derives and wire shapes. Add ccr-sync to ccr-app dependencies only when that extraction begins; keep the domain DTO owner singular. Native config/passphrase forms then consume this shared owner.
- Required preservation: HTTPS validation before save; Basic Auth Secret never used as an operation envelope passphrase; per-operation passphrase cleared on submit/close and never stored; v2-only new sensitive writes; explicit plaintext-v1 migration; typed both-sides conflict unless force; pull transaction and exact backup/rollback semantics.

### Checkin/balance still have a significant shared-owner gap

- `ccr-ui/src-tauri/src/commands/checkin.rs:197` batches accounts with a global Semaphore(5), 90-second per-account timeout, and `:290` wraps jobs with a 600-second deadline. Job updates/events and monitoring currently depend on AppHandle/AppState.
- `ccr-ui/src-tauri/src/checkin_jobs.rs:65` defines snapshots; its mutation methods own pending/processing/result/timeout presentation state. A safe extraction must preserve Tauri event and monitoring compatibility rather than copying it into a GPUI-only runner.
- Command anchors: execute `checkin.rs:588`, batch `:604`, start job `:644`, get status `:687`, get balance `:756`, records/history/stats and account/provider/crypto/import/WAF operations throughout the same file.
- `crates/ccr-checkin/src/services/checkin_service.rs:733` is the domain checkin method; `:1187` query_balance decrypts via CryptoManager, invokes provider HTTP, stores balance and updates account refresh time. A balance refresh is therefore a network and persistence action, not merely reading cached data.
- `checkin_service.rs:1410` batch_checkin also uses global concurrency 5. `AccountManager::with_db` (`managers/checkin/account_manager.rs:47`) accepts an independent DbAccess pool, but service-level query_balance constructs default provider/balance managers. A directory override alone does not prove DB isolation; a full fixture-safe service seam is needed before native fake-HTTP account tests.
- CheckinService/WAF error classification and encrypted cookies must stay domain-owned. Native WAF browser recovery requires separate capability handling and cannot silently claim WebView parity.

### Inventory identities

The existing parent `research/migration-inventory.json` is the authoritative per-command capture; do not regenerate it over historical hashes/evidence. Its eight install IDs are `llmusage_install_detect`, `llmusage_install_probe_capabilities`, `llmusage_install_plan`, `llmusage_install_execute`, `llmusage_install_cancel`, `llmusage_install_recent`, `llmusage_install_manual_catalog`, `llmusage_install_check`.

The 17 sync IDs are `sync_push`, `sync_pull`, `list_sync_assets`, `sync_push_asset`, `sync_pull_asset`, `sync_asset`, `sync_all_assets`, `sync_push_folder`, `sync_pull_folder`, `sync_status`, `list_sync_folders`, `add_sync_folder`, `update_sync_folder`, `delete_sync_folder`, `set_webdav_config`, `test_webdav_config`, `clear_webdav_config`. Inventory counts are command surface counts, not feature completion percentages.

### Related specs

- `.trellis/spec/ccr-app/backend/shared-desktop-services.md`: UI thread boundary, app-owned jobs, immutable terminal after cleanup, bounded notifications, identity/generation gates.
- `.trellis/spec/ccr-core/backend/managed-process.md`: managed child/tree lifecycle and cleanup.
- `.trellis/spec/ccr-sync/backend/backend-guidelines.md`, `webdav-hardening-contracts.md`, `test-fixtures.md`: owner, transport, transactional/envelope and fixture contracts.
- `.trellis/spec/ccr-ui/frontend/sync-security-contracts.md`: canonical WebDAV lifecycle, passphrase scope, direction/force truth table and state gating.
- `.trellis/spec/ccr-checkin/backend/backend-guidelines.md`: domain/DB/crypto/WAF boundaries and safe secret handling.
- External references/versions: none needed; this research changes no dependencies and makes no upstream-version claim.

## Caveats / Not Found

- PRD 8.4 says to retain a per-origin queue/throttle. Searches for per-origin/throttle/rate-limit queue owners in ccr-checkin, Tauri checkin and frontend source found no such implementation; observed batching only has global concurrency 5. Do not describe per-origin safety as an existing proven capability. Before migrating concurrent balance/checkin requests, locate a differently named owner or implement/test this missing contract explicitly.
- InstallService has no public shutdown/join method. Its execute task lives on the entered Tokio runtime and awaits actual executor completion to clear the slot. A native adapter must tie this spawned work to app shutdown tracking/cancellation; simply storing Arc<InstallService> is insufficient evidence that window-close cleanup completes.
- Install executor awaits stdout/stderr reader tasks without an explicit join timeout, and mpsc sends can wait while a receiver is alive but not drained. Keep an active owner-level drain or deliberately drop the receiver on detach; do not retain an unread receiver in a hidden view. These are current-owner constraints, not newly verified fixes.
- No behavior tests, builds, native windows, real network or real account actions were run for this research. All conclusions are source evidence and implementation recommendations.
