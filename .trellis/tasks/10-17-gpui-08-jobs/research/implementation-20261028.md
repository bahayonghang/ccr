# GPUI08 implementation, 2026-10-28

Active task: `.trellis/tasks/10-17-gpui-08-jobs`. User restored the existing
approved tree; main activated08 after consumable engineering dependencies. This
worker is already the Trellis implementer and does not dispatch other agents.
Shared workspace edits from04/05/07/06 are preserved. No commit, push, archive,
publish, signing, real account/configuration, install or network request.

## Context and smallest change boundary

Native injection selected08 but was truncated. Read its saved full hook chunks,
explicit PRD/design/implementation and both manifests, referenced contracts,
root code_map/RTK, applicable shared/owner indexes and parent recovery reports.
The historical `planning` prose in PRD does not override actual task.json and
latest approved activation note in implement.md.

Existing command policy, catalog, retained snapshot/runner and trusted process
gateway live in Tauri `commands/command_exec.rs` and `process/gateway.rs`; they
are framework-free apart from wire derives/event emission/spawn adapters. The
smallest missing behavior is a native typed caller through those same owners.
Do not create a second policy, raw shell passthrough or false progress runner.

## Prepared source boundary (unexported, NOT_RUN)

- `crates/ccr-app/src/jobs/policy.rs`: existing catalog and request validator,
  preserving exact allowlist/flags/aliases and typed-only destructive entries.
  Eight existing meaningful policy regressions moved with the pure owner.
- `jobs/process/{mod,gateway}.rs`: existing managed gateway/child registry,
  capability limits and old tests. Sidecar identity is an explicit
  `SidecarContext`: consumer current executable, repository root, debug policy,
  compile-time expected hash and consuming package version. Neutral ccr-app
  never captures those consumer `env!`/`option_env!` values. Missing context,
  release missing/mismatched hash and debug-only repo paths have authored tests.
- `jobs/commands.rs`: app-owned admission/token under one lock,64 retained jobs,
 30-minute terminal TTL, same request-key deduplication, explicit
  `cancel_requested`, immutable terminal, existing500-line/512KiB per-stream
  bounds, shared redaction and bounded readers. Runner returns after process
  wait/owned-tree cleanup and reader joins share a five-second cleanup budget.
  EOF/read errors are not silently converted to success. Drop is not cleanup
  proof. Existing shared runtime coalesces visible progress at100ms.
- `commands/tests.rs`: invalid/repeated admission, cancel before spawn with a
  must-not-exist marker, real owned fake process stdout/stderr/exit2, timeout,
  cancellation, output flood/bounds/drop visibility, held-reader cleanup
  failure/immutable terminal, capacity/TTL and UTF8/secret output assertions.
- `ccr-desktop/src/runtime/jobs.rs`: typed Local-only requests, app-owned shared
  CommandJobs, queued-ticket RAII when runtime shutdown skips queued work,
  static errors, status/cancel recovery. Production uses consumer-owned trusted
  sidecar context. Fixture uses only current executable's closed synthetic child
  mode, canonical outputs/desktop containment and existing ownership marker.
- `features/jobs/{mod,tests}.rs`: native command metadata, tokenized JSON-array
  args, start/cancel/status, retained state across view detach, virtual36px logs
  with8KiB displayed-line ceiling. Typed-only catalog items disabled; invalid
  drafts retain input with safe error and do not start a job. Fixture UI explicitly
  says synthetic process lifecycle, no real CCR/domain command executed.

All these modules remain **unexported** while main freezes the04/05/07 candidate.
The old Tauri owner remains unchanged until the authorized serialized handoff.
Temporary prepared source is not a delivered duplicate implementation; once
exported, Tauri must delegate/remove its old policy/process owner, keep wire
DTO/TS paths/events and run parity. No compile or behavior PASS is claimed yet.

## Shared integration request (main owns)

1. ccr-app existing workspace `reqwest`/`sha2`/`uuid`/`tracing` dependency edges,
   no version upgrade, plus `pub mod jobs`. Preserve all three lock records and
   existing root/Tauri sha20.11.0 versus desktop0.11.1 resolution policy.
2. desktop runtime/features `pub mod jobs`; Services once-created
   `pub(super) command_jobs: Arc<ccr_app::jobs::commands::CommandJobs>`.
3. shell JobsView entity via `new(services, window,cx)`, focus_main/suspend/refresh.
4. Before ordinary launch parsing, closed fixture mode:
   `runtime::jobs::fixture_child_mode(std::env::args_os().skip(1)) -> Option<i32>`;
   return code through process exit. This is a real fake child process for
   lifecycle validation, not real CCR command parity or performance evidence.
5. Release Tauri adapter rewiring only after candidate copy; consumer-side
   `SidecarContext` retains Tauri's own compile-time hash/version/repo semantics.

## Full68-command/five-route mapping and remaining code

Exact original inventory IDs, risk classes, source handlers and domain owners
are in `command-status-20261028.json`; original capture was not regenerated.
No command/route is accepted merely because this source was prepared.

Remaining after first command checkpoint: foreground/help adapters, actual
trusted-sidecar parity; InstallService native persistent plan/attempt/events
and shutdown; encrypted asset/folder sync and canonical WebDAV settings;
checkin provider/account/record/balance/CDK/WAF operations and per-origin
orchestration; monitored command history integration and complete native flows.

Checkin fixture prerequisite is concrete: AccountManager has explicit DbAccess,
but service helpers still construct Provider/Balance/Waf managers with global
DB and proxy discovery. Do not call these defaults from native fixture mode.
Add existing-owner explicit DB/client/proxy seams under reviewed ownership,
preserving encrypted cookies and per-origin throttle. WAF's old WebView cookie
collection needs a native/external handoff design with truthful unsupported
status; do not add a prohibited embedded WebView or infer cookies from a file.

## Actual validation so far

Own-file rustfmt repairs executed separately, exit0. Scoped `git diff --check`
exit0. These are formatting-only results. Cargo/tests/lint/build/native remain
NOT_RUN until main exports/edges and releases serialized gate ownership. The
first authoring attempt used an unavailable Input builder and a quoting error;
both corrected before integration, with no product behavior failure claim.

Next narrow commands after integration: root ccr-app jobs tests and strict
all-target/all-feature Clippy; Tauri command/process parity and command inventory/
bindings; desktop jobs consumer/fixture tests, strict lint/check then full suite.
Default parallelism and `--skip export_bindings` retained. Main owns native
candidate identity/actions; full08/S2/cross-platform/airy-glass acceptance open.
