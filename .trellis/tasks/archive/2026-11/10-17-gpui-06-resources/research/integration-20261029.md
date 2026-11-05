# GPUI06 instruction-file integration, 2026-10-29

Active task: `.trellis/tasks/10-17-gpui-06-resources`. Existing approved task tree resumed by the user; no new task or completion claim. The role hook selected08 and was truncated, so explicit06 artifacts/manifests and applicable specs were loaded. Main released the former source hold after confirming immutable combined candidate1157f008 exists. Other agents share the workspace; their changes are preserved.

## Implemented boundary

- Main integrates the shared/resource/native exports and Shell navigation. Those common files remain main-owned.
- `ccr-app::resources::system_prompts` owns the original fixed four-file registry, platform aliases, path/metadata/rule projection and save feedback augmentation. Native guarded saves consume the existing shared settings writer with captured fixture/application locks. Removed preparation-only unused JSON read and unscoped-save wrappers; native uses the actual non-Debug Snapshot rather than round-tripping plaintext through generic JSON.
- Tauri `commands/system_prompts.rs` now imports that exact registry/metadata/augmentation policy. Its Local-only check, runtime path/backup resolution, wire signatures and `settings_raw::{read_raw_file,write_raw_file_versioned}` I/O/error mapping stay in the adapter. The original save/create feedback keys and size/limit behavior are shared, without replacing Tauri errors with native static categories.
- Existing native adapter/view fixtures exercise original-token CAS, duplicate-create rejection, wrong-platform/remote refusal, missing reads without directory creation, scoped plaintext cleanup and real worker ownership. Prepared rendered tests exercise outside-focus decision entry with callback-before-draw, Cancel/Tab/Escape and stale-draft retention.
- Fixed first-entry focus: the resource main focus handle was mounted only when status text existed. The root now always owns it; `focus_main` enters the first enabled control. The leave test asserts this transition after plaintext/generation cleanup.
- Product source frozen for main gates at09:40:02UTC; only evidence changes after that point unless a concrete gate failure requires a documented repair.

## Verification status

- Owned-file rustfmt1.98 repairs executed separately; exit0. Scoped tracked-file `git diff --check` exit0. Original initial Python rewrite attempt failed on the Windows GBK default before any write; retry used explicit UTF-8 mode. It is a tooling failure, not a product result.
- Main `resume29-resources-root` exit0 ran four shared resource policy tests before the final removal of unused preparation-only wrappers. Final-source rerun required. The name filter selected zero integration tests in `resources_policy`; main was notified to run that target explicitly or the full app suite.
- Main desktop check reached the exported native module and exposed an unused ActiveTheme import; owner removed it. Its next all-target check stopped in the independent jobs module. Native resources compile/tests/lint and Tauri parity remain pending final-source gates.
- No native OS actions, real HOME/configuration/account, install/network or external request were performed by this worker. Main owns candidate identity and native operation evidence. Headless tests cannot establish IME/UIA, light/dark native pixels, DPI or performance.

## Remaining

Only the first four user-instruction commands are integrated in source. The other76 owned resource commands, all route accounting, install/uninstall/cancellation and full06 acceptance remain open. Do not mark06 completed based on this slice. Final gate receipts and main candidate/native evidence must be appended before changing the four-command acceptance fields.

## Final-source main-run contract checkpoint

Main serialized the commands and retained raw receipts under `outputs/desktop/continuation-20261028/`:

- `resume29-app-after`: `cargo test -p ccr-app --locked -- --skip export_bindings`, exit0,09:41:40–09:41:54UTC. Shared resource policy4 and exported integration1 passed. Full suite55 library passed/1 ignored,5 config-read passed/1 ignored,9 profile-service passed/1 ignored,1 resource integration passed; ignore entries are explicit native-fixture seeding helpers.
- `resume29-native-after`: desktop `cargo test --locked --lib -- --skip export_bindings`, exit0,09:41:40–09:42:08UTC.44 library tests passed, including resource runtime2 and rendered consumer2. These verify real fixture CAS, outside-focus confirmation ordering, draft retention, plaintext leave/generation cleanup and initial main-focus entry. They remain headless evidence.

These gates postdate the final06 product edits. Tauri parity, strict checks/lint, independent review and main native candidate actions remain pending. The four-command map records connected source and these specific tests; native_acceptance remains NOT_RUN, and original capture/source-line records are preserved.
