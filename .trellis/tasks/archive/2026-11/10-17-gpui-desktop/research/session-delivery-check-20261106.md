# Session delivery independent check — 2026-11-06

Reviewed the actual uncommitted source against `session-delivery-20261106.md`,
the task/design contracts, shared desktop spec and Codex session-recovery spec.
No native window/dialog was launched. The user's native deferral remains in
force. Performance fixture main/state/logger/catalog/DB initialization work was
excluded and preserved.

## Findings (fixed)

### P1 — Native validation leaked into legacy restore

`codex_session_trash_service.rs::restore_trash_entry_versioned` originally called
absolute/reparse validation before branching on `expected`. The existing CLI
`commands/codex/sessions.rs::build_service` accepts a relative `--codex-home`
without canonicalization. Consequently legacy trash succeeded but restore
failed with `ValidationError("Session paths must be absolute")`.

Added `legacy_restore_accepts_an_explicit_relative_codex_home`: it creates its
own temporary directory under the actual test cwd, changes no cwd/HOME, captures
fixture locks, and exercises legacy trash/restore. After correcting an initial
test-fixture omission (rollout parent creation), it reproduced that exact
product failure. Moved native checks into `Some(expected)` only; the native
reviewed path remains absolute/reparse guarded and legacy streaming semantics
remain intact. The final full narrow trash suite passed **9/9**, including the
existing >64 MiB legacy streaming regression. The source spec now records this
branch boundary.

### P2 — GUI trash built every row

`features/insights/session_operations/delivery.rs::render_session_trash` used an
unbounded `.iter().enumerate()` render loop. A large trash collection constructed
all rows, contrary to the native bounded-list contract.

Changed only this local composition to six-entry pages. Rows use `2 * ROW_HEIGHT`,
stable session/date/path identity, existing theme tokens and standard Kit
tooltips for clipped title/metadata. Previous/Next stay mounted and use disabled
state at the bounds. The existing session-page scroller remains the only scroll
owner; pagination avoids introducing a virtual list whose offscreen Review
buttons would have no built-in keyboard traversal. `clear_session_draft` resets
the page. Review callbacks continue capturing the exact original `TrashItem`.

Added `session_trash_pagination_bounds_large_collections_and_keeps_controls_reachable`
using one real trash entry and 10,000 synthetic metadata clones. It checks six
rendered Review buttons, actual Next/Previous navigation and clear/reset. This
is a render bound fixture, not an artificial filesystem generator or performance
sample. Existing restore-control coverage was also strengthened with actual
outside input focus, early Enter without restore, safe Cancel entry, Tab/
Shift+Tab, Escape and input-focus return. Desktop results are recorded below.

The first shared desktop compile found that the two new tooltip Divs needed
stateful `.id` before `.tooltip`; both local child IDs were fixed and formatted.
No shared Kit API or other worker's files changed.

## Findings (not fixed)

### P2 — Opening trash eagerly reads every rollout body

`runtime/insights/session_operations/delivery.rs::read_session_trash` calls
`CodexSessionTrashService::prepare_restores`. That method enumerates every trash
entry and calls `trash_file_version`, which reads/hashes each rollout up to the
native 64 MiB budget before returning the metadata list. It does not inspect a
cancellation predicate during enumeration. The GUI's six-row page therefore
bounds rendering only; it does **not** bound total list I/O or make cancellation/
shutdown responsive during a large body scan. One oversized/unreadable payload
also prevents listing otherwise restorable entries. The synthetic 10k metadata
render test provides no evidence about that actual backend cost.

This needs a public domain API/interaction-state change, so it was reported for
the main session to dispatch to a real implement role rather than silently
rewriting the boundary inside this check. Approved minimal follow-up contract:

1. Add an opaque metadata-only candidate with captured home and manifest token.
   List candidates without opening rollout bodies; check cooperative cancellation
   at each enumeration step. Keep native manifest safety/size bounds and legacy
   `restore_sessions` streaming behavior; do not invent a legacy cap.
2. Add single-candidate preparation that compares the original metadata/manifest,
   then reads/hashes only the selected bounded rollout. Preserve the original
   candidate authority; do not silently reread a newer manifest as consent.
3. Runtime list results carry metadata/candidate. A distinct typed PreparedRestore
   holds the item plus private full plan. Review invokes the worker preparation;
   the view shows confirmation only after the matching generation/decision
   returns. Leaving/cancelling preparation drops subscription and clears state.
4. Commit remains the existing retained job consuming the exact plan with
   manifest/payload revalidation, private no-overwrite CAS/index rollback and
   partial outcome semantics. It is not cancelled by navigating away.
5. Adapt existing tamper tests to list → selected prepare → mutate → restore.
   Add proof that listing does not read an unselected payload, cancellation stops
   enumeration, and a changed selected candidate never gains refreshed authority.

The expected production scope is Codex trash owner, runtime session delivery/
types, feature session delivery/state and their existing tests (about five
production modules). No second trash format, generic framework or new window.
The current public `prepare_restores` can remain compatible for existing callers;
the GUI must stop using it to prepare the entire list.

No further source defect was found in the reviewed Grok or usage capability
slice. Final integration/quality gates and current native behavior remain open
evidence, not source-review passes. Actual Save dialog interaction, native focus
pixels/UIA/IME and default-window presentation are user-deferred. Neither
headless control tests nor compiler success establish those outcomes.

## Session source review

- **Save binds the reviewed preview.** The view captures the exact export Arc,
  archive identity and generation/decision when opening `prompt_for_new_path`.
  Cancel admits no writer; stale selection/page results are ignored. The writer
  does not silently export a newer transcript. Navigation drops view plaintext;
  admitted writes are retained by the existing app-owned bounded job until real
  completion, and the retained Outcome has no transcript contents.
- **Destination safety.** Save rejects non-absolute/traversal destinations and
  linked/reparse ancestors. Fixture output compares the canonical existing
  parent to the canonical fixture root, including Windows ordinary-versus-
  verbatim path representations. Guarded absent-token CAS prevents replacing
  existing files and applies `secret: true`. The existing Unix assertion checks
  0600 on Unix; it is not Windows DACL inspection. Windows uses the existing
  private AtomicWriter owner, whose policy was not reimplemented here.
- **Reviewed restore authority, once prepared.** Opaque restore plans bind explicit Codex home,
  full manifest bytes, payload version and metadata. At commit, the same trash
  resource lock is held, manifest/record identity is compared, and checked
  bounded payload bytes are the actual bytes written. Native targets, trash,
  rollout and index paths reject linked/reparse ancestors. This does not close
  the eager whole-list payload preparation finding above. Relative manifest
  targets must remain under sessions/archived_sessions.
- **No overwrite and rollback.** Existing rollout paths or index session IDs
  stop restore. The native rollout uses absent-target CAS, index append compares
  its original content revision, and index-conflict rollback deletes only the
  exact newly written rollout. It preserves a concurrently edited index or
  externally replaced rollout. Native and legacy compatibility branches remain
  distinct; private new native files do not require changing the legacy API.
- **Truthful partial completion.** Mtime/index/cache/trash-cleanup errors can
  follow a source mutation; the native adapter returns Partial rather than
  asserting rollback. Successful source restore plus failed archive update is
  retained as success with `archive_updated=false` and an explicit warning.
  Archive refresh uses the same captured provider registry without environment
  overrides. Foreign homes and remote environment requests are refused.
- **Archive liveness.** The DB source projection adds the stored source_state.
  Container/group/member fast paths require corresponding live archive rows.
  Restored original bytes/mtime therefore reparse and upsert the original
  archive identity to live. Missing/deleted entries whose source remains absent
  do not enter discovered sources; identity reconciliation only marks previously
  live rows missing, preserving absent deleted rows. The change does not flush
  the whole fingerprint cache.
- **Exports/wiring.** `CodexSessionRestorePlan` is exported through the existing
  services/lib facade; its proof fields remain private. No second trash format,
  direct permanent deletion, usage SQL owner or runtime/window stack was added.

## Additional Grok source review

`runtime/profiles/overview.rs` uses captured ProfileLifecycleContext and the
existing Grok owner's `inspect_activation_state`, not a current/default badge.
The owning load/inspect chain reads profile, registry, runtime and entry-state
files without applying/reconciling them. Active/Inactive/Drifted/unsafe missing
baseline remain distinct. Profile credential mode is an enum, enabled/total
counts remain separate and no secret field enters the report. Remote/wrong-
platform admission precedes local source resolution. Workbench suspend resets
Grok data; generation rejects stale returns and other platforms do not render
its section. No new source defect found. Its source/control tests belong to the
coordinator's current candidate; this reviewer did not rerun them.

## Additional usage-capability source review

Reviewed `ccr-app/src/usage/{mod,capabilities}.rs`, Tauri's capability adapter,
native runtime/features capability modules and Insights integration.

- Shared detection uses the existing ccr-usage read-only DB capability owner and
  the closed installed-CLI version ProcessDescriptor/Gateway. It rejects failed,
  timed-out or either-stream-truncated probe results. No upstream crate, DB
  migration or import is introduced.
- `probe_cli=false` bypasses the ambient executable and labels CLI execution as
  disabled for fixture, preserving the distinction from actual absence. Missing
  DB support is not fabricated by finding a CLI. Upstream graceful cancellation
  remains unsupported independently of local import cancellation.
- Tauri retains its wire fields/TS ownership while mapping the shared Report.
  Native CLI version display is redacted and capped to 256 characters. The
  feature keys match ccr-usage FeatureKey::as_str. Report scope is explicitly the
  captured whole database rather than the current platform/date filter.
- Native work runs on the existing worker; render only consumes state. The
  capability subpage is reachable from Usage, refresh has real busy state,
  and suspend drops its task/data and advances generation. There is no hidden
  poll. No new source defect found; the coordinator's supplied shared **2/2**
  receipt predates the final timeout/truncation guard, so final tests must cover
  that latest helper and native control candidate.

## Verification

- Root narrow regression: intentional product red after fixture setup correction,
  then `cargo test --locked --offline -p ccr-codex codex_session_trash -- --skip export_bindings`
  **exit 0, 9 passed** on pinned Rust 1.98.0, default parallelism. Root target is
  the pre-existing external mbx symlink; initial sandbox write denial was
  resolved through approved escalation, without changing target or cache policy.
- Scoped rustfmt/diff check on reviewer-owned changes: **pass**. One earlier
  rustfmt attempt encountered a transient Windows mapped-file error; subsequent
  formatting succeeded. No gate threshold or warning suppression changed.
- Supplied unchanged evidence inspected by actual assertions: root trash 8/8
  before this extra regression; shared refresh 3/3 covers unchanged-live,
  restored/missing file bytes+mtime, absent deleted rows and SQLite members;
  desktop session 5/5 before reviewer pagination/focus changes; prior 1 MiB
  session control child pass predates these changes. Counts overlap and are not
  summed as independent coverage.
- Desktop session/pagination/focus controls and 1 MiB guard: **pending current desktop
  lease owner's run**. New test name was sent to `platform_continue` for the
  existing native stack guard. This reviewer did not run concurrent Cargo.
- Lint/type-check/final integration: **pending coordinator's stable aggregate**.
- Native UI/dialogs, real user data/network/SSH/WSL, benchmark samples and full
  `just ci`: **not_run by this reviewer**.

This check updated only local session composition/tests, the local legacy
restore regression/fix and its two owning specs, plus this report. Other workers'
changes and the performance fixture engineering remain intact. Per coordinator
handoff, no further production edits are being made here; the eager-listing API
fix is explicitly left to the next session implementer.
