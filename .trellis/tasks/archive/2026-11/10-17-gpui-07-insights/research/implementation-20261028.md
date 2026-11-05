# GPUI 07 continuation checkpoint, 2026-10-28

Status: implementation remains in progress. This is a concrete source checkpoint for independent check and a new native candidate, not complete 07 acceptance. No commit, push, publish, signing or archive. Other owners' changes and the historical failures/receipts remain intact.

## Delivered scope

- Existing incremental archive refresh moved from Tauri into `ccr-app/src/sessions/refresh.rs`; Tauri retains its exact wire DTOs and explicit mappings. Provider discovery/parsing and archive SQL remain with ccr-store/ccr-db. Native Sessions runs one initial refresh plus explicit refresh/status actions using the bounded app-owned runtime. View detachment does not cancel or invent a terminal. Counters/results remain available when revisiting. Existing archive refresh is non-cancellable; it is not usage import or usage-sync cancellation.
- Refresh completion originally interrupted a user-opened transcript. The actual headless failure was retained, and the callback now refreshes a list only when the same query generation is still current and no detail is open. New rendered pagination regression verifies actual visible rows across a full scrolled 100-record page and the final one-record page.
- Native Budget / pricing tab has actual load, patch/set, remove and reset consumers. Remove/reset capture the model/action and use mounted, generation-guarded confirmation focus. Invalid JSON retains its draft. All I/O stays in background app-owned jobs, fixture locks use captured roots, and displayed errors are static. Budget status can create the existing tracker directory; it is not described as a pure read. Legacy budget cost tracking and custom pricing remain separate from stored llmusage prices.
- Shared budget/pricing policy is extracted from the original Tauri command bodies. Both consumers call it. The whole supplied budget patch is validated before the first setter: bad booleans, negative/non-number limits, non-integer/out-of-range warning percentages reject without a write; 300 cannot truncate to 44. Valid fields still persist sequentially through the existing manager. This adds no multi-field transaction or CAS claim.
- Meaningful store regressions first reproduced unknown TOML extension loss for both managers (raw red logs preserved). The repair is private to the existing budget/pricing managers: keep original TOML, replace known keys, retain unknown top-level and surviving per-model/default-pricing fields, and use the existing guarded atomic writer. Successfully saved state becomes the private original, so deliberate model deletion is not reintroduced later. Public BudgetConfig/PricingConfig and wire projection shapes did not gain fields; unknown persisted extensions are not exposed in public projections.

## Fixture identity correction

The initial manually seeded archive used `home.join(".codex/sessions/...").to_string_lossy()` while provider WalkDir discovery produced its own exact Windows path spelling. Archive IDs intentionally hash exact physical-source strings. Automatic refresh could therefore add the actual discovered source while marking the manually spelled row missing, giving two fixture rows. This was a fixture identity mismatch, not evidence to redesign domain ID normalization.

The rendered fixture and seed helper now use the actual provider-discovered exact physical path and owner fingerprint. The original fixture/receipts are preserved. A **new** `outputs/desktop/insights/native-populated-20261028-v2` was generated. Its helper asserts 205 discovered/unchanged, zero upserts, stable first 200 archive IDs and a final page of five after refresh. Data remains synthetic: two usage buckets, 102 events, 205 sessions and a 250-message long session. This is native read/filter/pagination/refresh acceptance data, not S1/S2 performance data.

The coordinator's earlier native baseline report is `research/native-populated-20261028.md`; it validates its own older frozen artifact. It does not prove new refresh/policy controls. The briefly suspected blank last page did not reproduce in the rendered row-locator test, and the coordinator observed the final row after activating the target. No speculative pagination repair was made.

## Actual verification

All commands used existing paired Rust 1.98, `--locked`, default parallelism and the existing `--skip export_bindings` split. Root/Tauri use the process-only `outputs/desktop/profiles/resume-current-check/target`; desktop keeps its independent target. Full raw logs are in `outputs/desktop/insights/checks/`.

| Command/scope | Actual result | Raw log |
| --- | --- | --- |
| ccr-store full tests | 84 library + 7 integration + 1 doctest passed; 5 existing ignored | `store-unknown-fixed-20261028.log` |
| ccr-app usage tests | 8 passed; opt-in native seed helper ignored normally | `policies-validation-fixed-20261028.log` |
| Native insights focused suite | 9 passed, including actual policy controls, draft retention, confirmation Cancel/Tab/Escape, real-provider detail pages, visible final record, detached refresh and 10k candidate virtualization | `native-final-20261028.log` |
| ccr-app + ccr-store strict all-target/all-feature Clippy | Exit 0; two unnecessary extraction `.into()` calls were repaired after the retained failing run | `shared-final-clippy-fixed-20261028.log` |
| Desktop strict all-target Clippy | Exit 0 | `native-final-clippy-20261028.log` |
| Tauri locked all-target check after extraction/owner repair | Exit 0 | `tauri-final-check-20261028.log` |
| New native seed helper, explicit private v2 root | 1 passed; refresh identity and 200+5 pages asserted | `fixture-seed-v2-20261028.log` |
| Explicit owned-file rustfmt check and scoped editable `git diff --check` | Exit 0 | Tool receipts; formatting repair was separate |

Earlier in this continuation: shared sessions 3 and Tauri archive regression 10 passed (`session-refresh-20261028.log`, `tauri-session-refresh-20261028.log`). Those test counts are not added again to the current tests above. Initial compile/focus/fixture/format failures remain separate and are not rewritten as passes. A Windows mapped-source error during formatting was resolved by waiting for the actual Tauri check to finish, then formatting; no process or cache workaround was introduced.

Frozen source/manifests/locks and gate identity: `outputs/desktop/insights/checks/checkpoint-frozen-20261028.json`. All 49 command IDs are tracked in `research/command-status-20261028.json`: 12 existing read source mappings, 8 implemented refresh/policy mappings with acceptance open, and 29 explicit unimplemented native entries. Main owns parent inventory integration; the original capture was not regenerated.

## Open work and exact next entry

Independent check and a new frozen candidate's native 04/05/07 regression come next. No new native PASS is claimed for refresh/budget/pricing by this implementer. Real glass remains unsupported; other OS, IME/UIA, user-deferred size/DPI, performance, packaging, full bindings and final just ci remain separate parent acceptance gaps.

After this checkpoint, remaining local 07 implementation starts with the existing neutral usage-import admission/NDJSON/process lifecycle extraction, then Claude observer/subscription/monitoring, dedicated Codex sessions/rolling usage, capability/readiness/Home bootstrap and quota's explicit-root/fake-transport prerequisite. These 29 native command entries remain unimplemented; the new archive refresh does not satisfy usage import or its cancellation contract. No real HOME/account/quota/sync/network operation was used.
