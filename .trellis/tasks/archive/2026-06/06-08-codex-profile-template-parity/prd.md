# brainstorm: Codex profile template parity

## Goal

Understand why the Codex "Add Profile" flow opens a blank manual editor instead of offering provider templates, then define a repair task that brings Codex profile creation to parity with the Claude Code profile creation experience without changing unrelated UI or backend behavior.

## What I already know

* User screenshot shows the Codex Profiles page has an "Add Profile" button.
* Clicking "Add Profile" opens `CodexProfileEditorModal` in create mode with empty/manual fields and no visible provider template chooser.
* User expects Codex profile creation to have template-based creation similar to the Claude Code page.
* The repo has existing provider template configuration under `ccr-ui/src/configs/providerTemplates.ts`.
* Recent history includes `feat(ccr-ui): [AI] ✨ 补齐 Codex API 账号模板入口` and `feat(ccr-ui): [AI] ✨ 添加跨平台供应商模板`, so template data likely exists but is not wired into this Codex profile creation entry.
* `ccr-ui/src/views/CodexProfilesView.vue:17`, `:80`, and `:217` route every add/import action to `handleAdd`; `handleAdd` at line 587 only calls `openFormModal()`.
* `openFormModal()` in `ccr-ui/src/views/CodexProfilesView.vue:551-558` loads models, resets the form, opens the existing modal, and only loads an existing profile for edit mode. There is no template selection step.
* `CodexProfileEditorModal` props in `ccr-ui/src/components/codex/CodexProfileEditorModal.vue:590-607` only accept form/model/auth state. There are no provider-template props or events.
* The Claude Code profile editor already embeds `ProviderTemplateSelector` inside its profile modal at `ccr-ui/src/views/ClaudeCodeProfilesView.vue:325-335`, tracks `selectedProviderTemplate` and endpoint at `:456-457`, resets them at `:698-699`, and applies a selected template via `applyClaudeProviderTemplate()` at `:760-781`.
* Codex Auth already proves the Codex template component path works: `ccr-ui/src/views/CodexAuthView.vue:614-624` and `:1544-1559` use `ProviderTemplateSelector platform="codex"`, then apply non-secret fields with `mapTemplateToCodexProviderPatch` and `mapTemplateToCodexApiAccountPatch` at `:3116-3140`.
* Built-in template data includes Codex platform overrides: `ccr-ui/src/configs/providerTemplates.ts:61-150` defines `codexOverrides`; `:152-221` adds Codex-only OpenAI/Azure/local templates; `:269-272` attaches Codex overrides to shared Claude presets; `:324-328` exports the merged built-ins.
* `ccr-ui/src/utils/providerTemplates.ts:430-455` maps Codex templates only to saved-provider/API-account fields today. There is no Codex Profile-specific patch that can fill `name`, `description`, `base_url`, `provider`, `model`, or model selection state.
* Codex Profile requests require a selected model in the UI before save: `ccr-ui/src/views/CodexProfilesView.vue:630-634` validates `resolvedModelValue`, and `ccr-ui/src/types/codex.ts:100-122` defines the profile request fields.
* Existing tests are well placed for this repair: provider mapping tests in `ccr-ui/tests/provider-templates.smoke.test.ts`, modal rendering tests in `ccr-ui/tests/codex-profile-editor.smoke.test.ts`, and the API-level profile test in `ccr-ui/tests/codex-profiles-view.smoke.test.ts`.

## Assumptions (temporary)

* The intended fix is frontend-first: expose/apply existing provider templates in the Codex profile editor rather than adding a new backend API.
* Existing Claude Code profile creation already has a provider-template selection pattern that Codex should reuse or mirror.
* The repair should preserve manual Codex profile creation as a fallback.
* Template application should remain non-secret: it may fill base URL/provider/model metadata, but it must not insert API keys or auth tokens.

## Open Questions

* None.

## Requirements (evolving)

* Identify the current code path for Codex `Add Profile`.
* Compare it with the Claude Code profile creation template path.
* Define the smallest repair that makes Codex provider templates discoverable and usable during add-profile.
* Keep the task scoped to `ccr-ui` unless code evidence proves backend changes are required.
* Add Codex provider template selection to profile creation without removing manual entry.
* MVP scope is limited to embedding the template selector inside the existing Codex profile editor modal.
* Do not introduce a separate pre-selection wizard before the editor opens.
* Applying a template should fill stable non-secret fields: suggested profile name, description if available, base URL, provider name, optional provider type if available, and a usable model default.
* Applying a template should reconcile the model picker state: use an existing model catalog option when present, otherwise switch to custom model input and seed it with the template model.
* Template selection state should reset when opening a fresh form or editing an existing profile.
* Existing edit behavior should not show a misleading selected template unless the user explicitly chooses one.
* Add localized labels/helper text for the Codex profile template selector instead of hard-coded English.

## Acceptance Criteria (evolving)

* [ ] The PRD records the root cause with file-level evidence.
* [ ] The PRD identifies the affected frontend files and any test files to update.
* [ ] The implementation plan includes a verification path, likely `cd ccr-ui && bun run test` or a targeted smoke test plus `just frontend-check-quick` if scope expands.
* [ ] Manual Codex profile creation remains available.
* [ ] Codex "Add Profile" modal shows a `ProviderTemplateSelector` for `platform="codex"` near the top of the form, matching the Claude Code pattern.
* [ ] Selecting a Codex template fills base URL/provider/model fields and leaves auth token/API key blank.
* [ ] Selecting a template with a model not already in `listCodexModels()` switches the model selector to the custom model path.
* [ ] Existing Codex Auth provider/API account template flows remain unchanged.
* [ ] Smoke tests cover the new mapping and at least one modal-level template event/render path.

## Definition of Done (team quality bar)

* Tests added/updated where appropriate.
* Lint / typecheck / CI green for the affected surface.
* Docs/notes updated only if behavior changes require it.
* Rollout/rollback considered if risky.

## Out of Scope (explicit)

* Reworking the full Codex Profiles page layout.
* Adding a separate quick-create template chooser before the existing editor opens.
* Adding new provider-template data unless existing template coverage is proven insufficient during implementation.
* Changing Claude Code profile behavior except as a reference for parity.
* Changing backend profile persistence unless frontend analysis proves a contract gap.
* Storing or generating secrets from provider templates.
* Supporting deprecated `provider_env_key` creation from templates.

## Technical Notes

* Initial likely files: `ccr-ui/src/components/codex/CodexProfileEditorModal.vue`, `ccr-ui/src/utils/codexProfileEditor.ts`, `ccr-ui/src/configs/providerTemplates.ts`, and the Codex Profiles view/composables that open the modal.
* Need inspect Claude Code profile creation components to find the existing template chooser/application pattern.
* Current root cause: provider template data and Codex template selector support exist, but Codex Profiles never imports `ProviderTemplateSelector`, never stores `selectedProviderTemplate` / endpoint state, and has no `applyCodexProfileTemplate` path.
* Likely implementation files:
  * `ccr-ui/src/views/CodexProfilesView.vue`
  * `ccr-ui/src/components/codex/CodexProfileEditorModal.vue`
  * `ccr-ui/src/utils/providerTemplates.ts`
  * `ccr-ui/src/types/providerTemplates.ts`
  * `ccr-ui/src/i18n/locales/zh-CN.ts`
  * `ccr-ui/src/i18n/locales/en-US.ts`
  * `ccr-ui/tests/provider-templates.smoke.test.ts`
  * `ccr-ui/tests/codex-profile-editor.smoke.test.ts`

## Root Cause

Codex profile creation is missing the UI wiring, not the template data. The shared provider-template system already has Codex platform templates and is already used by Codex Auth, but `CodexProfilesView` opens `CodexProfileEditorModal` directly with a blank `createCodexProfileEditorForm()` and the modal has no provider-template selector.

## Technical Approach

Use the Claude Code modal pattern as the target shape, but keep the change scoped to Codex Profiles:

1. Extend provider-template mapping with a Codex Profile-specific patch.
   * Add `CodexProfileTemplatePatch` to `ccr-ui/src/types/providerTemplates.ts`.
   * Add `mapTemplateToCodexProfilePatch(template, endpoint?)` to `ccr-ui/src/utils/providerTemplates.ts`.
   * The patch should derive `base_url` from the selected endpoint / Codex override / template endpoint, `provider` from template name, `description` from a safe template description if available, `model` from Codex override `modelCatalog[0]` or template `modelCatalog[0]`, and `suggestedName` from `template.id`.
2. Wire the selector into the Codex profile modal.
   * Import and render `ProviderTemplateSelector` in `CodexProfileEditorModal.vue`, likely at the top of the scroll area before the identity section.
   * Add props for `selectedProviderTemplate`, `selectedProviderEndpoint`, and a `ProviderTemplateDraftContext`.
   * Add `select-template` and `manual-template` events, or mirror the existing `@select` / `@manual` naming used by the selector.
3. Add selection state and apply behavior in `CodexProfilesView.vue`.
   * Track `selectedProviderTemplate` and `selectedProviderEndpoint`.
   * Build `codexProfileTemplateDraft` from the current form (`name`, `base_url`, `provider`, model catalog/custom model).
   * Reset template state in `resetForm()` and when applying an existing profile to edit mode.
   * Implement `applyCodexProfileTemplate(selection)` to fill non-secret profile fields and set the model selector/custom model state correctly.
   * Leave `auth_mode` unchanged unless product decides otherwise. Recommended MVP: keep current default `no_auth` and let the user choose `openai_api_key` if they want to store a key; the template only fills provider/runtime metadata.
4. Add i18n keys under `codex.profiles` for template label/helper and optional manual helper text.
5. Add focused smoke tests.
   * Extend provider-template mapping tests to verify the Codex Profile patch maps OpenRouter or DeepSeek to non-secret profile fields and includes a model.
   * Extend Codex profile editor smoke test so the selector renders when provided and emits selection/manual events through the modal wrapper.

## Decision (ADR-lite)

**Context**: Codex Auth already has template selectors, and Claude Code Profiles already embeds templates in the existing edit modal. The missing behavior is isolated to Codex Profiles add/edit modal wiring.

**Decision**: Add the provider template selector inside the existing Codex profile editor modal. Do not introduce a separate pre-selection wizard.

**Consequences**: The fix is smaller, matches existing Claude Code UX, preserves manual creation, and avoids a second modal state machine. The trade-off is that templates are selected after the modal opens rather than before, but this is consistent with the current Claude Code page.

## Implementation Plan

* Step 1: Add Codex profile template patch type + mapper in provider template utilities.
  * Verify: `bun run test -- provider-templates.smoke.test.ts` from `ccr-ui/`.
* Step 2: Add template selector props/events to `CodexProfileEditorModal.vue` and render it above the identity section.
  * Verify: `bun run test -- codex-profile-editor.smoke.test.ts` from `ccr-ui/`.
* Step 3: Wire state/application logic in `CodexProfilesView.vue`, including model selection fallback to custom model input.
  * Verify: focused smoke tests plus `bun run type-check` from `ccr-ui/`.
* Step 4: Add i18n keys and run a narrow frontend test pass.
  * Verify: `bun run test -- provider-templates.smoke.test.ts codex-profile-editor.smoke.test.ts` and `bun run type-check`.
