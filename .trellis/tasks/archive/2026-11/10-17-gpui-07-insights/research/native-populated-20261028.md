# Populated native read verification, 2026-10-28

Main used immutable baseline candidate SHA256 `843f04f8229b97918e73ceaef4c0c0546d21ba880197358fcedc02cf7872ca00`. Its source/three-lock identity and unchanged-source build receipt are in `outputs/desktop/continuation-20261028/baseline-*.json`. This candidate predates the new archive-refresh and subsequent07 source changes.

The domain-owner-generated isolated fixture is `outputs/desktop/insights/native-populated-20261028/`; `fixture.json` records roots and exact values. No direct SQL was added outside ccr-usage, no upstream llmusage Rust dependency, real HOME, installed-CLI sync, auth or quota request was used.

Run receipts, actual actions, screenshots and stdout/stderr: `outputs/desktop/profiles/native/baseline-07-20261028/`. Explicit grouped Insights paths are recorded; process58160 exited0. Size/DPI user-deferred; this is not S1/S2 or memory/frame performance evidence.

| Scenario | Actual native result |
| --- | --- |
| Home populated |102 requests,200 tokens,205 indexed archive sessions; source comparison shows Codex101 and Claude1 requests |
| Usage exact values |Unfiltered USD0.35802468; provider `openai` + Apply yields USD0.123456789 and100 tokens, canonical fixture values |
| Records |Provider-filtered101 records in100+1 pages; first page visible and final single row visible after explicit activation |
| Archive |205 real provider-backed summaries paged100/100/5; first rows204/104/004 respectively, final page004–000 |
| Detail |Selected Native fixture204 with250 messages; first page ordinals150 onward, older page50 onward; selected message150 shows exact synthetic text and partial fidelity |
| Virtual scrolling |Older detail page scroll changes visible rows from50 onward to78 onward. This demonstrates actual scrolling; the10k visible-entity bound is a separate rendered test, not measured native performance |
| Plaintext exit |Back to archive removes selected message reader; close exits0 |

An immediate inactive last-page capture briefly showed the row count without the row. Subsequent active capture showed the final row; independent actual-row pagination regression did not reproduce a defect. No speculative pagination fix was made. As with04, unrelated immediate foreground captures were removed; only verified GPUI captures are retained. Do not promote capture artifacts to product failures.

These are selected read/filter/page/detail native passes on this baseline. New refresh race/generation guards need a new candidate. Native import/sync/observer/monitor/budget/pricing/quota, complete49-command migration, all platform/theme/language variants, OS UIA/IME, three-platform and performance acceptance remain open.
