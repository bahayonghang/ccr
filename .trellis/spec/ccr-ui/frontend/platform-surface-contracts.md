# Platform Surface Contracts

> Two-layer config for Settings / Profiles / Auth / MCP / Agents / Plugins / Commands. Nineteenth frontend contract.

---

## Scenario: cross-platform surface unification

### 1. Scope / Trigger

- Trigger: adding a platform surface, changing a shared base, or adding a per-platform config export.
- Applies to `ccr-ui/src/config/platformDescriptors.ts` (descriptor layer), `ccr-ui/src/configs/{settings,profiles,commands,mcp,agents,plugins,auth}.ts` (per-surface configs), and `ccr-ui/src/features/platform/**` (bases).
- Does not apply to `src/configs/slashCommands.ts` or `src/config/platformCapabilities.ts` (frozen references).
- Does not add IPC. Consumers keep calling existing `@/api` / `@/api/domains/*` wrappers.

### 2. Signatures

Descriptor layer:

```ts
type PlatformSurface =
  | 'settings' | 'profiles' | 'auth' | 'mcp' | 'agents' | 'plugins' | 'commands'

interface PlatformSurfaceDescriptor {
  id: string
  rootPath: string
  surfaces: readonly PlatformSurface[]
}
```

`rootPath` values match the live catalog: `/claude-code`, `/codex`, `/grok`, `/opencode`, `/antigravity`. Paths are not generated from this list in this task; `routeCatalog` still emits 76 records.

Per-surface config: one module per surface, one export per platform. Config objects have `cacheKey`, `i18nPrefix`, `features`, `load`/`save` or list CRUD. They do **not** have a `platform: 'codex'` identifier field.

### 3. Contracts

- Descriptor declares which surfaces a platform has. Per-surface config declares how that surface behaves.
- Base components live under `src/features/platform/`. Thin shells live under `src/features/<domain>/` and pass one config object.
- Base components must not branch on platform name literals (`claude` / `codex` / `grok` / `opencode` / `gemini` / `antigravity` / `claude-code`). Differences go through optional config fields, `features` flags, or props such as `hideChrome`.
- Auth is partially unified: `BaseAuth` covers session status, refresh, auth-off, local-only, confirm-off. Claude OAuth account snapshots and Codex OAuth/providers/quotas stay in the platform view tasks.
- MCP manager panels live in `features/platform/mcp/` and are re-exported from `features/mcp` so `features/*` can import `features/platform` only.
- Profiles shared components stay in `src/components/profiles/`. `features/platform/profiles/shared.ts` is the documented re-export (boundaries exemption).
- Settings load returns `SettingsSnapshot { values, source, managedLocks? }`.
  `source` preserves the typed response for the current form session.
  `saveSettingsValues(config, { values, dirtyKeys, snapshot })` returns
  `SettingsSaveResult`; platform mappers submit only changed fields. For an
  endpoint that replaces a submitted object, retain its unedited nested values
  from the snapshot. Do not mutate the snapshot or replace union values with
  boolean defaults.
- Codex Settings projects known fields on read and merges submitted leaf fields
  on save. Send only dirty leaves, including inside `tui`; do not resend snapshot
  siblings. Codex retains unknown extension fields on disk. Claude and OpenCode
  replace submitted top-level objects, so their nested patches retain snapshot
  siblings. Test fixtures must model the owning endpoint's merge behavior.
- A cleared Codex optional field sends `null` for that dirty leaf. The Codex
  backend removes the corresponding optional value. Unchanged fields stay absent
  from the patch. Do not send `undefined`, which the domain JSON converter omits.
  Codex enum and notification controls expose an explicit unset option.
- Claude and OpenCode reject a dirty top-level field whose clear operation would
  produce `undefined`. Their typed APIs have no established removal protocol.
  `SettingsValidationError` reaches the existing translated error notice, keeps
  the draft, and prevents all mutations for that save. Claude points to Source;
  OpenCode explains the API limit. Do not treat an empty string as key removal.
  Nested section replacement, empty arrays, and empty objects remain supported.
- Memoized Settings controls subscribe to their field value so form reset
  updates their registration and defaults. Test with a stable translation
  function; changing callback identities must not be required to populate inputs.
- Settings raw-source callbacks and notices belong to the per-platform config.
  The shared `features/platform/editor/` composite owns the editor, confirmation,
  version token, conflict recovery, and layer display. `features/editor/`
  re-exports the existing entry points for compatibility.
- `AgentDetailView` and `SystemPromptsView` stay with `08-22-views-secondary-platforms`.

### 4. Validation & Error Matrix

| Condition | Required result |
| --- | --- |
| Base file compares a platform name literal | ESLint `app/platform-unify-no-platform-branch` error |
| Settings save | Single helper `saveSettingsValues` in `settings-model.ts` |
| Grok settings dirty save | `dirtyPatch` feature + `buildGrokSettingsPatch`; invalid auto-compact blocks save |
| Non-local environment on a `localOnly` surface | `runtime-unavailable` via `probeLocalEnvironment` |
| `flattenCatalog()` | 76 paths, same as `route-inventory.md` |

### 5. Good/Base/Bad Cases

- Good: `ClaudeSettingsView` renders `<BaseSettings config={claudeSettingsConfig} />`.
- Good: Codex MCP extra fields are `features.statsStrip` / `authInjection` / `toolScope`, not `if (cacheKey === 'mcp-codex')`.
- Base: Slash commands use `slashCommands.ts` + `src/features/commands/BaseSlashCommands.tsx`.
- Bad: `if (platform === 'codex')` inside `BaseSettings`.
- Bad: adding `platform: 'grok'` to a config object so the base can switch.

### 6. Tests Required

- `tests/platforms/platform-surface-unify.smoke.test.ts`: 76 paths, descriptor roots, no platform-name branch in Base files, thin shells ≤100 lines, `visibleSettingsFields` / `saveSettingsValues` as the single settings implementation.
- `tests/platforms/platform-base-settings.smoke.test.tsx`: one `BaseSettings` rendered with two configs.
- `tests/platforms/settings-lossless.smoke.test.ts`: real mappers and domain
  transport, one-field saves, nested extensions, union values, and no-op saves.
- Codex model-only persistence requires a temporary-file save and reread through
  the production `update_codex_settings_at_path` helper. Assert the new model,
  every notification array entry, and semantic equality of all non-model TOML
  fields. Read/projection or in-memory merge tests alone do not cover disk writes.
  An explicit temporary path avoids Windows KnownFolder home resolution; this
  helper test does not establish `State`, IPC admission, cache invalidation, or
  native WebView behavior.
- `tests/platforms/settings-visible-i18n.smoke.test.tsx`: mount the real Settings
  view and translator, switch Chinese/English within one loaded session, and
  assert visible field labels, save actions, enum labels, and preserved unknown values.
- `tests/platforms/settings-capabilities.smoke.test.tsx`: mounted Settings routes,
  managed locks, current unknown options, source-mode success/conflict/invalid,
  Local-only gates, plaintext confirmation, and Grok policy/no-backup notices.
- `bun run type-check` and `bun run lint:ci`.

### 7. Change cost

- New platform: one `platformSurfaceDescriptors` row + one export per surface module.
- Shared behavior: edit the surface Base (and `settings-model.ts` for settings).
- One platform's difference: edit that config export only.

Sibling: `layering-contracts.md` (dependency direction). This document covers the two-layer surface config only.

## Scenario: Auth recovery and Settings edit sessions

### Scope and signatures

- Applies to `BaseAuth`, `BaseSettings`, `useSettingsSession`, Settings adapters, and environment event invalidation.
- `AuthSessionState = AuthSession | { status: 'unsupported_environment' }`.
- `SettingsConfig.load(context?: { environmentId: string })` returns the existing snapshot. `SettingsSaveInput.environmentId` is optional for compatibility; the mounted editor always sends its acknowledged id.
- Reuse `current-environment` Query as identity owner. Typed Settings keys include surface, environment id, and type. A component-local session retains `{ environment, snapshot }`; React Hook Form owns current values and dirty fields.

### Contracts

- Auth pending, probe/load errors, signed-out, signed-in, and unsupported are distinct. Backend unsupported remains unsupported after a successful local probe. Errors take precedence over an old unsupported result.
- Failed refresh may retain confirmed data only with stale/error feedback. Off failure preserves the session. Claim off synchronously before confirmation so duplicate clicks cannot open another confirmation or dispatch another request.
- Probe failure disables loading. Retry invalidates the session cache with `refetchType: 'none'` before retrying the probe; recovery then reloads once even with the production 30-second stale time.
- Server Query data is separate from the editable baseline. Background refetch never resets an established editor. A changed snapshot retains draft and baseline, displays a notice, and blocks saving until confirmed discard and a successful fresh load.
- Compare snapshot contents rather than object identity. Cache GC and an equivalent response must not create a false conflict.
- Codex, Grok, and OpenCode typed Settings are Local-only. Claude typed Settings retains existing environment service support. All raw Settings remain Local-only. Capability checks precede platform reads.
- Environment pending/error/change retains form and raw drafts and disables writes. Offer Return to draft environment and confirmed Discard and reload. Return waits for a successful idle probe/read before enabling Save; direct submission enforces the same guard.
- The shell invalidates identity and cancels environment-bound Settings reads first, then invalidates other domains after identity refresh. Keep the existing listener/cleanup owner.
- Read/save completions must match the session generation and environment Query revision. A late A response cannot replace a new session after A to B to A.
- Claude typed read/write sends `expectedEnvironmentId` through generated clients. The backend matches at admission and captures one execution environment for the full operation. Frontend checks alone cannot close the IPC admission race.
- Save failure retains draft/baseline. Successful mutation followed by failed reload blocks another save and offers reload without replay. Initial load, confirmed discard, and successful save are the only baseline binding paths.
- Preserve dirty-leaf/null, unsupported-clear, managed-lock, raw-editor and confirmation contracts. Do not put source snapshots or raw content into persistent storage, logs, or route state.

### Regression matrix

| Condition | Required evidence |
| --- | --- |
| Auth initial errors / cached probe recovery | Correct Retry and one session load; no signed-out claim |
| Dirty form plus refetch | Original input and baseline retained; confirmed reload |
| Unknown environment / return after GC | No mutation until successful idle target snapshot |
| Late A read or save after A to B to A | No reset, success notice, or newer-cache replacement |
| Saved mutation / failed follow-up read | No mutation replay; reload recovery |
| Local-only Settings in WSL or SSH | Unsupported; no platform load/save |

### Tests

- `query-contract-baseline.smoke.test.tsx`, `auth-query-contract.smoke.test.tsx`, and independent `auth-probe-recovery-check.smoke.test.tsx` cover initial failures, real Grok adapter, off pending/confirmation, and production cache recovery.
- `settings-session.smoke.test.tsx` and independent `settings-return-cache-check.smoke.test.tsx` use controlled promises for the real shell event, environment revisions, cache GC, direct submit, and reset consent.
- `settings-capabilities.smoke.test.tsx` exercises real platform routes, generated Claude arguments, Local-only scope, and retained T08 capabilities.
- Run platform/config/shell/API/editor tests, type-check, formal lint, i18n, cycle, and architecture checks. Report unrelated formal failures separately.
