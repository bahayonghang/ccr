# Profiles confirmation and conflict-feedback repair — 2026-10-27

## Boundary and diagnosis

Active task: `.trellis/research/archive/2026-11/10-17-gpui-04-profiles`.

The old confirmation entry requested focus before its subtree mounted, then gated deferred entry on that focus still being inside; outside restoration therefore suppressed entry. Separately, closing an editor discarded its draft/revision but retained conflict text claiming the draft still existed. Native failures are recorded in `native-solid-candidate-20261027.md` for immutable candidate `ed00067d…eaab`; they remain failures.

### Fresh native retry disproved the first repair

Main reported fresh debug candidate `770b043f…`, under `outputs/desktop/profiles/native/repaired-20261027-231848`: Settings → Chinese → Configurations → Off again left focus on sidebar Configurations; Tab moved to Favorite commands and Escape did not dismiss. Screenshots `05-off-initial-focus.png`, `06-off-tab.png`, `07-off-escape.png` preserve this actual failure. No Off mutation was confirmed. Source compilation succeeded according to Main; it did not establish working keyboard entry.

The specific remaining cause is **callback-before-draw ordering**, not a missing second focus trap: locked `gpui-pre-0.3.9/src/window.rs:1866–1874` drains `next_frame_callbacks` **before** `window.draw` at1898; `focus_next` at2445 traverses the **previous rendered** tab stops. Scheduling from `act` therefore still runs before the new confirmation is mounted, despite `on_next_frame` documentation saying “after the current frame.” The action callback can select an old sidebar tab stop; the confirmation then renders with focus outside, so Kit's active-trap lookup and the subtree Escape handler do not engage.

The previous test did not cover this ordering: `gpui-kit-0.7.3/src/test.rs:133–143` dispatches MouseUp and immediately renders before returning from `click`. Calling `simulate_next_frame` afterward reverses the decisive native boundary. This explains why headless coverage and actual native failure coexisted. The prior source-ready handoff is superseded by the render-owned scheduling below; no prior test attempt is reclassified as proof.

This repair owns only `ccr-desktop/src/features/profiles/mod.rs`, optional necessary changes in its `view.rs`, in-module consumer regressions, and this report. Preserve the existing partial changes: post-mount entry, draft-scoped conflict tracking, durable warning retention, decision trapping, cancellation and focus return. Bind deferred entry to the particular confirmation so cancellation/navigation/reopening cannot make an old callback enter a different decision. Strengthen rendered-control tests without manually seeding confirmation focus. Keep service/persistence APIs, common files, manifests/locks, frozen outputs and historical evidence unchanged.

No build, formatter, lint, test, native rebuild or window interaction is authorized in this assignment. Coordinator owns serialized verification and the fresh native retry. Source review and test additions are not passing checks or native acceptance.

## Delivered source

- Changed `ccr-desktop/src/features/profiles/{mod,view}.rs`. Existing focus trapping, Escape, Cancel/return and disabled underlying actions remain unchanged.
- Private symbols: `Confirmation::{generation,entry_pending}`; `ProfileTools::{confirmation_generation,draft_conflict}` and initialization in `new`; `act` (creates a pending confirmation, no entry callback); `ProfileTools::render` in `view.rs` (consumes pending entry once and schedules its weak, generation-bound callback from the frame that actually mounts the decision tree); `close`, `submit`, `accept` retain draft-scoped versus durable warning behavior.
- Preserved and completed the pre-existing partial repair rather than replacing unrelated work. No public/exported API changes, dependencies, common integration files or persistence changes.

## Regression filters and exact boundary

All tests are in `ccr-desktop/src/features/profiles/mod.rs`, under `features::profiles::tests`:

1. `confirmation_recovers_outside_focus_before_first_frame`: render a host with a real focusable outside sibling, focus it, then click rendered Off. After `run_until_parked`, render the mounted decision tree, explicitly restore the **outside** sibling before `simulate_next_frame`, deliver that callback, then render/assert initial Cancel focus. No inside handle/button is manually focused. Exercise forward/reverse Tab containment, Escape dismissal and outside focus return. Reopen/click Cancel before deferred entry and assert no late focus theft; reopen/leave before deferred entry, focus the destination outside sibling, and assert navigation remains undisturbed. This is an explicit simulation of the observed ordering, not a claim that the headless platform reproduces Windows focus restoration.
2. `editor_confirmation_blocks_underlying_save_and_returns_focus`: no artificial inside-focus seeding; deliver deferred entry, then retain forward/reverse Tab, Escape/trigger return, blocked underlying Save and restore-target confirmation checks. Positive Grok partial-write and committed-warning cases survive close/status refresh; recovery action truth remains intact.
3. `rendered_edit_save_conflict_cancel_and_delete_confirmation`: actual worker save/CAS refusal retains the dirty draft and original warning; rendered Cancel clears warning/failure, then an actual Status worker refresh leaves it cleared and external model unchanged. Actual source read/stale save retains source and warning; Cancel plus actual Status refresh clears feedback, reopening source reads the external change. Existing secret masking, delete confirmation and plaintext teardown assertions remain.
4. `confirmation_enters_after_native_callback_before_draw_order`, sharing `confirmation_focus_boundary` with the original outside-restoration test: obtain rendered Off's bounds, dispatch real pointer move/down with completed frames, then dispatch MouseUp **without** the Kit helper's automatic draw. Assert confirmation opened, drain callbacks with `simulate_next_frame` **before** drawing its first subtree, then draw. The subsequent callback/frame must enter Cancel even after outside-focus restoration. This preserves the actual callback-before-draw boundary; old action-scheduled entry is consumed against old tab stops, whereas render-scheduled entry remains queued until the tree exists. No inside focus is seeded. Tab/Shift+Tab/Escape/return and cancel/leave assertions are shared between both variants.

Coordinator can select those four exact test-name filters, or run `features::profiles::tests` once in the serialized desktop UI suite. Use the repository's `desktop-*` recipes/toolchain wrapper, locked dependencies and default parallelism.

## Evidence limits and handoff

Source is ready for coordinator formatting, compilation, lint and focused tests; none were run here. No native build/window was launched. The earlier failed render/harness attempts are **not** before/fix proof; this run has **no** red/green result. `confirmation-break-loop.md` describes earlier boundary observations, not verification of this final source.

For discriminating red/green of the native retry failure, keep the final raw-MouseUp/callback-before-draw test unchanged in an isolated coordinator fixture; compare the immediately preceding action-scheduled callback implementation with the render-scheduled implementation. The earlier inside-gated callback is a separate, older baseline. Do not revert/stash the shared worktree or substitute the auto-drawing `click` helper for the decisive event.

After serialized gates, freeze a fresh candidate and repeat actual Settings → configuration → Off from outside/sidebar focus, Tab/Shift+Tab, Escape and return focus; repeat Cancel before entry/navigation where practical. Repeat real stale-save CAS → Cancel/back → Refresh: external value stays visible and retained-draft text disappears. Check durable committed/partial warnings separately. The immutable candidate's failures remain open until this actual retry supplies evidence. Native keyboard acceptance, UIA/IME, unsupported glass, deferred size/DPI and broader migration inventory are not claimed complete.

Only the two Profiles source paths above and this report were written by this assignment. No commit, push, archive, real HOME/config/auth mutation occurred. Context7 documentation was queried read-only for GPUI lifecycle semantics; exact locked source, not its misleading callback wording, grounds this correction.
