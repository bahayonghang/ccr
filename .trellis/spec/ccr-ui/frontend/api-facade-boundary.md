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
- Guard test: `ccr-ui/tests/api-facade-boundary.smoke.test.ts`

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
- A manifest-typed command invoked from a handwritten wrapper outside the three frozen pilot clients -> smoke guard fails; route it through `src/api/generated/*`.

### 5. Good/Base/Bad Cases
- Good: add `src/api/domains/usage.ts` wrapper and expose it through `usageApi` in `src/api/index.ts`.
- Good: call a migrated command through its registry-generated client and project the concrete result in a domain wrapper.
- Good: add a temporary explicit compatibility re-export from `index.ts` with a migration reason.
- Base: keep existing `tauri.ts` legacy wrappers unchanged while stores migrate gradually.
- Bad: add `return invoke('new_backend_command')` directly to `tauri.ts`.
- Bad: update the allowlist without documenting why the command cannot live in a domain module.

### 6. Tests Required
- `cd ccr-ui && bun run test:smoke -- tests/api-facade-boundary.smoke.test.ts`
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
