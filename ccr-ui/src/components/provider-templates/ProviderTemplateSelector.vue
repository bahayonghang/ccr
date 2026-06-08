<template>
  <section class="provider-template-selector">
    <div class="provider-template-selector__head">
      <div class="provider-template-selector__copy">
        <span class="provider-template-selector__label">{{ label }}</span>
        <span
          v-if="helper"
          class="provider-template-selector__helper"
        >
          {{ helper }}
        </span>
      </div>
      <button
        type="button"
        class="provider-template-selector__trigger"
        data-testid="provider-template-trigger"
        :disabled="disabled"
        @click="openSelector"
      >
        <span class="provider-template-selector__trigger-icon">
          <SIcon
            name="Search"
            size="w-4 h-4"
          />
        </span>
        <span class="provider-template-selector__trigger-main">
          <span class="provider-template-selector__trigger-label">
            {{ selectedTemplate?.name || placeholder }}
          </span>
          <span class="provider-template-selector__trigger-sub">
            {{ selectedTemplateSummary }}
          </span>
        </span>
        <SIcon
          name="ChevronDown"
          size="w-4 h-4"
          class="provider-template-selector__chevron"
        />
      </button>
    </div>

    <div
      v-if="selectedTemplate"
      class="provider-template-selector__summary"
      data-testid="provider-template-selected-summary"
    >
      <span class="provider-template-selector__summary-badge">
        {{ selectedTemplate.source === 'custom' ? 'Custom' : 'Built-in' }}
      </span>
      <span class="provider-template-selector__summary-text">
        {{ selectedTemplate.name }}
        <template v-if="selectedEndpoint">
          · {{ selectedEndpoint }}
        </template>
      </span>
    </div>

    <BaseModal
      :model-value="selectorOpen"
      title="Provider templates"
      :description="`${platformLabel} provider template selector`"
      size="full"
      surface="solid"
      content-class="provider-template-modal"
      :show-close="false"
      @update:model-value="selectorOpen = $event"
    >
      <template #header="{ titleId }">
        <div class="provider-template-modal__header">
          <div class="provider-template-modal__heading">
            <span class="provider-template-modal__eyebrow">{{ platformLabel }}</span>
            <h2
              :id="titleId"
              class="provider-template-modal__title"
            >
              Provider templates
            </h2>
          </div>
          <span class="provider-template-modal__count">
            {{ results.length }} matches
          </span>
        </div>
      </template>

      <div class="provider-template-modal__body">
        <label
          class="sr-only"
          :for="searchInputId"
        >
          Search provider templates
        </label>
        <div class="provider-template-modal__search">
          <SIcon
            name="Search"
            size="w-4 h-4"
            class="provider-template-modal__search-icon"
          />
          <input
            :id="searchInputId"
            ref="searchInputRef"
            v-model="query"
            class="provider-template-modal__search-input"
            data-testid="provider-template-search"
            placeholder="Search by provider, host, model, tag..."
            :aria-activedescendant="activeOptionId"
            aria-controls="provider-template-listbox"
            @keydown="handleKeyDown"
          >
        </div>

        <div
          id="provider-template-listbox"
          ref="listRef"
          class="provider-template-modal__list"
          role="listbox"
          aria-label="Provider template results"
        >
          <button
            :id="optionId(0)"
            type="button"
            role="option"
            class="provider-template-modal__row provider-template-modal__row--manual"
            data-testid="provider-template-manual-row"
            :class="{ 'provider-template-modal__row--active': activeIndex === 0 }"
            :aria-selected="activeIndex === 0"
            data-index="0"
            @mouseenter="activeIndex = 0"
            @click="selectManual"
          >
            <span class="provider-template-modal__icon-box">
              <SIcon
                name="Pencil"
                size="w-3.5 h-3.5"
              />
            </span>
            <span class="provider-template-modal__row-main">
              <span class="provider-template-modal__row-title">Manual provider</span>
              <span class="provider-template-modal__row-sub">Keep editing fields without applying a template.</span>
            </span>
            <span class="provider-template-modal__pill">Manual</span>
          </button>

          <template v-if="results.length > 0">
            <button
              v-for="(option, index) in results"
              :id="optionId(index + 1)"
              :key="option.id"
              type="button"
              role="option"
              class="provider-template-modal__row"
              data-testid="provider-template-option"
              :class="{ 'provider-template-modal__row--active': activeIndex === index + 1 }"
              :aria-selected="activeIndex === index + 1"
              :data-index="index + 1"
              @mouseenter="activeIndex = index + 1"
              @click="selectOption(option)"
            >
              <span class="provider-template-modal__icon-box">
                <SIcon
                  :name="option.template.source === 'custom' ? 'Database' : 'Blocks'"
                  size="w-3.5 h-3.5"
                />
              </span>
              <span class="provider-template-modal__row-main">
                <span class="provider-template-modal__row-title">{{ option.label }}</span>
                <span class="provider-template-modal__row-sub">{{ option.subtitle }}</span>
              </span>
              <span class="provider-template-modal__meta">
                <span class="provider-template-modal__pill">{{ option.sourceLabel }}</span>
                <span class="provider-template-modal__pill provider-template-modal__pill--muted">
                  {{ option.categoryLabel }}
                </span>
                <span
                  v-if="option.template.source === 'custom'"
                  class="provider-template-modal__actions"
                  @click.stop
                >
                  <button
                    type="button"
                    class="provider-template-modal__icon-button"
                    data-testid="provider-template-edit-custom"
                    title="Edit custom template"
                    @click.stop="openCustomEditor(option.template)"
                  >
                    <SIcon
                      name="Pencil"
                      size="w-3.5 h-3.5"
                    />
                  </button>
                  <button
                    type="button"
                    class="provider-template-modal__icon-button provider-template-modal__icon-button--danger"
                    data-testid="provider-template-delete-custom"
                    title="Delete custom template"
                    @click.stop="deleteCustom(option.template.id)"
                  >
                    <SIcon
                      name="Trash2"
                      size="w-3.5 h-3.5"
                    />
                  </button>
                </span>
              </span>
            </button>
          </template>

          <div
            v-else
            class="provider-template-modal__empty"
            data-testid="provider-template-empty"
          >
            No template matched the current keywords.
          </div>
        </div>
      </div>

      <template #footer>
        <div class="provider-template-modal__footer">
          <div class="provider-template-modal__keys">
            <span><kbd>Enter</kbd> apply</span>
            <span><kbd>↑↓</kbd> select</span>
            <span><kbd>Esc</kbd> close</span>
          </div>
          <div class="provider-template-modal__footer-actions">
            <button
              type="button"
              class="provider-template-modal__secondary"
              data-testid="provider-template-new-custom"
              @click="openCustomEditor()"
            >
              New template
            </button>
            <button
              v-if="draftContext"
              type="button"
              class="provider-template-modal__primary"
              data-testid="provider-template-save-current"
              @click="openCustomEditor(undefined, true)"
            >
              Save current
            </button>
          </div>
        </div>
      </template>
    </BaseModal>

    <BaseModal
      :model-value="customEditorOpen"
      :title="editingCustomId ? 'Edit template' : 'Custom template'"
      description="Store non-secret provider metadata for later reuse."
      size="full"
      surface="solid"
      content-class="provider-template-editor-modal"
      @update:model-value="customEditorOpen = $event"
    >
      <div class="provider-template-editor">
        <div
          v-if="customError"
          class="provider-template-editor__error"
        >
          {{ customError }}
        </div>

        <div class="provider-template-editor__grid">
          <label class="provider-template-editor__field">
            <span>Name</span>
            <input
              v-model="customForm.name"
              class="provider-template-editor__input"
              data-testid="provider-template-custom-name"
              placeholder="OpenRouter"
            >
          </label>
          <label class="provider-template-editor__field">
            <span>ID</span>
            <input
              v-model="customForm.id"
              class="provider-template-editor__input"
              placeholder="openrouter"
            >
          </label>
          <label class="provider-template-editor__field">
            <span>Category</span>
            <select
              v-model="customForm.category"
              class="provider-template-editor__input"
            >
              <option value="official">
                Official
              </option>
              <option value="cn_official">
                CN official
              </option>
              <option value="aggregator">
                Aggregator
              </option>
              <option value="third_party">
                Third party
              </option>
              <option value="local">
                Local
              </option>
            </select>
          </label>
          <label class="provider-template-editor__field">
            <span>Website URL</span>
            <input
              v-model="customForm.websiteUrl"
              class="provider-template-editor__input"
              placeholder="https://..."
            >
          </label>
          <label class="provider-template-editor__field">
            <span>API key docs URL</span>
            <input
              v-model="customForm.apiKeyUrl"
              class="provider-template-editor__input"
              placeholder="https://..."
            >
          </label>
          <fieldset class="provider-template-editor__field provider-template-editor__field--platforms">
            <legend>Platforms</legend>
            <label
              v-for="item in platformItems"
              :key="item.id"
              class="provider-template-editor__check"
            >
              <input
                v-model="customForm.platforms[item.id]"
                type="checkbox"
                :data-testid="`provider-template-platform-${item.id}`"
              >
              <span>{{ item.label }}</span>
            </label>
          </fieldset>
        </div>

        <div class="provider-template-editor__stack">
          <label class="provider-template-editor__field">
            <span>Base URLs</span>
            <textarea
              v-model="customForm.baseUrlsInput"
              class="provider-template-editor__textarea"
              rows="4"
              placeholder="One URL per line"
            />
          </label>
          <label class="provider-template-editor__field">
            <span>Model catalog</span>
            <textarea
              v-model="customForm.modelCatalogInput"
              class="provider-template-editor__textarea"
              rows="4"
              placeholder="One model per line"
            />
          </label>
          <label class="provider-template-editor__field">
            <span>Aliases</span>
            <textarea
              v-model="customForm.aliasesInput"
              class="provider-template-editor__textarea"
              rows="3"
              placeholder="Search aliases, one per line"
            />
          </label>
          <label class="provider-template-editor__field">
            <span>Tags</span>
            <textarea
              v-model="customForm.tagsInput"
              class="provider-template-editor__textarea"
              rows="3"
              placeholder="Tags, one per line"
            />
          </label>
        </div>

        <div
          v-if="selectedPlatformOverrideItems.length > 0"
          class="provider-template-editor__override-list"
        >
          <label
            v-for="item in selectedPlatformOverrideItems"
            :key="item.id"
            class="provider-template-editor__field"
          >
            <span>{{ item.label }} override JSON</span>
            <textarea
              v-model="customForm.platformOverrideInputs[item.id]"
              class="provider-template-editor__textarea provider-template-editor__textarea--json"
              rows="7"
              spellcheck="false"
              :data-testid="`provider-template-platform-override-${item.id}`"
              placeholder="{}"
            />
          </label>
        </div>
      </div>

      <template #footer>
        <button
          type="button"
          class="provider-template-modal__secondary"
          @click="customEditorOpen = false"
        >
          Cancel
        </button>
        <button
          type="button"
          class="provider-template-modal__primary"
          data-testid="provider-template-save-custom"
          @click="saveCustom"
        >
          Save template
        </button>
      </template>
    </BaseModal>
  </section>
</template>

<script setup lang="ts">
import { computed, nextTick, reactive, ref, watch } from 'vue'
import BaseModal from '@/components/common/BaseModal.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { useFuzzySearch } from '@/composables/useFuzzySearch'
import { useProviderTemplates } from '@/composables/useProviderTemplates'
import type {
  ProviderTemplate,
  ProviderTemplateDraftContext,
  ProviderTemplateOption,
  ProviderTemplatePlatform,
  ProviderTemplatePlatformOverrides,
  ProviderTemplateSelection,
} from '@/types/providerTemplates'
import {
  buildProviderTemplateOptions,
  createCustomProviderTemplateFromDraft,
  formatListInput,
  parseJsonObject,
  parseListInput,
  PROVIDER_TEMPLATE_PLATFORM_LABELS,
  safeJson,
  slugifyTemplateId,
} from '@/utils/providerTemplates'

const props = withDefaults(defineProps<{
  platform: ProviderTemplatePlatform
  selectedTemplateId?: string | null
  selectedEndpoint?: string
  label?: string
  helper?: string
  placeholder?: string
  disabled?: boolean
  draftContext?: ProviderTemplateDraftContext | null
}>(), {
  selectedTemplateId: null,
  selectedEndpoint: '',
  label: 'Provider template',
  helper: '',
  placeholder: 'Choose a template',
  disabled: false,
  draftContext: null,
})

const emit = defineEmits<{
  select: [selection: ProviderTemplateSelection]
  manual: []
}>()

const {
  templates,
  saveCustomTemplate,
  removeCustomTemplate,
} = useProviderTemplates()

const selectorOpen = ref(false)
const customEditorOpen = ref(false)
const activeIndex = ref(0)
const searchInputRef = ref<HTMLInputElement | null>(null)
const listRef = ref<HTMLElement | null>(null)
const editingCustomId = ref('')
const customError = ref('')

const searchInputId = computed(() => `provider-template-search-${props.platform}`)
const platformLabel = computed(() => PROVIDER_TEMPLATE_PLATFORM_LABELS[props.platform])
const allOptions = computed(() => buildProviderTemplateOptions(templates.value, props.platform))
const { query, results } = useFuzzySearch<ProviderTemplateOption>(
  allOptions,
  [
    { name: 'label', weight: 4 },
    { name: 'subtitle', weight: 2 },
    { name: 'searchText', weight: 3 },
  ],
  {
    threshold: 0.36,
    ignoreLocation: true,
  },
)

const selectedTemplate = computed(() => (
  props.selectedTemplateId
    ? templates.value.find(template => template.id === props.selectedTemplateId) || null
    : null
))

const selectedEndpoint = computed(() => props.selectedEndpoint?.trim())

const selectedTemplateSummary = computed(() => {
  if (!selectedTemplate.value) return `${allOptions.value.length} reusable templates`
  const source = selectedTemplate.value.source === 'custom' ? 'Custom' : 'Built-in'
  return [source, selectedEndpoint.value].filter(Boolean).join(' · ')
})

const visibleItemCount = computed(() => results.value.length + 1)
const activeOptionId = computed(() => optionId(activeIndex.value))
const optionId = (index: number) => `provider-template-option-${props.platform}-${index}`

const platformItems: Array<{ id: ProviderTemplatePlatform; label: string }> = [
  { id: 'claude', label: 'Claude Code' },
  { id: 'codex', label: 'Codex' },
  { id: 'opencode', label: 'OpenCode' },
]

const selectedPlatformOverrideItems = computed(() => (
  platformItems.filter(item => customForm.platforms[item.id])
))

const customForm = reactive({
  id: '',
  name: '',
  category: 'third_party' as ProviderTemplate['category'],
  websiteUrl: '',
  apiKeyUrl: '',
  aliasesInput: '',
  tagsInput: '',
  baseUrlsInput: '',
  modelCatalogInput: '',
  platforms: {
    claude: false,
    codex: false,
    opencode: false,
  } as Record<ProviderTemplatePlatform, boolean>,
  platformOverrideInputs: {
    claude: '{}',
    codex: '{}',
    opencode: '{}',
  } as Record<ProviderTemplatePlatform, string>,
})

watch(selectorOpen, async open => {
  if (!open) return
  query.value = ''
  activeIndex.value = 0
  await nextTick()
  searchInputRef.value?.focus()
})

watch(results, () => {
  if (activeIndex.value >= visibleItemCount.value) {
    activeIndex.value = Math.max(0, visibleItemCount.value - 1)
  }
})

function openSelector() {
  if (props.disabled) return
  selectorOpen.value = true
}

function selectManual() {
  emit('manual')
  selectorOpen.value = false
}

function selectOption(option: ProviderTemplateOption) {
  emit('select', {
    template: option.template,
    endpoint: option.endpoint,
  })
  selectorOpen.value = false
}

function scrollActiveIntoView() {
  const el = listRef.value?.querySelector<HTMLElement>(`[data-index="${activeIndex.value}"]`)
  el?.scrollIntoView({ block: 'nearest' })
}

function handleKeyDown(event: KeyboardEvent) {
  if (event.key === 'ArrowDown') {
    event.preventDefault()
    activeIndex.value = Math.min(visibleItemCount.value - 1, activeIndex.value + 1)
    void nextTick(scrollActiveIntoView)
  } else if (event.key === 'ArrowUp') {
    event.preventDefault()
    activeIndex.value = Math.max(0, activeIndex.value - 1)
    void nextTick(scrollActiveIntoView)
  } else if (event.key === 'Enter') {
    event.preventDefault()
    if (activeIndex.value === 0) {
      selectManual()
      return
    }
    const option = results.value[activeIndex.value - 1]
    if (option) selectOption(option)
  } else if (event.key === 'Escape') {
    event.preventDefault()
    selectorOpen.value = false
  }
}

function formatPlatformOverrideInput(
  template: ProviderTemplate | undefined,
  platform: ProviderTemplatePlatform,
  fromCurrent: boolean,
): string {
  const templateOverride = template?.platforms[platform]
  if (templateOverride) return safeJson(templateOverride)

  const draft = props.draftContext
  if (fromCurrent && draft?.platform === platform) {
    return safeJson(draft.platformOverride)
  }

  return '{}'
}

function fillCustomForm(template?: ProviderTemplate, fromCurrent = false) {
  const draft = props.draftContext

  editingCustomId.value = template?.source === 'custom' ? template.id : ''
  customForm.name = template?.name || (fromCurrent ? draft?.name || draft?.defaultName || '' : '')
  customForm.id = template?.id || slugifyTemplateId(customForm.name)
  customForm.category = template?.category || draft?.category || 'third_party'
  customForm.websiteUrl = template?.websiteUrl || draft?.websiteUrl || ''
  customForm.apiKeyUrl = template?.apiKeyUrl || draft?.apiKeyUrl || ''
  customForm.aliasesInput = formatListInput(template?.aliases || draft?.aliases || [])
  customForm.tagsInput = formatListInput(template?.tags || draft?.tags || [])
  customForm.baseUrlsInput = formatListInput(template?.baseUrls || draft?.baseUrls || [])
  customForm.modelCatalogInput = formatListInput(template?.modelCatalog || draft?.modelCatalog || [])

  const selectedPlatforms = new Set<ProviderTemplatePlatform>(
    template
      ? platformItems
        .filter(item => Boolean(template.platforms[item.id]))
        .map(item => item.id)
      : [props.platform],
  )
  for (const item of platformItems) {
    customForm.platforms[item.id] = selectedPlatforms.has(item.id)
    customForm.platformOverrideInputs[item.id] = formatPlatformOverrideInput(template, item.id, fromCurrent)
  }
}

function openCustomEditor(template?: ProviderTemplate, fromCurrent = false) {
  customError.value = ''
  fillCustomForm(template, fromCurrent)
  customEditorOpen.value = true
}

function draftForCustomSave(existing?: ProviderTemplate): ProviderTemplateDraftContext | null {
  if (existing?.platforms[props.platform]) {
    return {
      platform: props.platform,
      defaultName: existing.name,
      name: existing.name,
      category: existing.category,
      websiteUrl: existing.websiteUrl,
      apiKeyUrl: existing.apiKeyUrl,
      aliases: existing.aliases,
      tags: existing.tags,
      baseUrls: existing.baseUrls,
      modelCatalog: existing.modelCatalog,
      platformOverride: existing.platforms[props.platform] as never,
    }
  }

  if (props.draftContext) return props.draftContext
  return null
}

function saveCustom() {
  customError.value = ''
  const name = customForm.name.trim()
  if (!name) {
    customError.value = 'Template name is required.'
    return
  }

  const existing = editingCustomId.value
    ? templates.value.find(template => template.id === editingCustomId.value)
    : undefined
  const draft = draftForCustomSave(existing)
  if (!draft) {
    customError.value = 'Open a provider form before saving a template.'
    return
  }

  const selectedPlatforms = platformItems
    .filter(item => customForm.platforms[item.id])
    .map(item => item.id)
  if (selectedPlatforms.length === 0) {
    customError.value = 'Select at least one platform.'
    return
  }

  const platformOverrides: ProviderTemplatePlatformOverrides = {}
  for (const platform of selectedPlatforms) {
    try {
      const override = parseJsonObject(customForm.platformOverrideInputs[platform])
      if (Object.keys(override).length > 0) {
        platformOverrides[platform] = override as never
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Invalid JSON.'
      customError.value = `${PROVIDER_TEMPLATE_PLATFORM_LABELS[platform]} override JSON is invalid. ${message}`
      return
    }
  }

  const template = createCustomProviderTemplateFromDraft(draft, selectedPlatforms, {
    id: customForm.id,
    name,
    aliases: parseListInput(customForm.aliasesInput),
    tags: parseListInput(customForm.tagsInput),
    category: customForm.category,
    websiteUrl: customForm.websiteUrl,
    apiKeyUrl: customForm.apiKeyUrl,
    baseUrls: parseListInput(customForm.baseUrlsInput),
    modelCatalog: parseListInput(customForm.modelCatalogInput),
    existing,
    platformOverrides,
  })

  saveCustomTemplate(template)
  customEditorOpen.value = false
}

function deleteCustom(id: string) {
  removeCustomTemplate(id)
}
</script>

<style scoped>
:global(.provider-template-modal) {
  max-width: min(760px, calc(100vw - 32px)) !important;
  border-radius: 18px !important;
  background: var(--color-bg-elevated) !important;
  border: 1px solid var(--color-border-default) !important;
  box-shadow: 0 28px 80px rgb(0 0 0 / 24%) !important;
  backdrop-filter: none !important;
}

:global(.provider-template-editor-modal) {
  max-width: min(820px, calc(100vw - 32px)) !important;
  max-height: calc(100vh - 32px) !important;
  overflow-y: auto !important;
  border-radius: 18px !important;
  background: var(--color-bg-elevated) !important;
  border: 1px solid var(--color-border-default) !important;
  backdrop-filter: none !important;
}

.provider-template-selector,
.provider-template-modal__body,
.provider-template-modal__header,
.provider-template-modal__footer,
.provider-template-editor {
  --template-bg: var(--color-bg-elevated);
  --template-bg-soft: var(--color-bg-surface);
  --template-bg-muted: var(--color-bg-overlay);
  --template-text: var(--color-text-primary);
  --template-text-soft: var(--color-text-secondary);
  --template-text-muted: var(--color-text-muted);
  --template-line: var(--color-border-subtle);
  --template-line-strong: var(--color-border-default);
  --template-accent: var(--color-accent-primary);
  --template-accent-soft: rgb(var(--color-accent-primary-rgb) / 10%);
  --template-accent-line: rgb(var(--color-accent-primary-rgb) / 28%);

  color: var(--template-text);
}

.provider-template-selector {
  display: grid;
  gap: 0.65rem;
}

.provider-template-selector__head {
  display: grid;
  gap: 0.55rem;
}

.provider-template-selector__copy {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.provider-template-selector__label {
  color: var(--template-text-soft);
  font-size: 0.75rem;
  font-weight: 650;
  letter-spacing: 0.08em;
  line-height: 1rem;
  text-transform: uppercase;
}

.provider-template-selector__helper {
  color: var(--template-text-muted);
  font-size: 0.8rem;
  line-height: 1.35rem;
}

.provider-template-selector__trigger {
  display: grid;
  grid-template-columns: 30px minmax(0, 1fr) auto;
  gap: 0.75rem;
  align-items: center;
  width: 100%;
  min-height: 58px;
  border: 1px solid var(--template-line-strong);
  border-radius: 14px;
  background: var(--template-bg-soft);
  padding: 0.7rem 0.85rem;
  text-align: left;
  transition: border-color 120ms ease, background 120ms ease, box-shadow 120ms ease;
}

.provider-template-selector__trigger:hover:not(:disabled) {
  border-color: var(--template-accent-line);
  background: var(--template-bg);
}

.provider-template-selector__trigger:focus-visible {
  outline: none;
  border-color: var(--template-accent-line);
  box-shadow: 0 0 0 3px rgb(var(--color-accent-primary-rgb) / 12%);
}

.provider-template-selector__trigger:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.provider-template-selector__trigger-icon,
.provider-template-modal__icon-box {
  display: grid;
  place-items: center;
  width: 30px;
  height: 30px;
  border: 1px solid var(--template-line);
  border-radius: 10px;
  background: var(--template-bg-muted);
  color: var(--template-text-muted);
}

.provider-template-selector__trigger-main {
  min-width: 0;
}

.provider-template-selector__trigger-label,
.provider-template-selector__trigger-sub,
.provider-template-selector__summary-text,
.provider-template-modal__row-title,
.provider-template-modal__row-sub {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.provider-template-selector__trigger-label {
  display: block;
  color: var(--template-text);
  font-size: 0.92rem;
  font-weight: 600;
  line-height: 1.2rem;
}

.provider-template-selector__trigger-sub {
  display: block;
  margin-top: 0.15rem;
  color: var(--template-text-muted);
  font-size: 0.78rem;
  line-height: 1.1rem;
}

.provider-template-selector__chevron {
  color: var(--template-text-muted);
}

.provider-template-selector__summary {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-width: 0;
  border: 1px solid var(--template-line);
  border-radius: 12px;
  background: var(--template-bg-muted);
  padding: 0.5rem 0.65rem;
}

.provider-template-selector__summary-badge,
.provider-template-modal__pill {
  flex-shrink: 0;
  border: 1px solid var(--template-line);
  border-radius: 999px;
  background: var(--template-bg-soft);
  color: var(--template-text-muted);
  padding: 0.15rem 0.45rem;
  font-size: 0.68rem;
  font-weight: 650;
  letter-spacing: 0.04em;
  line-height: 1rem;
  text-transform: uppercase;
}

.provider-template-selector__summary-text {
  min-width: 0;
  color: var(--template-text-soft);
  font-size: 0.78rem;
}

.provider-template-modal__header,
.provider-template-modal__footer {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.provider-template-modal__eyebrow {
  color: var(--template-text-muted);
  font-size: 0.68rem;
  font-weight: 700;
  letter-spacing: 0.16em;
  line-height: 1rem;
  text-transform: uppercase;
}

.provider-template-modal__title {
  margin: 0.25rem 0 0;
  color: var(--template-text);
  font-size: 1.15rem;
  font-weight: 650;
  line-height: 1.45rem;
}

.provider-template-modal__count {
  flex-shrink: 0;
  border: 1px solid var(--template-line);
  border-radius: 999px;
  background: var(--template-bg-soft);
  color: var(--template-text-muted);
  padding: 0.3rem 0.55rem;
  font-size: 0.72rem;
}

.provider-template-modal__search {
  position: relative;
}

.provider-template-modal__search-icon {
  position: absolute;
  top: 50%;
  left: 0.8rem;
  color: var(--template-text-muted);
  transform: translateY(-50%);
  pointer-events: none;
}

.provider-template-modal__search-input,
.provider-template-editor__input,
.provider-template-editor__textarea {
  width: 100%;
  border: 1px solid var(--template-line-strong);
  border-radius: 12px;
  background: var(--template-bg-soft);
  color: var(--template-text);
  font-family: inherit;
  font-size: 0.9rem;
  outline: none;
}

.provider-template-modal__search-input {
  padding: 0.8rem 0.9rem 0.8rem 2.55rem;
}

.provider-template-modal__search-input:focus,
.provider-template-editor__input:focus,
.provider-template-editor__textarea:focus {
  border-color: var(--template-accent-line);
  box-shadow: 0 0 0 3px rgb(var(--color-accent-primary-rgb) / 10%);
}

.provider-template-modal__list {
  display: grid;
  gap: 0.35rem;
  max-height: min(56vh, 500px);
  overflow-y: auto;
  margin-top: 0.9rem;
  padding-right: 0.15rem;
}

.provider-template-modal__row {
  position: relative;
  display: grid;
  grid-template-columns: 30px minmax(0, 1fr) auto;
  gap: 0.75rem;
  align-items: center;
  width: 100%;
  min-height: 62px;
  border: 1px solid transparent;
  border-radius: 12px;
  background: transparent;
  padding: 0.65rem 0.75rem;
  text-align: left;
  transition: border-color 120ms ease, background 120ms ease, box-shadow 120ms ease;
}

.provider-template-modal__row::before {
  position: absolute;
  top: 0.8rem;
  bottom: 0.8rem;
  left: 0;
  width: 3px;
  border-radius: 999px;
  background: transparent;
  content: '';
}

.provider-template-modal__row--active {
  border-color: var(--template-accent-line);
  background: var(--template-accent-soft);
  box-shadow: 0 8px 22px rgb(var(--color-accent-primary-rgb) / 8%);
}

.provider-template-modal__row--active::before {
  background: var(--template-accent);
}

.provider-template-modal__row--manual {
  border-color: var(--template-line);
  background: var(--template-bg-soft);
}

.provider-template-modal__row-main {
  min-width: 0;
}

.provider-template-modal__row-title {
  display: block;
  color: var(--template-text);
  font-size: 0.9rem;
  font-weight: 600;
  line-height: 1.2rem;
}

.provider-template-modal__row-sub {
  display: block;
  margin-top: 0.2rem;
  color: var(--template-text-muted);
  font-size: 0.78rem;
  line-height: 1.1rem;
}

.provider-template-modal__meta,
.provider-template-modal__actions {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
}

.provider-template-modal__pill--muted {
  color: var(--template-text-soft);
}

.provider-template-modal__icon-button {
  display: grid;
  place-items: center;
  width: 1.75rem;
  height: 1.75rem;
  border: 1px solid var(--template-line);
  border-radius: 0.55rem;
  background: var(--template-bg-soft);
  color: var(--template-text-muted);
}

.provider-template-modal__icon-button:hover {
  border-color: var(--template-accent-line);
  color: var(--template-accent);
}

.provider-template-modal__icon-button--danger:hover {
  border-color: rgb(var(--color-danger-rgb) / 25%);
  color: var(--color-danger);
}

.provider-template-modal__empty {
  border: 1px dashed var(--template-line-strong);
  border-radius: 12px;
  color: var(--template-text-muted);
  padding: 2rem 1rem;
  text-align: center;
}

.provider-template-modal__keys,
.provider-template-modal__footer-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem 0.8rem;
}

.provider-template-modal__keys {
  color: var(--template-text-muted);
  font-size: 0.74rem;
}

.provider-template-modal__keys kbd {
  border: 1px solid var(--template-line-strong);
  border-radius: 0.35rem;
  background: var(--template-bg-soft);
  color: var(--template-text-soft);
  padding: 0.05rem 0.35rem;
  font-family: var(--font-mono);
}

.provider-template-modal__secondary,
.provider-template-modal__primary {
  min-height: 2.2rem;
  border-radius: 999px;
  padding: 0.45rem 0.85rem;
  font-size: 0.82rem;
  font-weight: 600;
}

.provider-template-modal__secondary {
  border: 1px solid var(--template-line-strong);
  background: var(--template-bg-soft);
  color: var(--template-text-soft);
}

.provider-template-modal__primary {
  border: 1px solid var(--template-accent-line);
  background: var(--template-accent-soft);
  color: var(--template-accent);
}

.provider-template-editor {
  display: grid;
  gap: 1rem;
}

.provider-template-editor__grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.9rem;
}

.provider-template-editor__stack {
  display: grid;
  gap: 0.9rem;
}

.provider-template-editor__override-list {
  display: grid;
  gap: 0.9rem;
  border-top: 1px solid var(--template-line);
  padding-top: 0.95rem;
}

.provider-template-editor__field {
  display: grid;
  gap: 0.45rem;
  color: var(--template-text-soft);
  font-size: 0.76rem;
  font-weight: 650;
  letter-spacing: 0.06em;
  line-height: 1rem;
  text-transform: uppercase;
}

.provider-template-editor__field--platforms {
  grid-column: 1 / -1;
  border: 1px solid var(--template-line);
  border-radius: 12px;
  background: var(--template-bg-soft);
  padding: 0.8rem;
}

.provider-template-editor__field--platforms legend {
  padding: 0 0.2rem;
}

.provider-template-editor__input {
  min-height: 2.7rem;
  padding: 0.65rem 0.75rem;
}

.provider-template-editor__textarea {
  resize: vertical;
  padding: 0.7rem 0.75rem;
  font-family: var(--font-mono);
  font-size: 0.82rem;
}

.provider-template-editor__textarea--json {
  min-height: 10rem;
}

.provider-template-editor__check {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  margin-right: 1rem;
  margin-top: 0.55rem;
  color: var(--template-text);
  font-size: 0.82rem;
  font-weight: 500;
  letter-spacing: 0;
  text-transform: none;
}

.provider-template-editor__error {
  border: 1px solid rgb(var(--color-danger-rgb) / 24%);
  border-radius: 12px;
  background: rgb(var(--color-danger-rgb) / 10%);
  color: var(--color-danger);
  padding: 0.75rem 0.9rem;
  font-size: 0.85rem;
}

@media (width <= 700px) {
  :global(.provider-template-modal),
  :global(.provider-template-editor-modal) {
    max-width: calc(100vw - 20px) !important;
  }

  .provider-template-modal__header,
  .provider-template-modal__footer {
    flex-direction: column;
  }

  .provider-template-modal__row {
    grid-template-columns: 30px minmax(0, 1fr);
  }

  .provider-template-modal__meta {
    grid-column: 2;
    justify-self: start;
  }

  .provider-template-editor__grid {
    grid-template-columns: minmax(0, 1fr);
  }
}

@media (prefers-reduced-motion: reduce) {
  .provider-template-selector__trigger,
  .provider-template-modal__row {
    transition: none;
  }
}
</style>
