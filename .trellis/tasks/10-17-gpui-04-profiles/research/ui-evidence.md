# Native profiles UI implementation evidence

Date: 2026-10-18. Active task: `.trellis/tasks/10-17-gpui-04-profiles`.
This report records implementation and headless checks; main session owns native Computer Use and final task/parent acceptance. No commit/push/package/signing/account or real-user configuration writes were performed by this worker.

## Boundary and design

Applied trellis-before-dev, ui, design-md and the approved `ccr-desktop/DESIGN.md` (style:codex). Existing system fonts, semantic Kit tokens, stable 36px virtual rows and same-node button semantics remain. The visual thesis is a low-chroma continuous list/detail work surface; ordinary editors expand in content, and long fields scroll inside that area. High-frequency actions have no decorative animation.

Native ownership: `src/features/configs/mod.rs` (moved existing list view; `app.rs` reexports the same ConfigurationApp), `features/profiles/{mod,view}.rs` (draft/controller vs rendering), `features/converter.rs`, `runtime/profiles.rs` + module wiring, shell navigation, affected native tests and README/DESIGN factual notes. No manifests, locks, domain, ccr-app or Tauri files were edited by this worker.

All filesystem, DB, serialization and lifecycle work is inside02 RuntimeHandle workers. Read subscriptions cancel on leave; mutations remain application-owned and publish only the actual owner result. Input render/state operations do no I/O. Stale responses compare the complete request key; page leave drops subscription, invalidates key/generation and clears pending/source/export/credential buffers. A background can_off refresh preserves committed warning/recovery text.

## Reviewed native surface merge

One Claude catalog list hosts both legacy `/configs` and `/claude-code/profiles` through explicit **Configurations / Profiles** mode buttons. These share stored rows without collapsing policies: Configurations uses the strict ten-field parser and generic active-disable/default-delete policy; Profiles uses dedicated shared patch composition and actual platform lifecycle/normalization. Dedicated canonical secret export remains separate from generic redacted export. Codex/Grok selectors use their own catalog/current/runtime owner, never Claude fallback.

Main session explicitly reviewed the get_profile equivalence: **Inspect selected profile** first presents a plaintext warning, then reads exactly the selected ConfigSection and original catalog revision via ConfigManager::snapshot, serializes readonly selected TOML including unknown/datetime fields, and keeps it only in scoped noDebug output. Native format differs from legacy IPC JSON while preserving complete selected content/revision. It never seeds the editable patch or resubmits masks/plaintext automatically. Grok receives selected Inspect without inventing a whole-catalog source capability.

Structured edits bind the original catalog revision. Common nonsecret fields are prefilled; credential/base URL/auth replacements start blank and preserve existing values. Advanced explicit JSON patch supports existing fields via shared validate_fields; absent preserves/null removes. Grok existing kind is read-only, credential actions remain mutually exclusive, staged rename failures refresh actual state and offer appropriate recovery without asserting both names necessarily remain. Force delete is offered only once for actual Active/Drifted, never unsafe missing entry state. Source force acknowledges activation removal only; stale CAS still conflicts.

## Command mapping (42 entries; implementation is not per-entry native verification)

Status for the following table is `implemented_pending_independent_native_check` unless an explicit command-level result is stated later. Existing02 `list_configs` core verification is retained; do not replace its evidence with this table. Route completion requires actual independent/native evidence and reviewed merge, not the presence of controls.

| Command | Native implementation |
| --- | --- |
| `list_configs` | Configurations::refresh → Services::profile_rows → snapshot/current owner (existing02 read evidence retained) |
| `switch_config` | Configurations mode Apply / enable → ProfileCommand::Apply → context.apply/enable |
| `add_config` | Configurations mode New → GenericCreate → mutations::parse_patch/add |
| `delete_config` | Configurations mode Delete Confirm → GenericDelete → mutations::delete under operation lock |
| `rename_config` | Configurations mode Edit target name → GenericSave → original-token lifecycle update |
| `duplicate_config` | Configurations mode Duplicate → Duplicate → mutations::duplicate |
| `update_config` | Configurations mode Save → GenericSave → strict parse_patch + active-disable guard + original CAS |
| `validate_configs` | Configurations mode Validate → actual domain per-profile validation counts/results |
| `import_config` | Configurations mode Import warning → merge Save or separately confirmed Replace → ConfigService::import_config(backup=true) |
| `restore_config` | Configurations mode Restore → managed leaf + exact target confirmation → backups::restore |
| `export_config` | Configurations mode Redacted export → ConfigService::export_config(false) |
| `get_history` | Configurations mode Operation history → injected HistoryService; typed status only, no legacy diagnostic payload |
| `clear_history` | Configurations mode Clear history Confirm → injected HistoryService::clear |
| `claude_list_profiles` | Claude Profiles mode → profile_rows masked snapshot/current projection |
| `claude_get_profile` | Claude selected Inspect warning → exact selected ConfigSection readonly TOML + revision; noDebug scoped Text |
| `claude_add_profile` | Claude Profiles New → shared claude::build → context.create |
| `claude_update_profile` | Claude Profiles Edit / rename → shared claude::patch + native validate_fields → context.update(original revision) |
| `claude_delete_profile` | Claude Profiles Delete Confirm → actual PlatformConfig::delete_profile under operation lock |
| `claude_apply_profile` | Claude Profiles Apply → context.apply/enable, typed committed/warning/recovery result |
| `claude_profile_off` | Claude Profiles Off confirmation → context.off, can_off solely context.needs_login_prep |
| `claude_export_profiles` | Claude Profiles Canonical export warning → source::export(true); intentionally separate generic redacted export |
| `claude_get_profiles_raw` | Claude Profiles Source warning → source::read canonical catalog/token/path/existence |
| `claude_save_profiles_raw` | Source Save → source::save(original token); activation force requires a second explicit confirmation and never bypasses CAS |
| `codex_list_profiles` | Codex platform → profile_rows snapshot with Codex actual current owner; no Claude resolver reuse |
| `codex_add_profile` | Codex New → shared codex::build → context.create with actual Codex normalization/secret persistence owner |
| `codex_update_profile` | Codex Edit / rename → shared codex::patch → context.update(original revision) |
| `codex_delete_profile` | Codex Delete Confirm → actual injected Codex PlatformConfig::delete_profile |
| `codex_get_profile_env` | Codex Export environment / shell script warnings → context.export_profile_env(name,shell), preview only |
| `codex_apply_profile` | Codex Apply → injected context.apply/enable actual lifecycle outcome |
| `codex_profile_off` | Codex Off Confirm → injected context.off, no extra auth logout |
| `codex_export_profiles` | Codex Canonical export warning → source::export(true) |
| `codex_get_profiles_raw` | Codex Source warning → source::read canonical catalog/token/path/existence |
| `codex_save_profiles_raw` | Codex Source Save → original-token source::save + explicit activation force protocol |
| `grok_list_profiles` | Grok platform → safe snapshot rows and actual Grok current/needs_login_prep owner |
| `grok_get_profile` | Grok selected Inspect warning → exact selected ConfigSection readonly TOML + revision; no whole-catalog Source invented |
| `grok_add_profile` | Grok New → shared grok::build credential actions → context.create; explicit profile_kind supported |
| `grok_update_profile` | Grok Edit / rename → typed GrokPlatform::update_profile_versioned under application→Grok→catalog order; preserve staged partial outcomes |
| `grok_delete_profile` | Grok Delete Confirm → typed delete_profile_guarded; one separately confirmed force for Active/Drifted; UnsafeMissingEntryState never force |
| `grok_apply_profile` | Grok Apply / partial-rename retry → context.apply actual domain lifecycle |
| `grok_profile_off` | Grok Off Confirm → injected context.off existing bounded route cleanup |
| `convert_config` | Converter warning → ConfigConverter::convert in bounded read; actual Claude JSON ↔ Codex TOML preview, no file apply/execution |
| `clean_backups` | Configurations mode Clean backups Confirm → backups::clean(explicit context root) |

## Five route mappings

| Legacy route | Native surface | Status |
| --- | --- | --- |
| `/configs` | Claude → Configurations mode | Implemented; final independent/native acceptance pending |
| `/claude-code/profiles` | Claude → Profiles mode (reviewed shared-list merge above) | Implemented; final independent/native acceptance pending |
| `/codex/profiles` | Codex selector → dedicated profile workbench | Implemented; platform-specific native flows not yet run |
| `/grok/profiles` | Grok selector → dedicated profile workbench + selected Inspect | Implemented; platform-specific native flows not yet run |
| `/converter` | Converter navigation, supported formats only | Implemented; actual headless navigation clear tested, native pixels not yet run |

## Verification log (in progress until final freeze)

Windows MSVC, project Rust/Cargo1.98.0, independent desktop target/lock, normal helper identifies and bypasses only the verified mbx shim. No native GUI was launched by this worker. Rust suites use --locked/default parallelism/--skip export_bindings.

- First `just desktop-check`: sandbox cache identity metadata inaccessible (exit1), repeated with cache access; real compile found local Kit trait/arity errors (101), repaired and rechecked0.
- Focused `python scripts/desktop.py cargo test --locked --lib profiles -- --skip export_bindings`: initial test build lacked TestAppContextExt (101); repaired. Actual rendered case caught premature focus navigation (101), fixed persistent focus scope/next-frame Cancel entry and verified keyboard-triggered Escape return. Final focused5/5 exit0 before full Inspect extension.
- `just desktop-test` initial complete run: Python12/12 and AccessKit3/3 pass, application lib13/13 pass; native_shell2/3 pass, new converter test failed because Textarea accessibility_id is not an ElementId (101). Fixed the test to use real observed native label/actual ElementId, without wrapper/fake state. Focused native_shell navigation clear then exit0.
- `just desktop-check` logged `outputs/desktop/profiles-ui/check.log`: exit0.
- `just desktop-lint` initial log `lint.log`: exit101 for clone_on_copy/collapsible_if/test-module ordering; repaired. `lint-final.log`: exit0 before the explicitly approved final Inspect addition; final recheck still required.
- Explicit `just desktop-fmt` repairs ran separately and returned0. No validation command implicitly ran fmt or version-sync.
- Final post-Inspect full test/lint/fmt/build status will be appended below. No unrun item is passed.

## Scope limits

No scale/resize/DPI matrix was run, per the user's explicit core-only directive. OS native rendering, Computer Use edit/save/CAS/cancel/apply/normal exit and binary hash belong to main-session evidence. No IME/UIA, macOS/Linux native or release performance result is claimed; no Tauri-vs-GPUI samples were collected. Final root/domain/Tauri/just ci gates are owned by main/check and are not implied by desktop compilation.

## Final native worker freeze

- `just desktop-test` post-Inspect exit0, raw `outputs/desktop/profiles-ui/test-final.log`: Python12/12, compatible AccessKit3/3, application lib13/13, native_shell3/3, native_ui2/2; default Rust parallelism and --skip export_bindings retained. The expanded rendered profile case verifies masked replacement, real save/re-read, external conflict draft retention, durable warning retained through status read, keyboard Confirm/Cancel/Escape, actual deletion, selected Inspect plaintext warning + readonly input suppression + leave clear, canonical Source leave clear. Native runtime tests include full selected unknown/datetime/revision fidelity, same-token two-client CAS, generic active-disable/unknown field rejection and secret-safe legacy history projection.
- `just desktop-lint` post-Inspect exit0, raw `lint-accepted.log`. The only subsequent production change adds the already-projected safe endpoint label in the detail view; no backend behavior changed.
- `just desktop-fmt-check` exit0 (`fmt-check.log`), explicitly separate from previous fmt repair.
- Final `just desktop-check` exit0 after all native file changes (`check-accepted.log`).
- Final file hashes and current independent Cargo.lock hash: `outputs/desktop/profiles-ui/native-freeze.json`. Native files are frozen for main/check ownership. Main explicitly owns the final post-domain/check debug build, executable hash and Computer Use; this worker did not launch an old executable or label an intermediate build as native acceptance.

Full selected inspection closes the previously recorded native get_profile read gap. All42 command mappings above have concrete native code; this is still not an assertion that all42/5 have independent/native acceptance. Main must inspect the per-command results and unresolved domain/check/native evidence before task completion. Native full OS accessibility/IME/DPI/performance and platform acceptance remain unrun here.
