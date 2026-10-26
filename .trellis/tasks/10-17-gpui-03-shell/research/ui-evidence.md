# 03 UI implementation evidence

Date: 2026-10-18. Owner: shell_ui (trellis-implement). Shared workspace preserved; no commit, push, package, native launch, user-account operation or system setting change.

## Delivered boundary

- `ccr-desktop/src/theme.rs`: single approved DESIGN.md semantic palette, system UI/mono font resolution, Light/Dark/System session preference, separate Theme::change then Theme::update so Base and Component remain synchronized. Component motion durations are zero; no collapsing animated sidebar or decorative background.
- `src/shell/`: only connected Local Configurations, Favorite commands, Recent commands and Settings. No WSL/SSH selectors, placeholder business pages, fake execution, simulated success or synthetic history. Theme/language preferences explicitly apply to the current window session.
- `src/locales/`: small English/Chinese copy selector, including existing safe service-error translations. No raw domain error formatting.
- `src/app.rs`: existing actual service read chain retained, generation gate and read-drop lifecycle preserved. Virtual stable-name rows, selection details and Back/Escape focus restoration. Config filtering remains on the bounded worker.
- `src/shell/command_state.rs`: real UiStateRequests load/add/remove/clear actions, view read cancellation and application-owned mutation lifecycle. Favorite parameters are one argument per line and stored only, never shell-executed. Snapshot fields use the adapter's existing redaction owner; raw args/full_command are not displayed. New favorites do not create history.
- `src/shell/semantics.rs`: builder-only decoration of the existing styled Button's exact semantic node. No rendered wrapper/duplicate Button. Behavioral disable remains Kit-owned, same-node disabled/busy flags added. Disabled text uses opaque semantic muted colors instead of Kit's half-opacity default; buttons use the approved pill radius and native immediate active color feedback.
- `lib.rs` now opens Shell; `--probe` remains the original fixture-only input probe. README/DESIGN known gaps updated to actual state.

No Cargo manifests, lockfiles, runtime adapter/domain SQL, task state or parent inventory were edited by this UI owner. The domain owner added the injected UiStateRequests API. Main updated only desktop-test-ui's existing command to include native_shell.

## Actual validation

Environment: Windows, existing Rust 1.98.0 MSVC toolchain, exact locked GPUI Kit/Base/Component 0.7.3 and gpui-pre 0.3.9; existing repository-local target via desktop helper. No global HOME/PATH or build-cache setting changes. Rust suites keep default parallelism and `--skip export_bindings`.

| Command | Exit | Actual result |
| --- | --- | --- |
| `just desktop-check` | 0 | Locked all-target compile, after fixing exact Kit trait imports and action closure signature. Initial sandbox helper identity failure and compile failures were not passes. |
| `just desktop-fmt` | 0 | Explicit repair only, separate from validation. |
| `just desktop-fmt-check` | 0 | Formatting validation. |
| `just desktop-test-ui` | 0 | native_shell 2/2 and native_ui 2/2. Actual English/Chinese replacement, Light/Dark/System controls and Base/Component palette equality; F6/Shift-F6, nav arrows, Tab/Shift-Tab, and search editing. |
| `python scripts/desktop.py cargo test --locked --lib real_favorite_controls -- --skip export_bindings` (desktop cwd) | 0 | 1/1: actual control click creates persisted favorite, refresh retains UUID, remove persists deletion, no execution history is invented; private fixture database only. |
| `python ccr-desktop/scripts/desktop.py cargo test --locked --lib virtual_rows_keyboard -- --skip export_bindings` | 0 | 1/1: actual 10k-row renderer creates only visible range (<100), arrows select, Enter details, Escape/Back restore and reordered results preserve stable name. Synthetic view fixture, not a domain-storage acceptance claim. |
| `just desktop-test` | 0 | Python12/12, exact-source AccessKit close harness3/3, desktop Rust lib7/7, native_shell2/2, native_ui2/2; doc/main targets had zero tests. Log: `outputs/desktop/shell/ui-desktop-test.log`. |
| `just desktop-lint` | 101, then 0 | Fixed one collapsible-if and four useless test-handle conversions, reran strict all-target locked clippy. Final log: `outputs/desktop/shell/ui-desktop-lint-final.log`; initial failure preserved in `ui-desktop-lint.log`. |
| `just desktop-build-debug` | 0 | Actual binary build completed in 52.63s for main's native core validation. Log: `outputs/desktop/shell/ui-desktop-build-debug.log`. |

An attempted virtual-list test command initially used the desktop cwd with root-relative write paths, so no test was added or run (zero selected tests). That attempt is not evidence. It was corrected to repository cwd and the named actual test passed as recorded above. One text-edit expected-value assertion was corrected from an incorrect Chinese deletion expectation to the actual intended editor result, then both rendered suites passed.

## Keyboard handoff

F6 cycles navigation -> main -> status -> navigation; Shift-F6 reverses. From current config-search focus, the next F6 is status, then navigation. Nav Up/Down or Tab/Shift-Tab changes actual focus; Enter/Space invokes Kit button actions. In Config main, F6 focuses search; Tab reaches the available refresh/list controls. With list focus, Up/Down selects by stable name, Enter opens details, Escape or Back returns and restores list focus/selection. Text editor arrow/IME handling is untouched.

## Explicit native and contract gaps

No native UI operations were performed by this agent. Main owns actual Windows theme/language/navigation/read/filter/error-retry/favorite/IME/cancel/exit screenshots and process receipts. Headless input is not OS IME proof. No macOS/Linux native or screen-reader delivery claim is made here.

User 2026-10-18 explicitly deferred window-size/zoom/DPI matrix verification. No resize, zoom or DPI tests were added or executed. DESIGN responsive/minimum-window implementation remains.

Native appearance observation is implemented and retained only under System preference. The exact installed TestAppContext has private `test_window`/`test_platform` access, so external tests cannot directly call TestWindow::simulate_appearance_change. Tests cover actual System control application and palette synchronization, not an independently simulated OS appearance-change callback. No visibility setting was changed for this test.

Styled Button semantic flags are installed on the existing node, but Kit observation snapshots capture before synthetic callbacks and public debug_a11y_tree_json omits disabled/busy fields. `None` is not interpreted as either success or failure. Actual native UIA/screen-reader states remain separately unverified; editable Input remains the unchanged Kit editor, with no invented disabled/readOnly Input case. Existing log redaction has format-dependent limits; raw arbitrary metadata must not be described as universally credential-safe.

UI/product/tests/README/DESIGN frozen and released to independent shell_check. Only this research evidence remains owned by shell_ui pending the debug build receipt.

Debug build receipt: `ccr-desktop/target/debug/ccr-gpui.exe`; SHA-256 `0efb818550bfc371f61e9c871bb468d519d92e32f24a710c280116cba95758e8`, 56,204,288 bytes, independently measured after successful build. Lock SHA-256 `9c8f5b816b01a2db2285d0092fa223e791ffe3f48f02657ec4edad1708c7216f`. `git diff --check` exit0 after final handoff.
