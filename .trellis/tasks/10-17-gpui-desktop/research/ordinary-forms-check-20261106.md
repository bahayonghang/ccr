# Ordinary Settings/provider/MCP independent check — 2026-11-06

Reviewed the actual source and changes against the task contracts and `research/ordinary-forms-20261106.md`. This review covers ordinary platform forms, their existing Advanced round trip, provider metadata projections/fallback, Grok typed patch policy, MCP common fields/header merge and the selected-entry action placement. It does not extend to the concurrent Tauri performance isolation implementation or session candidate implementation. No native application was launched; the user explicitly deferred native verification.

## Findings (fixed)

### P1 — blank named credential replacements overwrote saved values

- File: `ccr-desktop/src/features/mcp/editor/form.rs`, with regression in `form/tests.rs`.
- A nonempty env/header name plus an empty replacement submitted an empty string. This contradicted the editor's omission contract and could erase a saved credential simply by naming it.
- Reproduced through actual Codex MCP controls: saving a blank `Authorization` replacement changed the persisted value from `stored-secret` to `""`. The pre-fix assertion failed on that exact persisted value.
- Ordinary replacements now omit empty values, including named rows. Advanced remains the explicit clearing mechanism. The final control regression saves blank named header and env replacements and verifies both stored sentinels remain. Existing inaccessible saved values are never preloaded; the private conflicting replacement remains in the editor after CAS failure and is absent from the stored file.

### P2 — HTTP-to-stdio retained an explicit HTTP transport type

- File: `ccr-desktop/src/features/mcp/editor/form.rs`, new regression `mcp_common_form_switches_http_to_stdio_without_stale_transport_type`.
- Changing an existing Claude HTTP entry to a stdio command removed its URL but retained `type = "http"`. The new actual control/writer regression failed on the persisted stale type.
- A changed nonempty stdio command now removes the selected URL key and non-Codex explicit type. Unknown extension siblings remain. The test reads the written JSON and invokes the shared Claude MCP listing owner: command is `node`, URL/type are absent, extension remains 7, and the existing owner can read the result. This is a local known-field correction, not a cross-source conversion policy change.

### P2 — MCP confirmation had no safe mounted-button entry or stale-focus guard

- File: `ccr-desktop/src/features/mcp/editor.rs`, regression in the existing common-form control test.
- The previous render immediately focused a trap container; it did not enter the safe Cancel button after the decision controls mounted. The added actual assertion reproduced `Cancel.focused() == false`.
- Applied the existing local Claude editor pattern: capture return focus, schedule entry from mounted confirmation render, enter Cancel through the trap's tab order, guard the callback with an opening generation, and invalidate it on dismissal/commit. Escape/Cancel restore the prior input. No generic modal framework or global wrapper changed.
- Final assertions exercise Enter before deferred entry without writing, safe initial Cancel, Tab and Shift+Tab wrap inside the two decision buttons, Escape restoration, and dismiss/reopen before the old callback runs. Raw mouse events model callback-before-draw ordering; the stale callback cannot steal prior focus, and the new mounted confirmation later enters Cancel.

The shared desktop service spec now records ordinary empty replacement omission, transport field exclusivity and the existing confirmation focus contract.

## Other approved source paths

- `features/platforms/forms.rs` and `forms/tests.rs`, `platforms/mod.rs`: ordinary field changes are compared with their initial values; absent bool/enum facts do not silently submit false/default choices. Advanced JSON is retained and synchronized back into ordinary controls without discarding unknown base keys. Invalid input and conflicts retain the current draft. Leaving clears the editor entities.
- `runtime/platforms.rs`, `runtime/platforms/grok.rs`: Grok accepts the actual managed key/type/range policy shared with the existing Tauri implementation. The guard checks protected default/reasoning/custom-model changes before the captured document CAS write; ordinary forms do not bypass active managed state.
- `runtime/platforms/providers.rs`: Codex metadata projection comes from the captured original provider record, omits keys and URLs containing user information/query/fragment, and falls back to the captured metadata when unsubmitted fields are required by the typed writer. Existing keys and creation metadata remain owned by the original versioned service. The typed store does not promise raw unknown-field retention, and the new ordinary form does not add that promise. OpenCode uses its existing nested merge owner.
- `runtime/mcp/editor.rs`: common fields expose only filtered command and the URL key name; stored args, URLs, env/header values do not enter replacement controls. Captured source/scope/version still govern saves. Codex `http_headers` patches merge the submitted map into the captured mapping so omitted secrets survive; malformed submitted header values fail before write. Claude/Codex/Antigravity retain their owner-readable URL/header formats, including an existing URL field name and Antigravity's new `serverUrl` shape.
- `features/mcp/mod.rs`: selected-entry Copy/Edit/Delete actions precede the longer metadata section, preserving default-window reachability established by the existing headless bounds assertions. Preset, copy, batch and Advanced entry points remain wired. Bounds assertions establish fixture layout reachability, not native pointer/OS acceptance.

No additional source defect was found in these approved paths. The ordinary controls intentionally expose a defined subset of settings; remaining keys retain their Advanced/source editing route.

## Verification

Independent narrow commands used the pinned Rust 1.98 toolchain, the existing desktop target, `--locked --offline --lib`, default test parallelism and `--skip export_bindings`:

| Check | Actual result |
| --- | --- |
| New MCP form filter, before data fixes | Red: 2 failures, persisted blank header overwrite and retained HTTP type |
| Same filter after data fixes, with new initial-focus assertion | Red: transport test passed; Cancel initial focus failed |
| Final `features::mcp::editor::form::tests` | PASS 2/2, including final env preservation and stale callback ordering assertions |
| Existing `features::mcp::tests` | PASS 4/4 after the production fixes; the later changes only strengthened the new form test |
| Scoped `rustfmt --edition 2024 --check --config skip_children=true` on the three edited MCP files | PASS |
| Rust type checking for changed production/test paths | PASS through the actual compiled lib-test artifact |
| Standalone strict clippy / whole-desktop gates | Not rerun here; parent owns the consolidated gates after the returned compile lease |

The unchanged owner receipts are reused as supplied evidence, not claimed as newly executed checks: ordinary platform forms 2/2, old platform controls 7/7, runtime MCP 2/2, Grok typed patch 1/1. Parent separately supplied Grok overview runtime 2/control 1 and capabilities control 1 plus shared capabilities 2; these are not added to this batch's independent test count.

Compilation emitted existing incremental-cache hard-link fallback warnings; it completed successfully. No aggregate 1 MiB guard result is claimed here. Its existing MCP form child now includes the enhanced regression. The separate new transport regression name was sent to the parent for its guard ownership decision.

## Findings (not fixed) / remaining acceptance

No unresolved code finding remains in this approved batch. Native screenshot/layout, OS focus/UIA, IME and native loading remain **user-deferred / not_run**. Headless controls and source inspection cannot establish a native visual shipping pass. The known mechanical design scanner covers zero Rust files; that is a coverage gap, not a clean scan verdict. Consolidated fmt/clippy/test/1 MiB guard results remain with the parent and must be reported separately when actually run.
