# GPUI06 resource implementation, 2026-10-28

Active task: `.trellis/research/archive/2026-11/10-17-gpui-06-resources`, dispatched Trellis implementer. Explicit user resumption applies to the existing approved tree. No new task, commit, push, publication, signing or archive. Shared workspace changes from04/05/07 and main are preserved. The native hook selected06 but was truncated; the full saved hook and explicit plans/manifests/specs were read before edits.

## Exact source inventory and smallest change

`command-map-20261028.json` maps all80 owned command definitions to current source files/lines and all38 owned route records, including14 redirects. It was derived by reading the existing captured inventory, not rerunning capture_inventory.py or overwriting its original hashes. Initial command statuses remain pending until real consumers are integrated and checked.

| Existing group | Actual owner/capability | Migration boundary |
| --- | --- | --- |
| User instruction files,4 commands | `commands/system_prompts.rs` fixed registry, existing `ccr-app::platform_settings::{read_text,write_versioned}` | First concrete shared/native slice; original tokens, absence, managed backups and size warnings |
| Claude MCP,4 commands plus unified use | `commands/claude_mcp_config.rs`,1510 lines including tests | Framework-free local/project/user precedence and approval, masked secrets, null/dirty merge and guarded CAS retries; extract existing owner rather than duplicate policy |
| Codex MCP | `commands/codex_mcp.rs` plus extracted Codex typed config owner | Preserve complete transport/auth fields, unknown TOML and existing validation/merge |
| Antigravity MCP | `commands/gemini.rs` and unified adapter | Keep `.gemini/antigravity-cli/mcp_config.json`; do not invent Gemini CLI capability |
| Agents/commands/plugins/hooks/output/statusline | Current platform-specific command modules and descriptor flags | Capture existing roots; preserve format/path/scope/source rules rather than one universal writer |
| Builtin prompts/presets | `ccr-skills` catalogue/managers | Pure catalogue reads can reuse existing owner; install/sync needs explicit-root owner injection before any fixture action |
| Skills routes | `SkillsMigrationView.tsx` | Current UI explicitly removed embedded install/market/source management and hands off to standalone Skillport. Detect/open IDs belong to09 shell in the frozen inventory. Preserve this handoff, do not resurrect unused SkillsService APIs as new product scope |

The source tree has no native resource consumer. The first gap is fixed user instruction-file list/get/create/save; the change belongs in shared resource policy plus a bounded native request and scoped editor. Main owns all common exports/manifests/three locks/shell navigation. Until main copies the04/05/07 candidate, new06 modules stay unexported. Main approved eventual necessary delegation of Tauri `commands/system_prompts.rs`; that file is still unchanged during the freeze.

## Prepared first slice, not delivered graph

- `crates/ccr-app/src/resources/system_prompts.rs` extracts the fixed four-platform registry, path/metadata/rules/size policy and original save-result keys. Actual reads and guarded token writes reuse the existing shared owner; explicit fixture locks are supported without HOME changes. No arbitrary UI path is accepted.
- `crates/ccr-app/tests/resources_policy.rs` path-loads this exact unexported policy to allow meaningful tests before a common export is released. A standalone policy pass is not native integration.
- `ccr-desktop/src/runtime/resources.rs` prepares list/open/create/save with captured roots, fixed IDs, environment/platform checks before default-root access, original revision and one app-owned writer through completion. Plaintext session/snapshot has no Debug/Serialize.
- `ccr-desktop/src/features/resources/mod.rs` prepares the existing solid reading/editor vocabulary, acknowledged plaintext entry, render-owned one-shot confirmation focus, explicit Cancel/Reload, stale-draft retention and leave teardown. No renderer/theme/CSS/dependency change. Native module/tests are not yet compiled or accepted.

UI skill direction: existing airy-glass field/single opaque sheet, system14px body,36px data rows and green controls. Utility sequence is orient → show real missing/existing state → acknowledged temporary editor → original-token save/reread. No new decoration or keyboard animation. There is no invented content to replace; fixtures are synthetic and explicit.

## Actual validation

Command: `cargo test -p ccr-app --test resources_policy --locked -- --skip export_bindings`, existing paired Rust/Cargo1.98, child-only alternate target `outputs/desktop/profiles/resume-current-check/target`. Exit0, four tests passed. Raw log: `outputs/desktop/resources/checks/prompt-policy-20261028.log`.

Assertions cover all fixed platform IDs and paths, missing reads without directory creation, sorted Claude rules, original absence/token create/save/reread, two original clients with one winner, no backup on conflict, failed preimage backup preserving old bytes, invalid UTF8 safe errors, size warning without source disclosure and unchanged result keys. No default HOME, real config/account or network call.

Repair formatting was separate: actual rustfmt1.98 on owned files, then scoped diff check exit0. An independent desktop fmt run briefly saw the in-flight path-loaded shared file before formatting; its raw failure remains under main continuation outputs. These validation results do not claim the prepared native graph compiled.

## Hold and next entry

Main requested an all-source/lock freeze for its combined candidate.06 confirmed product sources frozen; only research/doc writes continue until release. After release: finish native decision/CAS/teardown fixtures, integrate shared exports/runtime/features/nav through main, replace Tauri duplicated instruction policy with delegation, run root/Tauri/native narrow checks and independent review, then actual fixture native validation by main. Continue the remaining76 command consumers and every route mapping; do not mark06 completed from the first four-command slice.
