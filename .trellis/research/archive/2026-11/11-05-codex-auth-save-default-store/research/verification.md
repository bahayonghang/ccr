# Verification evidence

## Before implementation

- `just version-check`: PASS.
- `just fmt-check`: PASS (root and Tauri fmt; JSON checks).
- Implementer ran the newly added default-store unit regression against unchanged production code: FAIL, expected File but got Auto.
- Implementer ran the new compiled CLI regression against unchanged production code: FAIL, missing successful khanh save output.
- These two red runs establish the original bug before the production patch. Exact commands and green results belong in implementation evidence.

## Green checks

- See implementation-evidence.md for exact commands: ccr-codex 396 passed / 2 ignored; ccr-cli 429 passed / 2 ignored; actual CLI output suite 25 passed.
- Independent review: resolved raw-auth error redaction issue, no remaining blocking code findings; see review.md.
- `just lint-strict`: PASS on final implementation, all workspace/all targets/all features with warnings and unwrap denied; sensitive persistence policy check also passed.
- Full CI's `cargo check --workspace` and `cargo test --workspace --all-features -- --skip export_bindings`: PASS on final implementation, including all binary integration tests and doc-tests.
- `target/debug/ccr.exe` through the existing managed-cache symlink is compiled and used by the synthetic save tests. Installed CCR remains unchanged.

## Full gate and environment recovery

- Initial `just ci` passed version, fmt, OMP, harness, and Copilot checks, then could not access the external D:/mbx-cache directory at Clippy startup. This was an environment access failure, not a passing gate or a source diagnostic.
- Scoped elevated `just lint-strict` passed. Elevated `just ci` then passed version/fmt/OMP/harness/Copilot/strict lint/workspace check/workspace tests/release/root and Tauri lock audits, and stopped at workflow-governance tests.
- The 19 governance assertion failures came from the selected WSL `bash.exe` failing to mount Docker's missing ext4.vhdx (`ERROR_PATH_NOT_FOUND`), not from auth changes. Selecting existing Git Bash through process-local PATH under the sandbox instead failed MSYS object creation (`0xC0000022`). No global configuration was modified.
- Elevated `$env:PATH = 'C:\Program Files\Git\bin;' + $env:PATH; just workflow-governance-check`: PASS, 53 tests and 53 immutable action references. This restores the failed leaf; the original aggregate run is still recorded as failed, not relabeled passed.
- `just dependency-governance-check`: PASS, 6 tests; `just tauri-command-inventory-check`: PASS (external-cache access required).
- `just frontend-audit`: PASS under existing policy, 2 advisories covered by 2 active exceptions.
- `just vscode-ci`: PASS, type-check/tests/build/VSIX packaging. npm ci reported existing 5 advisories (3 moderate, 2 high); no dependency fixes were attempted in this auth task.
- `just tauri-ci`: PASS (dependency checks, fmt, check, clippy, all-feature behavior tests, separate binding generation/consistency gate, command inventory).
- `just frontend-check`: PASS (type/lint/style/cycles/boundaries/i18n/smoke, frontend build, docs audit/build).
- `just frontend-coverage`: PASS, 170 test files / 918 tests; statements 72.97%, branches 59.64%, functions 71.88%, lines 75.23%; existing thresholds unchanged.
- All constituent steps of `just ci` have now passed. The original aggregate was interrupted by the Bash environment issue, its failing leaf was recovered with process-local Git Bash selection, and all remaining suffix gates were run separately. Do not describe the original aggregate invocation itself as exit 0.
- Final `git diff --check`: PASS. Concurrent GPUI files/specs/tasks remain outside this task's edits.

## Release smoke

- Release build PASS (optimized build).
- Actual `target/release/ccr.exe codex auth save khanh` executed against a temporary synthetic ChatGPT auth.json and config.toml without the store setting: exit 0, saved snapshot bytes equal source, registry present, source/config bytes unchanged, sentinel access token absent from stdout/stderr.
- The smoke used explicit temporary HOME/USERPROFILE/CCR_ROOT/CCR_DATA_DIR/CCR_CODEX_DIR/CODEX_HOME/CCR_LOCK_DIR and removed inherited API-key overrides. No actual account state was touched.

No real-account credential save or installed-binary replacement is part of this verification.

## Delivery status

Implementation, regression coverage, independent review, and verification are complete.
At implementation handoff the source changes were uncommitted. The user's
follow-up explicitly authorizes committing the related changes and archiving
this task; the session journal records the resulting work commit.
Run the verified release binary directly:

```powershell
& 'D:\Documents\Code\Github\ccr\target\release\ccr.exe' codex auth save khanh
```

The installed executable on PATH still has its previous behavior until replaced
by a separately requested installation/update.
