<!-- Codex Profile 编辑器模态。
     模板结构与既有表单字段不变，样式统一消费共享编辑器基底（pe-*），
     不再维护平行的 --editor-* 令牌体系 / !important / 硬编码 RGBA 暗色覆盖块。
     保存前在模态内完成校验，失败时顶部汇总条并可跳转到第一个错误分段。 -->
<template>
  <BaseModal
    :model-value="modelValue"
    :show-close="false"
    size="xl"
    surface="glass"
    :close-on-backdrop="!saving"
    :close-on-escape="!saving"
    :persistent="saving"
    content-class="pe-modal !max-w-[980px] rounded-2xl"
    @update:model-value="handleModalModelValue"
  >
    <template #header="{ titleId }">
      <div class="pe-modal__head">
        <div class="flex min-w-0 items-start gap-4">
          <div class="pe-panel-icon flex h-12 w-12 shrink-0 items-center justify-center">
            <SIcon
              name="Settings"
              size="w-6 h-6"
            />
          </div>
          <div class="min-w-0">
            <p class="pe-modal__eyebrow">
              {{ isEditing ? $t('codex.profiles.editProfile') : $t('codex.profiles.addProfile') }}
            </p>
            <div class="mt-1.5 flex flex-wrap items-center gap-2">
              <h2
                :id="titleId"
                class="pe-modal__title"
              >
                {{ form.name.trim() || editingName || $t('codex.profiles.addProfile') }}
              </h2>
              <span
                class="pe-pill"
                :class="form.enabled ? 'pe-pill--accent' : 'pe-pill--danger'"
              >
                {{ form.enabled ? $t('codex.states.enabled') : $t('codex.states.disabled') }}
              </span>
            </div>
            <p class="pe-modal__desc mt-1.5 max-w-3xl">
              {{ $t('codex.profiles.subtitle') }}
            </p>
          </div>
        </div>

        <button
          type="button"
          class="pe-icon-btn shrink-0"
          :aria-label="$t('common.close')"
          :disabled="saving"
          @click="requestClose"
        >
          <SIcon
            name="X"
            size="w-4 h-4"
          />
        </button>
      </div>
    </template>

    <div class="pe-shell max-h-[calc(90vh-9rem)] overflow-hidden">
      <div
        v-if="showValidation && validationErrors.length > 0"
        class="pe-summary"
        role="alert"
      >
        <SIcon
          name="AlertTriangle"
          size="w-4 h-4"
        />
        <span>{{ validationErrors[0].message }}</span>
        <button
          type="button"
          class="pe-summary__jump"
          @click="scrollToSection(validationErrors[0].section)"
        >
          {{ $t('codex.profiles.validationJump') }}
        </button>
      </div>

      <div
        class="pe-nav"
        role="tablist"
        :aria-label="$t('codex.profiles.sections.identity')"
      >
        <button
          v-for="section in sectionItems"
          :key="section.id"
          type="button"
          role="tab"
          class="pe-nav__item"
          :class="{ 'pe-nav__item--active': activeSectionId === section.id }"
          :aria-selected="activeSectionId === section.id"
          @click="scrollToSection(section.id)"
        >
          {{ section.title }}
        </button>
      </div>

      <div
        ref="scrollRef"
        class="pe-scroll"
        @scroll="syncActiveSection"
      >
        <ProviderTemplateSelector
          v-if="providerTemplateDraft"
          class="mb-4"
          platform="codex"
          :selected-template-id="selectedProviderTemplate"
          :selected-endpoint="selectedProviderEndpoint"
          :draft-context="providerTemplateDraft"
          :label="$t('codex.profiles.templateSelector.label')"
          :helper="$t('codex.profiles.templateSelector.helper')"
          :placeholder="$t('codex.profiles.templateSelector.placeholder')"
          @select="$emit('select-template', $event)"
          @manual="$emit('manual-template')"
        />

        <section
          id="identity"
          ref="identityRef"
          class="space-y-3"
        >
          <div class="flex items-start gap-3">
            <div class="pe-panel-icon flex h-9 w-9 shrink-0 items-center justify-center">
              <SIcon
                name="Layers"
                size="w-4 h-4"
              />
            </div>
            <div class="min-w-0">
              <h3 class="pe-section__title">
                {{ $t('codex.profiles.sections.identity') }}
              </h3>
              <p class="pe-section__desc mt-1">
                {{ $t('codex.profiles.sectionHints.identity') }}
              </p>
            </div>
          </div>

          <div class="pe-panel-muted p-4">
            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
              <div class="pe-field">
                <label
                  for="codex-profile-name"
                  class="pe-field__label"
                >
                  {{ $t('codex.profiles.fields.name') }} <span class="text-accent-danger">*</span>
                </label>
                <input
                  id="codex-profile-name"
                  data-testid="codex-profile-name-input"
                  :value="form.name"
                  type="text"
                  class="pe-input"
                  :placeholder="$t('codex.profiles.placeholders.name')"
                  @input="updateTextField('name', $event)"
                >
                <p class="pe-field__hint">
                  {{ isEditing ? $t('codex.profiles.nameRenameHint') : $t('codex.profiles.nameCreateHint') }}
                </p>
              </div>

              <div class="pe-field">
                <label
                  for="codex-profile-description"
                  class="pe-field__label"
                >
                  {{ $t('codex.profiles.fields.description') }}
                </label>
                <input
                  id="codex-profile-description"
                  :value="form.description"
                  type="text"
                  class="pe-input"
                  :placeholder="$t('codex.profiles.placeholders.description')"
                  @input="updateTextField('description', $event)"
                >
              </div>
            </div>
          </div>
        </section>

        <section
          id="auth"
          ref="authRef"
          class="mt-5 space-y-3"
        >
          <div class="flex items-start gap-3">
            <div class="pe-panel-icon flex h-9 w-9 shrink-0 items-center justify-center">
              <SIcon
                name="ShieldCheck"
                size="w-4 h-4"
              />
            </div>
            <div class="min-w-0">
              <h3 class="pe-section__title">
                {{ $t('codex.profiles.sections.authentication') }}
              </h3>
              <p class="pe-section__desc mt-1">
                {{ $t('codex.profiles.sectionHints.authentication') }}
              </p>
            </div>
          </div>

          <div class="pe-panel space-y-4 p-4">
            <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
              <div class="pe-field">
                <label
                  for="codex-profile-auth-mode"
                  class="pe-field__label"
                >
                  {{ $t('codex.profiles.fields.authMode') }} <span class="text-accent-danger">*</span>
                </label>
                <select
                  id="codex-profile-auth-mode"
                  :value="form.auth_mode"
                  class="pe-select"
                  @change="updateSelectField('auth_mode', $event)"
                >
                  <option
                    v-for="authMode in availableAuthModeOptions"
                    :key="authMode"
                    :value="authMode"
                  >
                    {{ authModeLabel(authMode) }}
                  </option>
                </select>
                <p
                  v-if="isDeprecatedAuthMode"
                  class="pe-field__hint text-accent-warning"
                >
                  {{ $t('codex.profiles.deprecatedAuthModeHint', { mode: authModeLabel(form.auth_mode) }) }}
                </p>
              </div>

              <div class="pe-panel-muted p-3">
                <p class="pe-field__label">
                  {{ $t('codex.profiles.fields.openAiLoginMethod') }}
                </p>
                <p class="mt-2 flex flex-wrap items-center gap-2">
                  <span class="pe-tag rounded-full px-2.5 py-0.5 text-xs">
                    {{ displayOpenAiLoginMethod }}
                  </span>
                  <span class="pe-tag rounded-full px-2.5 py-0.5 text-xs">
                    {{ usesOpenAiAuthMode(form.auth_mode) ? $t('codex.profiles.openAiAuthOn') : $t('codex.profiles.openAiAuthOff') }}
                  </span>
                </p>
              </div>
            </div>

            <div class="pe-field">
              <label
                for="codex-profile-base-url"
                class="pe-field__label"
              >
                {{ $t('codex.profiles.fields.baseUrl') }}
                <span
                  v-if="requiresBaseUrl"
                  class="text-accent-danger"
                >*</span>
              </label>
              <input
                id="codex-profile-base-url"
                :value="form.base_url"
                type="text"
                class="pe-input pe-input--mono"
                :placeholder="$t('codex.profiles.placeholders.baseUrl')"
                @input="updateTextField('base_url', $event)"
              >
              <p class="pe-field__hint">
                {{ requiresBaseUrl ? $t('codex.profiles.baseUrlRequiredHint') : $t('codex.profiles.baseUrlOptionalHint') }}
              </p>
            </div>

            <div class="pe-field">
              <label
                for="codex-profile-auth-token"
                class="pe-field__label"
              >
                {{ $t('codex.profiles.fields.authToken') }}
                <span
                  v-if="requiresSecret"
                  class="text-accent-danger"
                >*</span>
              </label>
              <div class="relative">
                <input
                  id="codex-profile-auth-token"
                  :value="form.auth_token"
                  data-testid="codex-auth-token-input"
                  :type="showToken ? 'text' : 'password'"
                  class="pe-input pe-input--mono pr-20"
                  :placeholder="$t('codex.profiles.placeholders.authToken')"
                  @input="updateTextField('auth_token', $event)"
                >
                <div class="absolute right-2 top-1/2 flex -translate-y-1/2 items-center gap-1">
                  <button
                    type="button"
                    data-testid="codex-auth-token-visibility"
                    class="pe-icon-btn"
                    :title="showToken ? $t('codex.profiles.tokenActions.hide') : $t('codex.profiles.tokenActions.show')"
                    :aria-label="showToken ? $t('codex.profiles.tokenActions.hide') : $t('codex.profiles.tokenActions.show')"
                    @click="showToken = !showToken"
                  >
                    <SIcon
                      :name="showToken ? 'EyeOff' : 'Eye'"
                      size="w-3.5 h-3.5"
                    />
                  </button>
                  <button
                    type="button"
                    data-testid="codex-auth-token-copy"
                    class="pe-icon-btn"
                    :disabled="!form.auth_token.trim()"
                    :title="$t('codex.profiles.tokenActions.copy')"
                    :aria-label="$t('codex.profiles.tokenActions.copy')"
                    @click="copyToken"
                  >
                    <SIcon
                      name="Copy"
                      size="w-3.5 h-3.5"
                    />
                  </button>
                </div>
              </div>
              <p class="pe-field__hint">
                {{ authTokenHint }}
              </p>
            </div>

            <div
              v-if="requiresEnvKey"
              class="pe-field"
            >
              <label
                for="codex-profile-env-key"
                class="pe-field__label"
              >
                {{ $t('codex.profiles.fields.envKey') }} <span class="text-accent-danger">*</span>
              </label>
              <input
                id="codex-profile-env-key"
                :value="form.env_key"
                type="text"
                class="pe-input pe-input--mono"
                :placeholder="$t('codex.profiles.placeholders.envKey')"
                @input="updateTextField('env_key', $event)"
              >
              <p class="pe-field__hint">
                {{ $t('codex.profiles.envKeyHint') }}
              </p>
            </div>
          </div>
        </section>

        <section
          id="runtime"
          ref="runtimeRef"
          class="mt-5 space-y-3"
        >
          <div class="flex items-start gap-3">
            <div class="pe-panel-icon flex h-9 w-9 shrink-0 items-center justify-center">
              <SIcon
                name="Bot"
                size="w-4 h-4"
              />
            </div>
            <div class="min-w-0">
              <h3 class="pe-section__title">
                {{ $t('codex.profiles.sections.runtime') }}
              </h3>
              <p class="pe-section__desc mt-1">
                {{ $t('codex.profiles.sectionHints.runtime') }}
              </p>
            </div>
          </div>

          <div class="pe-panel space-y-4 p-4">
            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
              <div class="pe-field">
                <label
                  for="codex-profile-model"
                  class="pe-field__label"
                >
                  {{ $t('codex.profiles.fields.model') }} <span class="text-accent-danger">*</span>
                </label>
                <select
                  id="codex-profile-model"
                  :value="selectedModelOption"
                  data-testid="codex-profile-model-select"
                  class="pe-select pe-select--mono"
                  @change="emitSelectedModelOption"
                >
                  <option
                    v-for="model in modelCatalog"
                    :key="model"
                    :value="model"
                  >
                    {{ modelOptionLabel(model) }}
                  </option>
                  <option :value="CUSTOM_MODEL_OPTION">
                    {{ $t('codex.profiles.customModelOption') }}
                  </option>
                </select>
                <input
                  v-if="selectedModelOption === CUSTOM_MODEL_OPTION"
                  :value="customModelInput"
                  type="text"
                  class="pe-input pe-input--mono"
                  :placeholder="$t('codex.profiles.placeholders.customModel')"
                  @input="emitCustomModelInput"
                >
                <p class="pe-field__hint">
                  {{ selectedModelOption === CUSTOM_MODEL_OPTION ? $t('codex.profiles.customModelHint') : $t('codex.profiles.modelPresetHint') }}
                </p>
              </div>

              <div class="pe-field">
                <label
                  for="codex-profile-reasoning-effort"
                  class="pe-field__label"
                >
                  {{ $t('codex.profiles.fields.reasoningEffort') }}
                </label>
                <select
                  id="codex-profile-reasoning-effort"
                  :value="form.model_reasoning_effort"
                  data-testid="codex-reasoning-effort-select"
                  class="pe-select"
                  @change="updateSelectField('model_reasoning_effort', $event)"
                >
                  <option value="">
                    --
                  </option>
                  <option
                    v-for="effort in REASONING_EFFORT_OPTIONS"
                    :key="effort"
                    :value="effort"
                  >
                    {{ effort }}
                  </option>
                </select>
                <p class="pe-field__hint">
                  {{ $t('codex.profiles.reasoningEffortHint') }}
                </p>
              </div>
            </div>

            <div class="pe-field">
              <label
                for="codex-profile-wire-api"
                class="pe-field__label"
              >
                {{ $t('codex.profiles.fields.wireApi') }}
              </label>
              <input
                id="codex-profile-wire-api"
                :value="form.wire_api"
                type="text"
                class="pe-input pe-input--mono"
                :placeholder="$t('codex.profiles.placeholders.wireApi')"
                @input="updateTextField('wire_api', $event)"
              >
            </div>
          </div>
        </section>

        <section
          id="metadata"
          ref="metadataRef"
          class="mt-5 space-y-3"
        >
          <div class="flex items-start gap-3">
            <div class="pe-panel-icon flex h-9 w-9 shrink-0 items-center justify-center">
              <SIcon
                name="SlidersHorizontal"
                size="w-4 h-4"
              />
            </div>
            <div class="min-w-0">
              <h3 class="pe-section__title">
                {{ $t('codex.profiles.sections.metadata') }}
              </h3>
              <p class="pe-section__desc mt-1">
                {{ $t('codex.profiles.sectionHints.metadata') }}
              </p>
            </div>
          </div>

          <div class="pe-panel space-y-4 p-4">
            <div class="grid grid-cols-1 gap-4 md:grid-cols-3">
              <div class="pe-field">
                <label
                  for="codex-profile-provider"
                  class="pe-field__label"
                >
                  {{ $t('codex.profiles.fields.provider') }}
                </label>
                <input
                  id="codex-profile-provider"
                  :value="form.provider"
                  type="text"
                  class="pe-input"
                  :placeholder="$t('codex.profiles.placeholders.provider')"
                  @input="updateTextField('provider', $event)"
                >
              </div>
              <div class="pe-field">
                <label
                  for="codex-profile-provider-type"
                  class="pe-field__label"
                >
                  {{ $t('codex.profiles.fields.providerType') }}
                </label>
                <input
                  id="codex-profile-provider-type"
                  :value="form.provider_type"
                  type="text"
                  class="pe-input"
                  :placeholder="$t('codex.profiles.placeholders.providerType')"
                  @input="updateTextField('provider_type', $event)"
                >
              </div>
              <div class="pe-field">
                <label
                  for="codex-profile-tags"
                  class="pe-field__label"
                >
                  {{ $t('codex.profiles.fields.tags') }}
                </label>
                <input
                  id="codex-profile-tags"
                  :value="form.tags_input"
                  type="text"
                  class="pe-input"
                  :placeholder="$t('codex.profiles.placeholders.tags')"
                  @input="updateTextField('tags_input', $event)"
                >
              </div>
            </div>

            <div class="pe-panel-muted flex items-center justify-between gap-4 p-3">
              <div class="min-w-0">
                <p class="text-sm font-semibold text-text-primary">
                  {{ $t('codex.profiles.fields.enabled') }}
                </p>
                <p class="pe-section__desc mt-1">
                  {{ $t('codex.profiles.enabledHint') }}
                </p>
              </div>
              <label class="inline-flex shrink-0 cursor-pointer items-center gap-2">
                <input
                  :checked="form.enabled"
                  type="checkbox"
                  class="h-4 w-4 rounded border-border-default/50 text-accent-primary focus:ring-accent-primary/30"
                  @change="updateCheckboxField('enabled', $event)"
                >
                <span class="text-sm text-text-secondary">
                  {{ form.enabled ? $t('codex.states.enabled') : $t('codex.states.disabled') }}
                </span>
              </label>
            </div>
          </div>
        </section>
      </div>

      <div class="pe-footer">
        <p class="mr-auto text-xs text-text-secondary">
          {{ $t('codex.profiles.modalFooterHint') }}
        </p>
        <button
          type="button"
          class="pe-btn"
          :disabled="saving"
          @click="requestClose"
        >
          {{ $t('common.cancel') }}
        </button>
        <button
          type="button"
          class="pe-btn pe-btn--primary"
          :disabled="saving"
          @click="handleSaveClick"
        >
          <SIcon
            v-if="saving"
            name="RefreshCw"
            size="w-3.5 h-3.5"
            class="animate-spin"
          />
          {{ $t('codex.actions.save') }}
        </button>
      </div>
    </div>
  </BaseModal>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import BaseModal from '@/components/common/BaseModal.vue'
import ProviderTemplateSelector from '@/components/provider-templates/ProviderTemplateSelector.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { copyText } from '@/utils/clipboard'
import { useUIStore } from '@/stores/ui'
import type { CodexProfileAuthMode } from '@/types'
import type { ProviderTemplateDraftContext, ProviderTemplateSelection } from '@/types/providerTemplates'
import type { CodexProfileEditorForm } from '@/utils/codexProfileEditor'
import {
  CUSTOM_MODEL_OPTION,
  REASONING_EFFORT_OPTIONS,
  usesOpenAiAuthMode,
} from '@/utils/codexProfileEditor'
import '@/components/profiles/profile-editor-shell.css'

type SectionId = 'identity' | 'auth' | 'runtime' | 'metadata'

interface SectionItem {
  id: SectionId
  title: string
}

interface Props {
  modelValue: boolean
  editingName: string | null
  saving: boolean
  form: CodexProfileEditorForm
  updateField: (field: keyof CodexProfileEditorForm, value: string | boolean) => void
  availableAuthModeOptions: CodexProfileAuthMode[]
  modelCatalog: string[]
  currentModelOption?: string
  selectedModelOption: string
  customModelInput: string
  /** 视图解析后的最终模型值（预设或自定义输入），保存前校验用 */
  resolvedModel: string
  requiresBaseUrl: boolean
  requiresSecret: boolean
  requiresEnvKey: boolean
  authTokenHint: string
  isDeprecatedAuthMode: boolean
  displayOpenAiLoginMethod: string
  authModeLabel: (mode: CodexProfileAuthMode) => string
  selectedProviderTemplate?: string | null
  selectedProviderEndpoint?: string
  providerTemplateDraft?: ProviderTemplateDraftContext | null
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'update:selectedModelOption': [value: string]
  'update:customModelInput': [value: string]
  close: []
  save: []
  'select-template': [selection: ProviderTemplateSelection]
  'manual-template': []
}>()

const { t } = useI18n()
const uiStore = useUIStore()

const showToken = ref(false)
const scrollRef = ref<HTMLElement | null>(null)
const identityRef = ref<HTMLElement | null>(null)
const authRef = ref<HTMLElement | null>(null)
const runtimeRef = ref<HTMLElement | null>(null)
const metadataRef = ref<HTMLElement | null>(null)

const isEditing = computed(() => Boolean(props.editingName))

const sectionItems = computed<SectionItem[]>(() => ([
  { id: 'identity', title: t('codex.profiles.sections.identity') },
  { id: 'auth', title: t('codex.profiles.sections.authentication') },
  { id: 'runtime', title: t('codex.profiles.sections.runtime') },
  { id: 'metadata', title: t('codex.profiles.sections.metadata') },
]))

const activeSectionId = ref<SectionId>('identity')

/* ========================================================================
 * 保存前校验：失败时顶部汇总条 + 可跳转到第一个错误字段所在分段
 * ======================================================================== */

interface ValidationError {
  section: SectionId
  message: string
}

const showValidation = ref(false)

const validationErrors = computed<ValidationError[]>(() => {
  const errors: ValidationError[] = []
  if (!props.form.name.trim()) {
    errors.push({ section: 'identity', message: t('codex.profiles.validation.nameRequired') })
  }
  if (props.requiresBaseUrl && !props.form.base_url.trim()) {
    errors.push({ section: 'auth', message: t('codex.profiles.validation.baseUrlRequired') })
  }
  if (props.requiresSecret && !props.form.auth_token.trim()) {
    errors.push({ section: 'auth', message: t('codex.profiles.validation.authTokenRequired') })
  }
  if (props.requiresEnvKey && !props.form.env_key.trim()) {
    errors.push({ section: 'auth', message: t('codex.profiles.validation.envKeyRequired') })
  }
  if (!props.resolvedModel.trim()) {
    errors.push({ section: 'runtime', message: t('codex.profiles.validation.modelRequired') })
  }
  return errors
})

const handleSaveClick = () => {
  if (validationErrors.value.length > 0) {
    showValidation.value = true
    void scrollToSection(validationErrors.value[0].section)
    return
  }
  showValidation.value = false
  emit('save')
}

const emitSelectedModelOption = (event: Event) => {
  const target = event.target as HTMLSelectElement
  emit('update:selectedModelOption', target.value)
}

const emitCustomModelInput = (event: Event) => {
  const target = event.target as HTMLInputElement
  emit('update:customModelInput', target.value)
}

const modelOptionLabel = (model: string) => {
  if (model !== props.currentModelOption) return model
  return t('codex.profiles.currentModelOption', { model })
}

const updateTextField = (field: keyof CodexProfileEditorForm, event: Event) => {
  const target = event.target as HTMLInputElement
  props.updateField(field, target.value)
}

const updateSelectField = (field: keyof CodexProfileEditorForm, event: Event) => {
  const target = event.target as HTMLSelectElement
  props.updateField(field, target.value)
}

const updateCheckboxField = (field: keyof CodexProfileEditorForm, event: Event) => {
  const target = event.target as HTMLInputElement
  props.updateField(field, target.checked)
}

const requestClose = () => {
  if (props.saving) return
  emit('update:modelValue', false)
  emit('close')
}

const handleModalModelValue = (value: boolean) => {
  emit('update:modelValue', value)
  if (!value) {
    emit('close')
  }
}

const scrollToSection = async (id: SectionId) => {
  activeSectionId.value = id
  await nextTick()

  const map: Record<SectionId, HTMLElement | null> = {
    identity: identityRef.value,
    auth: authRef.value,
    runtime: runtimeRef.value,
    metadata: metadataRef.value,
  }

  // jsdom 下 scrollIntoView 未实现，缺失时静默跳过（分段高亮已由 activeSectionId 完成）
  map[id]?.scrollIntoView?.({ block: 'start', behavior: 'smooth' })
}

const syncActiveSection = () => {
  const container = scrollRef.value
  if (!container) return

  const entries: Array<{ id: SectionId; el: HTMLElement | null }> = [
    { id: 'identity', el: identityRef.value },
    { id: 'auth', el: authRef.value },
    { id: 'runtime', el: runtimeRef.value },
    { id: 'metadata', el: metadataRef.value },
  ]

  const containerTop = container.getBoundingClientRect().top
  let best: { id: SectionId; distance: number } | null = null

  for (const entry of entries) {
    if (!entry.el) continue
    const top = entry.el.getBoundingClientRect().top - containerTop
    const distance = Math.abs(top)
    if (!best || distance < best.distance) {
      best = { id: entry.id, distance }
    }
  }

  if (best) {
    activeSectionId.value = best.id
  }
}

const copyToken = async () => {
  const token = props.form.auth_token.trim()
  if (!token) return

  const ok = await copyText(token)
  if (ok) {
    uiStore.showSuccess(t('codex.profiles.messages.tokenCopied'))
  } else {
    uiStore.showError(t('codex.profiles.messages.tokenCopyFailed'))
  }
}

watch(() => props.modelValue, (isOpen) => {
  showToken.value = false
  showValidation.value = false
  if (!isOpen) return

  activeSectionId.value = 'identity'
  void nextTick(() => {
    scrollRef.value?.scrollTo({ top: 0 })
  })
})

// 模板填充后由视图驱动跳到认证段
defineExpose({ scrollToSection })
</script>
