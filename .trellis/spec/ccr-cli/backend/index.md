# ccr-cli Backend Spec Index

> CLI/application domain crate.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Backend Guidelines](./backend-guidelines.md) | CLI command boundaries, output/logging rules, errors, tests, and verification | Complete |
| [CLI Output Presentation](./cli-output-presentation.md) | Status markers, fields, suggestions, output capabilities, and machine-output compatibility | Complete |
| [Test Fixtures](./test-fixtures.md) | Process-wide env and filesystem fixtures for CLI tests | Complete |
| [CLI Diagnostics](./diagnostics-contract.md) | Typed reports, pure reads, auth rules, platform capabilities, and exit codes | Complete |
| [Profile Initialization](./profile-init.md) | Claude/Codex/Grok profile scaffolding, templates, guarded creation, and registry registration | Complete |
| [Profile Application Lifecycle](./profile-application-lifecycle.md) | Shared apply/enable/rename, active-profile guards, compensation, outcomes, and replay | Complete |
| [Grok Profile Runtime](./grok-profile-runtime.md) | Grok profile validation, runtime switching, restoration, CAS, and secret boundaries | Complete |
| [Claude Auth Runtime Diagnosis](./claude-auth-runtime.md) | Claude Code auth-source priority, confidence, ownership, secret-free diagnosis, and action warnings | Complete |
| [Profile Off Login-Prep](./profile-off-login-prep.md) | Shared Claude/Codex/Grok `profile off` login-prep cleanup, `needs_login_prep`, and backup/secret write rules | Complete |
| [Auth Off Official Logout](./auth-off.md) | Shared Claude/Codex/Grok `auth off` official-runtime logout, `needs_auth_off`, file vs native spawn, and backup deletion | Complete |
| [Clean Storage](./clean-storage.md) | `ccr clean storage` deletion set, root order, confirmation, and keeper paths | Complete |

## Pre-Development Checklist

- Read [Backend Guidelines](./backend-guidelines.md) before changing command definitions, command handlers, CLI services, CLI managers, or command output.
- Read [CLI Output Presentation](./cli-output-presentation.md) before changing human result lines, fields, progress, warnings, errors, Doctor status rendering, or command suggestions.
- Read [Test Fixtures](./test-fixtures.md) before adding tests that mutate process env or home-directory paths.
- Read [CLI Diagnostics](./diagnostics-contract.md) before changing validate, doctor, their reports, or binary status.
- Read [Profile Initialization](./profile-init.md) before changing profile init commands, embedded examples, or platform registry bootstrap.
- Read [Profile Application Lifecycle](./profile-application-lifecycle.md) before changing apply/enable/rename or guarded active-profile mutations in CLI, TUI, or desktop adapters.
- Read [Grok Profile Runtime](./grok-profile-runtime.md) before changing Grok profile validation, runtime switching, restoration, deletion, or credential display.
- Read [Claude Auth Runtime Diagnosis](./claude-auth-runtime.md) before changing Claude auth/profile switching, runtime summaries, doctor auth-source checks, or auth-source output in any client.
- Read [Profile Off Login-Prep](./profile-off-login-prep.md) before changing `profile off`, `needs_login_prep`, TUI apply/auth-switch off, or Tauri `*_profile_off`.
- Read [Auth Off Official Logout](./auth-off.md) before changing `auth off`, `needs_auth_off`, TUI Auth logout, or Tauri `*_auth_off`.
- Read [Clean Storage](./clean-storage.md) before changing `ccr clean storage` or the paths it deletes.
