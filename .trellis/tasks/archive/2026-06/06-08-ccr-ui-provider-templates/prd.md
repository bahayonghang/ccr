# ccr-ui Provider Templates

## Goal

为 `ccr-ui` 中 Claude Code、Codex、OpenCode 的供应商配置流程增加可复用的非敏感供应商模板。用户可以通过下拉/列表选择器和关键词筛选快速选中模板，一次性填充供应商名称、Base URL、官网、API key 文档、模型默认值等字段，同时仍能手动填写并自定义保存模板。实现不能照搬 `cockpit-tools` 的全量按钮平铺界面。

## Status

* Trellis task: `.trellis/tasks/06-08-ccr-ui-provider-templates`
* Task status: `in_progress`
* Current phase: implementation and quality verification. User confirmed execution with "请开始实施".

## Research References

* [`research/cockpit-provider-templates.md`](research/cockpit-provider-templates.md) - Cockpit implementation notes and ccr-ui mapping analysis.

## What I Already Know

### User Intent

* 需要先分析 `ref/repo/cockpit-tools` 的“新增模型供应商”实现。
* 目标范围是 `ccr-ui` 的 Claude Code、Codex、OpenCode 供应商配置入口。
* 用户明确不想使用截图里的“所有供应商按钮全部铺开”交互。
* 期望交互是下拉列表、关键词筛选，并支持用户自定义添加模板。

### Cockpit Findings

* Cockpit 的 `CodexApiProviderPreset` 是静态非密钥模板，字段包括 `id`、`name`、`baseUrls[]`、`modelCatalog`、`website`、`apiKeyUrl` 和展示/分类标记。
* 选择模板只填表：provider name、base URL、模型目录、官网、API key 文档、协议默认值等。
* API key 不属于模板，由用户另外输入或保存在 saved provider/account 中。
* UI 使用 `.api-provider-chip-list` 把所有模板按钮直接渲染出来，这是本任务明确要避免的模式。

### ccr-ui Current State

* Claude Code 已有旧模板数据和按钮网格：
  * `ccr-ui/src/types/providerPresets.ts`
  * `ccr-ui/src/configs/providerPresets/claude.ts`
  * `ccr-ui/src/components/configs/ProviderPresetSelector.vue`
  * `ccr-ui/src/components/AddConfigModal.vue`
* Claude Code 当前 profile 编辑主路径在：
  * `ccr-ui/src/views/ClaudeCodeProfilesView.vue`
  * `ccr-ui/src/components/claude/ClaudeProfileEditorSections.vue`
* Codex 当前把 saved model provider 称为 provider preset，但它可以包含 API keys：
  * `ccr-ui/src/views/CodexAuthView.vue`
  * `ccr-ui/src/types/codex.ts`
  * `ccr-ui/src-tauri/src/commands/codex_auth.rs`
  * `crates/ccr-codex/src/models/codex_model_provider.rs`
  * `crates/ccr-codex/src/services/codex_model_provider_store.rs`
* OpenCode 当前有少量内置推荐项：
  * `ccr-ui/src/types/opencode.ts`
  * `ccr-ui/src/views/OpenCodeProvidersView.vue`
  * `ccr-ui/src/api/domains/opencode.ts`
* 可复用的搜索/弹层模式已经存在：
  * `ccr-ui/src/components/codex/profiles/CommandPalette.vue`
  * `ccr-ui/src/composables/useFuzzySearch.ts`
  * `ccr-ui/src/components/common/ListSearchHeader.vue`
  * `ccr-ui/src/components/common/BaseModal.vue`

## Assumptions

* 模板是非敏感配置资产，不保存、不预填 API key、Auth token、OAuth token 或任何账号凭据。
* 选择模板会覆盖该平台的模板托管字段，但保留密钥字段和不属于模板的用户输入。
* 自定义模板需要本地持久化，并与 saved providers / accounts / API keys 分离。
* Claude Code、Codex、OpenCode 中同名供应商大量重叠，模板数据应尽量复用，但平台字段映射必须保持显式。

## Requirements

### Template Domain Model

* 增加共享的 provider template 类型，核心字段建议包括：
  * `id`
  * `name`
  * `aliases`
  * `category`
  * `websiteUrl`
  * `apiKeyUrl`
  * `tags`
  * `baseUrls`
  * `modelCatalog`
  * `isOfficial`
  * `isPartner`
  * `platforms`
* `platforms` 下保存平台特定映射，例如：
  * `claude`: `base_url`、`provider`、`provider_type`、默认模型字段。
  * `codex`: `baseUrl`、`websiteUrl`、`apiKeyUrl`、可选 model catalog / protocol metadata。
  * `opencode`: `id`、`name`、`npm`、`baseURL`、可选 `models` 和非敏感 JSON defaults。
* 内置模板和用户自定义模板应能合并成同一个选择列表，但要在 UI 上区分来源。
* 自定义模板不允许包含密钥字段。
* 自定义模板采用全局共享模型：一个模板可以声明适用于 Claude Code、Codex、OpenCode 中的一个或多个平台。
* 平台差异通过 `platforms.<platform>` override 表达；没有对应 override 的模板不应出现在该平台的选择结果中，除非用户明确选择手动/自定义。

### UX

* 替换现有全量按钮网格/推荐卡片列表为紧凑的 searchable selector。
* Selector 必须支持关键词筛选，至少覆盖：
  * provider name
  * aliases
  * category / tags
  * base URL / host
  * model names
  * website host
* Selector 应支持键盘导航、空状态、当前选择状态和清晰的 "Custom / Manual" 入口。
* 长列表不应在页面主体直接全量展开；可使用 dropdown popover 或 BaseModal command-palette style selector。
* 多 endpoint 模板要允许用户选择 endpoint，不应静默隐藏其他 `baseUrls`。
* 选择模板后显示被填充字段的结果或摘要，避免用户不知道哪些字段被改动。

### Platform Integration

* Claude Code:
  * 在当前 profile 编辑流中提供模板选择能力。
  * 旧 `AddConfigModal` 如仍可达，也应复用同一 selector 和 mapper，避免保留旧按钮网格。
  * 可填字段包括 `base_url`、`provider`、`provider_type`、默认模型字段和描述/name 建议。
  * 不填 `auth_token`。
* Codex:
  * 在 provider create/edit form 中提供模板选择能力。
  * 新模板概念必须和当前 saved model providers 区分，避免继续把可含 API keys 的记录叫作模板。
  * 可填字段包括 `name`、`baseUrl`、`websiteUrl`、`apiKeyUrl`。
  * 不填 `apiKey`，不写入 `api_keys`。
* OpenCode:
  * 在 provider create form 中提供模板选择能力，替换当前推荐预设卡片。
  * 可填字段包括 `id`、`name`、`npm`、`baseURL`、非敏感 `modelsJson`、非敏感 `extraOptionsJson` / `rootExtraJson`。
  * 不填 `apiKey`。

### Custom Templates

* 用户可以从空白表单创建自定义模板。
* 用户可以从当前已填表单保存为模板，但必须排除密钥字段。
* 用户可以编辑和删除自定义模板。
* 删除自定义模板不影响已经保存的 profiles/providers/accounts。
* 内置模板不可被直接删除；如需要调整，用户可以复制成自定义模板。
* 自定义模板是全局共享资产；用户保存模板时可以选择适用平台，并为每个平台保存独立 override。
* 同一个全局模板在不同平台的字段不完全一致时，以平台 override 为准，不把某个平台的字段强行写入其他平台。

## Acceptance Criteria

* [x] PRD 和 research 文件说明 Cockpit 的模板结构、填表逻辑、保存模型和 UI 模式。
* [x] 新的 provider template 类型不包含 API key/Auth token 等敏感字段。
* [x] Claude Code、Codex、OpenCode 的模板选择入口都使用 searchable selector，不再把所有供应商按钮/卡片全量铺开。
* [x] 关键词筛选可以命中名称、别名、分类、base URL、模型名和官网 host。
* [x] 选择模板会填充平台对应的非敏感字段，并保留密钥字段为空或原值。
* [x] 自定义模板可以添加、编辑、删除，并能在后续创建供应商时复用。
* [x] 自定义模板按全局模板 + 平台适用性/override 工作，同一个模板可服务多个平台。
* [x] Codex 的 saved provider/API key store 与 provider template store 保持分离。
* [x] 相关 smoke/unit tests 覆盖筛选、键盘选择、平台字段映射和密钥不被预填。

## Definition of Done

* Tests added/updated for mapping, filtering, selector interaction, and custom template persistence.
* `cd ccr-ui && bun run type-check` passes.
* `cd ccr-ui && bun run test:smoke` passes, or targeted smoke tests pass first before escalating.
* For UI implementation, web preview is checked with `bun run dev:web -- --host 127.0.0.1 --strictPort` and Browser at `http://127.0.0.1:5173/`.
* Docs/notes updated if terminology changes from "provider preset" to "provider template".
* Rollout/rollback considered for any new persisted custom template file/store.

## Implementation Verification

* `cd ccr-ui && bun run test:smoke -- tests/provider-templates.smoke.test.ts` - passed, 8 tests.
* `cd ccr-ui && bun run test:smoke -- tests/legacy-shells.smoke.test.ts` - passed, 11 tests.
* `cd ccr-ui && bun run type-check` - passed.
* `cd ccr-ui && bun run lint` - passed.
* `git diff --check` - passed with only LF/CRLF warnings from Git on existing tracked files.
* Web preview at `http://127.0.0.1:5173/codex/auth` with system Chrome:
  * Opened the Codex Model providers tab and the provider template selector.
  * Opened "Save current" custom editor and enabled OpenCode platform applicability.
  * Confirmed Codex and OpenCode override JSON textareas are visible.
  * Saved a global custom template with explicit Codex/OpenCode overrides.
  * Confirmed localStorage stored the overrides and stripped the test API key value.
  * Captured screenshots in `ccr-ui/output/playwright/`.
* Web preview limitation: Codex data loading logs Tauri `invoke()` errors in plain browser mode; template UI interaction still works and this matches the ccr-ui web-preview boundary.

## Follow-up: Codex API Account Template Entry

Screenshot review found a remaining UX gap after the initial implementation:

* Claude Code profile creation shows `ProviderTemplateSelector` inside the profile modal.
* Codex templates are available in the `Model providers` tab, but the `Add account -> API Key` modal only shows `Saved providers`.
* This makes Codex look like it has no provider templates when users create an API key account directly.

Implementation target:

* Add the Codex provider template selector to the API key account form.
* Keep template state separate from the saved provider form state.
* Applying a template fills only non-secret API account fields:
  * provider name
  * API base URL
* Applying a template must not set, clear, or overwrite the API key, saved-provider checkbox, or switch-after-add checkbox.
* Keep the right-side `Saved providers` list because it intentionally represents saved provider records that may include API keys.

Verification target:

* Extend Codex auth smoke coverage to open `Add account -> API Key`, apply a Codex template, and assert the API add payload includes the selected provider/base URL while preserving the manually entered API key.
* Keep provider template mapper coverage proving no secret fields are emitted.

## Technical Approach

Recommended approach: shared template catalog + platform-specific mappers.

* Add a shared template module under `ccr-ui/src/types/` and `ccr-ui/src/configs/`.
* Convert existing Claude presets and OpenCode presets into the shared model.
* Add Codex built-in templates from Cockpit-like metadata, adapted to ccr-ui field names and terminology.
* Build one reusable `ProviderTemplateSelector` with search, grouping, keyboard navigation, custom/manual option, and selected-template summary.
* Add mapper functions per platform so forms stay decoupled from template internals.
* Add a dedicated custom-template persistence layer separate from Codex saved provider storage.

Alternative considered: keep three independent platform-local preset systems.

* Pros: smaller first patch per page.
* Cons: duplicates provider metadata, repeats search UI, makes custom templates inconsistent, and keeps current naming drift.

## Decision (ADR-lite)

**Context**: Claude Code、Codex、OpenCode 的供应商大量重叠，但每个平台的字段和运行时配置并不相同。用户希望能自定义模板，并明确选择了“全局共享 + 平台适用性/override”。

**Decision**: 自定义模板和内置模板都按全局 provider template 建模。模板核心字段描述供应商本身，`platforms` 字段声明适用平台和该平台的字段 override。平台表单只消费对应平台 override，并保留密钥字段。

**Consequences**: 这会让 DeepSeek、OpenRouter、SiliconFlow 等跨平台供应商只维护一份模板，减少重复；实现上需要明确的 mapper 和测试来防止平台字段串写，也需要 UI 在保存自定义模板时表达“适用平台”和各平台 override。

## Expansion Sweep

### Future Evolution

* The same template catalog could later support import/export, template packs, or remote template sync.
* Platform-specific overrides leave room for future Claude/Codex/OpenCode schema drift without widening every core template field.

### Related Scenarios

* Saved provider/account flows should remain separate from templates but may offer "save as template" for non-secret fields.
* Existing profile/provider edit flows should show which template a config resembles, but MVP does not need automatic matching badges everywhere.

### Failure And Edge Cases

* Dirty form overwrite: selecting a template should only update known template-owned non-secret fields.
* Duplicate custom templates: normalize id/name/base URL enough to prevent accidental exact duplicates.
* Invalid custom URL/JSON: validate before saving a custom template.
* Web mode: Tauri-only persistence may need a test/runtime fallback, but product behavior should target desktop persistence.

## Open Questions

* None for MVP. Remaining implementation details should be derived from code/spec context during Phase 2.

## Implementation Plan

1. Data model and mapping tests
   * Add shared provider template types and built-in catalog.
   * Add platform mapper functions for Claude Code, Codex, and OpenCode.
   * Verify: unit tests for non-secret mapping and search fields.
2. Searchable selector
   * Build reusable selector using existing BaseModal/listbox/search patterns.
   * Verify: smoke test for filtering, keyboard navigation, empty state, and selection.
3. Platform integration
   * Replace Claude old selector and add current profile editor integration.
   * Add Codex provider form integration with terminology separation.
   * Replace OpenCode recommended card list with selector.
   * Verify: targeted smoke tests per surface.
4. Custom templates
   * Add custom template add/edit/delete and persistence.
   * Add "save as template" from safe non-secret form fields.
   * Verify: persistence tests and no-secret regression tests.

## Out of Scope

* Remote template marketplace or online sync.
* Automatic API key acquisition, token storage, or credential migration.
* Full redesign of Claude/Codex/OpenCode provider management pages.
* Live provider health checks or model discovery.
* Importing Cockpit sponsor/affiliate logic wholesale.

## Technical Notes

* Root navigation read: `code_map.md`.
* ccr-ui scoped guidance read: `ccr-ui/AGENTS.md` and `ccr-ui/code_map.md`.
* Reference implementation inspected:
  * `ref/repo/cockpit-tools/src/utils/codexProviderPresets.ts`
  * `ref/repo/cockpit-tools/src/components/codex/CodexModelProviderManager.tsx`
  * `ref/repo/cockpit-tools/src/services/codexModelProviderService.ts`
* ccr-ui files inspected:
  * `ccr-ui/src/types/providerPresets.ts`
  * `ccr-ui/src/configs/providerPresets/claude.ts`
  * `ccr-ui/src/components/configs/ProviderPresetSelector.vue`
  * `ccr-ui/src/components/AddConfigModal.vue`
  * `ccr-ui/src/views/ClaudeCodeProfilesView.vue`
  * `ccr-ui/src/components/claude/ClaudeProfileEditorSections.vue`
  * `ccr-ui/src/views/CodexAuthView.vue`
  * `ccr-ui/src/types/codex.ts`
  * `ccr-ui/src-tauri/src/commands/codex_auth.rs`
  * `crates/ccr-codex/src/models/codex_model_provider.rs`
  * `crates/ccr-codex/src/services/codex_model_provider_store.rs`
  * `ccr-ui/src/types/opencode.ts`
  * `ccr-ui/src/views/OpenCodeProvidersView.vue`
  * `ccr-ui/src/api/domains/opencode.ts`
  * `ccr-ui/src/components/codex/profiles/CommandPalette.vue`
  * `ccr-ui/src/composables/useFuzzySearch.ts`
  * `ccr-ui/src/components/common/ListSearchHeader.vue`
  * `ccr-ui/src/components/common/BaseModal.vue`
