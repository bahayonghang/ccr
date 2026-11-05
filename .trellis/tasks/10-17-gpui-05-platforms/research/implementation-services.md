# GPUI05 service implementation

Date: 2026-10-27. Active task: `10-17-gpui-05-platforms`.
Owner: dispatched Trellis implementer. Shared workspace; preserve03/04 changes.
This document is implementation planning/evidence, not native acceptance.

## Boundary and sequence

The54-command/eight-route source map in03 `research/platforms-source-map.md`
remains the inventory. The smallest missing behavior is a native caller for the
existing settings/auth owners, with original editor revisions and scoped secret
lifetimes. Native rendering does not become a file/auth/OAuth owner.

1. Add `ccr-app/platform_settings`: explicit paths, non-Debug text snapshots,
   existing core content-token writer, safe syntax errors and conflict outcomes.
   Tauri raw commands and native consumers use this same primitive. Existing
   typed platform policy is extracted/reused separately, never copied into UI.
2. Move the framework-free OAuth state machine from Tauri to the existing
   `ccr-cli/application` owner (existing dependencies suffice). Keep TS/wire DTOs,
   AppHandle events and ProcessGateway browser policy in Tauri; native injects
   captured roots and keeps a tracked runtime job alive through cleanup/commit.
3. Native settings/auth requests bind environment/platform/generation through
   the existing bounded runtime. Fixture roots derive only from explicit launch
   inputs. Account/profile sources, CAS, managed Grok locks and official-vs-local
   logout retain domain ownership. No real credentials, login/logout request,
   global HOME or system preference changes are used for testing.
4. Add capability-driven native settings/auth/provider surfaces after03's shell
   write owner is transferred. Keep03 geometry/radii, fixed row heights,
   theme owner and04 editor lifecycle unchanged.
5. Map54 commands/eight routes to actual actions and regression evidence;
   Antigravity's two legacy settings commands need an explicit advanced action
   or reviewed consolidation, never an invented descriptor surface. Device-code
   login is not invented where the repository has no such owner.

## Checks

Use explicit temporary roots; meaningful fixtures cover raw invalid/no-write,
two original-revision clients, missing-vs-empty, legacy fallback/primary races,
dirty-only and unknown sibling retention, managed locks, OAuth callback identity,
timeout/cancel/commit/cleanup and tracked app shutdown. Domain/Tauri parity,
typed bindings and native control tests stay separate from real native/IME/OS
evidence. Keep default parallelism, `--locked`, `--skip export_bindings`.

Root Cargo's stale target link is preserved; process-only alternate target is
`outputs/desktop/profiles/resume-current-check/target`. Desktop retains its own
workspace/lock/target. No dependencies, manifests or locks are changed without
the coordinator's single-owner handoff. No commit/push/archive is performed.

## Results

### Recovered checks, 2026-10-27

| Command / scope | Exit | Actual result |
| --- | --- | --- |
| `cargo test -p ccr-app --locked platform_settings -- --skip export_bindings` | 0 | Five shared-settings tests passed: missing/invalid no-write and secret-safe diagnostics, original-token two-client conflict, absent versus empty, fallback/primary conflicts, Codex typed patch retention. `outputs/desktop/platforms/settings-recovery-tests.log`. |
| `cargo test -p ccr-cli --locked application::codex_oauth:: -- --skip export_bindings` | 0 | Four extracted-owner tests passed: RFC7636 S256 vector, callback identity/cancel/secret cleanup, deadline cleanup/listener release, and waiting for an in-progress commit at shutdown. `outputs/desktop/platforms/oauth-recovery-tests.log`. |
| Windows-filtered root/Tauri `cargo metadata --offline --format-version 1 --filter-platform x86_64-pc-windows-msvc` | 0 / 0 | Required07 shared Home dependency/export batch resolves; existing package records remain unchanged apart from `ccr-app` dependency edges. |
| Desktop `cargo metadata --locked --offline --format-version 1 --filter-platform x86_64-pc-windows-msvc` | 0 | Existing934 packages preserved; only required `ccr-usage7.5.0`, `chrono-tz0.10.4`, `phf0.12.1`, `phf_shared0.12.1` added. Direct native `ccr-usage` / `chrono=0.4.45` edges subsequently verified with the same locked command exit0. |

Root checks use the existing real Rust/Cargo1.98 executable and process-only
`CARGO_TARGET_DIR=outputs/desktop/profiles/resume-current-check/target`. Default
parallelism and export filters are preserved. No global toolchain/cache changes.

Dependency receipts are under `outputs/desktop/platforms/dependencies/`:
`usage-lock-integration.json`, the before-lock snapshots, and final metadata.
The first all-platform offline root metadata attempt could not fetch uncached
`js-sys0.3.91`; the Windows-filtered retry passed. Offline desktop resolution
initially proposed ten unrelated cached-version downgrades. That graph was
rejected and retained as `desktop-rejected-offline-resolution.lock`; the exact
existing records were restored, only the four necessary records and explicit
edges were added, and final `--locked` verification passed. No downgrade is
delivered. Primary API metadata for existing sha2 graphs remains in
`sha2-0.11.0.json` / `sha2-0.11.1.json`;07 owns the additional chrono/tz/phf
official-source receipt in `outputs/desktop/insights/dependencies/receipt.json`.

The moved `CodexConfig` model and nested parsed-file types are internal serde
types, not TS-exported wire DTOs. Tauri retains its command/wire DTOs and export
paths. Existing settings/OAuth consumer parity, typed binding generation,
complete54-command mapping, new native05 pages and actual native verification
remain **NOT_RUN / implementation in progress** until their corresponding
checks run. These focused shared-owner tests are not complete05 acceptance.

### Integrated source boundary, 2026-10-27 continuation

The coordinator released the old04 freeze for final integration, then explicitly
requested a concrete executable source boundary rather than completing all54
commands in this pass. This boundary is **source-ready, NOT_RUN**, not05
acceptance. No build, formatter, lint, test, native launch, real account action,
network request or global HOME/config mutation was performed by this worker.

Native `features/platforms` now consumes `runtime/platforms`: capability-driven
settings/account/provider controls, a scoped acknowledged plaintext source
editor, original loaded revisions, conflict draft retention and explicit reload
after committed writes. Settings/auth roots are captured on the worker and
retained in the editor session through mutation. Fixture launch uses the existing
paired absolute profile/registry paths and derives only
`<profiles-parent>/desktop-platform-fixture/{home,ccr,locks}`. Merely constructing
or reading an absent fixture does not initialize these roots. Domain errors are
projected to static typed messages; neither raw/parser errors nor credentials
enter logs/preferences. A Claude switch with diagnosis warnings/suppressors is
reported as committed-with-warning, never silently successful.

Shared JSON dirty-leaf composition retains unknown nested siblings. OpenCode
provider add/edit/delete composes the canonical `provider` object, falls back to
legacy `providers` on read, retains unedited credential/options/model siblings
and deletes only the named ID. This is an actual configuration consumer, **not**
a non-secret template catalog. Native provider plaintext remains scoped; no
template persistence is introduced. Codex uses the existing extracted typed
patch owner. Empty/native invalid patches do not write. Grok exposes source
editing with active/drifted managed model/custom-route comparison before the
original-revision guarded write; a full typed Grok settings form is not claimed.

Read subscriptions are dropped on leave; mutation subscriptions detach while the
bounded app-owned job finishes. Generation/environment/platform/request identity
rejects stale completions. Leaving clears snapshots, pending plaintext, inputs,
confirmation and editor feedback. Decisions capture their target name, use the
existing focus trap, delay initial focus until mounted, handle Escape and restore
the trigger. Actual OS focus behavior remains unverified.

The runtime request identity is now its own enum, preserving Claude, Codex,
Grok, Gemini, Qwen and Droid and adding OpenCode. This does not expand the
profile-domain enum. Profile conversion stays explicit and rejects OpenCode.
LSP references were attempted by the coordinator but its server exited code0;
fallback reference search covered root/Tauri/desktop sources and tests. Tauri
has no consumer of this runtime identity. Added regression rejects OpenCode
through both catalog and profile request paths before any Claude read.

07 integration is common-owner wiring only: shared sessions export, native
runtime/features exports, an Insights shell entry, and the grouped absolute
`--insights-usage-root`, `--insights-archive-db`, `--insights-provider-home`
launch inputs. Partial groups, relative roots and probe combinations reject
before I/O. Absent inputs remain explicitly unconfigured, never discover HOME.
The existing ccr-usage test-fixtures feature is enabled for desktop dev tests;
it adds no dependency/version/checksum. Other manifests/locks were not changed
in this continuation.

#### Eight-route disposition

| Existing route | Native action at this boundary | Remaining |
| --- | --- | --- |
| `/claude-code/settings` | Platforms → Claude Code → Settings: load, dirty JSON leaves, acknowledged source save with original revision | Typed field widgets, layer probes, remote transport |
| `/claude-code/auth` | Platforms → Claude Code → Accounts: masked list/current, save runtime, switch, confirmed delete snapshot | Official logout and full source diagnosis UI |
| `/grok/auth` | Platforms → Grok → Accounts: explicit-root runtime credential-file presence | Account mutations and logout injection |
| `/grok/settings` | Platforms → Grok → Settings: acknowledged source read/save, original revision, managed-route guard | Typed patch/schema controls, layers |
| `/codex/auth` | Platforms → Codex → Accounts: masked list/current, save runtime, switch, confirmed delete snapshot | OAuth native lifetime, imports/API keys, rename, process/port/browser, logout |
| `/codex/settings` | Platforms → Codex → Settings: extracted typed dirty patch and original-revision source save | Model catalog and typed widgets/layers |
| `/opencode/providers` | Platforms → OpenCode → Providers: IDs plus add/merge/delete under original document revision | Enabled visibility widgets, non-secret templates |
| `/opencode/settings` | Platforms → OpenCode → Settings / TUI: independent read/dirty patch/source sessions | Legacy keybinding alias and theme catalog; no two-file atomicity claim |

#### Fixed54-command action mapping

The rows below cover every command ID in the frozen54 inventory. `Implemented`
means source exists, not compiled/native PASS. Missing commands are explicitly
unexposed rather than a disabled success button or fake fallback.

| Command IDs | Native disposition |
| --- | --- |
| `claude_get_settings_raw_text`, `claude_save_settings_raw_text`, `codex_get_config_raw_text`, `codex_save_config_raw_text` | Implemented through corresponding Settings → acknowledged source; original snapshot token retained |
| `claude_list_settings_layers`, `codex_list_config_layers` | Not implemented; no ambient project/system probing in fixture mode |
| `claude_get_settings`, `claude_update_settings` | Implemented Settings load + dirty-leaf JSON patch; native widget projection/remote environment parity still open |
| `claude_list_auth_accounts`, `claude_get_auth_current`, `claude_save_auth`, `claude_switch_auth`, `claude_delete_auth` | Implemented Accounts list/current/save/switch/confirmed snapshot delete through existing service |
| `claude_auth_off` | Not implemented natively; existing official Tauri/CLI owner retained unchanged |
| `codex_list_models` | Not implemented |
| `codex_get_settings`, `codex_update_settings` | Implemented captured config load + extracted typed dirty patch under original revision |
| `codex_list_auth_accounts`, `codex_get_auth_current`, `codex_save_auth`, `codex_switch_auth`, `codex_delete_auth` | Implemented Accounts list/current/save/switch/confirmed snapshot delete through existing service |
| `codex_auth_off`, `codex_rename_auth`, `codex_detect_process` | Not implemented natively |
| `codex_oauth_login_start`, `codex_oauth_login_completed`, `codex_oauth_login_cancel`, `codex_oauth_submit_callback_url`, `codex_is_oauth_port_in_use`, `codex_release_oauth_port`, `codex_open_external_url` | Existing extracted controller/Tauri adapter retained; native captured backend/app-owned cleanup integration not implemented |
| `codex_import_auth_payload`, `codex_import_auth_from_local`, `codex_add_auth_with_api_key` | Not implemented natively |
| `codex_list_model_providers`, `codex_save_model_provider`, `codex_delete_model_provider` | Codex credential-store consumers not implemented; OpenCode provider CRUD is separate frontend-only workflow and does not satisfy these three IDs |
| `grok_auth_current` | Implemented explicit-root presence check, not token validity or login success |
| `grok_auth_off` | Not implemented natively |
| `grok_get_settings`, `grok_update_settings` | Source editor is usable; existing typed response/schema patch parity is not implemented |
| `grok_get_config_raw_text`, `grok_save_config_raw_text` | Implemented source read/save under original revision plus conservative managed-route guard |
| `grok_list_config_layers` | Not implemented |
| `opencode_get_settings`, `opencode_update_settings`, `opencode_get_tui_settings`, `opencode_update_tui_settings` | Implemented independent Settings/TUI dirty-leaf sessions; source and primary/fallback revisions retained |
| `opencode_get_keybindings`, `opencode_update_keybindings`, `opencode_list_themes` | Not implemented; TUI object editing is not claimed as alias/theme-catalog parity |
| `gemini_get_settings`, `gemini_update_settings` | Implemented explicit Advanced: Antigravity source action; no fabricated descriptor settings/auth surface |

Official-vs-local auth off remains deliberately unchanged: native does not call
an ambient helper from a fixture or substitute local removal for official
logout. OAuth callback reception is not account commit success. The existing
extracted controller's focused tests remain historical evidence only; native
cancel/commit/app-close cleanup and all remaining54 actions stay open.

#### Prepared regressions and coordinator checks

- Shared JSON policy fixture: nested dirty retention, canonical/legacy provider
  CRUD, retained secrets/options/models, original-revision conflict and invalid
  clearing without a write.
- Native adapter fixture: pure missing read, two clients sharing the original
  snapshot, one winner, retained unknown nested fields, descriptor truth.
- Runtime consumer regression: OpenCode rejects both Claude catalog/profile
  dispatch; grouped Insights paths require all absolute roots.
- Headless rendered controls: plaintext source requires acknowledgement,
  decision focus enters the real mounted trap, closing clears plaintext and
  rejects the previous response identity. These tests have **not been run**.

Coordinator should format only owned changed files, then serialize desktop
check/test/lint/build, affected ccr-app/CLI/Tauri settings/OAuth suites and typed
binding parity. Preserve `--locked`, default parallelism and
`--skip export_bindings`. Actual native source/CAS/account confirmation/close,
language/theme and07 reads require a new candidate; prior frozen artifacts,
historical native failures and user-deferred size/DPI matrices remain unchanged.

#### Coordinator gate correction

The coordinator's integration gate reported that the new shared sessions owner
imports `ccr_store` without a direct ccr-app manifest edge. Added only
`ccr-store = { path = "../ccr-store" }` and the ccr-app dependency entry in the
root, Tauri and desktop lockfiles. The native manifest and ccr-gpui lock record
already contained their direct ccr-store edge and were intentionally unchanged.
No package/version/checksum records changed and no gate/formatter was rerun by
this worker. Common manifests/locks/source are frozen again for the coordinator.

The subsequent native all-target check reported `.role` on an unkeyed `Div`
in the new confirmation. Matched the existing Profiles semantic construction:
stable `platform-confirmation` ID, `TestSupportExt::test_support`, then
`AlertDialog` role and accessible label. The focus trap and decision semantics
remain intact. This worker did not rerun the gate or formatter.

The coordinator's Tauri services test build then reported12 test compilation
errors (`outputs/desktop/continuation-20261027/tauri-services-tests.log`).
Restored the missing `tokio::sync::oneshot` test import. Moved the silent-socket
terminal/deadline and stale-cleanup-owner regressions from Tauri into the CLI
controller's existing private test module, adapting its explicit in-memory
fixture with cleanup-failure and commit-count controls. Both retain their
listener-release/secret-cleanup/no-commit and replacement-login-preservation
assertions. No private controller methods were exposed. Tauri retains its public
controller, runtime-admission, cancellation/commit and fake-HTTP adapter tests.
Changed the two notification equality assertions to non-formatting `assert!`
comparisons rather than adding Debug to a parsed-file payload type. No product
wire API/dependency changes, gate reruns or formatter runs were performed.

### Coordinator verification after handoff

Shared app36 tests passed (two existing ignored), strict all-target/all-feature Clippy passed; CLI OAuth owner6 passed, Tauri services52 passed and actual `oauth` selection19 passed. Native Rust36 passed; desktop strict Clippy, fmt-check and debug build passed. Full command logs and scope limits are in parent `research/resume-20261027.md` and `outputs/desktop/continuation-20261027/`.

Actual native Claude dirty-field patch committed the synthetic model and the isolated settings file matched. Screenshot/receipt: `outputs/desktop/profiles/native/repaired-20261027-231848-frame-fixed/`. This does not verify native OAuth/account/provider/logout completion; all inventory gaps above stay open.
