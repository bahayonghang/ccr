# 04 independent native review and self-fix

Date: 2026-10-18. Active task: `.trellis/tasks/10-17-gpui-04-profiles`.
Role: dispatched trellis-check; no recursive sub-agent dispatch. Shared work and
all prior implementation retained. Main owns task status, inventory, specs,
final debug executable and actual Computer Use. Domain/Tauri/manifests/locks
belong to the separate checker. No GUI launch, user configuration/account access,
HOME/USERPROFILE mutation, commit, push, packaging, signing or archival occurred.

## Findings fixed

1. **Confirmation could submit an underlying editor.** Actual rendered Advanced
   confirmation followed by clicking underlying Save started a write. The first
   regression failed101 at the `busy` assertion, recorded in
   `outputs/desktop/profiles/native-check-confirm-before-permitted.log`. Save now
   refuses a pending confirmation, and editor Cancel/Save/Advanced/reload/force
   and recovery actions are disabled while a decision is pending. Disabled
   buttons retain their same native node and flags, and the existing semantic
   owner additionally sets `tab_stop(false)`.
2. **Confirmation's hand-written Tab handler did not trap Root actions.** The
   actual Tab regression reached a disabled Profile name input. Installed Kit
   source shows Tab/Shift-Tab are Base Root actions; an ordinary on_key_down
   handler is not their owner. Replaced that branch with existing Kit
   `FocusTrapElement` on the two actual decision buttons, with a persistent
   confirmation focus handle/tab group. No vendor patch, copied traversal
   engine, duplicate controls or additional dependency. Actual forward and
   reverse traversal stays on Cancel/Confirm, Escape returns keyboard focus,
   and the unchanged draft remains. The existing rendered full edit/Inspect
   case still verifies Cancel, deletion confirmation and readonly input
   suppression.
3. **Restore's final decision omitted its chosen filename and lost focus.**
   RestoreExecute now captures the entered managed leaf through the same
   confirmation entry path, names it and the replacement/preimage consequence
   in the actual rendered alert, and returns to keyboard Save on Escape. The
   observation is a transparent `.test_support()` decoration on that existing
   alert node. Closing also resets the source activation-conflict flag.
4. **Committed Grok retry kept offering another activation.** A staged
   rename-apply-failure sets a recovery action. Accepting a subsequent committed
   Applied/AppliedWithWarning outcome did not remove it. An actual rendered
   controller-transition regression reproduced101 in
   `native-check-retry-before.log`. Committed success now clears that obsolete
   recovery action; its durable warning survives an automatic Status refresh.
   This test supplies typed owner outcomes to the real view. It is not an
   assertion that native pixels or a real Grok failure were exercised; actual
   Grok domain failure/concurrency tests belong to the other checker.
5. **Merged generic Validate used dedicated platform validation.** Configurations
   now emits the native-only typed GenericValidate request, which consumes the
   existing `ConfigService::validate_all` counts/per-name bool results. Dedicated
   Profiles keeps `PlatformConfig::validate_profile`. Diagnostics do not project
   arbitrary validator error text. Tauri's existing wire response is unchanged.
   The adapter regression compares the result to the actual service report.
6. **Fixture cleanup missed the catalog owner's same-directory backups.**
   ConfigSource::Fixture declares a catalog parent outside its auxiliary
   `desktop-profile-fixture/` root. The old CleanBackups branch scanned only the
   auxiliary root. A private worker-selected `cleanup_root` now binds Local to
   the unchanged `context.paths().root`, and Fixture to its already-declared
   profiles parent. It is passed to execute, not added as a UI/command path
   picker. Domain context root, locks, runtime, history, analytics and off backup
   paths all remain unchanged; the second client's lock identity stays
   `profilesParent/desktop-profile-fixture/locks`.

The new cleanup regression calls the actual Services::profile_command twice:
real Apply first, then CleanBackups. It creates a backup via ConfigService,
lists/resolves that managed leaf, independently inventories the two actual
domain-created backup leaves within the existing depth3 policy, verifies the
reported deletion count and absence, and retains catalog revision, runtime
bytes and an unrelated nonbackup file. All paths are owned temporary fixtures.
No SQL or config-store implementation was copied. Initial test assumptions
expected one backup and then inspected only the root directory; the observed
domain apply preimage adds a nested backup. Those assertion failures101 are
retained in `native-check-profiles-last.log`, `native-check-clean-diagnostic.log`
and `native-check-profiles-verified.log`. The corrected filesystem inventory
and final7 profile regressions pass; no product policy/depth was relaxed.

## Behavior and 42-command / 5-route review

Read task PRD/design/implement, all16 curated check references, task evidence,
the three worker evidence reports, 02 source map and runtime contracts, 03
usable-core review, and approved style:codex DESIGN. Applied before-dev/check,
ui/design-md to the bounded existing-surface repairs. No visual redesign.

The native mapping table in `ui-evidence.md` supplies all42 concrete actions.
Independently traced those groups through rendered control -> typed request ->
worker -> existing domain/shared owner. None of these source mappings is treated
as42 separately executed native scenarios:

| Count | Command group | Reviewed native behavior |
| --- | --- | --- |
| 13 | list_configs, switch_config, add_config, delete_config, rename_config, duplicate_config, update_config, validate_configs, import_config, restore_config, export_config, get_history, clear_history | Explicit Claude Configurations policy; strict ten-field parse, distinct generic validation, original-token update/rename, active disable/delete guards, latest domain clone, Merge/Replace backup, managed-leaf restore, redacted export, typed-only history and confirmations. |
| 10 | claude_list_profiles, claude_get_profile, claude_add_profile, claude_update_profile, claude_delete_profile, claude_apply_profile, claude_profile_off, claude_export_profiles, claude_get_profiles_raw, claude_save_profiles_raw | Dedicated Claude owner/patches; selected Inspect and canonical secret source/export follow plaintext warning; actual current/can_off, journal apply/off, original draft/source token and force-only activation acknowledgement. |
| 10 | codex_list_profiles, codex_add_profile, codex_update_profile, codex_delete_profile, codex_get_profile_env, codex_apply_profile, codex_profile_off, codex_export_profiles, codex_get_profiles_raw, codex_save_profiles_raw | Own catalog/current/runtime; dedicated normalized field patch, injected lifecycle/secret store owner, scoped env/script preview, canonical source/export. No generated command execution. |
| 7 | grok_list_profiles, grok_get_profile, grok_add_profile, grok_update_profile, grok_delete_profile, grok_apply_profile, grok_profile_off | Own current/runtime and safe URL; immutable existing kind, explicit credential actions, staged rename outcomes, single Active/Drifted force confirmation, no UnsafeMissingEntryState force. No invented whole-catalog source/export. |
| 2 | convert_config, clean_backups | Existing pure Claude JSON <-> Codex TOML preview in bounded read; no invented unsupported formats. Cleanup uses explicit source-bound root with unchanged shared depth/extensions/success count. |

Reviewed route merge: `/configs` and `/claude-code/profiles` share the same Claude
catalog but expose explicit Configurations/Profiles modes. Generic strict patch,
validation, protected delete and redacted export remain distinct from dedicated
normalization/lifecycle and secret canonical output. `/codex/profiles` and
`/grok/profiles` use actual platform selectors/owners; `/converter` uses the
connected converter navigation. Five concrete surfaces exist; full per-route
native acceptance and inventory status remain main-owned.

Reviewed current native code has no synchronous file/DB/network/process work in
render. Request work stays inside the02 bounded RuntimeHandle; stale gates are
cleared on leave, reads cancel and job view drops detach an app-owned runner.
Original draft revision is retained through conflicts, untouched/masked fields
are not resubmitted, and unknown/datetime retention uses the actual repository.
Generic rename uses the lifecycle's lock internally and checks active-disable
inside its patch; it does not nest an extra outer application lock. The added
active rename+disabled patch regression rejects with no renamed section or
revision change. Invalid field/source and stale token regressions remain no-write.

NoDebug scoped payloads are cleared on Cancel/navigation; a late completion
cannot reopen them. Safe endpoint rendering uses the existing Grok URL display
helper. History projects typed success/failed/warning without legacy diagnostic
payload strings. Full selected Inspect includes exact unknown/datetime content
and its original catalog version, but never seeds the replacement credential
input. The rendered Inspect input-suppression test remains passing.

## Verification

Environment: Windows MSVC, repository Rust/Cargo1.98.0. Commands run from the
independent desktop directory through its normal verified helper. MSRV uses
actual Cargo1.95.0/rustc1.95.0 and child-only RUSTUP_TOOLCHAIN/PATH pairing;
actual executable paths and verbose versions are captured in
`outputs/desktop/profiles/native-check-toolchains.json`.

| Final tree command | Exit | Evidence |
| --- | --- | --- |
| just desktop-fmt-check | 0 | native-check-fmt-accepted.log |
| just desktop-check | 0 | native-check-check-accepted.log; locked all targets |
| just desktop-lint | 0 | native-check-lint-accepted.log; locked all targets, -D warnings |
| python scripts/desktop.py cargo test --locked -- --skip export_bindings | 0 | native-check-rust-tests-accepted.log; lib15, native_shell3, native_ui2, default parallelism |
| Rust1.95: python scripts/desktop.py cargo check --locked --all-targets | 0 | native-check-msrv-accepted.log; exact paired compiler receipt |
| git diff --check -- ccr-desktop | 0 | tool output; no Rust changes after final gates, only README/DESIGN factual corrections |

`native-check-accepted-gates.json` preserves commands/times/exit codes for the
final Rust tree. Explicit desktop-fmt repairs were separate commands, not hidden
inside checks. Before the final generic-validation/cleanup-only Rust changes,
the independent full `just desktop-test` also passed Python12, AccessKit3, lib14,
native_shell3 and native_ui2 (`native-check-test-final.log`). Python helper and
vendored AccessKit files were untouched by those last Rust changes, so that
actual support-test evidence is consumed rather than needlessly rebuilt.

The first sandboxed regression attempt could not inspect mbx shim metadata
(exit1), not a product test failure. Required cache access was then granted and
the real compile/test outcomes recorded. No auto-review rejection or global
MBX/cache configuration change occurred. A quoting parser failure and Python
default-GBK read failure changed no files; subsequent edits used UTF-8.

Source freeze receipt: `outputs/desktop/profiles/native-check-freeze.json`,
15 native source/test/document hashes. Desktop lock remained
`14900fb870ff12effe9129a663aec7fb92d11dd3abef79fa1dce41ecf3c69a56`.
No lock/manifest/justfile/generated-binding changes were made by this reviewer.
Current native product ownership is frozen for main's final build and Computer
Use; no Cargo process remains from this reviewer.

## Findings not fixed / remaining acceptance

- No remaining concrete blocker found in the native source scope after fixes
  and passing final gates. Main must build current source, hash that executable
  and perform its actual fixture core flows. Existing worker screenshots or an
  old executable do not verify these changes.
- This reviewer did not run OS native pixels, IME, UIA/screen reader, window
  size/zoom/DPI, macOS/Linux native, release performance or packaging. Size/DPI
  is user-deferred. No performance or complete04/parent pass is asserted.
- Domain/Tauri failure atomicity, first-create, Grok concurrency/rollback,
  shared/TS/bindings checks belong to the other checker; root aggregate gates
  and final `just ci` belong to main. In particular the separately observed
  intermittent bindings drift is not repaired or declared passed here.
- README/DESIGN now distinguish03 ordinary input from04 readonly scoped output
  evidence. Main should keep the code_map description current (it still calls
  desktop read-only) and may sync the native confirmation/generic-validation
  contract into its owned specs. This reviewer did not edit those main-owned
  files, task status or migration inventory.

AC4.1/4.2/4.4 have concrete native adapter/rendered evidence above; AC4.3 has
native invalid/stale/no-write evidence plus separate domain failure injection;
AC4.5 has42 source action mappings and five reviewed surface mappings. Actual
Computer Use, combined owner evidence and final inventory/AC judgement remain
main-owned. This report does not mark all42 commands/five routes verified or
the task complete.
