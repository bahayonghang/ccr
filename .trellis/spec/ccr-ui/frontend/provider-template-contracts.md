# Provider Template Contracts

> Executable contracts for reusable non-secret provider templates in CCR UI.

---

## Scenario: Shared provider templates

### 1. Scope / Trigger
- Trigger: adding or changing provider template data, custom template persistence, template selectors, or template-to-form mappers for Claude Code, Codex, or OpenCode.
- Applies to `ccr-ui/src/types/providerTemplates.ts`, `ccr-ui/src/configs/providerTemplates.ts`, `ccr-ui/src/utils/providerTemplates.ts`, `ccr-ui/src/composables/useProviderTemplates.ts`, `ccr-ui/src/components/provider-templates/ProviderTemplateSelector.vue`, and the platform views that consume them.

### 2. Signatures
- Built-in catalog: `BUILT_IN_PROVIDER_TEMPLATES: ProviderTemplate[]`
- Custom storage key: `ccr.providerTemplates.custom.v1`
- Platform overrides: `ProviderTemplate.platforms.claude`, `ProviderTemplate.platforms.codex`, `ProviderTemplate.platforms.opencode`
- Mappers:
  - `mapTemplateToClaudeProfilePatch(template, endpoint?)`
  - `mapTemplateToClaudeLegacyConfigPatch(template, endpoint?)`
  - `mapTemplateToCodexProviderPatch(template, endpoint?)`
  - `mapTemplateToCodexApiAccountPatch(template, endpoint?)`
  - `mapTemplateToCodexProfilePatch(template, endpoint?)`
  - `mapTemplateToOpenCodeProviderPatch(template, endpoint?)`

### 3. Contracts
- Templates are global non-secret metadata. They must not store or prefill API keys, auth tokens, bearer tokens, secrets, passwords, authorization headers, or `x-api-key` style header values.
- A template is visible on a platform only when the matching `platforms.<platform>` override exists.
- Platform differences belong under `platforms.<platform>`. Do not write Claude-specific fields into Codex or OpenCode form state through shared core fields.
- Custom templates persist in `ccr.providerTemplates.custom.v1`, separate from Codex saved providers, saved accounts, and API key stores.
- Codex saved providers may include API keys, but provider templates never do. UI copy must keep "provider template" separate from "saved provider".
- Codex templates can be applied in both the saved provider form and the API key account form. The API key account mapper may only fill `providerName` and `apiBaseUrl`; the API key, save-provider checkbox, and switch-after-add checkbox remain user-controlled.
- Codex templates can be applied in the Codex Profile editor modal. The profile mapper may only fill `base_url`, `provider`, optional `provider_type`, `description`, `model`, and `suggestedName`; `auth_token`, `env_key`, `auth_mode`, and other credential fields remain user-controlled.
- OpenCode template application must preserve the OpenCode settings contract: provider ID is the map key, `npm` is a provider root field, credentials stay under the provider form, and root extras remain explicit JSON.
- Multiple endpoint templates become multiple selectable options. Selection may set a non-secret base URL, but must not modify user credential fields.

### 4. Validation & Error Matrix
- Custom override JSON is invalid or not an object -> show an editor error and do not write the custom template.
- Template or override includes sensitive keys such as `apiKey`, `authToken`, `authorization`, `x-api-key`, `secret`, or `password` -> strip them before persistence or mapper output.
- Template lacks `platforms.opencode` -> it must not appear in the OpenCode selector.
- Saving a custom template with no selected platforms -> reject in the editor.
- Applying a Codex template -> fill `name`, `baseUrl`, `websiteUrl`, and `apiKeyUrl`; leave `apiKey` untouched.
- Applying a Codex template in the API key account flow -> fill `providerName` and `apiBaseUrl`; leave `apiKey` untouched.
- Applying a Codex template in the Profile editor -> fill profile metadata and selected model state; leave `auth_token`, `env_key`, and `auth_mode` untouched.
- Applying a Codex Profile template whose model is absent from the current model catalog -> select the custom model path and seed the custom model input.
- Applying an OpenCode template -> fill `id`, `name`, `npm`, `baseURL`, and non-secret JSON fields; leave `apiKey` untouched.

### 5. Good/Base/Bad Cases
- Good: one custom "Gateway" template with `platforms.codex.baseUrl` and `platforms.opencode.baseURL` overrides.
- Good: strip `extraOptions.apiKey` and `rootExtra.headers.authorization` before storing an OpenCode custom template.
- Base: a built-in Claude-only template is only shown in Claude Code flows.
- Bad: store a custom template in Codex `model_providers.json`.
- Bad: call a Codex saved provider a "template" when it can contain API keys.
- Bad: store OpenCode `npm` under `options.npm` instead of the provider root.

### 6. Tests Required
- `cd ccr-ui && bun run test:smoke -- tests/provider-templates.smoke.test.ts`
  - Assert search indexes name, aliases, hosts, model names, and platform override fields.
  - Assert platform filtering respects `platforms.<platform>`.
  - Assert custom template persistence strips sensitive fields across all platform overrides.
  - Assert invalid override JSON does not persist.
- `cd ccr-ui && bun run test:smoke -- tests/codex-auth-view.smoke.test.ts` when adding or changing Codex account-form template entry points.
  - Assert the API key add flow can apply a Codex provider template.
  - Assert template application does not prefill or overwrite the API key.
- `cd ccr-ui && bun run test:smoke -- tests/codex-profile-editor.smoke.test.ts` when adding or changing the Codex Profile editor template entry point.
  - Assert the existing editor modal renders the `platform="codex"` selector.
  - Assert the modal forwards template selection/manual events without owning credential writes.
- `cd ccr-ui && bun run test:smoke -- tests/legacy-shells.smoke.test.ts` when replacing legacy selector/card surfaces.
- `cd ccr-ui && bun run type-check`
- `cd ccr-ui && bun run lint`

### 7. Wrong vs Correct

#### Wrong
```typescript
const template = {
  id: 'gateway',
  apiKey: providerForm.apiKey,
  platforms: {
    codex: { baseUrl: providerForm.baseUrl },
  },
}
```

#### Correct
```typescript
const template = createCustomProviderTemplateFromDraft(draft, ['codex', 'opencode'], {
  name: 'Gateway',
  category: 'third_party',
  platformOverrides: {
    codex: { baseUrl: 'https://codex.gateway.example.com/v1' },
    opencode: {
      id: 'openai',
      npm: '@ai-sdk/openai-compatible',
      baseURL: 'https://opencode.gateway.example.com/v1',
    },
  },
})
```
