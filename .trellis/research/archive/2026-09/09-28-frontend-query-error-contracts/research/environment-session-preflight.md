# T09 environment session preflight

2026-09-28. Read-only source preparation while T03 finishes its prerequisite. No implementation or acceptance is implied.

- `ccr-ui/src/api/runtime/environment.ts` already exports `getCurrentEnvironment` and `switchEnvironment`; use the existing runtime API.
- `ccr-ui/src-tauri/src/commands/environment.rs` emits `env:changed` after a successful switch with `env_id`, `env_type`, and `status`.
- `ccr-ui/src/shell/eventBridge.ts` currently invalidates all Query keys on `env:changed` and `env:refresh-requested`. T07 owns the command-event edits in this file until freeze. Coordinate any environment-event change; do not replace command handling.
- Claude/Codex profiles and AgentSessions already use the Query key `['current-environment']`. The local-only dashboard spec identifies `getCurrentEnvironment()` as the environment identity owner. Avoid a second persistent environment store.
- `SettingsSource.tsx` currently uses the shared key `['settings-source-environment']`; raw editor sessions need the same environment boundary as forms. Preserve T08 source editing, CAS, managed-lock and unsupported-clear behavior.
- `BaseSettings.tsx` unconditionally resets on `valuesQuery.data`, and saves against the latest Query snapshot. T09 must separate the acknowledged edit baseline from refreshed server data; keeping only dirty field values does not establish a safe baseline.
- `grokAuthConfig.load` currently converts backend `unsupported_environment` into `{ loggedIn: false, canAuthOff: false }`. Fix the domain result so BaseAuth can show the authoritative unsupported state after a successful local probe.
- The environment-aware UI must retain dirty input during pending/error transitions, reject stale async completions and prevent writes when the edit session no longer matches the active environment. Use controlled promise tests; do not exercise real profiles.

Required evidence remains the approved PRD AC1-AC3 and independent review. This preparation does not alter the existing Insights task or event ownership.
