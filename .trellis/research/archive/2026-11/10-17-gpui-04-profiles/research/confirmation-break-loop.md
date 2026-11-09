# Bug Analysis: confirmation entry and scoped recovery feedback

This records the resumed04 investigation. A new native repair-candidate retry is still required; it does not convert current fixes or headless results into complete native acceptance.

## 1. Root Cause Category

- **D — Test Coverage Gap**: the earlier confirmation test manually focused the trap after opening it. That established the transition whose initial delivery was actually broken.
- **E — Implicit Assumption**: the old entry path requested focus before the confirmation subtree was mounted. Its next-frame callback required that focus to remain inside, so an outside restoration could suppress the repair.
- Feedback has a related lifetime assumption: closing the draft cleared draft/revision but left a message that asserted the discarded draft was still retained. Ordinary status refresh intentionally preserved durable warnings, so clearing every message would break a different contract.

Evidence: immutable candidate `ed00067d…eaab` actually opened Off confirmation with sidebar focus; native Tab escaped and Escape failed. Main safely clicked Cancel. Actual stale-save CAS refused correctly and retained the draft, but Cancel/Refresh then left obsolete recovery text. See `native-solid-candidate-20261027.md` and its real screenshots/receipts.

## 2. Why Fixes Failed

1. The earlier trap repair tested traversal after artificial inside-focus seeding. It covered containment but hid initial entry failure.
2. Early resumed test attempts failed to render/enter confirmation at all; missing ElementId and test-frame/host setup failures are not valid product reproductions.
3. An outside-focus test initially failed before a simulated post-frame callback, but the corrected same harness also passed old production code. That result was explicitly withdrawn as before/fix discrimination.
4. A separate boundary case restores outside focus before the first callback, matching the observed native state. Old code fails that test. It is a deterministic boundary simulation, not an assertion that the headless backend recreates OS focus restoration.

Raw attempts remain under `outputs/desktop/profiles/resume-current-check/recovery-*.log`. The checker report owns the exact final source/test outcomes; no failed attempt is overwritten as a PASS.

## 3. Prevention Mechanisms

| Priority | Mechanism | Specific action | Status |
| --- | --- | --- | --- |
| P0 | Lifecycle | Enter real decision focus after mount; weak entity/current confirmation guard prevents late focus after cancellation/leave | Checker implementing/verifying |
| P0 | Feedback ownership | Clear only draft-scoped conflict/reload text when draft closes; retain committed/partial warnings | Checker implementing/verifying |
| P0 | Native check | Repeat actual Settings→configuration→Off→Tab/Shift+Tab/Escape sequence on a fresh frozen candidate | Pending |
| P1 | Regression | No manual inside-focus seeding; distinguish old/repaired timing boundary and preserve cancellation/leave checks | In progress |
| P1 | Design contract | Explicit entry/return and scoped feedback requirements in DESIGN.md | Done |
| P1 | Review checklist | Event-driven boundary checklist in cross-layer thinking guide | Done |

## 4. Systematic Expansion

- Similar review targets are05 authentication/provider confirmations and future06/08 destructive actions. They must consume the verified lifecycle/Kit semantics, rather than copy a pre-mount focus assumption. This is a review trigger, not authorization for unrelated rewrites.
- Keep outcome/error categories typed and scoped to their owner. A durable partial result and a discarded editor recovery instruction have different lifetimes.
- Acceptance should ask what initial state and frame/event ordering a test actually establishes. Counting passing tests cannot answer whether the critical transition was exercised.

## 5. Knowledge Capture

- [x] Updated `ccr-desktop/DESIGN.md` with native initial focus and scoped feedback contracts.
- [x] Updated `.trellis/spec/guides/cross-layer-thinking-guide.md` with concrete test-boundary checks.
- [x] Preserved actual native failures and raw harness/product failures separately.
- [ ] Append new candidate hash/native retry outcome after real verification.

This repository has no `src/templates/markdown/spec/` tree to synchronize. User continuation explicitly excludes automatic commits/archival, so spec changes remain reviewable in the working tree; no commit is performed by this skill.
