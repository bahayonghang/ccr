# Native acceptance feasibility

2026-10-26. Read-only filesystem/source assessment requested by the main session. No native application was launched, no executable rebuilt, no package installed, and no user catalog/database/preference was touched. This file is the only mutation in this follow-up.

**Current airy-glass native artifact acceptance: NOT_RUN.** Available executables predate the October 26 frontend changes, and the existing Tauri startup does not expose complete fixture isolation. Browser evidence cannot close this gap.

## Bounded artifact search

Inspected code-map anchors: `ccr-ui/src-tauri/target`, its resolved managed target, `ccr-ui/output`, `outputs/ccr-ui`, `outputs/desktop`, `outputs/mbx-repair`. Recursive inspection excluded dependency/build internals (`node_modules`, `deps`, `incremental`, `.fingerprint`, `build`). Read `tauri.conf.json`, Cargo manifests/config, the native window probe, native startup/state/tray code, domain fixture tests and the local `dirs` implementation. This is a bounded local assessment, not a claim that no other executable exists anywhere on the machine.

The workspace target is a symbolic link to:

`D:\mbx-cache\targets\v1\44061aa3da5bb563d33700d15e5ae7660d5d3da33a41e506f28329282a48c2b6`

Normal link traversal returned no contents in this session, but the resolved target is readable. Its executables exist:

| Executable relative to resolved target | Bytes | Modified UTC | SHA-256 | Version |
|---|---:|---|---|---|
| `debug/ccr-desktop.exe` | 75,962,880 | 2026-10-18T06:41:31.1959251Z | `7ABB359E11B7FC5820B644F2805CE82A28E42FA51AF5DE052E253FB5DF671CC3` | 7.5.0 |
| `release/ccr-desktop.exe` | 100,052,992 | 2026-10-20T09:12:09.8272175Z | `CF87D64D9A8DDE47896164344FD2BDEC103E534F84A306216F9FB1D3AD3DDC06` | 7.5.0 |

`outputs/ccr-ui/src-tauri/target/release/ccr-desktop.exe` has the same length, timestamp and SHA-256 as the release row. The corresponding 7.5.0 NSIS installer also dates to October 20 and was not run.

Observed source/output timestamps at assessment: `src/styles/tokens.css` 2026-10-26T12:04:24Z, `src/shell/shell.css` 2026-10-26T11:54:34Z, `dist/index.html` 2026-10-26T11:57:03Z. Timestamps are staleness evidence, not a claim that the latest complete build was accepted. The release binary cannot embed the later frontend files. An old debug binary loading a live dev URL would also not establish current backend/build identity.

## Why existing launch paths are not safely isolated

- `src-tauri/src/main.rs:134` startup calls `ccr_config::migrate_catalogs`, initializes the application and usage database pools, creates AppState, installs desktop shell/tray, schedules initial Codex tray refresh and restores pending OAuth listeners. Merely avoiding UI buttons does not make launch side-effect-free.
- `crates/ccr-db/src/database/mod.rs:54` resolves the usage archive root through `CCR_DATA_DIR` / `CCR_ROOT`, but `get_db_path` at line 68 still creates `dirs::home_dir()/.ccr-ui/ccr-ui.db`. Pool initialization runs migrations with that actual home directory. Those two overrides alone do not isolate the application database or legacy import inputs.
- `src-tauri/src/state.rs:216` resolves shell preferences through `dirs::home_dir()/.ccr/desktop-shell.json`; startup loads/quarantines corrupt preferences, and tray placement can persist settings. This path does not consume `CCR_ROOT` or `CCR_DATA_DIR`.
- Domain unit/contract tests do expose explicit roots such as `CCR_PROFILE_CONTRACT_ROOT`, `CLAUDE_CONFIG_DIR`, `CCR_SETTINGS_PATH`, `CCR_BACKUP_DIR`, `CCR_CODEX_DIR` and `GROK_HOME` (`commands/config/contract_tests.rs:11`). They are test-process fixtures, not a complete Tauri launch mode.
- The installed `dirs-6.0.0/src/win.rs:5` implements `home_dir()` through `dirs_sys::known_folder_profile()`. `dirs-sys-0.5.0/src/lib.rs` calls Windows `SHGetKnownFolderPath(FOLDERID_Profile)`. Changing a child `USERPROFILE` environment variable alone is therefore not sufficient proof of home isolation.
- Existing `outputs/desktop/{profiles,services,shell}/launch-native-fixture.ps1` target `ccr-desktop/target/debug/ccr-gpui.exe` with GPUI fixture flags. They cannot be applied to the Tauri executable.

No launch-wide Tauri fixture switch or explicit WebView user-data directory was found in the inspected startup/state/tray entry points. Do not patch a new backend isolation mechanism as part of this visual-only task.

## Existing probes and their limits

- `ccr-ui/scripts/verify-release-window.ps1` starts an executable, waits five seconds, enumerates its visible `Tauri Window`, checks minimum 800×600 and force-stops the process. It does not isolate application data, inspect the rendered theme, record window DPI, or validate tray interaction; it was inspected but not run.
- `ccr-ui/.tmp-desktop-probe.mjs` can attach Playwright over CDP at port 9222 and measure the native WebView DOM, chart bounds, screenshots and `window.devicePixelRatio`, using the dev URL at port 15173. It assumes a separately launched app and does not provide fixture isolation. It was inspected but not run.
- `src-tauri/src/desktop_shell.rs` defines the compact panel as 456×620 and clamps it to the monitor work area. Existing tests cover top/bottom taskbar placement, missing work area, small work area, manual position preservation/clamping and close-to-tray. These are pure geometry/control tests, not actual Windows 100%/150% DPI, focus, hit-target or popup rendering acceptance.
- `tauri.conf.json` retains main window 1200×800/minimum 800×600, decorations enabled and `transparent: false`; this assessment changed none of them.

## Concrete follow-up avenue

Build the current frontend and a matching Tauri executable, then run the existing window/CDP probe from a disposable Windows account or VM/session whose OS Known Folders and WebView profile are isolated. Record executable/source hashes, actual WebView version, window/client dimensions, DPI at 100% and 150%, system-theme transitions, native focus/resize and tray open/close/placement behavior. No install is required merely to use a freshly built executable, but that native build/isolated session has not been performed here. macOS/Linux native checks also remain NOT_RUN.

Until that evidence exists, report web implementation/checks separately and keep AG04 native acceptance open.
