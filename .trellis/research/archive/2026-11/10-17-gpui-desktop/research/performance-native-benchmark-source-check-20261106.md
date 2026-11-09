# Native Benchmark source independent check — 2026-11-06

Scope: distinct `ConfigSource::Benchmark`, launch/startup, Services and affected
runtime consumers, desktop Cargo inputs and isolated tests. Review began at HEAD
`65a5fe1f` with the implementation and concurrent native delivery changes already
uncommitted. No other worker's changes were reverted; no commit was made.
Native GUI/CUA, real host probes, WSL/SSH, CLI imports, OAuth listeners, external
network and user HOME mutations were not used in this check.

## Findings fixed

1. `runtime/platforms.rs`: Benchmark OAuth start/control had no source guard.
   Shared Fixture OAuth merely substitutes its authorization URL; it still owns
   a listener and saved pending state. Also a retained foreign PlatformSession
   bypassed `platform_request`'s root check. Both Benchmark entries now return a
   read-only `Unsupported` terminal before claim/run/control effects. Existing
   Local/Fixture behavior and the public return type stay intact; this is still
   read admission, not a fabricated synchronous admission error.
2. Retained editor sessions were bound to the original document, but their public
   Services save entry points did not validate the current Benchmark source.
   `mcp/editor.rs`, `transfer.rs`, `batch.rs` now compare a private captured
   Benchmark root for all retained source/destination sessions before revision
   reads or saves. `resources.rs` validates prompt owner roots and snapshot path;
   `resources/claude_files.rs` checks the actual retained domain document path
   (not its public display path); `resources/platform_files.rs` checks the
   private retained file path before publishing operation state/admitting work.
   Legacy Local/Fixture sessions retain their existing behavior.
3. `runtime/benchmark.rs` owned Path comparison and `lib.rs` nested title condition
   were corrected for the strict Clippy findings without changing behavior.

The existing fixture-isolation spec now documents the distinct native source,
retained editor checks, Benchmark-specific OAuth refusal, and comparison limits.

## Source audit

- Parsing is explicit, exclusive and filesystem-free. Benchmark load runs on the
  bounded worker before preferences/window creation; failure has no Local arm.
- Absolute/traversal-free paths, reparse/symlink ancestors and descendants,
  required layout directories, bounded JSON, marker/scenario/count identities,
  gpui/tauri sibling identities, logical hashes and native DB artifact hashes are
  checked before main-pool/log initialization. Original root-bound opaque IDs are
  preserved. This is not a proof against concurrent hostile filesystem swaps.
- Roots/catalog/profile lifecycle/settings/auth, preferences, UI state,
  monitoring/observer, usage/archive/session, sync and resource selectors have
  Benchmark arms or explicit captured-root checks. Alternate insights paths
  cannot override Benchmark. The old Fixture-only `if let` branches were checked
  individually: delay/subroot compatibility is intentional, project/transfer
  paths have earlier Benchmark checks, test-only import fallback rejects it.
- Command jobs, CLI import/probes, installation/tools, SSH, network sync,
  check-in network service/recharge and quota are refused for Benchmark before
  their effect owners. A reqwest client object in a cached resource owner is not
  evidence of a network request; cached source operations keep their domain
  refusal policy.
- `sha2` reuses pinned 0.11.1 in the resolved desktop graph; serde_json remains
  pinned and enables float roundtrip. Canonical logical hashes sort object keys,
  preserve arrays/values and match the generator contract. Concurrent sysinfo
  changes are owned by the System delivery, not this review.

## Findings not fixed / next owners

- Visible clock wiring is still incomplete: `runtime/insights.rs::read_workbench`
  uses `Local::now`/Local filters; `session_window` uses `Utc::now`. Exposing the
  immutable manifest clock does not close this. Common visible clock owner and
  acceptance remain required.
- Native startup streams DB hashes and walks session metadata; Tauri admission
  currently has different validation costs. Do not subtract this cost or claim
  comparable process-create-to-present performance.
- Main disclosed and source inspection confirms a remaining Check-in mutation
  seam: `extended.rs::Export` calls `ExportManager::export_file -> export ->
  convert_accounts_for_export`, whose `CryptoManager::new` occurs before
  `write_new_export` enters its own `crypto/locks` scope. Empty captured stores
  may therefore acquire ambient crypto locks. Import's legacy account encryption
  also needs the Check-in owner audit. Do not wrap asynchronous work in TLS or
  invent a conflicting nested lock root. This check does not approve all
  Check-in mutation paths as isolated; main assigned a separate follow-up owner.
- No paired data was regenerated and no 161-second projection traversal was
  repeated here. The implementation owner's actual S0/S1/S2 readback receipt is
  historical domain evidence, not this review's performance/native evidence.
  S2 streaming remains `not_emitted`; all benchmark results remain
  `not_run/not_comparable`. Other-OS and native visual/loading acceptance remain
  unverified.

## Verification

- Actual OAuth red: the safe preclaimed synthetic owner returned `Full`, failing
  “source refusal must precede OAuth claim”; it never started a listener.
- Actual retained-session red: all seven rejection assertions were false for MCP
  mutate/copy/batch, prompt create/save, Claude file and platform file. Only owned
  temporary directories were used. A test setup correction added the required
  `.md` extension before this genuine red run; the earlier setup failure is not
  counted as a behavior regression.
- Green: `cargo test --offline --manifest-path ccr-desktop/Cargo.toml --lib
  runtime::benchmark -- --skip export_bindings --nocapture` — **6 passed,
  0 failed, 2 ignored**, plus **1 passed** from the explicitly spawned poisoned
  child. New rejection tests include the current Benchmark and a synthetic
  nonfixture foreign OAuth session, all controls, all seven retained editor
  routes, and complete directory/file-byte equality for both temporary trees.
- Existing compatibility on that fresh compiled artifact:
  `runtime::mcp` **13/13**, `runtime::resources` **4/4** passed. These are isolated
  domain tests; no native window was opened. Actual S0/S1/S2 acceptance remains
  intentionally ignored and was not rerun.
- Scoped rustfmt and `git diff --check` passed. Final strict Clippy/typecheck
  `cargo clippy --offline --manifest-path ccr-desktop/Cargo.toml --lib --tests
  -- -D warnings` passed with exit **0** (12.28s), including the final borrowed
  path comparison and coordinator's credentials module ordering fix.
- Parallel transient diagnostics test-module/type errors and an SSH seam missing
  EnvironmentId qualification initially prevented compilation; those are not
  presented as Benchmark test failures or as passing checks.
