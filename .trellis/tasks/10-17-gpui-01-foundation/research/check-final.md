# Foundation independent check — 2026-10-17

Scope: approved task 01 only. Product-file ownership was explicitly handed from trellis-implement to trellis-check before formal verification. No other participant's changes were reverted; the initial review changed no product source, and the later narrow Linux doctor correction is recorded below. No commit, push, publication, task archival, real-account access or HOME mutation occurred.

## Findings (fixed before final review)

- **Target isolation:** preflight found ordinary build/run/check/lint/test entries bypassed the cleanup guard. The implementer added `scripts/desktop.py verify` before all seven build/test entries. The shared validator checks workspace_root, target_directory and Cargo's newer build_directory against the exact resolved independent target. The reviewer reran all 21 foreign-environment cases and verified rejection before compilation.
- **Failure propagation:** the implementer added explicit Windows `$LASTEXITCODE` forwarding to both justfiles. The reviewer reran the actual controlled Cargo failure test through the root and child recipes and confirmed exact exit **23**, rather than accepting a dry-run graph as proof.
- **Doctor and documentation:** Windows doctor now reads actual MSVC/SDK/GPU inventory and explicitly separates inventory from native rendering. README describes the delivered foundation probe. Both version-sync fixtures register and test the independent manifest. The earlier preflight observations do not remain open code defects.

No additional mechanical defect was found after lint, type-check and the focused regression checks passed.

## Findings (not fixed / acceptance limits)

1. **AC1.5 is incomplete:** Windows build and native probe receipts exist, but at this review cutoff macOS/Linux compilation receipts do not. Main session is investigating an available Linux environment; this reviewer did not duplicate its operations. A later receipt must be evaluated separately. No macOS support claim can be inferred from Windows success.
2. **Required performance baseline/probe measurements are not_run:** the baseline report identifies fixture-path and timing seams needed to avoid real HOME/database/OAuth/WebView state. This is beyond a mechanical correction to foundation command plumbing. Preserve not_run/not_comparable and coordinate the path-provider work with its owning domain. Do not claim foundation's baseline requirement or any parent performance budget has passed.
3. **Native coverage is partial:** the main session's Windows receipt establishes a visible real GPUI window, actual Chinese IME composition/commit, focus/selection and normal close. Four logical sizes, DPI matrix, themes, full keyboard flow and native accessibility remain unverified; accessibility capture returned null. These need task 03/10 investigation and native receipts, not headless assertions.
4. **Eight upstream unmaintained dependency warnings remain:** instant, paste, rustls-pemfile, rustybuzz, ttf-parser, zbus_names, zvariant and zvariant_derive. The audit receipt preserves their advisory IDs and RustSec revision; there is no ignore or weakened threshold. Replacing/upgrading the Kit-bound transitive stack would require compatibility judgment, so the reviewer did not alter it.
5. **Unix command behavior lacks a native execution receipt:** Bash fixture tests pass under Git Bash, and source preserves nonzero exits without Windows suffixes on Unix, but the root/child public command propagation was actually exercised only on Windows. Do not extend that proof to macOS/Linux until their executions exist.

## Verification — reviewer executed

Host: existing Windows x86_64 MSVC environment, repository development compiler 1.98.0. All commands ran from repository root except the explicitly noted desktop cwd.

| Command | Exit | Result |
| --- | --- | --- |
| `python -B -m unittest discover -s tests -p 'test_*.py'` (desktop cwd) | 0 | **5/5**, 50.932 s; 21 real foreign target/build environment cases, exact exit 23, metadata cleanup rejection and command graph |
| `just desktop-lint` (sandbox) | 1 | Metadata guard passed; existing mbx cache access denied. Preserved as failed attempt, not a source failure or passing receipt |
| `just desktop-lint` (approved cache access retry) | 0 | Strict `cargo clippy --locked --all-targets -- -D warnings`, 1.89 s Cargo result |
| `just desktop-check` (approved cache access) | 0 | `cargo check --locked --all-targets`, 1.49 s |
| `just desktop-test-ui` (approved cache access) | 0 | **1/1** Kit headless Unicode/focus/redraw/backspace/close test, default test parallelism and `--skip export_bindings` |
| `just desktop-fmt-check` | 0 | Read-only format validation; reviewer did not run fmt repair |
| `./scripts/version/version-sync.ps1 -Check -Verbose` | 0 | All current versions, including independent GPUI manifest, equal 7.5.0 |
| `cargo tree --locked --offline -p ccr --edges normal --prefix none` + forbidden-package assertion | 0 | 834 root normal CLI entries, no GPUI/Tauri |
| `cargo metadata --locked --offline --no-deps --format-version 1` (root) | 0 | Root workspace retained; default member is `crates/ccr` |
| `cargo tree --locked --offline --edges normal --prefix none` (desktop) + forbidden-package assertion | 0 | 920 entries, no Tauri/GPUI Fast |
| `git diff --check` | 0 | No tracked-file whitespace errors; CRLF notices retained |

The reviewer also parsed the retained official registry response: newest/max_stable **0.7.3**, selected version not yanked, upstream rust_version absent and Apache-2.0 license. `ccr-desktop/dependencies.md` records normal/dev features and the missing upstream MSRV declaration instead of inventing one. Cargo.lock SHA256 independently matches **f05306cb38264a91afd699843ab7e67f380e180698ed31b9e39d6f62fd39cacd**.

## Existing receipts independently inspected

- `outputs/desktop/foundation/desktop-ci.log`: ordered fmt-check/check/clippy/Python/Rust tests/release build, ending in optimized release success; no implicit fmt/version-sync repair.
- `msrv-check.log`: actual Rust 1.95 all-target check completed successfully in 2m42s. Resolver compatibility is not used as its substitute.
- `version-pester.log`: 7 passed, 0 failed/skipped/not-run. `version-bats.log`: 8/8. Both scripts and fixtures were reviewed for the new canonical manifest target.
- `audit.log`: 827 packages, 8 upstream warnings, no ignored vulnerabilities. Dependency report and implementation evidence preserve the advisory revision and fetched database status.
- `research/native-probe.md`: root `just desktop-dev` actually launched the native window and exited 0 after Alt+F4, with a separate failed sandbox-cache attempt recorded. `probe-desktop-dev.png` belongs to the newly hashed debug executable; earlier IME captures retain their own executable hash.
- `research/baseline.md`: actual Tauri release build passed; application/performance sampling did not run. Native helper availability is not the baseline blocker.

## Acceptance recommendation

Windows engineering gates for **AC1.1–AC1.4 pass** with the limits above. **AC1.5 and the baseline/probe measurement requirement remain incomplete**. Keep task 01 open; do not treat foundation or the parent migration as completed. Parent task state, migration inventory and later-task dependency decisions remain main-session owned.

The new exact-target/build-directory and explicit Windows native-exit conventions should be captured in a desktop-specific Trellis spec when that spec surface is established. This review's write scope did not include `.trellis/spec/`, so no broader specification restructuring was attempted.

## Follow-up fix: Linux X11 development-library prerequisite

Main session supplied an actual final-link failure after Linux cargo check had succeeded: `-lxkbcommon-x11` was unavailable. `outputs/desktop/foundation/linux-linker-diagnosis.json` traces the linked crate to gpui-pre-linux -> xkbcommon 0.8.0 -> `#[link(name = "xkbcommon-x11")]`; its pkg-config probe failed and dpkg reported the development package absent. `linux-runtime-abi.json` confirms the runtime `.so.0` and FFI symbols exist while the unversioned linker entry does not.

The pkg-config name was independently verified against the [official Ubuntu noble amd64 development-package file list](https://packages.ubuntu.com/noble/amd64/libxkbcommon-x11-dev/filelist): `xkbcommon-x11.pc` and `libxkbcommon-x11.so` are both delivered by `libxkbcommon-x11-dev`. This was not inferred merely from the Rust link name.

### Findings (fixed)

- `ccr-desktop/scripts/desktop.py`: added `xkbcommon-x11` to Linux's existing development-library probe and `--print-errors` to the existing pkg-config check. Missing development metadata now makes doctor return nonzero with the actual missing-module diagnostic. The check remains read-only and installs nothing.
- `ccr-desktop/README.md`: documents the X11 development entry and distinguishes it from the existing runtime SONAME. The official package-file source is linked directly.
- `ccr-desktop/tests/test_commands.py`: added one narrow regression with all other tool/workspace/display prerequisites satisfied and only `xkbcommon-x11` absent. It verifies doctor returns **1** and requests the correct module with diagnostics enabled. No real process runs in this simulated Linux branch; HOME is never changed.

### Follow-up verification

| Command | Exit | Result |
| --- | --- | --- |
| `python -B -m unittest discover -s tests -p 'test_*.py' -k linux_doctor` (desktop cwd) | 0 | 1/1, 0.004 s; missing-X11-development-library failure is propagated |
| `python -B -m unittest discover -s tests -p 'test_*.py'` (desktop cwd) | 0 | **6/6**, 32.592 s; existing target isolation and exact exit-23 regressions remain green |
| Python `compile()` over both changed Python sources, without bytecode writes | 0 | Syntax passes |
| `git diff --check -- ccr-desktop/scripts/desktop.py ccr-desktop/tests/test_commands.py ccr-desktop/README.md` | 0 | No tracked whitespace error; new files were also parsed/exercised directly |

The first full-suite attempt exited 1 because the new test mocked subprocess while Windows `platform.platform()` internally consulted a subprocess. The fixture now also supplies a stable platform description, and both the focused regression and full suite passed afterward. This failure was test isolation, not a Linux/Windows product result, and is retained here rather than represented as pass.

Rust sources/manifests/locks were untouched; prior Clippy/type-check results remain applicable and no Rust recompilation was repeated for this Python/README fix. WSL builds and any outputs-local linker workaround remain main/baseline-agent owned. Such a workaround does not satisfy the absent standard development package and must not make doctor pass. This follow-up does not establish macOS/Linux native or performance acceptance. Narrow product-file write ownership is released back to the main session after this receipt.
