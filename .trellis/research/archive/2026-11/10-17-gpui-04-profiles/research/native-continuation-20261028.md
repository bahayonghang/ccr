# GPUI04 native continuation, 2026-10-28

The user resumed the approved existing tree; no commit, push, signing, publication or archive is authorized. Main used the Computer Use skill and an isolated domain-generated fixture. No real configuration/account/credential or external service was used. Size/DPI remains user-deferred.

## Frozen candidate and actual actions

Candidate `843f04f8229b97918e73ceaef4c0c0546d21ba880197358fcedc02cf7872ca00` was built with `just desktop-build-debug`, exit0. `outputs/desktop/continuation-20261028/baseline-source-before.json` records source and all three locks; `baseline-candidate.json` verifies unchanged source through build and identifies the immutable copied executable. The candidate predates the two new04 checker fixes and subsequent05/07 source changes.

Run: `outputs/desktop/profiles/native/baseline-04-20261028/`. `launch.json`, `receipt.json`, `actions.json`, screenshots and raw stdout/stderr are retained. Process80288 exited0. Fixture paths and exact arguments are in the launch receipt; HOME was not overridden.

1. Selected synthetic current `a-first`, opened Edit, replaced description with `native-cas-draft-20261028`.
2. Invoked the prebuilt explicit-root second-client fixture helper in a separate process. `second-client.log` and receipt show exit0, real catalog revision change and matching fixture lock ownership. No Cargo ran to perform this mutation.
3. Native Save did not commit the stale draft: the editor retained its text and changed to explicit reload/discard recovery. The conflict message was outside the current scrolled viewport, so this run does not claim a screenshot of its exact text.
4. Cancel returned to the previous selected details without stale retained-draft feedback. Explicit Refresh read `External native CAS fixture update`; `a-first` remained Current. Screenshots04–09 establish draft retention and external value, not the entire42-command domain.
5. Entered Settings then returned through sidebar, opened Off with outside focus: Cancel initially focused. Tab entered Confirm; Escape dismissed without confirming Off. Active-window screenshots11/14/15 record the transitions.

## Capture limitation

The helper returned the GPUI HWND with inherited Codex application identity. Some immediate post-input captures showed unrelated foreground content even though reactivation showed the expected GPUI transition. Those unrelated screenshots were removed from the evidence directory and never used as product evidence. A fresh explicit target activation/capture established the actual state before subsequent actions. Delayed inactive captures are not treated as a product regression or an acceptance pass.

## Remaining

Independent checker subsequently reproduced/fixed nested-editor return focus and stale Grok recovery across platform change; see `check-20261028.md`. This baseline does not validate those later changes. A new checked/frozen candidate must repeat CAS/Cancel/Refresh, focus paths and the broader native configuration/profile/backup/converter matrix. Durable warning preservation has rendered regression evidence; no new native partial-warning fixture is claimed. Whole04 and parent acceptance remain open.
