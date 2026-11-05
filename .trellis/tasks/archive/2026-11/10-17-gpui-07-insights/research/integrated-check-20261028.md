# Independent integrated checkpoint, 2026-10-28

Scope: the frozen 07 archive-refresh/budget/pricing slice and the 05 Codex
provider-store owner repairs. This is not complete 07, 05, 04 or parent
acceptance. Main retains source/lock integration, native actions and final
judgment. No commit, push, publication, signing, archive, GUI action, real HOME,
real account, external sync or quota request was performed by this checker.

The injected hook was truncated and selected 04. Its saved source was read in
chunks; the explicit 07 task plans/manifests, current implementation/check/native
reports, and relevant usage/job/session/shared-runtime/store/Codex contracts
were loaded directly. The 05 task plans and previous independent report were
also read for the provider-store boundary. Concurrent 06/08 modules retained
their owners; no shared manifests, locks, exports or shell files were edited.

## Findings verified

No additional concrete product defect was found in the reviewed frozen slice.
No product source edits were needed in this checker turn.

- `usage::policies::update_budget` validates the complete supplied object
  before constructing the manager or calling a setter. Boolean type, finite
  nonnegative limits or null, and integer warning range 0..100 are checked;
  300 cannot wrap to 44. The retained tests compare exact original bytes after
  invalid later fields, not merely an error return. Valid field setters retain
  the existing sequential I/O semantics; no multi-field transaction or CAS
  guarantee was invented.
- Budget/Pricing managers keep the original TOML privately. Known keys are
  replaced from typed state, unknown top-level values remain, and surviving
  model/default-pricing rows retain their unknown extensions while known
  optional prices can be removed. Deliberate model removal/reset does not
  resurrect a deleted model through the original document. The private saved
  original is advanced only after the existing guarded atomic writer returns
  success. Public persisted models and wire projections did not gain extension
  fields. This does not add cross-client read/modify/write transactions or
  whole-operation rollback guarantees to the legacy managers.
- Native policy commands reject non-local/unconfigured inputs and use captured
  provider roots plus fixture-local scoped leaf locks. Existing legacy cost
  tracking remains distinct from read-only upstream llmusage stored pricing.
  The view reports sequential/partial budget I/O accurately; native errors are
  static categories and do not print the shared raw parser/path error.
- Archive refresh remains provider-registry/repository owned. Failed discovery
  is not authoritative missing-state reconciliation. Unchanged source/container
  state can skip parsing. Source bodies remain provider-owned; only CCR archive
  summary/fingerprint/state data is persisted through its repository transaction.
- Refresh admission/retained status and the bounded app-owned runner remain
  alive after a view subscription detaches. The view terminal callback reloads
  only the same generation in Sessions with no detail selected, so a newer
  filter/page/detail is not replaced. This existing refresh has no cancellation
  API; it does not satisfy usage-import cancellation/deadline/cleanup acceptance.
- The new v2 populated fixture uses exact paths and fingerprints returned by
  the actual provider registry. Its helper checks 205 unchanged/zero upserts and
  stable archive pagination after refresh. The earlier duplicate was a manual
  fixture spelling mismatch; no speculative domain ID-normalization change was
  introduced, and old fixture/logs remain preserved.
- The 05 provider owner now reads upsert/delete state under its operation lock,
  binds nested guarded writes to the captured leaf-lock directory, propagates
  backup failure before replacement, and applies secret-file policy through the
  existing owner. Invalid JSON reports location without source values. Public
  full-store save remains a distinct replacement operation; no stale-editor CAS
  is claimed for this older API. This credential store is not a non-secret
  provider template catalog.

## Executed independent verification

Raw logs/command receipts: `outputs/desktop/continuation-20261028/`.
Existing paired Rust/Cargo 1.98, process-only PATH and root alternate target
were used; desktop retained its independent target. Locked checks, default
parallelism and `--skip export_bindings` remained intact.

| Receipt/log stem | Actual result |
| --- | --- |
| `independent-desktop-fmt` | Exit 1 from concurrent 06 owned `resources/system_prompts.rs` formatting; notified its owner, no 07 repair inferred |
| `independent-desktop-fmt-after` | Exit 0 after 06 separately formatted only its owned files |
| `independent-desktop-check` | `just desktop-check`, locked all-target check, exit 0 |
| `independent-desktop-lint` | `just desktop-lint`, locked strict all-target Clippy, exit 0 |
| `independent-desktop-tests` | `just desktop-test`, exit 0: 38 library + 4 native_shell + 2 native_ui Rust tests; 12 Python and 3 AccessKit regressions; no failures |
| `independent-provider-tests` | Three provider-store tests passed, including backup failure and safe parser diagnostic |
| `independent-provider-lint` | `cargo clippy -p ccr-codex --locked --all-targets --all-features -- -D warnings`, exit 0 |

The 07 implementer's unchanged frozen-source results were also consumed rather
than unnecessarily rerun: store 84 library + 7 integration + 1 doctest passed
with 5 existing ignored; shared usage 8 passed; focused native insights 9
passed; v2 seed helper 1 passed. Their original raw failure/repair logs remain
under `outputs/desktop/insights/checks/`. These are owner-run results, not
checker reruns. The all-source hashes in the 07 checkpoint still matched after
the independent gates; provider-store identity and exact gate receipts are in
`outputs/desktop/insights/checks/independent-integrated-20261028.json`.

## Findings not fixed / remaining scope

- Existing 05 API-key/provider consumer gap:
  `ccr-ui/src-tauri/src/commands/codex_auth.rs:1348` ignores
  `provider_store.upsert_provider(provider)` when `save_provider` was requested,
  then proceeds to account finalization. A correctly propagated store backup
  failure can therefore be ignored by that legacy adapter. This file and its
  fatal-versus-committed-warning operation policy were outside the handed-off
  provider-store review. Main has recorded it for the next 05 implement owner;
  no silent public/operation semantics change was made here. The native API-key
  and provider action remains unimplemented, not PASS.
- Current 07 mapping still has 29 native commands unimplemented. Import/sync,
  observer/subscription/monitoring, dedicated Codex mutable sessions/rolling
  usage, capabilities/readiness/Home bootstrap and quota prerequisites remain
  genuine local work. Refresh and legacy policy controls do not satisfy them.
- 05 official logout and the remaining account/provider/settings/port/browser
  commands remain open according to its complete 54-command map. File presence,
  callback receipt, fixture cancellation or owner tests do not prove account
  commit or official logout success.
- This checker did not build or operate a new native binary. Main must freeze
  the integrated source/locks and candidate identity, use the v2 fixture, and
  verify the repaired 04 focus/CAS plus new 05/07 native controls on that exact
  candidate. Earlier baseline native results prove their own artifact only.
- Actual application glass remains unsupported on the inspected locked public
  API. Other OS, IME/UIA, performance, packaging, full bindings/final just ci and
  user-deferred size/DPI remain separate acceptance conditions.

Main should append executable spec notes for the budget prevalidation/private
TOML-preservation and refresh query-generation contracts. The current slice is
ready for the main session's next candidate freeze, with the limits above.
