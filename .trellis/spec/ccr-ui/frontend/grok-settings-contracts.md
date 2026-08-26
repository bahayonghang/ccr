# Grok Settings Contracts

> Cross-layer contract for the Grok typed Settings form and its raw `config.toml`
> source mode. After platform unification, assertions split in two: the shared
> `BaseSettings` side and the Grok-only `grokSettingsConfig` side. The two sides
> do not overlap; together they cover every assertion in this document.

---

## Scenario A: BaseSettings side (shared Settings surface)

Verification owner: `08-22-platform-unify`.

### 1. Scope / Trigger

- Trigger: changing `src/features/platform/settings/BaseSettings.tsx`,
  `src/configs/settings-types.ts` feature flags (`localOnly` / `dirtyPatch` /
  `managedLocks` / `rawSource`), or `saveSettingsValues` in `settings-model.ts`.
- Applies to every thin Settings shell that renders
  `<BaseSettings config={...} />`, including Grok.

### 2. Signatures

```ts
features?: {
  rawSource?: boolean
  localOnly?: boolean
  dirtyPatch?: boolean
  managedLocks?: boolean
}

probe?: () => Promise<{ ok: true } | { ok: false; envType: string }>
```

Grok's thin shell:

```tsx
// src/features/grok/GrokSettingsView.tsx
return <BaseSettings config={grokSettingsConfig} />
```

### 3. Contracts

- `BaseSettings` must not branch on platform name literals. Local-only gating,
  dirty-only saves, managed-lock disablement, and raw-source chrome are all
  feature flags / optional config fields.
- A `localOnly` surface calls `config.probe` (Grok: `probeLocalEnvironment`)
  before any settings or raw-file request. A non-local result is
  `runtime-unavailable`; the view issues no Grok filesystem call after the gate.
- `dirtyPatch: true` saves only dirty keys through the config `save` helper.
  `BaseSettings` must not serialize the whole form into a document.
- `managedLocks: true` disables the locked fields in the form. The backend
  remains the enforcement boundary (`managed_locked`).
- Invalid integer-range fields (Grok auto-compact `0..100`) disable save in the
  base field control and are rejected again in the backend.

### 4. Tests Required

- `tests/platforms/platform-base-settings.smoke.test.tsx`: one `BaseSettings` rendered
  with two configs.
- `tests/platforms/platform-surface-unify.smoke.test.ts`: no platform-name branch
  in Base files; `visibleSettingsFields` / `saveSettingsValues` stay the single
  settings implementation.

---

## Scenario B: Grok side (`grokSettingsConfig` + typed / raw wrappers)

Verification owner: `08-22-views-secondary-platforms`.

### 1. Scope / Trigger

- Trigger: changing `src/features/grok/GrokSettingsView.tsx`,
  `src/configs/settings-grok.ts`, `src/utils/grokSettings.ts`, the Grok
  settings/raw domain wrappers, or the corresponding Tauri commands.
- Applies to typed form saves, Local-only behavior, managed model fields, raw
  source editing, and Grok configuration-layer notices.

### 2. Signatures

```typescript
type GrokSettingsPatch = {
  set: Record<string, string | number | boolean>
  unset: string[]
}

getGrokSettings(): Promise<GrokSettingsCommandResponse>
updateGrokSettings(patch: GrokSettingsPatchDto): Promise<GrokSettingsUpdateResponse>
getGrokConfigRaw(): Promise<RawFileGetResult>
saveGrokConfigRaw(content: string, token: string): Promise<RawFileSaveResult>
listGrokConfigLayers(): Promise<ConfigLayersResult>
```

Backend commands are `grok_get_settings`, `grok_update_settings`,
`grok_get_config_raw_text`, `grok_save_config_raw_text`, and
`grok_list_config_layers`.

`grokSettingsConfig.features` is
`{ rawSource: true, localOnly: true, dirtyPatch: true, managedLocks: true }`.

### 3. Contracts

- The typed form owns only these dotted keys: `models.default`,
  `models.default_reasoning_effort`, `ui.theme`,
  `session.auto_compact_threshold_percent`, `session.load_envrc`,
  `cli.auto_update`, `cli.channel`, `cli.show_tips`,
  `hints.new_session_worktree_mode`, and `hints.fork_worktree_mode`.
- Track dirty keys against the last successful response. Build `{ set, unset }`
  from dirty keys only; blank strings and `null` go to `unset`. Never send a
  section or reconstructed document.
- Typed writes use read/merge/versioned-CAS with at most three attempts. Each
  attempt rereads the full TOML tree and rechecks activation before applying the
  patch, preserving unknown tables and keys across concurrent edits.
- `models.default` and `models.default_reasoning_effort` are locked unless
  activation is `inactive`. The frontend disables them when
  `managed_keys_locked` is true; the backend still returns `managed_locked` for
  a bypass or activation race.
- Existing unknown enum values remain visible as a current-value option. They
  are not rewritten unless the user explicitly selects a supported value or
  unsets the key.
- All five commands are Local-only at the Tauri boundary. The view also probes
  the environment first (Base side) and issues no Grok file request for a
  non-local environment.
- Grok raw saves are verbatim, version-token guarded, secret writes with
  `BackupPolicy::None`. Pass an explicit no-backup notice to `ConfigSourcePanel`;
  the optional props must not change Claude/Codex consumers.
- Treat `managed_user`, `managed_system`, `requirements_user`, and
  `requirements_system` as policy layer IDs. Show the policy notice only when
  one of those layers exists.

### 4. Validation & Error Matrix

| Condition | Required result |
| --- | --- |
| Dirty typed save succeeds | `saved`; toast, clear dirty state through a full reload |
| Three CAS attempts conflict | `conflict`; keep user edits and show reload-only recovery |
| Managed model key is changed outside `inactive` | `managed_locked`; retain a Profiles/off recovery link |
| Environment is not Local | `unsupported_environment`; no Grok filesystem call after the gate |
| Auto-compact is non-integer or outside `0..100` | Disable save and reject again in the backend |
| Patch contains an unknown or duplicate set/unset key | Backend validation error without echoing values |
| Raw TOML is invalid | `invalid` with line/column marker; no write or backup |
| Raw token is stale | `conflict`; preserve external bytes and create no backup |

### 5. Good/Base/Bad Cases

- Good: changing only `ui.theme` sends `{ set: { 'ui.theme': 'dark' }, unset: [] }` and preserves `[mcp_servers]`, `[permission]`, and unknown keys.
- Good: an external write between read and save causes a reread/remerge, or a visible conflict after the retry budget.
- Base: missing `config.toml` renders unset defaults and the first typed or raw save creates it.
- Bad: serialize the form into `[models]`, `[ui]`, or a complete TOML document and overwrite unknown configuration.
- Bad: rely on disabled controls as the managed-lock or Local-only enforcement boundary.
- Bad: put `if (platform === 'grok')` inside `BaseSettings` instead of `grokSettingsConfig` feature flags.

### 6. Tests Required

- `tests/platforms/grok-settings-api.smoke.test.ts`: generated-client forwarding, raw unsupported normalization, tokens, invalid markers, and unknown-status rejection.
- `tests/ui/code-source-editor.smoke.test.tsx`: shared editor mount and CSP nonce; default consumers remain unchanged.
- `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml commands::grok::tests -- --test-threads=1`: whitelist/value validation, unknown-key preservation, CAS retry/conflict, activation recheck, raw invalid/stale behavior, and no-backup policy.
- Run `just tauri-bindings-check` before `just frontend-check-quick`, sequentially. The bindings command regenerates/moves `src/types/generated`; running `tsc` concurrently can observe a transient missing tree.
- For source-mode changes, also run the CodeMirror CSP smoke and the production checks from `raw-config-editor-contracts.md`.

### 7. Wrong vs Correct

#### Wrong

```typescript
await updateGrokSettings({
  models: form.models,
  ui: form.ui,
  session: form.session,
})
```

#### Correct

```typescript
const dirtyKeys = new Set<GrokSettingsKey>(['ui.theme'])
await updateGrokSettings(buildGrokSettingsPatch(form, dirtyKeys))
// => { set: { 'ui.theme': 'dark' }, unset: [] }
```
