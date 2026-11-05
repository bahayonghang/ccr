# Native solid candidate: actual partial acceptance

Date: 2026-10-27. Candidate SHA256 `ed00067d7a6a2f88356c69d1b6e24417680ba7f9c3c170e49a3ca893e9d0eaab`; copied executable and exact source/gates are linked by03 `research/airy-glass-check.md` and `outputs/desktop/airy-glass/2026-10-27/check/candidate-receipt.json`. Main used the Windows Computer Use skill on uniquely returned actual GPUI windows, with screenshots after each action. This is the new solid palette; old debug/release evidence is separate.

## Passed on this candidate

| Scenario | Actual observation / evidence |
| --- | --- |
| Read and selected detail | Two synthetic profiles, valid registry fallback `a-first` Current, credential masked. `solid-core-20261027/screenshots/01-dark-list.jpg` and subsequent actual detail capture. |
| Metadata edit/save/re-read | Description became `native-solid-save-20261027`, while `a-first` stayed Current and `z-last` stayed Disabled. The original current-marker defect did not reproduce. Screenshot02. |
| Two real clients / CAS | Main entered a `-dirty` native draft; frozen helper SHA256 `89dd134542ae6ecd02a0e78994b6e7d3de9c135b68c50f553b9a67c30474c4ef` committed through ConfigManager in a second process with explicit private fixture paths and the matching lock directory. Helper exited0 and catalog revision changed. Native Save refused with `Catalog changed. Your draft is retained`, and the dirty field remained. Screenshots03/04 and `second-client{.log,-receipt.json}`. |
| Cancel/refresh data state | Cancel/back then Refresh reread `External native CAS fixture update`; external write was not overwritten and `a-first` stayed Current. Screenshot05. The obsolete status text is a separate failure below. |
| Apply/current/runtime | Selecting disabled `z-last` then Apply/enable reported `Profile applied`, moved Current to `z-last` and removed Disabled. Explicit private registry had `current_profile=z-last`, enabled=true; private runtime model/endpoint matched the selected profile. `applied-safe-state.json` records booleans without credential values; screenshot06. An initial assertion used a mistyped expected model (`model_z`); actual nonsecret value was `model-z`, matching the displayed fixture. The corrected check passed. |
| Appearance/language | Actual dark and light solid paper scene, rounded silhouette, green focus/selection, readable opaque panels; Chinese settings and profile labels rendered. Screenshots01/07/08. This is CJK rendering, not IME composition or OS accessibility acceptance. |
| Missing catalog / Retry | Fresh `solid-retry-20261027` showed a real safe missing-catalog error. Main copied only the closed seed DB into that run's previously absent missing directory, reobserved, then clicked Retry. Two real rows and expected current returned. Both screenshots and final receipt retained. No active database was copied. |
| Pending read / exit | Fresh `solid-cancel-20261027` had a controlled30000ms read delay and visible Reading catalog. Main closed before natural completion; receipt lifetime25.9259218s, exit0, no residual candidate process. This proves cancellation/cleanup for this fixture; it is not a performance sample or a100ms cancellation latency claim. |

Each run has native stdout/stderr, executable hash, explicit profiles/registry inputs, final exit0 and final fixture hashes under `outputs/desktop/profiles/native/`. Summary is `solid-core-summary-20261027.json`. No real user config/account, global HOME, size/DPI change, commit or archive was used.

## Actual failures still requiring fixes and new evidence

1. After cancelling the stale editor and explicitly refreshing, the correct external value is visible but the old status still says the draft is retained. Screenshot05 captures this misleading recovery text. Data CAS passed; this feedback state did not.
2. With Light/Chinese active after returning from Settings, clicking Off opened its confirmation while keyboard focus remained outside the decision subtree. Native Tab moved focus to sidebar Favorite commands, and Escape did not dismiss the confirmation. Screenshots09/10 capture the open confirmation and escaped focus. Main clicked Cancel safely; no off mutation or delete was submitted. This is a real keyboard acceptance failure, not an untested IME/UIA category. The existing headless precondition did not cover this actual focus path. Trellis checker owns reproduction/fix.

Therefore neither complete04 core keyboard acceptance nor parent/03 completion is claimed. The candidate executable remains immutable so these failures can be distinguished from subsequent repairs and05 code. Native glass is unsupported on the inspected stack; system-preference integration, OS UIA/IME, other OS, full command coverage, S1/S2 performance and packaging remain independently open. Native window/DPI matrices remain user-deferred.
