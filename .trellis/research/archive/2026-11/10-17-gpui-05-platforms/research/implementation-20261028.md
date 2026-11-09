# GPUI 05 continuation checkpoint — 2026-10-28

Active task: `.trellis/research/archive/2026-11/10-17-gpui-05-platforms`. Implementer owns 05 source; shared workspace edits preserved. Injection contained the wrong 04 task and a truncated hook output, so the explicit 05 task artifacts, both manifests, source map and relevant owner contracts were loaded. No task was recreated, no dependency/version/lock changes were made, and no commit, push, publish, sign or archive was performed.

## Implemented boundary

- `ccr-cli::application::codex_oauth::NativeSession` connects the existing controller to a captured-root backend and the existing bounded app runtime. One tracked start runner stays alive until listener join, pending-secret cleanup and any already-started account commit finish. View close requests cancellation; it does not detach the writer. A pre-admission claim also permits cancellation before the job starts.
- Native Codex Accounts now provides Start OAuth, a scoped authorization URL, manual callback submission, explicit account completion, cancellation and failed-cleanup retry. Callback acceptance never presents account success. Fixture token exchange rejects before constructing an HTTP request, and its displayed authorization text contains no real external authorize URL. No browser is automatically launched.
- The Tauri account finalizer delegates the same extracted account-name/import/switch policy. Existing domain `CodexAuthService` still owns credential persistence and switching; Tauri wire DTOs remain in place. Failure after import and before switch remains an error, not a claim of successful activation.
- Controller terminal cleanup now clears the retained verifier/state/authorization/callback buffers. Native terminal feedback clears the callback textarea as well as the scoped authorization URL. Account/environment navigation invalidates previous UI generations.
- Independent check reproduced 05 confirmation entry failing with raw MouseUp → native callback-before-draw order and outside focus. Production repair now schedules one-shot initial entry from mounted confirmation render with the decision-generation guard. Existing 04 source was not changed by this implementer.
- Existing Codex model-provider owner defects were reproduced before repair: failed backup was ignored and parser diagnostics echoed a synthetic secret. `save_locked` now propagates the existing guarded secret-write/backup policy; provider upsert/delete read under the existing operation lock. Explicit captured leaf-lock injection preserves fixture isolation without a second persistence owner. Native provider action wiring is still pending at this checkpoint.

## Recorded checks

| Scope | Actual result | Raw log |
| --- | --- | --- |
| Initial extracted OAuth owner selection | PASS 6 | `outputs/desktop/platforms/continuation-20261028/oauth-initial.log` |
| OAuth owner including native lifetime fixtures | PASS 8 | `outputs/desktop/platforms/continuation-20261028/oauth-owner-tests.log` |
| Provider regression before fix | FAIL 2, PASS 1 | `outputs/desktop/platforms/continuation-20261028/provider-before.log` |
| Provider regression after first fix | PASS 3 | `outputs/desktop/platforms/continuation-20261028/provider-after.log` |
| Independent 05 focus before fix | FAIL, Cancel was not focused | `outputs/desktop/platforms/check-20261028/focus-before.log` |
| Independent rendered native consumers after fix | PASS 3 | `outputs/desktop/platforms/check-20261028/consumer-after-compiled.log` |

The eight OAuth tests include callback identity, deadline/listener cleanup, cancellation with commit join, rejected fixture network exchange, callback not being commit success, pending disk cleanup and terminal in-memory secret clearing. The three rendered consumer tests cover source acknowledgement/teardown, outside-focus initial entry/Tab/Escape, and Start → Waiting → retained invalid callback draft → Cancel → terminal clearing with the actual shared runtime, owned loopback port and pending store. These are headless rendered GPUI fixtures, not actual native OS operation evidence.

Latest frozen-source CLI/Codex owner tests + strict all-target/all-feature Clippy and independent desktop all-target/strict checks are being finalized; append their actual result below rather than infer it from an earlier tree. Repair formatting was explicit and separate from validation. Default test parallelism, `--locked` and `--skip export_bindings` were preserved.

## Inventory and open acceptance

`command-map-20261028.json` maps all 54 original command IDs separately to source/action, authored tests, executed checks, native acceptance and remaining work. At this checkpoint: 30 implemented native actions, 3 partial, 21 pending. This is implementation accounting, not final command acceptance. In particular official logout, remaining account/import/API-key/provider commands, typed Grok settings, layer/model/keybinding/theme probes and external-browser/port-discovery parity remain open. No absent native action is counted through a service-only test.

Full native OAuth success, real accounts/services, native populated settings/account/provider parity, full command/domain acceptance, other OS, IME/UIA, real application glass and performance remain unverified. Fixture network prohibition is a deliberate validation boundary. Window/DPI remains user-deferred. Whole task and parent acceptance remain open.
