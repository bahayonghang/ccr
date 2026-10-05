# T09 independent environment owner review

Date: 2026-09-28. Scope: source review before the frontend implementation freeze. No real user configuration or remote environment was accessed. Source locations describe the version reviewed before the backend repair.

## Confirmed write-target ownership

| Surface | Backend owner and target | Environment contract at review |
| --- | --- | --- |
| Claude typed Settings | `commands/claude_settings.rs:5-25` delegates to `commands/claude.rs:37-160`; target is an `ExecutionEnvironment` | Get and update select the current environment. Update accepts no expected environment ID. The remote path selects current environment separately for branch selection, read, and write. |
| Codex typed Settings | `commands/codex_settings.rs:5-34` calls `codex_config_path`; `commands/codex.rs:336` resolves local platform paths | Reads and writes fixed local config. No remote target selection or environment guard. |
| OpenCode typed Settings | `commands/opencode.rs:757-797`; paths at `:122-151` use `dirs::home_dir()/.config/opencode` | Reads and writes fixed local runtime and TUI files. No remote target selection or environment guard. |
| Claude/Codex raw Settings | `commands/settings_raw.rs:281-338`, Local guard `:29-49` | Checks current environment is Local, then captures a fixed local path. Existing CAS token and guarded writer remain authoritative. |
| Grok typed and raw Settings | `commands/grok.rs:1424-1487` | Checks non-local status, then creates a fixed local Grok platform/path. Does not redirect the write into WSL/SSH. |

All backend paths in the table are relative to `ccr-ui/src-tauri/src/`.

## Confirmed Claude mutation gap

`claude_update_settings(state, settings)` contains no expected environment identity. `update_settings` first selects an active environment to choose the Local path (`claude.rs:137-140`). Its remote branch calls `load_settings(state)` (`:142`), whose read selects the active environment again (`:69-71`, `:118-120`). After an awaited remote read, `save_settings(state)` (`:144`) selects the active environment a third time through `write_active_claude_settings_raw` (`:95-100`, `:123-129`).

`switch_environment` changes the registry under its write lock and releases that lock before publishing the environment event (`environment.rs:44-79`). The remote settings operation does not hold one selected target across its read and write. A registry switch between these awaits can therefore combine environment A data with a write to environment B. Frontend preflight alone also leaves a window between the identity response and mutation admission. These are source-confirmed control-flow gaps; no native or real remote reproduction was performed during this early review.

Recommended minimum contract: accept the acknowledged environment ID on the affected Claude read/mutation path; compare the expected ID while selecting the backend target; capture one `Arc<dyn ExecutionEnvironment>`; reuse that exact target for all reads/writes in the operation. Preserve the local atomic update owner. Add controlled dual-environment tests for mismatch-before-read and switch-between-read-and-write.

The root accepted this finding and assigned backend work to `implement_t09_backend`, with frontend contract plumbing assigned to `implement_t09`. Final acceptance requires their frozen source and executed regressions.

## Fixed local capabilities

Codex and OpenCode typed Settings cannot currently claim remote support. Giving their Query entries a WSL/SSH identity does not change the fixed local backend path. The root selected explicit Local-only frontend capability/probe configuration for these two surfaces. The raw and Grok paths retain their existing Local-only behavior. This review does not claim that the existing local gates form a global transaction with environment switching.

## Frontend checks still required

- Freeze old-session form and raw writes while environment identity is invalidated, pending, failed, or changed.
- Keep dirty values and the acknowledged baseline separate from refreshed Query snapshots.
- Reject delayed reads/writes from an obsolete edit session; test return-to-origin and explicit discard.
- Retain raw editor content when an environment probe refresh fails or becomes unsupported.
- Do not describe mocked UI IPC success as native filesystem or remote acceptance.
