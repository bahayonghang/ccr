# GPUI08 integrated command checkpoint, 2026-10-29

Active task: `.trellis/tasks/10-17-gpui-08-jobs`. Existing approved tree was resumed by the user. Main confirmed immutable integrated candidate1157f008 already existed, releasing the historical06/08 source hold. This implementer loaded the saved truncated-hook output and actual task artifacts; no nested agent was spawned. Shared workspace edits from main/04/05/06/07 were preserved. No commit, push, archive, real configuration/account operation, network request or external install was performed.

## Source change boundary

Main owns exports, dependencies, lockfiles, Services, shell and launch wiring. This worker owns the existing prepared jobs policy/process/runner, native jobs adapter/view, and Tauri command/process adapters. Main explicitly extended ownership to `crates/ccr-core/src/core/log_redact.rs` for the reproduced shared-owner redaction defect.

- `jobs::policy` is now the single allowlist/catalog/argument validator. Tauri retains its original TS/wire DTO definitions and maps the catalogue, rather than maintaining a second command list.
- `jobs::process` is the single capability/executable/managed-child/registry owner. Tauri's wrapper captures its own executable, repository root, build debug policy, compile-time sidecar hash and package version before calling the shared gateway; native captures its own context. Neutral ccr-app does not read a desktop consumer's compile-time variables. Release resolution remains adjacent-only, hash-required, without PATH fallback.
- Tauri start/status/cancel, pruning and event bridging now consume the shared `CommandJobs`, removing the duplicate retained registry/runner. The existing Tauri runtime-policy admission permit moves to the runner through process/tree/reader cleanup. The adapter subscribes before polling the runner, emits only nonterminal progress deltas, drains pending deltas and emits the final snapshot after the runner returns. Shared `TimedOut` maps to existing wire `Failed` plus `process_timeout`; the original status enum and snapshot fields remain unchanged. The private shared `cancel_requested` field is represented through the existing system log and is not added to the Tauri wire DTO.
- Native compile repairs use the actual `gpui_kit::prelude::FluentBuilder`, explicit virtual range typing and remove an unused import. The jobs page remains a typed-token JSON-array caller with bounded virtual logs and explicit synthetic process labeling. It does not execute real domain commands in fixture mode.
- Main's actual app test reproduced `snapshot_redacts_secret_output_and_caps_long_utf8_lines` failure: shared `redact_log_text` handled JSON/Cookie/Bearer/sk/JWT but not command-style `api_key=value`. The correction is in that existing core owner, with key=value/key:value, quoted keys/values, escaped quotes, whitespace, incomplete line and Unicode-long-value regressions. Existing recognizers and masking rules remain. The original failed jobs assertion was retained.
- A second read-path issue was found during review: adding a truncation marker to a capped multi-byte line could truncate off a UTF8 boundary. The existing marker branch now backs up to a char boundary. No input/output cap was raised.

## Verification boundary

Own-file rustfmt and scoped `git diff --check` ran successfully. Main owns serialized Cargo gates and raw receipts under `outputs/desktop/continuation-20261028/resume29-*`. Initial red/compile receipts are retained; final results must be appended after actual runs. Prepared or compiled source is not native, trusted-sidecar, full task08 or performance acceptance.

Tauri tests retain real consumer controls/admission-through-cleanup barriers and wire parity. Shared owner tests retain invalid/repeated admission, cancel-before-spawn marker absence, real fake process stdout/stderr/nonzero/timeout/cancel/flood, bounds/drops, held reader and immutable terminal assertions. Existing process tests moved with the owner rather than being deleted for a smaller test count.

## Remaining scope

Native foreground help remains unimplemented, as do install/sync/checkin and other entries in the68-command/five-route ledger. Main owns actual native candidate identity/actions. Whole08, S2 latency, full light/dark/platform coverage, bindings/final gate and final acceptance remain open.

## Received current-source gate results

Main's serialized actual receipts show `resume29-app-after.json` exit0 (55 library,15 integration passed;3 existing ignored); `resume29-native-after.json` exit0 (44 native library tests passed); `resume29-core-redact.json` exit0 (5 redaction tests passed); `resume29-desktop-lint.json` exit0 (locked strict all-target Clippy). The app result preceded the added oversized4-byte stream regression, so its final-source rerun remains required. Tauri all-target `resume29-tauri-check.json` exit0 exposed now-orphaned local Tokio construction wrappers after owner delegation; this worker searched all consumers, removed only those new orphans, and redirected the test to the shared constructor. Tauri parity/strict final gates are still owned by main.

A direct source-body comparison with HEAD confirmed all11 original command DTO field/variant/attribute bodies identical (`CommandArgSchema`, `CommandFlagSchema`, `CommandInfo`, `CommandJobStatus`, `CommandJobSnapshot`, `StartCommandJobResponse`, `OutputChannel`, `CommandJobDelta`, `CommandExecutionResult`, `CommandCatalog`, `CommandHelpResponse`). This static comparison supports compatibility but does not replace the transactional binding gate.

The native view detaches on leave and recovers through an explicit Refresh status after return; automatic re-subscription has not been implemented. This checkpoint does not satisfy the full subscription-recovery requirement merely by retaining the job owner. `research/source-checkpoint-20261029.json` records the frozen08-owned source identity for independent review; main's all-source/lock candidate remains the authoritative build identity.
