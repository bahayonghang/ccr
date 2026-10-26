# Foundation check preflight

Date: 2026-10-17. Role: trellis-check. Implementation is still in progress; this is a read-only preparation record, not acceptance.

## Loaded context

- Native hook saved output; task PRD/design/implement and implement/check context lists.
- Parent technical design, architecture/performance research, desktop DESIGN.md.
- Dependency-governance and local-build-cache specs; guide/backend indexes.
- trellis-start, trellis-before-dev, trellis-check skills.
- `C:\Users\lyh\.codex\RTK.md` is absent; no instructions were inferred from it.
- Memory registry search returned no task-relevant hits.

## Current source observations

- Package/bin are `ccr-gpui`; the manifest has its own `[workspace]`, local `target` configuration and exact Kit requirement.
- Root recipes delegate to the child justfile. The ordered desktop-ci graph uses fmt-check/check/lint/test/build and no explicit repair step.
- Tests preserve default Rust parallelism and `--skip export_bindings`.
- Cleanup verifies metadata workspace and target, resolves symlinks/junctions, and passes the verified absolute target explicitly to cargo clean. Final review must test hostile metadata and inherited target overrides.
- Probe uses Kit InputState/Input, subscribes to change events, and retains its subscription. There are no configuration reads in its render path.
- PowerShell/Bash version registries include the new manifest. Both test fixtures were being updated during this review; final checks must use the settled files.

## Pending verification and evidence

1. Locked resolution/build and strict Clippy/check, including native_ui integration API compatibility.
2. Dependency report: implementation-day registry candidates, chosen requirements, resolved features/MSRV/source, exceptions and lock checksum.
3. Root default CLI graph contains neither GPUI nor Tauri; new desktop graph contains no Tauri.
4. Actual command failure propagation, child cwd and cleaning boundary tests. Current dry-run tests alone do not establish runtime failure propagation.
5. Both version-sync suites and root version-check against current files.
6. README must describe the delivered implementation state; its earlier read still said the Cargo project and commands did not exist.
7. Release baseline precedes GPUI timing and includes the complete Tauri process tree, environment/data hashes and raw samples.
8. Windows native visible window, CJK composition, focus and close evidence; macOS/Linux build receipts. Headless tests cannot establish these.

No lint, type-check, Rust test, native interaction, performance or OS acceptance result is claimed by this preflight. Product-file ownership remains with the implementation agent until the main session hands it over.

## Additional source review requested by main session

- **Target isolation gap:** `desktop-dev/build/build-debug/check/lint/test/test-ui` invoke Cargo without the metadata guard. Cargo environment overrides `CARGO_TARGET_DIR` and `CARGO_BUILD_TARGET_DIR` take precedence over the checked-in target setting. Cleanup itself rejects foreign metadata and supplies its validated target explicitly, but ordinary public commands can still write into the root/Tauri target. Recommendation: run the shared target validator before each build/test entry; test rejection with a controlled Cargo command stub. Do not edit the user's global environment.
- **Windows doctor limitation:** the Windows branch prints the Visual Studio/SDK/DirectX prerequisites but does not detect them; it currently succeeds when Cargo/rustc/just and metadata succeed. This is useful tool/workspace evidence, not linker/GPU/native readiness. Its existing final text explicitly excludes native GPU/IME/accessibility/DPI evidence. Main session should avoid treating this exit code as the full platform-prerequisite contract.
- The main session reports the Tauri baseline receipt is `research/baseline.md`, currently `not_run` because fixture isolation has no supported equivalent of its real-home startup. Native helper availability is a separate question; do not conflate the two.
