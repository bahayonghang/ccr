# API Facade Boundary

> Domain-first frontend API wrappers with a frozen legacy Tauri facade.

---

## Scenario: `tauri.ts` compatibility facade freeze

### 1. Scope / Trigger
- Trigger: adding or changing frontend API wrappers under `ccr-ui/src/api/**`.
- Applies to `ccr-ui/src/api/tauri.ts`, `ccr-ui/src/api/domains/*`, and `ccr-ui/src/api/index.ts`.
- `tauri.ts` exists for legacy imports only; it is not the place for new business API wrappers.

### 2. Signatures
- Compatibility facade: `ccr-ui/src/api/tauri.ts`
- Domain modules: `ccr-ui/src/api/domains/<domain>.ts`
- Public frontend entry: `ccr-ui/src/api/index.ts`
- Guard test: `ccr-ui/tests/api/api-facade-boundary.smoke.test.ts`

### 3. Contracts
- New business API wrappers must live in `src/api/domains/*` or a generated typed client.
- `src/api/index.ts` exposes domain APIs through namespace exports such as `configApi`, `codexApi`, `syncApi`, `platformApi`, `usageApi`, and `systemApi`, or via an explicit compatibility re-export when needed.
- `src/api/tauri.ts` must keep a compatibility-only header that tells maintainers not to add new direct `invoke()` calls.
- The smoke guard strips comments before collecting `invoke()` calls, so JSDoc examples do not affect the allowlist.
- Direct `invoke()` commands in `tauri.ts` are frozen by an allowlist. Adding a command there requires a deliberate compatibility exception and a test update; the default fix is moving the wrapper to a domain module.

### 4. Validation & Error Matrix
- New direct `invoke()` in `tauri.ts` -> `api-facade-boundary.smoke.test.ts` fails.
- Missing compatibility header marker -> smoke test fails.
- New wrapper in `src/api/domains/*` and exported through `src/api/index.ts` -> accepted.
- Generated typed client added later -> must keep generated drift checks outside this manual facade guard.
- A manifest-typed command invoked from any handwritten wrapper -> smoke guard fails; route it through `src/api/generated/*`. There are no typed pilot exceptions.

### 5. Good/Base/Bad Cases
- Good: add `src/api/domains/usage.ts` wrapper and expose it through `usageApi` in `src/api/index.ts`.
- Good: call a migrated command through its registry-generated client and project the concrete result in a domain wrapper.
- Good: add a temporary explicit compatibility re-export from `index.ts` with a migration reason.
- Base: keep existing `tauri.ts` legacy wrappers unchanged while stores migrate gradually.
- Bad: add `return invoke('new_backend_command')` directly to `tauri.ts`.
- Bad: update the allowlist without documenting why the command cannot live in a domain module.

### 6. Tests Required
- `cd ccr-ui && bun run test:smoke -- tests/api/api-facade-boundary.smoke.test.ts`
- `cd ccr-ui && bun run type-check`
- `cd ccr-ui && bun run lint`
- For broad API changes, also run `cd ccr-ui && bun run test`.

### 7. Wrong vs Correct
#### Wrong
```typescript
// src/api/tauri.ts
export const newFeature = () => invoke('new_feature')
```

#### Correct
```typescript
// src/api/domains/newFeature.ts
export const newFeature = () => invoke('new_feature')

// src/api/index.ts
export * as newFeatureApi from './domains/newFeature'
```

---

## Scenario: generated invoke runtime facade

### 1. Scope / Trigger

- Trigger: adding or changing a Tauri invoke call, command confirmation policy, or generated command client.
- Applies to all `ccr-ui/src/api/**` imports of `@tauri-apps/api/core`.

### 2. Signatures

```typescript
export const invoke = <T>(
  command: string,
  args?: InvokeArgs,
  options?: InvokeOptions,
): Promise<T>
```

Only `src/api/invokeRuntime.ts` may import core `invoke`; domain wrappers and generated clients import this facade.

### 3. Contracts

- The facade looks up the command in generated `COMMAND_MANIFEST` metadata.
- For `confirmation: 'user_gesture'`, it merges `confirmationToken: desktop-confirm:<command>` into JSON arguments immediately before invoking Tauri.
- Existing arguments remain intact; the runtime token wins if a caller supplies a conflicting `confirmationToken`.
- `none` and `opaque_capability` commands pass arguments unchanged. Opaque proof remains owned by the backend-issued plan/challenge workflow.
- `ArrayBuffer`, `Uint8Array`, and array payloads cannot be augmented and are rejected locally for gesture-confirmed commands.
- This facade does not implement timeout or cancellation. Those semantics belong to the completion-aware Rust runtime/business boundary.

### 4. Validation & Error Matrix

- Direct core `invoke` import outside `invokeRuntime.ts` -> API boundary smoke test fails.
- Gesture-confirmed command with JSON args -> exact command token is injected.
- Gesture-confirmed command without args -> invoke receives an object containing only the token.
- Gesture-confirmed command with raw/binary args -> synchronous `TypeError` before Tauri invoke.
- Opaque capability command -> no synthesized gesture token; the submitted `planId` or challenge is preserved.

### 5. Good/Base/Bad Cases

- Good: a generated sync client calls the runtime facade and receives token injection from manifest policy.
- Good: install execute forwards a backend-issued `planId` unchanged.
- Base: a read-only generated client invokes with no additional payload.
- Bad: import `@tauri-apps/api/core` directly in each generated client.
- Bad: add Promise timeout races in the frontend and claim backend cancellation.

### 6. Tests Required

- `cd ccr-ui && bun run test:smoke -- tests/api/command-runtime-policy.smoke.test.ts tests/api/api-facade-boundary.smoke.test.ts`.
- `cd ccr-ui && bun run type-check`.
- Search `src/api/**` and assert `invokeRuntime.ts` is the only core invoke import.
- For broad generated-client changes, run `just frontend-check`.

### 7. Wrong vs Correct

#### Wrong

```typescript
import { invoke } from '@tauri-apps/api/core'
await invoke('sync_push', { force: true })
```

#### Correct

```typescript
import { invoke } from '@/api/invokeRuntime'
await invoke('sync_push', { force: true })
```

---

## Scenario: OpenCode settings map editors

### 1. Scope / Trigger
- Trigger: editing `ccr-ui/src/api/domains/opencode.ts` or a UI editor that writes OpenCode settings map fields such as `provider`, `mcp`, or `plugin`.
- Applies to OpenCode provider IDs, display names, root config fields, and arbitrary official schema extensions.

### 2. Signatures
- Provider list: `listOpenCodeProviders<T>(): Promise<T>`
- Provider write: `addOpenCodeProvider<T>(providerId: string, config: unknown): Promise<T>`
- Provider update: `updateOpenCodeProvider<T>(providerId: string, config: unknown): Promise<T>`
- OpenCode provider config shape: `OpenCodeProviderConfig` / `OpenCodeProviderRequest` in `ccr-ui/src/types/opencode.ts`

### 3. Contracts
- The OpenCode provider ID is the key under `settings.provider.<id>`. Do not derive that key from `config.name`; `name` is only the display name stored inside the provider object.
- For custom OpenAI-compatible providers, write `npm: '@ai-sdk/openai-compatible'` at the provider root. Keep credentials and endpoints under `options.apiKey` and `options.baseURL`.
- Editors must preserve unknown provider root fields such as `api`, `env`, `whitelist`, and `blacklist`. If the editor exposes a root-extra JSON field, saving should merge those root extras before managed fields.
- Model configs must allow official model-level fields: `limit`, `options`, `headers`, `variants`, and `provider` overrides.

### 4. Validation & Error Matrix
- Passing a single object with both `id` and `name` to `resolveNameAndConfig` -> display name can become the provider map key when `name` is present.
- Saving only `options` and `models` -> root fields like `npm` are dropped, breaking custom provider loading.
- Editing an existing provider without preserving root extras -> official fields not surfaced in the form are lost.

### 5. Good/Base/Bad Cases
- Good: `addOpenCodeProvider('openai', { name: 'OpenAI Compatible', npm: '@ai-sdk/openai-compatible', options, models })`
- Base: built-in providers can omit `npm`; their config still uses the provider ID key explicitly.
- Bad: `addOpenCodeProvider({ id: 'openai', name: 'OpenAI Compatible', options, models })` when the intent is to save under `provider.openai`.
- Bad: store `npm` inside `options` instead of at the provider root.

### 6. Tests Required
- Add a focused smoke test that creates an OpenAI-compatible provider and asserts `addOpenCodeProvider` receives the explicit provider ID plus a config object containing root-level `npm`.
- Add or update edit coverage when root extras are preserved across a save.
- Run `cd ccr-ui && bun run type-check` and the focused smoke test for the touched editor.

### 7. Wrong vs Correct
#### Wrong
```typescript
await addOpenCodeProvider({
  id: 'openai',
  name: 'OpenAI Compatible',
  options: { npm: '@ai-sdk/openai-compatible' },
})
```

#### Correct
```typescript
await addOpenCodeProvider('openai', {
  name: 'OpenAI Compatible',
  npm: '@ai-sdk/openai-compatible',
  options: {
    baseURL: 'https://api.example.com/v1',
    apiKey: '{env:OPENAI_API_KEY}',
  },
})
```

## Scenario: config mutation outcomes and versioned edit drafts

### 1. Scope / Trigger

- Trigger: changing `api/domains/config.ts`, `features/configs/**`, generic config actions, or edit draft persistence.
- The generic page uses explicit `claude` requests. Dedicated platform profile pages retain their own APIs.

### 2. Signatures

```typescript
enableConfig(platform: ConfigPlatform, name: string): Promise<ConfigMutationResult>
disableConfig(platform: ConfigPlatform, name: string): Promise<ConfigMutationResult>
getConfig(platform: ConfigPlatform, name: string): Promise<ConfigInfo | null>

type ConfigEditDraft = {
  values: ConfigFormValues
  baseline: ConfigFormValues
  version: string
}
readConfigEditDraft(value: unknown, name: string): ConfigEditDraft | null
toConfigPatch(values: ConfigFormValues, baseline?: ConfigFormValues): ConfigPatchInput
```

The domain delegates to generated clients. `tauri.ts` re-exports compatibility names. `enableConfig` sends `enable: true`; disable sends a typed patch with `enabled: false`.

### 3. Contracts

- Visible disabled rows expose Enable through `ConfigsView -> ConfigList -> ConfigCard`. The view obtains warning confirmation before `handleEnable`. A historically current but disabled row must remain enableable.
- Switch/enable refresh the query after receiving an outcome. Only a committed activation updates the current-config store. `applied_with_warning` shows warning severity; unchanged/recovery shows an error and no success toast. Never activate twice to handle a warning.
- A form baseline binds the loaded name, values, and repository version. Save sends only changed fields plus that original `expectedVersion`. Changed empty optional fields become null; unchanged fields are omitted.
- Drafts preserve `{ values, baseline, version }` together. Validate the complete schema and matching target name. A draft with no bound version is not restored as an editable snapshot.
- Reopening a draft after an external change retains the draft's old baseline and old token. A fresh read must not authorize a stale draft with a new token. CAS failure retains the draft for user review.
- Reload requires confirmation before discarding the draft. After confirmation, clear the old baseline and loaded-name capability, read the current profile, and bind the new token to that fresh baseline.
- At every load start, clear the baseline and loaded-name capability. Missing/failed reads keep Save disabled; the submit handler checks snapshot/name again so direct form submission cannot bypass the guard.
- `valuesFromConfig` sets `auth_token` to an empty string instead of copying the masked response. Add/Edit global drafts remove plaintext auth_token. New credentials stay in component form state and are excluded from restored drafts.
- Keep refresh ownership in the action hook/mutation path. Do not add an extra activation or duplicate refresh in the view after a completed mutation.

### 4. Validation & Error Matrix

| Condition | Required behavior |
| --- | --- |
| Enable action confirmed | One generated switch request with explicit Claude and enable true |
| Enable confirmation cancelled | No mutation request |
| Committed warning | Refresh, warning toast, no retry |
| Unchanged/recovery outcome | Refresh, error toast, no current-store success update |
| Draft token is stale | Send old token; backend rejects; retain draft |
| Draft lacks schema/token/name binding | Ignore invalid draft; load a fresh baseline |
| Read fails after selecting a different name | No save capability; no update even on direct submit |
| User confirms Reload | Discard draft and load a fresh baseline/token |
| Secret entered | Mutation may include the new secret; global drafts contain no plaintext secret |

### 5. Good/Base/Bad Cases

- Good: edit description at version A, close, observe external version B, reopen, save with A, and preserve the draft after CAS rejection. Confirm Reload before editing from B.
- Base: an untouched masked token is omitted from the patch and remains unchanged on disk.
- Bad: reopen an old draft, assign the latest version, then overwrite external edits without conflict.
- Bad: an enabled-only hook exists but the disabled row still dispatches ordinary switch.

### 6. Tests Required

- `tests/configs/configs-actions.smoke.test.tsx`: click actual page buttons through the domain/generated/runtime path; assert explicit platform, enable flag, confirmation cancellation, current-store updates, and warning/recovery behavior.
- `tests/configs/edit-config-draft.smoke.test.tsx`: real form edit/unmount/external update/reopen/conflict/reload/save; assert both baseline and token remain bound. Cover failed target loads, direct submit rejection, and no secret in global drafts.
- API smoke: config wrappers use registry-generated clients; no new direct core invoke or typed-command exceptions.
- Backend fixtures separately prove runtime/current/enabled consistency; mocked UI IPC success alone cannot prove backend persistence.
- Run frontend type-check, the config/API smoke set, and formal lint. Keep native WebView and visual verification as separate evidence.

### 7. Wrong vs Correct

Wrong: restoring draft values and replacing `draft.version` with the version returned by a later read.

Correct:

```typescript
const patch = toConfigPatch(draft.values, draft.baseline)
await updateConfig({
  platform: 'claude',
  name,
  data: patch,
  expectedVersion: draft.version,
})
```

Acquire a new version only when the user accepts discarding the old draft and the fresh load succeeds.
