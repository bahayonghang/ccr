# GPUI 02 independent check

Status: **02 core scope accepted**, 2026-10-18, after the final patched core native receipts and paired Rust 1.95 check. AC2.1–2.5 are satisfied within this child's configuration-read/runtime scope. This does not accept parent migration/platform/performance/accessibility or later mutation/job domains. Original failed/interrupted attempts remain recorded below.

## Scope and ownership

Loaded `check.jsonl`, its sources including the newly added shared-services spec, the full saved check hook, task PRD/design/implementation plan, relevant package indexes and current product diff. Both implementation worker receipts were read after product freeze. The coordinator explicitly assigned test-fixture repair and then the complete 02 product scope to this reviewer. Native operations remain exclusively with the coordinator.

## Findings under review

- The shared configuration projection preserves the original Tauri field set, ordering, snapshot revision, empty description default, existing `Secret::Display` masking, and the domain-owned current-profile resolver. The Tauri wire DTO/export path stays in its adapter. Await actual fixture parity results and bindings gate.
- The runtime admits at most 128 tasks, limits blocking work to four, uses a 128-entry notification channel and separately retained terminal state. The producer coalesces latest progress at 100 ms intervals; the subscription checks terminal before receiving a wake. Read drop requests cancellation, whereas job page drop detaches. Runner completion owns terminal publication. Await final worker tests and source freeze before acceptance.
- Native configuration calls construct/read the domain service in the blocking closure. Render builds retained UI state only. The current native adapter refuses nonlocal/non-Claude requests instead of pretending to provide remote support.
- SQLite read-only live-WAL connections may create `profiles.db-wal` (zero bytes) and `profiles.db-shm` (32768 bytes), or change SHM mapping mtime on close. The assertions must continue distinguishing these from business-file/revision changes. No claim of zero physical filesystem writes is justified.

## Findings fixed

`just test` executes all workspace tests with all features and default parallelism. Existing process-global HOME/USERPROFILE mutation was reachable beyond the repaired Tauri config contracts:

- `ccr-cli/src/lib.rs::TestHome::new_with_home_env`, used by auth-off, profile-off, profile contracts, Claude paths/auth, doctor, Grok auth accounts, backup and conflict tests.
- `ccr-cli/src/lib.rs` host-environment fixture tests and `services/install_detect.rs` home-hint cases.
- `ccr-cli/tests/dispatch_routing.rs` environment fixture.
- `ccr-codex/src/platforms/codex.rs` three default-home/login-preparation cases.
- `ccr-usage/src/paths.rs` default-discovery case.

Fixed using `scripts/test-support/home_process.rs`, included only in test/test-support code. Each affected test dispatches its exact Rust test name in a fresh child with child-only HOME/USERPROFILE; the parent retains its environment and ordinary test parallelism. A successful child must report exactly one passing test. Canonical fixture roots have a parent-created ownership marker. Repeated lifecycle cleanup verifies the root and resolved child directories remain in that owned root before removal. TestHome now receives that root instead of calling global HOME setters. Codex, usage and dispatch fixture ownership follows the same model. Gemini HOME/empty-HOME/missing-HOME precedence remains three isolated cases; installation home-hint and Codex default-home cleanup coverage remain intact. Existing CCR-specific fixture locks remain where originally needed; there is no serial whole-suite gate.

Downstream audit found twelve further Tauri profile lifecycle/rename tests using that legacy fixture. With coordinator approval, added only test-entry guards in `commands/profile_lifecycle.rs`, `claude_profiles.rs` and `codex_profiles.rs`; production bodies and assertions are unchanged. Actual targeted regression passed all 37 matching profile tests. No global HOME/USERPROFILE set/remove call remains in the root/Tauri source scan.

Correction to preparation audit: `ccr/tests/diagnostics_contract.rs`, other CLI black-box tests and core log-path tests already used `Command::env` and required no repair. They were left unchanged.

First `just lint-strict` returned 1 because the enlarged TestHome made `ContractFixture::Legacy` trigger `large_enum_variant`. Boxed only that test fixture variant, preserving behavior and avoiding a lint allowance. Re-running the original strict gate returned 0.

## Scope limits and retained failures

- Coordinator observed a real native slow-read window close panic in upstream `accesskit_windows` (invalid HWND, HRESULT 0x80070578). The initial launch wrapper did not preserve the child's exit code, so its wrapper zero is not evidence of normal shutdown. This is now repaired with the narrow adapter patch, deterministic regression and three independently inspected successful patched native core receipts below. No original failure was reclassified as a pass.
- Initial transactional `just tauri-bindings-check` returned 1 for only `claude_observer/HeatmapCell.ts` and `InsightDto.ts`; it restored the original generated tree. A separate transaction exported the seven existing Observer DTOs from the actual already-built test executable and saved before/after under `outputs/desktop/services/bindings-drift`. The raw difference was trailing spaces; applying the gate's existing normalization made each pair byte-identical (326 and 621 bytes). The original full gate was then re-run unchanged and returned 0. The first transient mismatch is retained without claiming an unproven cause. No generated file, DTO, schema or gate was changed, and generated files have no final diff.
- Kit 0.7.3 disabled button semantics do not expose its disabled state in AccessKit, although the actual disabled click cannot advance read generation. That shell/accessibility item remains open for 03; no headless assertion substitutes for OS accessibility evidence.
- Complete command/usage job registries, re-subscription/history and log storage are later tasks. 02 establishes runner lifetime and terminal ownership only; it does not claim those domains migrated.

## Verification

- `cargo test -p ccr-cli --all-features --locked --lib -- --skip export_bindings`: 0, 390 passed, 1 existing ignored; `outputs/desktop/services/check-cli-fixtures.log`.
- `cargo test -p ccr-codex -p ccr-usage -p ccr-tui --all-features --locked --lib -- --skip export_bindings`: 0; Codex 392 passed/2 ignored, TUI 277 passed/4 ignored, usage 65 passed; `check-home-domains.log`.
- `just version-check`: 0; `check-version.log`.
- `cargo +1.95 check -p ccr-app --locked --all-targets`: 0, 58.54s; `check-ccr-app-msrv.log`. First inspected installed toolchains and confirmed the installed `1.95` alias is actual rustc/cargo 1.95.0; no toolchain installation/update. This is distinct from the ordinary Rust 1.98.0 gates and foundation-only MSRV evidence. The frozen independent desktop graph was subsequently checked as recorded below.
- `just fmt-check`: 0; `check-root-fmt.log`. Explicit rustfmt repair was run separately on owned files, never embedded in verification.
- `just desktop-fmt-check`: 0; `check-desktop-fmt.log`.
- `just lint-strict`: first 1 (fixture enum), then 0 after the scoped fix; `check-root-lint.log`, `check-root-lint-final.log`.
- `just test`: 0, `check-root-test.log`; workspace/all-features/default parallelism with original `--skip export_bindings`. Across 47 Rust test/doc-test suites: 2223 passed, 0 failed, 25 ignored and 33 filtered. Ignored/internal generator/export entries are not counted as passed acceptance.
- `just tauri-bindings-check`: initial 1, final 0 (`check-bindings.log`, `check-bindings-final.log`), original transactional gate restored the baseline each time.
- `cargo fmt --manifest-path ccr-ui/src-tauri/Cargo.toml -- --check`: 0 after all twelve additional Tauri fixture entry guards; `check-tauri-fmt-final.log`.
- `cargo --config .cargo/tauri-ci.toml test --manifest-path ccr-ui/src-tauri/Cargo.toml --locked profile_ -- --skip export_bindings`: 0, 37 passed, including all twelve newly guarded downstream lifecycle/rename cases; `check-tauri-profile-fixtures.log`.
- `just tauri-command-inventory-check`: 0, actual registry test 1 passed; `check-tauri-inventory.log`.
- Coordinator subsequently assigned both CLI/Codex `backend/test-fixtures.md` specs. Updated their seven-section contracts to the actual child-only HOME model, entry guard, canonical marker checks, exact passing-test assertion, safe ownership/cleanup, default parallelism, and original Gemini/install/Codex fallback coverage. Section-order assertion and scoped `git diff --check` both returned 0; no code changed and no broad Rust suite was repeated for prose-only changes.
- Machine-readable root test audit: `outputs/desktop/services/check-root-test-audit.json` records `just test`, underlying exact Cargo arguments, actual exit 0 from the completed session, log SHA256, all 47 suite names/counts and the 2223/0/25/33 totals.
- Generated tree receipt: `outputs/desktop/services/check-generated-tree-audit.json`; actual `git diff --exit-code -- ccr-ui/src/types/generated` returned 0 with empty output, and `git status --porcelain --untracked-files=all -- ccr-ui/src/types/generated` returned 0/empty. All 230 files have a path/content tree hash. No generated edit/untracked artifact remains.
- Independent lock audit against HEAD: `outputs/desktop/services/check-root-tauri-lock-audit.json`. Root adds exactly six required package records without changing an old record; Tauri adds exactly three and adds only `ccr-app` to its existing application dependency list. No existing versions/checksums or unrelated dependencies changed. The first diagnostic incorrectly demanded zero Tauri app-record changes, then was corrected to assert this exact required dependency addition rather than a blanket exception.
- Worker receipts independently inspected: shared config 5 passed and real Tauri configuration suite 18 passed; ccr-app dependency tree excludes umbrella/Tauri/GPUI/WebView. Runtime 9 tests, desktop unit 3 tests and Kit interaction 2 tests passed, desktop-check/lint/test/test-ui passed. See `domain-evidence.md` and `runtime-evidence.md` for commands, original failures and exact limits. No duplicate build was run merely to restate these receipts.
- Native operation: coordinator responsibility; no application launched by reviewer.
- No commit, push, installation, global environment change, or archive performed.

## Acceptance mapping

| Criterion | Review conclusion |
| --- | --- |
| 2.1 real fixture parity | Passed at service and actual Tauri adapter; all wire fields, order, revision, fallback and masking are exercised. Independently inspected real native fixture list/detail/filter/error-retry screenshots agree with that projection. |
| 2.2 boundaries | Passed for reviewed source and dependency receipts: no duplicate SQL/persistence/masking; framework-free ccr-app and worker-only I/O. |
| 2.3 responsiveness/stale/drop | Passed for 02 reads: deterministic environment/platform/generation/request rejection and subscription cancellation/join; rendered Kit input/disabled/close checks; native pending-read input evidence; final patched normal/error/cancel closes each exit 0 with empty stderr and coordinator-confirmed no surviving process. Full remote execution is still 09, and no UI latency budget is inferred from qualitative interaction. |
| 2.4 bounded events/terminal | Passed deterministic coalescing, admission, queue-full terminal, cancellation, late result and join tests. No performance-budget or OS proof inferred from unit tests. |
| 2.5 existing compatibility | Actual Tauri config suite and further profile regressions passed; transactional bindings and command inventory gates passed, generated tree unchanged. |

No parent AC1–AC8 completion is claimed. Platform/DPI/native accessibility/performance/package/full `just ci` acceptance remains with the parent and subsequent tasks.

## Independent AccessKit patch review and command-boundary follow-up

Read the complete registry-to-vendor `source.patch`, actual private tests, `UPSTREAM.json`, retained licenses, patch/removal notes, harness, desktop manifest/lock and `accesskit-close-fix.md`. This was read-only product review after the implementation owner froze the adapter. No duplicate Cargo/native operation or root/Tauri suite was run by this reviewer.

- Independent SHA256 inspection verifies all 19 recorded upstream files: only `context.rs`, `lib.rs`, `node.rs`, `text.rs` and `util.rs` differ, plus the new private regression module. Original manifests, dependencies and upstream tests remain unchanged.
- The private coordinate helper returns `Result<Point>` and maps failed ClientToScreen to UIA error 0x80040201. BoundingRectangle, ElementProviderFromPoint, RangeFromPoint and GetBoundingRectangles propagate failure with `?`. Existing absent-bounds default/empty responses are preserved; valid coordinate arithmetic is unchanged. No fabricated coordinate success, production panic catch, or accessibility disabling was found.
- The stock utility regression really ran and failed (101) at the original unwrap with 0x80070578. Fixed tests passed 3/3 and assert literal 0x80040201. They keep a strong Context and call the actual four COM interfaces, covering the in-flight lifetime overlap rather than merely Weak-upgrade failure. The successful-empty-bounds case is tested separately. These tests do not create a valid HWND or prove ordinary native screen coordinates.
- The harness copies the production source bytes, removes only the upstream visible-window test-module entry and four unrelated dev-dependency tables, and verifies every remaining source before/after testing. Its package identity/version/source/checksum set must be present in the production lock. It records its transformations and does not edit the registry cache or production manifest to run tests.
- Independent TOML comparison against the retained pre-patch desktop lock confirms the only change is accesskit_windows 0.34.0 losing registry source/checksum for its local patch; dependency lists and all other package records are identical. Final lock SHA256 is `95c30e2ec5d68b3547a3cb219aac7898d867ff906c809af1591dc1b3dfbc41af`.
- Independently observed `ccr-desktop/target` is currently a real directory resolving to itself, with no symlink/junction. Patched executable SHA256 matches the actual native receipt: `dd53feb570608970a1ce7cced1996399f541244b912d87afa8042988a38bf185`.

The implementation owner's initial elevated Cargo invocation caused mbx to adopt the local target. The original isolation gate rejected it; the exact adopted directory was restored without changing global settings. All earlier patch build/check/lint/test results using an explicitly selected real-toolchain PATH remain conditional evidence. Default-PATH recipes now use `desktop.py`'s verified Cargo resolver and the harness shares it; root CLI/Tauri command behavior is unchanged.

Initial resolver review found two local issues and sent them to its current sole writer for repair/revalidation: preserve a file-identity-confirmed ordinary Rustup proxy/toolchain even when mbx status is unavailable; avoid identifying a wrapper solely by `.cmd/.bat` suffix. Also route doctor's Cargo version query through the resolver so that it cannot bypass the new boundary. The owner fixed all three, and this reviewer re-read the final implementation: ordinary selected Cargo retains its path/environment; only the official installed/current shim's same-file identity authorizes the toolchain substitution; unverifiable selected shims fail closed.

The first default-path check then exposed a real compiler mismatch (E0514): direct Cargo 1.98 invoked a rustup proxy from registry dependency cwd, which chose default rustc 1.99. The owner paired that confirmed toolchain's bin directory only in the subprocess PATH; no global setting/PATH or artifact clean changed. This reviewer verified the final `cargo_environment` function and each metadata/build/clean/doctor/harness use. Its matching-compiler and unchanged-parent-PATH assertions supplement the normal-Cargo/stub 23, ambiguous identity, missing toolchain and existing 21 override/clean cases.

After final freeze, independently inspected actual default-PATH logs and the owner's settled exit receipts: `just desktop-check` 0 (`default-desktop-check-fixed.log`), `just desktop-test` 0 (Python 12/12, vendor 3/3, app 3/3, rendered Kit 2/2 in `default-desktop-test.log`), `just desktop-lint` 0 (`default-desktop-lint.log`) and `just desktop-build-debug` 0 (`default-desktop-build-debug.log`). Original E0514 and conditional-path runs remain retained. Product code is consistent and no unresolved defect was found in this final narrow patch/resolver review. No repeat Cargo command was run by this reviewer.

The final default-PATH debug rebuild supersedes the earlier native-interrupted artifact: independently read SHA256 **`94acabe375bc032da8cf868934eae6fb2de4392ab55cd7440849e3e9b84fc741`**. The two interrupted native receipts refer to their actual earlier **dd53** build and must not be relabeled as native verification of 94ac. A fresh filesystem observation still finds a real workspace-local target directory, not a link.

Those earlier native attempts remain explicitly **interrupted**: both `patched-normal` and `patched-core-normal` were stopped by Computer Use's physical Escape lock before a target capture. Coordinator verified the owned fixture process then force-cleaned it: native exit -1 / launcher 1. This is neither an accepted normal close nor proof of a patched crash. No further UI operation was attempted after the stop response within that turn. The user's next turn authorized core-flow continuation; main reset its old helper kernel and resumed native verification. The new actual native receipts, rather than those interrupted attempts, support final lifecycle acceptance below.

The user's latest steering prioritizes core functional flows and defers resizing/scaling/DPI matrices. Those matrices are user-deferred/not run; they are not passed and this does not require renewed implementation approval. No 03 implementation was started by this reviewer.

## Resumed core verification: final desktop MSRV

On the new authorized continuation, ran the frozen desktop graph through the normal public entry with **child-process-only `RUSTUP_TOOLCHAIN=1.95`**, inherited default command PATH and existing `desktop.py` resolver/toolchain pairing. No installed toolchain was changed or downloaded, and no app/theme/vendor code was modified.

- Prelude explicitly resolved and executed Cargo and rustc under `C:/Users/lyh/.rustup/toolchains/1.95-x86_64-pc-windows-msvc/bin`; both reported **1.95.0**. `cargo_environment` paired them in the child PATH. This avoids direct Cargo 1.95 accidentally using another rustc from dependency cwd.
- `just desktop-check` → `python scripts/desktop.py cargo check --locked --all-targets`: actual wrapper exit **0**, Cargo check duration **40.64s**. Shared ccr-core/config/app, patched GPUI Windows adapter and application/test targets compile. Actual log: `outputs/desktop/services/check-desktop-msrv.log`.
- Post-check observation retains a real workspace-local directory (not a symlink or junction), unchanged final lock hash **95c30e2e…bc41af**, and unchanged built native executable **94acabe3…b84fc741**. Checking did not replace the binary used by the coordinator's native run.
- Root 2223-test, bindings and other unrelated gates were not repeated. This is MSRV compilation evidence, not an additional native test or performance result.

Desktop 1.95 compatibility is now verified for the final Windows graph. The coordinator subsequently completed the native core checks below; no synthetic pass is inferred from compilation.

## Final native core receipts and acceptance

Independently read `native/core-resume-summary.json` and all three `core-resume-{normal,error,cancel}/receipt.json` files, asserted actual native exits **0**, exact final executable SHA256 **94acabe3…b84fc741**, `home_overridden=false`, `performance_sample=false`, and independently read each stderr log as trimmed-empty. Coordinator separately verified that no ccr-gpui process remained after the runs. The receipt check returned 0; no native operation was performed by this reviewer.

Also visually inspected actual GPUI screenshots in `outputs/desktop/services/native/windows-core/`: masked detail with matching `a-first`, empty `zz-no-match` filter, missing-catalog error, recovered two-row catalog with disabled `z-last` and current fallback `a-first`, and a pending-read state explicitly labeled Controlled slow read. This is native UI evidence, not a browser mock.

- **Normal:** two real catalog rows, matching/empty filters, masked detail and ordinary Alt+F4; actual native exit 0.
- **Error/retry:** missing catalog displayed a safe static error. Main confirmed the query created no missing catalog directory; it then copied only the domain-generated synthetic database into the owned fixture path. Refresh recovered both rows/current fallback. Actual native exit 0; this was fixture repair, not a product auto-initialization path.
- **Read cancellation/close:** screenshot shows pending catalog read. Receipt specifies a 30,000ms fixture delay; process lifetime was **28.52s**, less than that delay, followed by native exit 0/empty stderr. This exercises close while the artificial read was still pending, supplementing deterministic runner cancellation/cleanup and subscription reclamation tests. It does not measure the 100ms performance budget.

Main honored the Computer Use interruption/input guard, resuming only after explicit user steering and inspecting only the target application. Previous dd53 forced-cleanup receipts remain interrupted. The new 94ac receipts provide the actual post-patch native acceptance for this child's core flows.

**Conclusion:** no unresolved defect remains in reviewed 02 scope. Proceed to the next approved dependency task without archiving 02 while its context is referenced. DPI/resize/theme matrices remain user-deferred/not run, native screen-reader/disabled semantics belong to 03, macOS/Linux/full performance/package acceptance remains unproven, and parent AC1–AC8 are not declared complete.
