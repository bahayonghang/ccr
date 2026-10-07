# Read-only CLI Diagnostics

## Scope and ownership

- `services::validate_service::diagnose` returns `DiagnosticReport` with typed severity/category and existing error codes.
- `DoctorService::run` returns the existing `DoctorReport`.
- Terminal commands render reports. `crates/ccr/src/main.rs` chooses the process exit status before file logging initializes.
- Platform adapters own auth-mode validation. `ConfigValidator::validate_section_with` accepts an injected domain validator without adding a CLI dependency to `ccr-config`.
- Legacy `ConfigValidator::validate_section` and `ValidateService` methods keep their public compatibility signatures. New CLI diagnostics use the typed report entry point.

## Behavioral contract

1. Validate examines explicit Claude/Codex/Grok repositories from `Platform::auth_profile_supported`. Registry order and legacy current-platform fields do not select a platform.
2. Missing optional profile/runtime files produce warnings. Corrupt and unreadable existing files produce errors. Never convert a failed read to `None` or an unconfigured state.
3. Structural auth validation is separate from activation policy. Valid subscription/session profiles require no artificial API-key fields. A disabled stored profile produces a warning; a disabled current profile produces an error.
4. An empty file or simplified profile map without a declared current marker remains inactive. Diagnostics never synthesize or repair a current marker.
5. The first error in registry -> Claude -> Codex -> Grok order chooses the validate exit code. Preserve `CcrError` values: profile format 14, IO 51, domain validation 90, missing current target 62, runtime JSON/TOML parse 40/41. Warnings alone return zero. Doctor retains failure code 1.
6. Services and command adapters do not call `process::exit`. The compatibility command returns a `Result` error for embedded callers. Binary diagnostics consume the typed report.
7. Reports and parser errors never include credential values or source lines. Fixtures use synthetic secrets. Diagnostic reads create no lock, backup, history, operation record, or log file.
8. Doctor defaults to configured Claude/Codex/Grok. Explicit choices derive from `Platform::all`. Gemini/Droid are labelled `legacy_adapter`; Qwen remains unimplemented. No new legacy writer caller is added.
9. Grok runtime checks use `inspect_activation_state`; the service does not infer real official-session validity or read `mcp_credentials.json`.
10. Runtime file reads are independent of profile presence. A missing, invalid, or disabled profile does not suppress runtime syntax/read checks. For a valid current profile, classify runtime IO/parser errors before asking the platform to resolve its operational current state.
11. Doctor current-marker conflict messages identify the selected source: Claude uses a valid file marker before a valid registry marker; Codex/Grok use registry-first diagnostic selection.
12. Validate reuses `base::resolve_file_current_profile` for Claude. A valid file candidate wins over a valid registry candidate, including a disabled candidate. A selected existing but disabled profile fails with 90; no enabled-profile fallback is permitted. A stale or conflicting explicit marker alongside a valid candidate emits `warning/invalid` without repair. Explicit candidates with no existing target fail with 62; absent markers remain inactive. Codex/Grok retain their existing recorded-intent policy.

## Scenario: ConflictChecker settings paths

### 1. Scope / Trigger

Change `managers::conflict_checker`, Doctor conflict reads, or their process fixtures.

### 2. Signatures

- `ConflictChecker::check_conflicts(&self) -> Result<ConflictReport>` retains the report DTO.
- Private `collect_env_vars(&self, platform: Platform) -> Result<IndexMap<String, String>>` reads one selected settings file.

### 3. Contracts

- Claude uses `ClaudeRuntimePaths::from_env()?.settings_file`: nonempty `CCR_SETTINGS_PATH`, then nonempty `CLAUDE_CONFIG_DIR` plus `settings.json`, then the existing system-home fallback.
- Codex uses `CodexConfigManager::resolve_codex_dir()?.join("settings.json")`: existing nonblank `CCR_CODEX_DIR`, then `CODEX_HOME`, then the existing system-home fallback. Retain the legacy `settings.json` filename.
- Gemini uses nonempty `HOME`, then nonempty `USERPROFILE`, then `dirs::home_dir()`. Append `.gemini/antigravity-cli/settings.json`.
- Keep key extraction, conflict severity, suggestions, warnings, and report serialization. Reads do not create settings files or directories.
- Windows home overrides alone cannot replace the Claude/Codex Known Folder fallback. Synthetic fixtures must select explicit platform paths as well as Gemini home.

### 4. Validation & Error Matrix

| Condition | Result |
| --- | --- |
| Selected settings file missing | Empty map; no write or fallback file read |
| Selected file unreadable | Existing IO error, collected as the existing warning |
| Selected JSON invalid | Existing JSON error, collected as the existing warning |
| Multiple synthetic models differ | Existing warning conflict and original platform values |
| Platform unsupported | Existing `PlatformNotSupported` error |

### 5. Good / Base / Bad Cases

- Good: isolate Claude/Codex with explicit paths and Gemini with a temporary home.
- Base: an ordinary invocation without overrides retains existing platform fallbacks.
- Bad: hardcode `<system home>/.claude/settings.json` while the caller selected `CCR_SETTINGS_PATH`.

### 6. Tests Required

- `cargo test -p ccr-cli --lib managers::conflict_checker -- --skip export_bindings`: path priority, empty override fallback, missing settings, unchanged synthetic conflict values, and no writes.
- Actual Doctor binary tests use synthetic settings and assert JSON, status text, stdout, exit 0/1, and unchanged file inventory.
- Default parallelism applies. Use the existing `TestHome` lock for parent-process environment tests.

### 7. Wrong vs Correct

Wrong: build the Claude settings path directly from `dirs::home_dir()`.

Correct: read `ClaudeRuntimePaths::from_env()?.settings_file` and keep the existing parsing and conflict algorithm.

## Required checks

- Actual binary fixtures with isolated child HOME/USERPROFILE/CCR_ROOT and platform paths: platform × auth mode × valid/warning/invalid/corrupt/unreadable/missing.
- Actual profile switch followed by validation for API-key and subscription/session modes.
- Disabled current, ordinary disabled, missing marker, empty file, stale marker, and registry order cases.
- Before/after inventories include file paths, bytes, modification times, and directories. On Windows, read each path with `fs::metadata(&path)` for fresh timestamps; `DirEntry::metadata()` can return cached directory timestamps. Retain directory timestamp assertions.
- Windows denied sharing and Unix denied read permission tests; missing and denied access must differ.
- Doctor Grok default/explicit routes, legacy capability labels, warning zero exit, failed nonzero exit, and secret-safe malformed documents.
- Run affected unit/binary tests, strict Clippy, formatting, and task-level aggregate checks. Record unrun platforms separately.
