# Claude execution plan

1. [x] Wait for approved plan and shared foundation; read parent research, Claude auth runtime and fixture specs.
2. [x] Rename only Claude Profile identity and update EN/ZH expectations; preserve configured IDs and Auth names.
3. [x] Feed shared runtime strip from cached Claude semantics; implement grouped model/route/activity/usage content.
4. [x] Verify official/API-key/pending/conflicting/unknown states, profile env/model mappings, current != selected, secret sentinel and missing history.
5. [x] Render both themes/locales and boundary sizes; check header/body columns and all optional model fields remain reachable.
6. [x] Run `cargo test -p ccr-tui -- --skip export_bindings`, affected `cargo test -p ccr-cli -- --skip export_bindings`, `just fmt-check`, `just lint-strict`.
7. [x] Trellis check reviews actual adapter and rendering results; parent owns integrated gates and performance comparison.

No global Claude Code text replacement. No concurrent edits to the shared app/ui files. Keep read-model and platform presentation rollback separate.

Evidence: parent `research/integration-validation.md` and `research/check-final.md`. Workspace tests and strict lint passed; all standard CI recipes passed as recorded by the parent. Native terminal appearance is not established by TestBackend exports.
