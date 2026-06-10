# Fix ccr-ui OpenCode provider custom config

## Goal

Make `ccr-ui` generate and persist OpenCode provider configuration that matches the current OpenCode schema for custom OpenAI-compatible URLs, API keys, and model catalogs, so a user can configure an endpoint like the supplied example without hand-editing missing fields afterward.

## Research References

- [`research/opencode-custom-provider-config.md`](research/opencode-custom-provider-config.md) - official OpenCode providers docs and schema findings.
- https://opencode.ai/docs/providers/ - custom provider examples and troubleshooting guidance.
- https://opencode.ai/config.json - current JSON schema for `provider`, `options`, `models`, `variants`, and `agent`.

## What I Already Know

- The user wants research-backed analysis before implementation and explicitly asked to create an implementation task.
- The user supplied a real API key; it is treated as a secret and must not be persisted. All task examples use `<YOUR_API_KEY>` or environment placeholders.
- Current source-backed implementation surface is `ccr-ui/src/views/OpenCodeProvidersView.vue`, `ccr-ui/src/types/opencode.ts`, `ccr-ui/src/api/domains/opencode.ts`, and related smoke tests.
- The screenshot/modal text `Use API Key` was not found in current `ccr-ui/src`; it may be from a stale bundle, another route, or another product surface. Do not plan implementation around that phrase unless it is found in source later.
- `ccr-ui/src-tauri/src/commands/opencode.rs` reads and writes OpenCode settings as JSON and merges patches, so arbitrary provider fields such as `npm` should survive backend persistence.
- Existing dirty worktree files (`ccr-ui/src-tauri/Cargo.toml`, `icon.png`, `ccr-ui/output/playwright/*`) predate this task and must be preserved.

## Current Problems

- `OpenCodeProviderConfig` and `OpenCodeProviderRequest` do not model `npm` explicitly, although OpenCode's schema supports it.
- `OpenCodeProvidersView.vue` saves `id`, `name`, `options`, and `models`, but not a top-level `npm` field. This means OpenAI-compatible endpoints can be saved without the package OpenCode needs to load them.
- `OPENCODE_PROVIDER_PRESETS` contains an `openai-compatible` provider ID. Official OpenCode examples use arbitrary provider IDs and select the implementation with `npm: "@ai-sdk/openai-compatible"`, so the current preset conflates identity with package choice.
- The UI exposes `extra options JSON`, but that writes into `options`, not the provider root. Users cannot add `npm` through the current form without bypassing the page.

## Requirements

- Add first-class support for a root-level provider `npm` package field.
- For OpenAI-compatible provider creation, prefill `npm` with `@ai-sdk/openai-compatible`.
- Keep `options.apiKey`, `options.baseURL`, `models`, model `limit`, model `options`, and model `variants` accepted as JSON.
- Preserve arbitrary existing provider root fields during edit, including official fields not surfaced in the form.
- Keep secret handling conservative: mask displayed API keys, do not log or fixture real keys, and use placeholders in tests.
- Do not automatically write to the user's OpenCode config outside the existing explicit Provider save action.

## Acceptance Criteria

- [ ] Creating an OpenAI-compatible provider through `OpenCodeProvidersView.vue` calls `addOpenCodeProvider` with top-level `npm: "@ai-sdk/openai-compatible"`.
- [ ] Editing an existing provider with `npm` preserves and displays that package value.
- [ ] The provider request type includes `npm?: string`, and model typing continues to allow `options`, `headers`, `variants`, and per-model provider overrides.
- [ ] Existing unknown root provider fields are preserved unless the user explicitly edits/removes them through a dedicated root-extra JSON field.
- [ ] Smoke/unit coverage verifies the saved payload includes `npm`, `options.baseURL`, `options.apiKey`, and `models`, with only placeholder secrets.
- [ ] No implementation file, test, screenshot, or task artifact contains the real API key from the user prompt.

## Technical Approach

Recommended MVP: extend the existing provider editor rather than creating a separate config generator.

- In `ccr-ui/src/types/opencode.ts`, add `npm?: string` to `OpenCodeProviderConfig`, `OpenCodeProviderRequest`, and preset metadata if needed. Broaden `OpenCodeModelConfig` only as needed for official model fields.
- In `ccr-ui/src/views/OpenCodeProvidersView.vue`, add a compact `npm package` field near provider id/display name, initialize it from presets, populate it in edit mode, and include it at provider-root level when saving.
- Add a provider-root "extra JSON" field only if needed to preserve root fields cleanly; keep `extra options JSON` scoped to `options`.
- Change the OpenAI-compatible preset from "provider id = openai-compatible" semantics to a preset that can still default `id` to `openai` or `custom-openai` while setting `npm: "@ai-sdk/openai-compatible"`. Avoid breaking existing configs that already have an `openai-compatible` ID.
- Keep `ccr-ui/src/api/domains/opencode.ts` unchanged unless implementation proves it drops root fields; current analysis says it preserves arbitrary JSON.

## Decision (ADR-lite)

Context: OpenCode separates provider identity from implementation package. CCR UI currently captures credentials and models but not the provider package.

Decision: Treat `npm` as a first-class provider-root field in the existing Providers page, and auto-fill `@ai-sdk/openai-compatible` for the OpenAI-compatible preset.

Consequences: This is a small UI/type/test change, avoids a second generator flow, and keeps existing provider IDs compatible. It does not solve full schema editing for every OpenCode provider field, but root-extra preservation can cover advanced fields without overbuilding.

## Implementation Plan

1. Update TypeScript contracts.
   - Files: `ccr-ui/src/types/opencode.ts`.
   - Verify: TypeScript compile catches invalid request shapes.

2. Update Providers UI form state and save path.
   - Files: `ccr-ui/src/views/OpenCodeProvidersView.vue`.
   - Verify: saved provider request includes root-level `npm` plus nested `options`.

3. Update presets for OpenAI-compatible setup.
   - Files: `ccr-ui/src/types/opencode.ts` and any UI code consuming `OPENCODE_PROVIDER_PRESETS`.
   - Verify: preset click pre-populates the package and does not force users into a misleading provider ID.

4. Add/extend smoke tests.
   - Likely file: `ccr-ui/tests/legacy-shells.smoke.test.ts`.
   - Verify: mount providers shell, open OpenAI-compatible preset, fill placeholder baseURL/API key/models, save, assert `addOpenCodeProvider` payload.

5. Run narrow verification, then escalate if touched surface grows.
   - From `ccr-ui/`: `bun run test -- legacy-shells.smoke.test.ts`.
   - If types/components change broadly: `bun run type-check`.
   - If final delivery spans UI contracts: `bun run test`.

## Out of Scope

- Persisting the user's real API key into any config or fixture.
- Automatically modifying `~/.config/opencode/opencode.json` outside the existing explicit save flow.
- Implementing a full OpenCode schema editor.
- Fixing the screenshot's `Use API Key` modal unless its source is found in this repository during implementation.
- Changing OpenCode runtime/auth internals.

## Confirmed Scope

The user confirmed the implementation should only fix the existing Providers editor. Do not add a separate copyable `opencode.json` example panel.

## Definition of Done

- Tests added/updated where appropriate.
- Narrow frontend test passes.
- Type-check passes if TS contracts are touched.
- Verification output is recorded in the final implementation summary.
- Secret audit confirms the real user key was not persisted.
