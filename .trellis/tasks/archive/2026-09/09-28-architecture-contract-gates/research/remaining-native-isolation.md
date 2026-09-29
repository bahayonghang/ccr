# Native WebView validation boundary

## Scope

The user selected the remaining native WebView/CSP acceptance and requested continuation. The test uses synthetic configuration without provider credentials, personal configuration, SSH hosts, or real OAuth login.

The planned native run uses Linux WebKitGTK under WSLg. It does not establish Windows WebView2, macOS WKWebView, physical input, or display-hardware behavior.

## Isolation decision

- crates/ccr-core/src/core/logging.rs:147 resolves logs through dirs::home_dir().
- crates/ccr-db/src/database/mod.rs:68 resolves the application database through dirs::home_dir(); the archive respects CCR_DATA_DIR/CCR_ROOT.
- ccr-ui/src-tauri/src/state.rs:216 resolves desktop preferences through dirs::home_dir().
- ccr-ui/src-tauri/src/commands/codex_auth.rs:475 resolves pending OAuth through PlatformPaths and CCR_ROOT.
- ccr-ui/src-tauri/src/main.rs:194 restores pending OAuth, but the isolated root has no pending state. The database has no SSH hosts or accounts.
- crates/ccr-usage/src/paths.rs supports LLMUSAGE_HOME; the fixture points to a new empty directory.
- The installed Linux dirs-sys 0.5.0 implementation reads nonempty HOME before its account-database fallback. This was inspected in the local Cargo source.

remaining-native-webview.py constructs an environment allowlist. HOME, USERPROFILE, XDG paths, CCR roots, Claude/Codex paths, llmusage root, and temporary storage resolve inside a fresh test directory. PATH excludes user-installed provider CLIs. The driver receives a private D-Bus session. No parent credentials or proxy variables are copied. DISPLAY connects to the WSLg X server.

The script requires a successful native-build record and matching executable hash. It captures synthetic settings, DOM, screenshots, and fixture file names. The driver session and process group are closed in a finally block. Fixture files remain available for verification.

## Production asset boundary

Native build runs cargo build --manifest-path ccr-ui/src-tauri/Cargo.toml --bin ccr-desktop --features custom-protocol. It omits .cargo/tauri-ci.toml, which substitutes ci-dist for the frontend. The configured ../dist bundle and tauri.conf.json CSP are used. The binary remains a debug build with automation enabled by the driver; this is not release-package acceptance.

Planned checks are native IPC availability, CodeMirror rendering from the synthetic Claude file, rejection of an untrusted inline script by CSP, and a UI save that preserves unknown JSON and environment fields.

These checks remain pending until a completed remaining-native-webview-N.json records results. Web mocks, static checks, and tool installation do not count as native acceptance.

## External boundaries

Read-only gh api repos/bahayonghang/ccr/actions/runners returned zero self-hosted runners. No accessible macOS host is registered for this task. Required macOS process smoke remains open. The workflow runs on pull requests; no push or PR creation is authorized for the dirty working tree.

Dedicated remaining_check dispatch returned agent thread limit reached. Existing implement_t09 provides a labeled non-author review of the current delta. That review does not change the role-dispatch result.
