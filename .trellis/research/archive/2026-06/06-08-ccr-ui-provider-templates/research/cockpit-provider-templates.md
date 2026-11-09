# Cockpit Provider Templates Research

## Scope

Reference repo: `ref/repo/cockpit-tools`.

Target repo area: `ccr-ui` provider setup flows for Claude Code, Codex, and OpenCode.

User constraint: do not copy Cockpit's full provider chip grid. Use a searchable dropdown/listbox style selector and allow custom user templates.

## Cockpit Implementation

### Static Template Data

`ref/repo/cockpit-tools/src/utils/codexProviderPresets.ts` defines a Codex-only static preset list:

```ts
interface CodexApiProviderPreset {
  id: string
  name: string
  baseUrls: string[]
  modelCatalog?: string[]
  website?: string
  apiKeyUrl?: string
  isOfficial?: boolean
  isPartner?: boolean
  isService?: boolean
}
```

Important behavior:

- Templates are static metadata and do not include API keys.
- A template can expose multiple `baseUrls`; the UI lets the user select the endpoint after selecting the provider.
- Helper functions normalize base URLs and resolve a preset from either id or base URL.
- A special custom id represents manual provider entry.

### Modal Fill Behavior

`ref/repo/cockpit-tools/src/components/codex/CodexModelProviderManager.tsx` keeps form state separate from templates. `handleSelectProviderPreset()`:

- updates `selectedPresetId`,
- finds the static preset,
- fills provider name, first base URL, model catalog, website, API key URL, and protocol defaults,
- clears non-template capability fields such as vision details and integration type,
- does not fill API keys.

Cockpit also supports sponsor templates from live sponsor metadata, but those are still converted into non-secret form fields before saving.

### UI Pattern To Avoid

The Cockpit modal renders all templates in an `.api-provider-chip-list`:

- Custom entry button.
- Sponsor template buttons.
- Every static preset button.
- Extra endpoint chips when a preset has multiple base URLs.

This matches the screenshot style the user rejected. It is fine as a data/behavior reference, but not as a ccr-ui interaction model.

### Saved Providers

Cockpit's saved model providers are separate from static presets. Saved providers can hold user-specific API keys and runtime wiring; static presets only help initialize the form.

## ccr-ui Current State

### Claude Code

Existing preset plumbing:

- `ccr-ui/src/types/providerPresets.ts` defines the old `ProviderPreset` and `PlatformPresets` model.
- `ccr-ui/src/configs/providerPresets/claude.ts` contains many Claude Code presets.
- `ccr-ui/src/components/configs/ProviderPresetSelector.vue` renders all presets as a button grid.
- `ccr-ui/src/components/AddConfigModal.vue` uses that selector and fills `base_url`, `model`, `small_fast_model`, `provider`, `provider_type`, and `description`.

Current active profile editing flow:

- `ccr-ui/src/views/ClaudeCodeProfilesView.vue` owns the profile modal.
- `ccr-ui/src/components/claude/ClaudeProfileEditorSections.vue` owns connection/auth/model fields.
- Relevant fill targets are `base_url`, `provider`, `provider_type`, default model fields, and description/name where appropriate.
- `auth_token` must remain user-entered.

Implication: implementation should not only preserve the old `AddConfigModal` path. The current Claude profile editor should use the same template selection capability.

### Codex

Current provider management:

- `ccr-ui/src/views/CodexAuthView.vue` manages saved provider records through `codexListModelProviders`, `codexSaveModelProvider`, and `codexDeleteModelProvider`.
- `ccr-ui/src/types/codex.ts` defines `CodexModelProviderRecord` with `api_keys`.
- `ccr-ui/src-tauri/src/commands/codex_auth.rs` persists providers through `CodexModelProviderStoreService`.
- `crates/ccr-codex/src/models/codex_model_provider.rs` and `crates/ccr-codex/src/services/codex_model_provider_store.rs` store provider records at `model_providers.json` with atomic writes, backups, and locking.

Important naming risk:

- The UI currently calls saved provider records "provider presets".
- These records can include API keys, and deleting one can delete stored API keys.
- The new template feature should use distinct terminology and storage so "template" never implies credential storage.

Recommended Codex template fill targets:

- `providerForm.name`
- `providerForm.baseUrl`
- `providerForm.websiteUrl`
- `providerForm.apiKeyUrl`

Do not fill:

- `providerForm.apiKey`
- existing stored `api_keys`

### OpenCode

Current provider presets:

- `ccr-ui/src/types/opencode.ts` defines `OPENCODE_PROVIDER_PRESETS`.
- `ccr-ui/src/views/OpenCodeProvidersView.vue` renders the presets as a "recommended presets" card list.
- Selecting a preset calls `openCreate(preset.id)` and fills `id`, `name`, and `npm`.
- Provider config is written through `ccr-ui/src/api/domains/opencode.ts` by updating `settings.provider`.

Recommended OpenCode template fill targets:

- `form.id`
- `form.name`
- `form.npm`
- `form.baseURL`
- optional `modelsJson`
- optional `extraOptionsJson` / `rootExtraJson` for non-secret provider-specific schema defaults

Do not fill:

- `form.apiKey`

### Reusable ccr-ui Patterns

Use existing interaction primitives instead of inventing a new search stack:

- `ccr-ui/src/components/codex/profiles/CommandPalette.vue` provides a proven BaseModal-backed searchable listbox with grouped results and keyboard navigation.
- `ccr-ui/src/composables/useFuzzySearch.ts` wraps Fuse.js and is already a dependency.
- `ccr-ui/src/components/common/ListSearchHeader.vue` provides compact search header styling.
- `ccr-ui/src/components/common/BaseModal.vue` is the modal primitive for focused selection flows.
- `ccr-ui/tests/codex-command-palette.smoke.test.ts` is a useful smoke-test pattern for keyboard navigation, BaseModal mounting, and scroll-lock cleanup.

## Derived Design Direction

1. Add a shared non-secret provider template domain model.
2. Keep built-in templates and user-created templates separate from saved providers/accounts/API keys.
3. Add one reusable searchable selector component that can be embedded as a dropdown/listbox or modal palette depending on available space.
4. Map the selected template into each platform's native form fields through platform-specific mapper functions.
5. Replace current button grid/card-list preset surfaces with the searchable selector.

## Risks

- The word "preset" is overloaded in Codex. New UI copy should distinguish "template" from "saved provider".
- A global template model can become too broad if it tries to model every platform option equally. Keep the shared core small and put platform-specific values under platform override objects.
- Template selection can overwrite user-entered fields. MVP should preserve secrets and only overwrite fields owned by the selected template.
- OpenCode provider schemas can be arbitrary JSON. Templates should fill only known safe keys by default and keep raw JSON overrides explicit.
