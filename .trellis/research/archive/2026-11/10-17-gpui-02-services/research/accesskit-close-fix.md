# Windows UIA close-race dependency repair

Date: 2026-10-18. Active task: `10-17-gpui-02-services`.
Implementation owner: `accesskit_close_fix`; independent review and native UI
verification belong to `services_check` and the main session, respectively.

## Scope and cause

The real native Alt+F4 failure supplied by the main session reaches
accesskit_windows 0.34.0 `ClientToScreen(...).unwrap()` with HRESULT 0x80070578,
then aborts across the COM ABI. An in-flight provider retains a strong Context
after HWND destruction. Full exact-source evidence and upstream references are
in `native-close-race.md`; ordinary runtime task cleanup is a separate contract.

The independent desktop workspace now patches only that exact adapter crate to
`ccr-desktop/vendor/accesskit_windows`. Private utility/Context methods return
Windows `Result<Point>`, mapping failed conversion to existing
`UIA_E_ELEMENTNOTAVAILABLE` (0x80040201). BoundingRectangle,
ElementProviderFromPoint, RangeFromPoint and GetBoundingRectangles propagate `?`.
Absent node/text bounds preserve their existing successful empty responses.
No public API/feature/dependency/MSRV change, invented coordinates, production
panic catch, accessibility disablement, or GPUI/Kit fork was introduced.

`UPSTREAM.json` retains every original file SHA-256, archive checksum
`106c2b961215864d1c2e703ee63269c25c4e80a577ffb2c1017b9c17dcdf83a1`,
upstream commit `c88605b96d04431f9c3c792464a0f2f253480e94`, path
`platforms/windows`. Original manifests/lock/examples/tests are unchanged.
The registry omitted referenced license texts; Apache, MIT and Chromium notices
were fetched from the pinned commit root. An attempted adapter-subdir Chromium
URL returned 404; the root URL succeeded. Initial sandbox license requests
failed with socket access denied; authorized retries succeeded. No global cache
source was modified. `CCR-PATCH.md` documents the exception and removal process.
Official crates.io API was freshly checked on 2026-10-18 (exit 0): latest stable
0.35.1; only compatible non-yanked 0.34.* is 0.34.0, MSRV 1.85 and archive
checksum identical to the original lock. Raw response:
`outputs/desktop/services/accesskit-close/accesskit-windows-registry.json`.

## Deterministic regression

Original no-panic test actually executed against the unchanged utility and
failed at the original unwrap: **exit 101**, 0 passed / 1 failed, HRESULT
0x80070578. This was a runtime panic, not a compile-signature mismatch.
Raw: `outputs/desktop/services/accesskit-close/stock-regression.log`.

Fixed private tests: **3 passed**, exit 0:

- exact invalid/null HWND conversion returns 0x80040201 without panic;
- a deliberately retained Context and real COM provider interfaces exercise
  all four coordinate methods and assert the exact unavailable HRESULT;
- node/text absent bounds still return default rectangle/null array.

No native window is created, shown or controlled. The first fixed compilation
returned 101 because the exact Windows binding returns BOOL (requiring `.ok()`)
and consumer NodeId differs from AccessKit NodeId; both were corrected and
rerun. Raw: `fixed-regression-first.log`, `fixed-regression.log`.
The final tests explicitly assert literal HRESULT **0x80040201**, avoiding an
assertion that merely calls the same error helper as production. Final exact
source rerun: 3/3, exit 0, `fixed-regression-final.log`. Focused formatting check
returned 0 after a separate one-line formatting repair. `source.patch` contains
the complete final registry-to-vendor Rust diff. `target-recovery.json` retains
the verified exact cache target, original before/after artifact hash and restored
workspace-local target check.

Cargo initially rejected `cargo test -p accesskit_windows --lib` because this
patched dependency is not a workspace member and needs upstream dev dependencies
(101, no tests executed). Adding it as a member would pull unrelated winit test
dependencies into the production lock. The reproducible gate instead uses
`scripts/test_accesskit_patch.py`, also invoked by `just desktop-test`:
byte-identical source copy in a temporary directory under the verified desktop
target; omit only upstream `mod tests` and its four unrelated dev-dependency
tables; reconcile the temporary lock offline; reject all package
version/source/checksum selections absent from the production lock; test
`--locked --offline --lib ccr_invalid_hwnd -- --skip export_bindings` using
default parallelism; recheck source hashes. All production modules and private
CCR tests remain byte-identical. Non-Windows explicitly reports not_applicable.
The final isolated harness selected **22 packages**, all present in the desktop
lock. An initial non-escalated reproducible run returned 101; authorized cache
access retry returned 0 (3/3). Raw: `reproducible-regression*.log`.

## Toolchain/cache incident and recovery

Machine default `cargo` is an mbx shim, distinct from actual Rust Cargo. The
initial elevated test invocation adopted the desktop target into
`D:/mbx-cache/targets/v1/ee43b576e028adad95351a2e838b0e5b0a3d7e5781bf26d84f2ccfaee43a9047`
and placed a symlink at `ccr-desktop/target`. The desktop isolation check correctly
rejected it with exit 1. No isolation rule was relaxed. Read-only mbx settings
showed target.keep contains root/Tauri only; no setting was changed.

Before restoration, verified the exact symlink target, directory association
(debug/release/tmp/Linux artifacts) and original executable SHA-256
`daae9c7a5d037835951a12a79fd808207a55a560057797776a344d4eefbf48ed`.
Removed only the verified symlink using .NET Directory.Delete (not recursive),
then moved that exact cache directory back using Directory.Move in the same
PowerShell process. Restored artifact hash is identical; target is now a real
workspace-local directory; `python scripts/desktop.py verify` returns 0.
No unrelated cache, global setting, HOME, user configuration or account changed.

Subsequent initial patch checks used **process-only PATH** priority for the
existing actual toolchain:
`C:/Users/lyh/.rustup/toolchains/1.98.0-x86_64-pc-windows-msvc/bin`,
Cargo `1.98.0 (797e8a9bc 2026-08-05)`, Rust
`1.98.0 (88d9e12ae 2026-08-18)`. They are conditional runs, not proof that the
default command path works. The main session separately assigned services_check
the verified mbx resolver repair in desktop.py; this owner wires the justfile
and reuses that resolver for the harness. Default-path validation follows that
handoff below.

## Initial actual verification (real-toolchain PATH condition)

Logs under `outputs/desktop/services/accesskit-close/`:

| Command | Exit | Result / log |
| --- | --- | --- |
| `just desktop-check` | 0 | locked all-targets; `desktop-check.log` |
| `just desktop-lint` | 0 | locked all-targets / -D warnings; `desktop-lint.log` |
| `just desktop-test-ui` | 0 | actual GPUI headless tests 2/2; `desktop-test-ui.log` |
| `just desktop-test` | 0 | Python command boundaries + patched adapter 3/3 + app units 3/3 + UI 2/2; `desktop-test.log` |
| `cargo build --locked --bin ccr-gpui` | 0 | debug binary; `desktop-build-debug.log` |
| focused `rustfmt --edition 2024 --check` | 0 | five changed adapter/test files; separate earlier formatting repair |
| `git diff --check` | 0 | tracked shared edits preserved; CRLF notices only |

Debug artifact: `ccr-desktop/target/debug/ccr-gpui.exe`, SHA-256
`dd53feb570608970a1ce7cced1996399f541244b912d87afa8042988a38bf185`.
Main was notified of the exact hash for native rechecks. Headless tests are not
native UIA/IME/close proof. No valid native window test or application launch was
performed by this implementation agent.

`lock-diff.json` proves the **only changed production lock package** is
accesskit_windows 0.34.0, removing registry source/checksum for the local patch.
All other records (versions, checksums, sources, dependency lists) are identical.
Final desktop lock SHA-256:
`95c30e2ec5d68b3547a3cb219aac7898d867ff906c809af1591dc1b3dfbc41af`.
Root/Tauri manifests and locks were not edited by this owner. Earlier root/Tauri
gate evidence belongs to services_check and is not claimed as rerun here.

## Final command-path repair and verification

The coordinator explicitly transferred desktop.py and existing command-test
ownership from services_check to this implementation owner. The final
`resolve_cargo()` uses the official `mbx setup --status` first line and file
identity, bypassing only the confirmed installed/current shim through
project-cwd `rustup which cargo`. No filename/suffix guess identifies a shim.
Ordinary Cargo is retained. If mbx status is unavailable/ambiguous, only file
identity with the rustup proxy or its actual selected Cargo permits execution;
otherwise it fails closed. A missing real toolchain is explicit. No global mbx
configuration, PATH, HOME or toolchain default is changed.

All desktop Cargo recipes, metadata, clean, doctor and the regression harness
use the same resolver. Existing locked/argument/exit semantics remain; actual
stub exit **23**, all **21 environment override cases**, and foreign-target clean
rejection are covered by the retained tests.

The first default-path desktop-check returned **101**, E0514: direct Cargo 1.98
spawned the rustup compiler proxy from dependency registry cwd, choosing global
default rustc 1.99 rather than project 1.98. No artifact clean or dependency
upgrade was used. `cargo_environment()` now prioritizes the confirmed real Cargo
toolchain's bin directory **only in that child's PATH after shim bypass**.
This pairs compiler/rustdoc/subcommands; ordinary Cargo inherits the original
environment. Raw failure: `default-desktop-check.log`; fixed pass:
`default-desktop-check-fixed.log`.

Normal PATH, authorized existing cache access, final actual wrapper results:

| Command | Exit | Result / log |
| --- | --- | --- |
| `python scripts/desktop.py verify` | 0 | Real workspace-local target; same normal-path invocation as default regression run |
| `just desktop-check` | 0 | Locked all-targets; `default-desktop-check-fixed.log` |
| `just desktop-test` | 0 | Python **12/12**, adapter **3/3**, app **3/3**, UI **2/2**; `default-desktop-test.log` |
| `just desktop-lint` | 0 | Locked all-targets, -D warnings; `default-desktop-lint.log` |
| `just desktop-build-debug` | 0 | Locked debug build; `default-desktop-build-debug.log` |

Final built executable SHA-256 (supersedes earlier DD53/D9FB artifacts):
`94acabe375bc032da8cf868934eae6fb2de4392ab55cd7440849e3e9b84fc741`.
Desktop target remains a real local directory. In the restricted sandbox,
mbx's status can instead report a namespace-local missing installation; the
resolver then correctly refuses an ambiguous selected shim (exit 1). The host
authorization retry above has actual installed/current identity and succeeds;
no permission/identity refusal is counted as a passed build.

services_check independently reviewed the final resolver, child environment,
doctor and harness, and read the actual default check/test logs. It reported no
remaining defect in these changes; no redundant Cargo rerun was claimed by that
reviewer. Final source/manifests/lock/justfile/helper/tests are frozen for handoff.

## Remaining verification

Native repeated UIA queries/Alt+F4, valid-window coordinate
behavior, stderr/exit/process cleanup checks belong to the main session.
The earlier `accessibility:null` helper output is unresolved; this patch does
not claim full native a11y, screen-reader, DPI, three-platform or performance
acceptance. No commit, push, signing, publishing or task archival was performed.
