# Codex execution plan

1. [x] Confirm parent shared foundation is ready and final plan approved; read parent research and this child's manifests.
2. [x] Implement runtime span/layout states and Codex field grouping; reuse table/activity components.
3. [x] Add scoped token splits/costs and visible scope note in compact and wide layouts.
4. [x] Exercise runtime bound/unbound/pending/mismatch, selected != current, invalid effort, missing provider, query failure, long CJK/URL and token sentinel.
5. [x] Verify Codex Auth shared banner consumer and cached runtime invalidation after Profile apply/off, mouse geometry, locale/theme change and post-apply warning semantics.
6. [x] Run `cargo test -p ccr-tui -- --skip export_bindings`, affected `cargo test -p ccr-codex -- --skip export_bindings` if domain code changes, then `just fmt-check` and `just lint-strict`.
7. [x] Capture actual TestBackend cells and colored cell-derived images; append parent before/after report and complete Trellis check. Native-terminal/font capture remains unverified. Parent owns final integrated CI (all recipes passed; aggregate plus resumed frontend suffix).

Do not write personal runtime files or use actual account credentials in fixtures. Revert only this child changes if a platform regression appears; retain others' work.

Evidence: parent `research/integration-validation.md` and `research/check-final.md`. Workspace tests and strict lint passed; all standard CI recipes passed as recorded by the parent. Native terminal appearance is not established by TestBackend exports.
