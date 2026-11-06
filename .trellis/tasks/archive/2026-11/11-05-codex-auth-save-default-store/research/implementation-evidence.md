# Implementation evidence — 2026-11-05

## Root cause and fix

`CredentialStoreKind::from_config_value(None)` returned Auto although Codex CLI defaults to File. The auth service also returned Auto for missing config. CLI save rejected this before inspecting the existing auth file and returned success on errors.

- Shared default is now File; `from_toml_value` retains unsupported behavior for non-string TOML instead of collapsing them to missing.
- Auth service uses Result for read/parse/type failures; only missing config defaults to File. Explicit auto/keyring remains unsupported. Unknown strings are rejected without echoing their contents.
- All production value-conversion callers audited: auth service, runtime service, three platform paths, auth-off. No remaining `as_str` collapse at these store conversion sites.
- Save returns errors through the existing dispatcher (nonzero) without printing a second error. Invalid config/auth gets its actual diagnostic; unsupported storage no longer suggests logout. Duplicate-account overwrite suggestion remains.
- Parsing auth JSON emits line/column only; a scalar JSON string can contain a credential and must never enter an error message.
- No personal config/auth payloads read or changed by implementation; no install, commit, or push.

## Changed implementation/test files

- `crates/ccr-codex/src/models/codex_auth.rs`
- `crates/ccr-codex/src/services/codex_auth_service.rs`
- `crates/ccr-codex/src/services/codex_runtime_service.rs`
- `crates/ccr-codex/src/platforms/codex.rs`
- `crates/ccr-cli/src/application/auth_off.rs`
- `crates/ccr-cli/src/commands/codex/auth/save.rs`
- `crates/ccr/tests/commands/output_presentation.rs`

## Red → green evidence

Before production edits:

1. `cargo test -p ccr-codex --locked --lib credential_store_defaults_to_file_without_enabling_unknown_values -- --skip export_bindings`
   - FAIL, exit 1, 1 failed: expected File, got Auto.
2. `cargo test -p ccr --locked --test commands output_presentation::codex_save_file_login_without_explicit_store_preserves_runtime_and_registers_account -- --skip export_bindings`
   - FAIL, exit 1, 1 failed: actual binary did not emit successful save of khanh.

After production edits:

1. `cargo test -p ccr-codex --locked --lib -- --skip export_bindings`
   - PASS, 396 passed, 2 ignored; default regression and invalid config service refusal included.
   - This run preceded the final line/column-only auth diagnostic change; that final change was subsequently compiled/exercised by CLI tests below.
2. `cargo test -p ccr --locked --test commands output_presentation -- --skip export_bindings`
   - PASS, 25 passed, 134 filtered out.
   - Actual binary saves khanh under isolated CCR/Codex paths for missing config, omitted store, and explicit file; checks parsed saved credential equality (including extension data), registry account, unchanged runtime bytes and unchanged/missing config.
   - Rejects explicit auto/keyring, unknown/wrong-type/malformed config, unreadable config (directory), malformed/scalar auth, duplicate name and storage write failure with nonzero status; sentinel credentials absent from captured output. Force overwrite remains successful.
3. `cargo test -p ccr-cli --all-features --locked --lib -- --skip export_bindings`
   - PASS, 429 passed, 2 ignored, 24 filtered out; includes auth-off default/type regression and existing native-store safeguards.
4. `cargo fmt --all -- --check` and `git diff --check`
   - PASS.

Initial sandboxed Cargo could not write the existing mbx managed cache. `rtk bypass` established Access denied at `D:\mbx-cache\targets\v1\...`; scoped escalated cargo tests ran successfully. No new cache/toolchain configuration was introduced.

Root session owns final full `just ci` and any remaining lint/all-command gates. Focused clippy was intentionally not started to avoid redundant Cargo work with that gate. Ignored tests are not counted as passed.

## Binary and limitations

Compiled test CLI exists at `D:\mbx-cache\targets\v1\6b736f3e8608d890383d61470c216d50cd84b38db0e955508f465ce6ab4ea1b5\debug\ccr.exe`, through the workspace `target` link. Installed `~/.cargo/bin/ccr.exe` has not been replaced.

No real-account save or upstream token verification was performed. Explicit auto/keyring still requires a separately designed credential-source resolver. Full official layered configuration/enterprise policy resolution remains outside this task.
