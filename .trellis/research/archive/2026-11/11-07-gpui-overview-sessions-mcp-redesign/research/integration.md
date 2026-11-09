# Integration receipt — redesign continuation (2026-11-08)

Scope: `HANDOFF-gpui-redesign.md` §5.1 A–F, the existing task's final desktop gate and native visual evidence. Base: `911e9d7b72404b0fa1982d30a622684c17771ce0`, branch `dev`.

## Environment findings

- In the sandbox, `mbx setup --status` resolves a sandbox-specific LOCALAPPDATA and reports its Cargo shim missing. `scripts/desktop.py` consequently rejects the ambiguous shim before Cargo runs. No project scripts were changed: prepend `C:\Users\lyh\.cargo\bin` to this process's PATH to select the installed Rustup proxy that the existing resolver accepts.
- The sandboxed `desktop-test` Python command-contract suite encountered Git Bash/MSYS `NtCreateDirectoryObject` access denial (`0xC0000022`) in two simulated Linux/macOS recipe subcases. The unsandboxed aggregate rerun below passed these tests. No coverage was removed to avoid the host restriction.
- Preserve default Rust test parallelism and the existing `--skip export_bindings` argument. No threshold, skip list, or stack limit was relaxed.

## Final gate

`just desktop-ci`: **exit 0**, **339.0268024 seconds**, with installed Rustup Cargo prepended to process PATH. Code delivered in `a145abba166eef4828efdaeba8bccf2b7783cf0f`.

| Gate | Actual result |
| --- | --- |
| desktop-fmt-check | passed |
| desktop-check | passed, all targets |
| desktop-lint | passed, `-D warnings` |
| Python command contracts | 16 passed, 23.405s |
| AccessKit patch harness | 3 passed, 6 filtered out |
| Rust lib | 294 passed, 0 failed, 4 ignored, 87.89s |
| native_shell | 12 passed, 1 ignored, 2.36s |
| native_ui | 2 passed, 0.07s |
| runtime_shutdown | 2 passed, 1 ignored, 0.27s |
| main/doc tests | 0 tests, successful |
| desktop-build | passed, release optimized, 3m 03s |

The aggregate began before the final stale-count/empty-pane guards landed. Its release step compiled the final behavior. After those changes, `just desktop-fmt-check`, `git diff --check`, and `just desktop-lint` passed again; the focused `failed_list_read_keeps_provider_filters_and_codex_trash_visibility` test passed **1/1**, **30.17s**, on the final tree. It asserts retained checked provider chip/trash availability, no stale rows, an empty count label and no stale empty detail pane. The independent review has no unresolved source findings (`check-continuation.md`).

`desktop-test-ui` was not repeated as a separate recipe: the aggregate's full `cargo test --locked -- --skip export_bindings` ran both native_shell and native_ui binaries with the same flags. Four lib and two integration ignored cases remain the pre-existing explicit/native-helper fixtures; they are not acceptance passes.

Release executable SHA-256: `CFFF57EB9FDA0C94263D91FC01F8BB984108641423BCD3806AB137092C215CC0`. Native launch used an identical copy at `outputs/desktop/redesign-native/ccr-gpui-redesign.exe`, so the target executable remains free for later builds. The full local log is `research/desktop-ci.log` (ignored generated evidence).

## Native evidence

The final release launched after the user requested reopening when the first Computer Use application approval timed out. Three light/en wide-page screenshots were saved. Capture normalized its image dimensions and was interrupted by user input/foreground changes; the exact logical dimensions and the requested theme/size matrix were **not verified**. See `native-visual.md`. The task remains `in_progress`, with no archive and no push.

The temporary theme/geometry preference preparation was restored byte-for-byte before the successful reopen: original and restored SHA-256 both `5a72f2dd62026dd21d86ea210454255007265db4045cc161135dad4958c5b7f0`. Subsequent user window changes were not overwritten.

## User-requested archive — 2026-11-09

The user explicitly requested committing all changes and archiving this task. The earlier in-progress/do-not-archive notes are historical and superseded by that instruction. Implementation and the recorded desktop gates passed. The unchecked native theme/size/Chinese matrix, calibrated gap measurements and other native acceptance remain **UNVERIFIED**; archival does not turn them into passes. Preserve the unchecked acceptance items and remaining gaps for future work. No push.
