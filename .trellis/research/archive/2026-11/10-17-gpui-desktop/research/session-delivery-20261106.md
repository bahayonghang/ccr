# Native session delivery — 2026-11-06

Scope: existing parent task, baseline HEAD `96376961cd49b7914e3457efba2fc92e9d8ae076`; preserve other uncommitted batches. No commit or native UI launch in this implementation subtask.

## Delivered

- Selected Codex Markdown preview now offers native Save as new file, using locked `gpui-pre 0.3.9` `App::prompt_for_new_path` (`src/app.rs:1904`). Cancelling admits no writer. Selection and preview are generation/decision scoped; the retained bounded job saves the explicit preview bytes, without rereading a newer session. Completion snapshots contain no transcript text.
- Save uses private guarded CAS with an absent-file expectation; existing files conflict. Reject relative/parent traversal and linked/reparse ancestors. Fixture destinations compare canonical existing parent to canonical fixture root (Windows native dialog paths need not carry the captured verbatim prefix). No fixture output escapes that root.
- Sessions exposes CCR trash independently of an active selected row. Metadata list, explicit reviewed restore confirmation, focus trap, Cancel/Escape and focus return reuse native controls/theme. No permanent delete action added.
- Codex owner creates opaque home/manifest/payload-bound restore plans; native reads use the existing shared 64 MiB session-operation budget. The exact checked payload is written privately without overwrite, index append uses CAS, and a concurrent index edit is preserved. Legacy CLI/Tauri `restore_sessions` keeps its streaming copy behavior without the native payload cap.
- Restores remain application-owned across view detachment; source success with failed archive refresh is separate from partial source failure. Archive refresh uses the same captured provider registry without environment overrides.
- End-to-end tests exposed a real shared refresh bug: restoring original bytes and mtime left `deleted_by_user` archive rows unchanged because all three fingerprint fast paths ignored archive state. DB source projection now includes `source_state`; container/group/member skipping requires live archive rows. Restored/missing sources reparse to live; absent deleted sources remain deleted and normal unchanged live sources retain their fast path.

## Current evidence

Pinned Rust 1.98; offline/locked; default test parallelism; binding exports skipped separately.

- Root `cargo test --locked --offline -p ccr-codex codex_session_trash -- --skip export_bindings`: **8 passed**. Includes original recovery/mtime/index/conflict cases and >64 MiB legacy streaming restoration compatibility.
- Root `cargo test --locked --offline -p ccr-app sessions::tests::refresh -- --skip export_bindings`: **3 passed**. File restoration with original bytes/mtime, missing restoration, preserved absent deleted rows, unchanged live fast path, SQLite member/container reactivation, unchanged provider bytes.
- Desktop `cargo test --locked --offline --lib session_operations -- --skip export_bindings`: **5 passed**, final run 1.60s. Covers preview Save dialog mock/cancel/save, copy, trash cancellation, GUI restore review/cancel/confirm, plaintext clearing, retained result, CAS conflict, junction/fixture escape/remote refusal, manifest/payload changes, home substitution, existing rollout/index conflicts, archive reactivation and source-success/archive-unavailable distinction.
- Exact existing headless session UI test with `RUST_MIN_STACK=1048576` and a child-only poisoned `CCR_LOCK_DIR`: **1 passed**. Already included in the existing native stack guard list; no new guard entry required.
- `git diff --check`: passed at batch verification.
- Design-atlas existing airy-glass source followed. Mechanical component scan reports **0 files** (Rust/GPUI unsupported), so its zero findings provide no visual evidence. Source review checks real labels/data, native semantics, bounded feedback and focus handling; independent review remains parent-owned.

Native screenshot, actual Save dialog behavior and real window interaction: **user-deferred**, not verified. No actual user data, network, desktop window or system file dialog accessed. Parent final lint/integration gate and independent check remain required; old CI does not verify this batch.

## Metadata-candidate follow-up — ReadyForReview

Follow-up baseline observed HEAD `20d6f23a0a4892385219ef1d4f1327500d137a99`; other workers' source and the independent reviewer's relative-home, pagination and focus fixes were preserved. This section supersedes eager whole-list preparation in the initial implementation, without replacing the historical test receipts above.

- `list_restore_candidates(cancelled)` returns opaque `CodexSessionRestoreCandidate` metadata/home/manifest authority and **never opens rollout bodies**. It checks cancellation before enumeration, per entry and around sorting. Bounded manifest reads check cancellation between 64 KiB chunks; native inputs must be regular files and reject linked/reparse paths. No legacy interface acquires these new list semantics or restrictions.
- Explicit Review invokes `prepare_restore(candidate, cancelled)`: compare the original candidate's manifest/record, read/hash only the selected bounded payload, then recheck the original manifest. Changed manifest bytes (even identical parsed metadata) cannot gain fresh consent. Missing/unreadable/oversized unselected payloads do not block listing. An oversized selected payload fails only its review. A prepared payload change remains rejected by the existing commit token.
- Runtime `TrashItem` stores the metadata candidate; distinct `PreparedRestore` stores a private full plan. The view enters confirmation only after the matching generation/decision completes its background read. Cancel-review and scope clear drop that read and invalidate late results. The existing retained writer still consumes the exact prepared plan off-page; commit now reads only the selected manifest instead of rescanning the whole trash.
- Six-row pagination and existing native controls remain intact. The reviewer's unrun `.disabled()` snapshot assertion returned `None` because disabled is emitted through existing synthetic accessibility nodes; it now verifies actual Previous-at-first-page behavior. No production semantics or gate threshold was relaxed. Focus regression still tests early Enter, safe Cancel entry, Tab/Shift+Tab, Escape and return to the outside input.

Latest scoped checks (Rust 1.98, locked/offline, default parallelism):

- Root trash suite: **11/11 passed**, final run after regular-file and final cancellation guards, 0.38s. New real filesystem cases prove metadata listing succeeds with an unreadable unselected payload, selected Review fails on that payload/oversize, changed manifest bytes retain old authority, selected bounded reading observes cancellation before scanning the full oversized body, and cancellation stops before opening invalid manifests across a 100-entry fixture. Legacy relative home and >64 MiB streaming restore continue passing.
- Desktop session suite: **7/7 passed**, final run 1.97s before only the two last domain guard additions (root-tested above). Includes actual queued review `Terminal::Cancelled` with no writer, metadata-to-prepare-to-commit tamper/conflict/archive flows, Cancel-review, scope clear during preparation, and both existing focus and six-row/10k metadata pagination controls. The 10k fixture proves rendering bounds only; it is **not** domain I/O or performance evidence.
- Scoped rustfmt and diff checks: passed. Final aggregate GPUI fmt/lint/tests, current 1 MiB child guard and independent API review are **pending coordinator's stable candidate**; they must cover the two latest domain guards. No current native/UIA/IME/Save-dialog or benchmark pass is claimed.

Production changes stay in Codex trash owner, runtime session delivery/types, and native session delivery/state, plus two mechanical facade exports for Candidate. Tests and the owning recovery spec were updated. No native SQL, cache invalidator, new trash format, real-home read, network action, or native window was introduced. Current stage is **ReadyForReview**, not full task acceptance or release readiness; local commit remains coordinator-owned after review.

## Windows 1 MiB render-stack gate recovery

The later aggregate `gpui-test` correctly found a stack overflow in the existing session control child. Default-stack session 7/7 did not cover that constraint. Reproduced with the already compiled `ccr_gpui-c8abaaf8ffb826a3.exe`, `RUST_MIN_STACK=1048576` and a child-only poisoned `CCR_LOCK_DIR`; exit was `-1073741571` (stack overflow). Rustc itself always retained its normal stack.

Hunt diagnosis used temporary stage and local-stack-anchor markers (all removed). Fixture/window creation, opening Sessions and list loading succeeded; selecting detail entered the real production `InsightsView::render → product::sessions → transcript → session operations → render_session_trash` chain, then overflowed. The trash list was not open. Local-anchor displacement from operations into the monolithic trash function was approximately 250 KiB, on top of the transcript builder chain. Pagination, tooltip and row builder temporaries shared that entry frame even when no trash entries were rendered; this was a production frame accumulation, not the async test body's setup.

The fix extracts only the existing pagination and single-row builder expressions into natural helper functions in `features/insights/session_operations/delivery.rs`. Parent/child nodes, IDs, attributes, six-row page bounds, event order and callbacks remain unchanged. No new container, larger stack, skipped route, test-only shortcut, threshold change or synthetic passing renderer was added. `InsightsView` and `product` retain their original structure. The optional builder frame is no longer reserved by the unloaded-trash entry path.

Green evidence on the same test executable after recompilation, with no temporary logging:

- Exact `selected_codex_controls_export_cancel_and_clear_plaintext`, 1 MiB plus poisoned default locks: **1/1 passed**, 1.64s. It traverses the actual detail/preview/save/trash/restore/focus route that overflowed.
- Exact pagination sibling `session_trash_pagination_bounds_large_collections_and_keeps_controls_reachable`, same 1 MiB/lock constraints: **1/1 passed**, 0.41s.
- Default-parallel `session_operations` filter on that executable: **7/7 passed**, 1.96s.

The coordinator owns the final aggregate guard/full gates and the three unrelated install/jobs/close test failures from that earlier aggregate. No native UI, real dialog or visual evidence is claimed; user deferral remains unchanged.
