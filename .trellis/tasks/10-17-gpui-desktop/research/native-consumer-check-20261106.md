# Native consumer independent check — 2026-11-06

Reviewed the System info, Check-in credential editor, Codex diagnostics and Root
pending cleanup receipts against their owning sources. Began at HEAD `65a5fe1f`
with concurrent uncommitted work; no unrelated changes were reverted, no commit
was made. The coordinator later included a read-only check of the fixed SSH CLI
detector and confirmation-layout repair. This report is source/headless evidence,
not native loading, real host diagnostic or performance acceptance.

## Finding fixed: missing Check-in key replaced by metadata update

`AccountManager::update` called `get_crypto -> CryptoManager::new` before checking
whether cookies were supplied. The new editor's read correctly failed without
a key, but its advertised blank-to-preserve fallback then created a replacement
key even for a name-only save. This was not just a hypothetical exception path:
both a native request regression and an independent injected-memory-pool domain
regression failed because `crypto.key` unexpectedly existed.

With coordinator approval, the fix is in the actual domain owner,
`crates/ccr-checkin/src/managers/checkin/account_manager.rs::update`:

- `cookies_json: None` updates metadata while preserving encrypted bytes without
  constructing crypto.
- `Some` uses `CryptoManager::load_existing`; missing/damaged keys fail before
  the DB update, leave all account fields/encrypted bytes intact and do not create
  or replace the key.

No public signature/model/schema changed. Create, export/import and CDK were not
refactored. The check-in backend spec and credential receipt now distinguish this
contract from captured lock correctness.

## Source checks

### System info

The explicit bounded read is gated on Local and `!is_fixture()` before its probe;
Benchmark is included. It constructs `System::new`, refreshes CPU twice with at
least the library minimum interval, refreshes memory only, and never enumerates
processes or starts CLI/network/timers. Cancellation is checked around the CPU
wait. CPU values are finite/range checked and averaged in f64; absent capacities,
invalid ratios, empty CPU sets and unavailable facts remain unknown. GiB and
logical CPU/app-architecture labels are accurate; host RAM is distinct from
process RSS. Parent suspend clears facts/tasks/error and invalidates generations.
No additional issue found in this scope.

### Check-in scoped editor and captured locks

Only the selected account is decrypted through the explicit captured DB/crypto
owner and `load_existing`. The DTO has no Debug/Serialize and is held only by the
request/editor, never retained management/operation projections. Loading blocks
editing/save; editor request nonce plus account ID reject late responses. Cookie
JSON, API user and ordinary Session input are masked. The Session patch inserts
into the original JSON object and retains other cookie keys; advanced JSON
remains an explicit path. Cancel/leave/success drop the editor and subscription.

Fixture/Benchmark `Storage` captures locks. Create/update/Import/CDK synchronous
domain calls enter that scope without crossing await/spawn. Export's initial
key creation is scoped separately before the exporter's own file-write scope;
it is not wrapped in a conflicting nested lock root. Import validates the
retained source path against the current source before the domain re-read.
Native read failures render safe errors instead of crypto paths/plaintext.

### Root pending proof and native Codex diagnostics

Pending snapshots retain private content revision plus login/nonce fingerprint;
no Debug/Serialize proof is exposed. Snapshot preparation does not expire data.
Expiry and conditional cleanup use the exact guarded delete revision and return
conflict on a later file. `CleanupScope` binds pending/locks/store/runtime;
the opaque plan checks that scope and current login/nonce. An active cleanup
delegates to the existing commit-aware controller and joins it before conditional
clear. A committing controller refuses cancellation before the file is cleared.
`CapturedBackend` uses its own save/load proof; it cannot unconditionally erase a
later nonce. This is cleanup safety, not a claim that all OAuth writes are CAS.

Diagnostic admission rejects fixture/Benchmark, remote and wrong-platform work
before roots or probes. Fixed preflight binds only 127.0.0.1:1455 and distinguishes
occupied/unavailable/available; the actual OAuth owner still performs final bind.
Parsing validates status, timeout, truncation, stderr warnings and UTF-8; Windows
uses the parsed local listening endpoint, not a port substring. Cancellation is
requested only from matching gateway registry handles; unknown PIDs are never
killed. Reports explicitly avoid claiming process exit or port release. Save
warnings are advisory and keep the original session/name/CAS. Review focus enters
safe Cancel after mount, restores trigger focus, and generation checks prevent
stale terminal results from acting after leave/platform change. No new issue
found in this scope after the previous Benchmark OAuth guard fix.

### Additional read-only SSH source review

The detector accepts no user command/tool argument: fixed Claude/Codex/agy/OpenCode
`command -v` checks use the existing quoted SSH target and known-host gateway.
Opaque connection UUID authority is captured before runtime queueing and checked
after the serial lock; reconnect cannot retarget a queued check. Process output
timeouts/truncation fail in the existing transport; accepted facts are bounded
UTF-8 single absolute paths, not versions or banners. Native environment/source
checks precede live transport. UI cancel/leave/selection generation clears old
facts. Moving the existing confirmation above the editor is a local reachability
repair, with a visible/in-viewport assertion; no window-size/timeout threshold
was relaxed. Source reviewed only: the owner already supplied SSH3 + adapter1 +
25 actual 1MiB child passes, which were intentionally not repeated here.

## Verification actually run

Pinned Rust1.98/MSVC, locked/offline Cargo and default parallelism; binding exports
remain skipped. No GUI/CUA, live PID/fixed-port probes, real SSH, user cookies,
OAuth exchange, provider HTTP or global HOME/USERPROFILE changes were used.

- Native missing-key regression **red 0/1**: name-only save unexpectedly generated
  a key. Root injected-pool domain regression independently **red 0/1** on the
  same invariant. Both used only owned temporary files; the new domain test scopes
  its own locks.
- `cargo test --locked --offline -p ccr-checkin account_manager -- --skip
  export_bindings` — **9/9 passed**, including the new None/Some and damaged-key
  cases plus existing re-encryption/CRUD compatibility.
- `cargo clippy --locked --offline -p ccr-checkin --all-targets --all-features
  -- -D warnings` — **exit 0**.
- Fresh native `cargo test --locked --offline --manifest-path
  ccr-desktop/Cargo.toml --lib credentials -- --skip export_bindings` — **9/9**
  passed, including the **3 relevant credential/transfer cases**. The final UI
  test actually edits Session, saves, reopens and checks Session/other-key values,
  advanced access, masking, switch-account staleness and leave clearing.
- That fresh artifact: `monitoring::system` **5/5**, `diagnostics` **7/7** passed
  (the latter filter includes one existing profile diagnostic-policy regression).
  All new host-effect paths used pure/synthetic DTOs or early fixture refusal.
- Owner's unchanged Root pending **7/7** and OAuth **10/10** plus strict Clippy
  receipts were checked against source; their controlled listener tests were not
  rerun here. They are not newly claimed executions by this reviewer.
- Native `cargo clippy --locked --offline --manifest-path ccr-desktop/Cargo.toml
  --lib --tests -- -D warnings` — **exit 0** (10.93s). Scoped rustfmt and
  `git diff --check` passed. An initial native Clippy run stopped only on the concurrent shared-clock
  `insights.rs` unnecessary match parentheses; its owner corrected it without a
  warning suppression.

## Remaining boundaries / not fixed

- Existing AddAccount/Export/Import/CDK paths still use `CryptoManager::new` in
  parts of their recovery flow. Captured locks do not prove a lost key cannot be
  regenerated when other encrypted accounts exist. Their recovery semantics need
  a separate domain-owner decision; this check approves only the repaired update
  and read invariants, not all crypto recovery behavior.
- Real OS/sysinfo output, actual fixed-1455/process-tool behavior, dialogs,
  native focus/pixels/UIA/IME and other OS behavior remain unverified/deferred.
  No benchmark or first-present/action-ready sample was collected. Final
  workspace/GPUI aggregate gates and updated 1MiB guard belong to the coordinator.

## Final OAuth regression adaptation review

Read-only follow-up: reviewed the 12-line change to
`features/platforms/tests.rs::oauth_cancel_joins_fixture_cleanup_and_clears_callback_draft`
and the actual native view/runtime/controller/backend call chain. **No coverage
weakening or new source defect found; no production guard was changed.**

The visible Start action still runs the new diagnostic preflight. The adapted
test waits for its completion and asserts no OAuth status/active owner. The
separate diagnostics control test asserts the exact `Error::Isolated` refusal;
the old cancellation test does not pretend that this refusal started a listener.
It then calls the real `view.start_oauth` to exercise the retained owner layer,
rather than injecting a success/status or bypassing the cancellation owner.

That layer receives the same manually captured PlatformSession roots as before.
Before the view exists, the test allocates a loopback port through port 0, writes
that exact port/redirect into a live synthetic pending state under those roots,
and saves it with the captured lock scope. `Controller::start` loads this live
pending state before its create fallback and calls `backend.bind(pending.port)`;
the captured backend binds 127.0.0.1 at that port. Therefore this path uses the
preseeded ephemeral port, not the production fixed-1455 preflight endpoint.
Fixture exchange remains refused by the existing backend policy.

All subsequent checks remain real and unchanged: Waiting state, actual Cancel
button, Cancelled terminal, callback draft and URL clearing, inactive owner,
pending-file deletion, successful rebind of the captured port, cleanup message
and final runtime-owner shutdown/join. Existing five-second waits and product
permissions were not relaxed. Coverage is explicitly split between preflight
refusal and retained-owner cancel/join, since fixture UI Start-to-listener is now
intentionally disallowed by the preflight policy.

Verification here was source/diff/log inspection, without rerunning Cargo or a
listener. Coordinator supplied exact test **1/1**. The actual rerun log
`outputs/desktop/integration-20261106-gpui-current-test-rerun1.log` was inspected:
lib **228 passed / 0 failed / 4 ignored**, native-shell integration **9 passed**,
input integration **2 passed**, shutdown integration **2 passed**; stack wrapper
passed (coordinator reports its 26 actual children). Current fmt-check and strict
all-target lint logs also finish successfully. These supersede the earlier
OAuth-failed aggregate, while preserving its historical receipt. They are
headless/controlled integration evidence, not native GUI or performance evidence.
