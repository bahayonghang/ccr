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

`rootPath` values match the live catalog: `/claude-code`, `/codex`, `/grok`, `/opencode`, `/antigravity`. Paths are not generated from this list in this task; `routeCatalog` still emits 75 records.

Per-surface config: one module per surface, one export per platform. Config objects have `cacheKey`, `i18nPrefix`, `features`, `load`/`save` or list CRUD. They do **not** have a `platform: 'codex'` identifier field.

### 3. Contracts

- Descriptor declares which surfaces a platform has. Per-surface config declares how that surface behaves.
- Base components live under `src/features/platform/`. Thin shells live under `src/features/<domain>/` and pass one config object.
- Base components must not branch on platform name literals (`claude` / `codex` / `grok` / `opencode` / `gemini` / `antigravity` / `claude-code`). Differences go through optional config fields, `features` flags, or props such as `hideChrome`.
- Auth is partially unified: `BaseAuth` covers session status, refresh, auth-off, local-only, confirm-off. Claude OAuth account snapshots and Codex OAuth/providers/quotas stay in the platform view tasks.
- MCP manager panels live in `features/platform/mcp/` and are re-exported from `features/mcp` so `features/*` can import `features/platform` only.
- Profiles shared components stay in `src/components/profiles/`. `features/platform/profiles/shared.ts` is the documented re-export (boundaries exemption).
- `AgentDetailView` and `SystemPromptsView` stay with `08-22-views-secondary-platforms`.

### 4. Validation & Error Matrix

| Condition | Required result |
| --- | --- |
| Base file compares a platform name literal | ESLint `app/platform-unify-no-platform-branch` error |
| Settings save | Single helper `saveSettingsValues` in `settings-model.ts` |
| Grok settings dirty save | `dirtyPatch` feature + `buildGrokSettingsPatch`; invalid auto-compact blocks save |
| Non-local environment on a `localOnly` surface | `runtime-unavailable` via `probeLocalEnvironment` |
| `flattenCatalog()` | 75 paths, same as `route-inventory.md` |

### 5. Good/Base/Bad Cases

- Good: `ClaudeSettingsView` renders `<BaseSettings config={claudeSettingsConfig} />`.
- Good: Codex MCP extra fields are `features.statsStrip` / `authInjection` / `toolScope`, not `if (cacheKey === 'mcp-codex')`.
- Base: Slash commands remain on `slashCommands.ts` + `BaseSlashCommands.vue` until that view migrates.
- Bad: `if (platform === 'codex')` inside `BaseSettings`.
- Bad: adding `platform: 'grok'` to a config object so the base can switch.

### 6. Tests Required

- `tests/platform-surface-unify.smoke.test.ts`: 75 paths, descriptor roots, no platform-name branch in Base files, thin shells ≤100 lines, `visibleSettingsFields` / `saveSettingsValues` as the single settings implementation.
- `tests/platform-base-settings.smoke.test.tsx`: one `BaseSettings` rendered with two configs.
- `bun run type-check` and `bun run lint:ci`.

### 7. Change cost

- New platform: one `platformSurfaceDescriptors` row + one export per surface module.
- Shared behavior: edit the surface Base (and `settings-model.ts` for settings).
- One platform's difference: edit that config export only.

Sibling: `layering-contracts.md` (dependency direction). This document covers the two-layer surface config only.
