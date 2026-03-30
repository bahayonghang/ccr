<template>
  <BaseModal
    :model-value="modelValue"
    :show-close="false"
    size="xl"
    surface="glass"
    :close-on-backdrop="!saving"
    :close-on-escape="!saving"
    :persistent="saving"
    content-class="codex-profile-editor-modal !max-w-[980px] !max-h-[90vh] rounded-[32px]"
    @update:model-value="handleModalModelValue"
  >
    <template #header="{ titleId }">
      <div class="editor-shell-header flex items-start justify-between gap-4">
        <div class="flex min-w-0 items-start gap-4">
          <div class="editor-hero-icon flex h-14 w-14 shrink-0 items-center justify-center rounded-[20px]">
            <SIcon
              name="Settings"
              size="w-7 h-7"
            />
          </div>
          <div class="min-w-0">
            <p class="editor-shell-eyebrow text-xs font-semibold uppercase tracking-[0.26em]">
              {{ isEditing ? $t('codex.profiles.editProfile') : $t('codex.profiles.addProfile') }}
            </p>
            <div class="mt-2 flex flex-wrap items-center gap-2">
              <h2
                :id="titleId"
                class="editor-shell-title text-2xl font-semibold tracking-tight"
              >
                {{ isEditing ? editingName : $t('codex.profiles.addProfile') }}
              </h2>
              <span
                class="editor-pill px-3 py-1 text-xs font-medium"
                :class="form.enabled ? 'editor-pill--success' : 'editor-pill--danger'"
              >
                {{ form.enabled ? $t('codex.states.enabled') : $t('codex.states.disabled') }}
              </span>
            </div>
            <p class="editor-shell-description mt-2 max-w-3xl text-sm leading-6">
              {{ $t('codex.profiles.subtitle') }}
            </p>
          </div>
        </div>

        <button
          type="button"
          class="editor-close-button inline-flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl transition-colors disabled:cursor-not-allowed disabled:opacity-50"
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

    <div class="flex max-h-[calc(90vh-8rem)] flex-col overflow-hidden">
      <div class="editor-nav-rail mb-4 flex flex-wrap gap-2 border-b border-border-default/35 pb-4">
        <button
          v-for="section in sectionItems"
          :key="section.id"
          type="button"
          class="editor-nav-button inline-flex min-h-[40px] items-center gap-2 rounded-full px-3.5 py-2 text-sm transition-[background-color,border-color,transform] duration-200 hover:-translate-y-px"
          :class="activeSectionId === section.id
            ? 'editor-nav-button--active'
            : 'editor-nav-button--idle'"
          @click="scrollToSection(section.id)"
        >
          <span class="editor-nav-button__icon flex h-7 w-7 items-center justify-center rounded-full">
            <SIcon
              :name="section.icon"
              size="w-3.5 h-3.5"
            />
          </span>
          {{ section.title }}
        </button>
      </div>

      <div
        ref="scrollRef"
        class="editor-scroll-area min-h-0 flex-1 overflow-y-auto pr-1"
        @scroll="syncActiveSection"
      >
        <section
          id="identity"
          ref="identityRef"
          class="editor-section space-y-4"
        >
          <div class="editor-section-head flex items-start gap-3">
            <div class="editor-section-icon flex h-10 w-10 items-center justify-center rounded-2xl">
              <SIcon
                name="Layers"
                size="w-5 h-5"
              />
            </div>
            <div class="min-w-0">
              <h3 class="text-base font-semibold text-text-primary">
                {{ $t('codex.profiles.sections.identity') }}
              </h3>
              <p class="mt-1 text-sm text-text-secondary">
                {{ $t('codex.profiles.sectionHints.identity') }}
              </p>
            </div>
          </div>

          <div class="editor-panel editor-panel-muted rounded-[28px] p-5">
            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
              <div>
                <label
                  for="codex-profile-name"
                  class="mb-1.5 ml-1 block text-xs font-semibold tracking-wide text-text-muted"
                >
                  {{ $t('codex.profiles.fields.name') }} <span class="text-red-500">*</span>
                </label>
                <input
                  id="codex-profile-name"
                  :value="form.name"
                  :disabled="isEditing"
                  type="text"
                  class="editor-input w-full rounded-[20px] px-4 py-3 text-sm"
                  :placeholder="$t('codex.profiles.placeholders.name')"
                  @input="updateTextField('name', $event)"
                >
              </div>

              <div>
                <label
                  for="codex-profile-description"
                  class="mb-1.5 ml-1 block text-xs font-semibold tracking-wide text-text-muted"
                >
                  {{ $t('codex.profiles.fields.description') }}
                </label>
                <input
                  id="codex-profile-description"
                  :value="form.description"
                  type="text"
                  class="editor-input w-full rounded-[20px] px-4 py-3 text-sm"
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
          class="editor-section mt-6 space-y-4"
        >
          <div class="editor-section-head flex items-start gap-3">
            <div class="editor-section-icon flex h-10 w-10 items-center justify-center rounded-2xl">
              <SIcon
                name="ShieldCheck"
                size="w-5 h-5"
              />
            </div>
            <div class="min-w-0">
              <h3 class="text-base font-semibold text-text-primary">
                {{ $t('codex.profiles.sections.authentication') }}
              </h3>
              <p class="mt-1 text-sm text-text-secondary">
                {{ $t('codex.profiles.sectionHints.authentication') }}
              </p>
            </div>
          </div>

          <div class="editor-panel rounded-[28px] p-5">
            <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
              <div>
                <label
                  for="codex-profile-auth-mode"
                  class="mb-1.5 ml-1 block text-xs font-semibold tracking-wide text-text-muted"
                >
                  {{ $t('codex.profiles.fields.authMode') }} <span class="text-red-500">*</span>
                </label>
                <select
                  id="codex-profile-auth-mode"
                  :value="form.auth_mode"
                  class="editor-input w-full rounded-[20px] px-4 py-3 text-sm"
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
                  class="mt-2 text-xs text-amber-300"
                >
                  {{ $t('codex.profiles.deprecatedAuthModeHint', { mode: authModeLabel(form.auth_mode) }) }}
                </p>
              </div>

              <div class="editor-panel-muted rounded-[20px] p-4">
                <p class="text-xs font-semibold uppercase tracking-[0.24em] text-text-muted">
                  {{ $t('codex.profiles.fields.openAiLoginMethod') }}
                </p>
                <p class="mt-2 flex flex-wrap items-center gap-2 text-sm text-text-secondary">
                  <span class="editor-inline-chip rounded-full px-3 py-1 text-xs font-medium">
                    {{ displayOpenAiLoginMethod }}
                  </span>
                  <span class="editor-inline-chip rounded-full px-3 py-1 text-xs font-medium">
                    {{ usesOpenAiAuthMode(form.auth_mode) ? $t('codex.profiles.openAiAuthOn') : $t('codex.profiles.openAiAuthOff') }}
                  </span>
                </p>
              </div>
            </div>

            <div class="mt-5">
              <label
                for="codex-profile-base-url"
                class="mb-1.5 ml-1 block text-xs font-semibold tracking-wide text-text-muted"
              >
                {{ $t('codex.profiles.fields.baseUrl') }}
                <span
                  v-if="requiresBaseUrl"
                  class="text-red-500"
                >*</span>
              </label>
              <input
                id="codex-profile-base-url"
                :value="form.base_url"
                type="text"
                class="editor-input editor-input--mono w-full rounded-[20px] px-4 py-3 text-sm"
                :placeholder="$t('codex.profiles.placeholders.baseUrl')"
                @input="updateTextField('base_url', $event)"
              >
              <p class="mt-2 text-xs text-text-muted">
                {{ requiresBaseUrl ? $t('codex.profiles.baseUrlRequiredHint') : $t('codex.profiles.baseUrlOptionalHint') }}
              </p>
            </div>

            <div class="mt-5">
              <label
                for="codex-profile-auth-token"
                class="mb-1.5 ml-1 block text-xs font-semibold tracking-wide text-text-muted"
              >
                {{ $t('codex.profiles.fields.authToken') }}
                <span
                  v-if="requiresSecret"
                  class="text-red-500"
                >*</span>
              </label>
              <div class="relative">
                <input
                  id="codex-profile-auth-token"
                  :value="form.auth_token"
                  data-testid="codex-auth-token-input"
                  :type="showToken ? 'text' : 'password'"
                  class="editor-input editor-input--mono w-full rounded-[20px] px-4 py-3 pr-24 text-sm"
                  :placeholder="$t('codex.profiles.placeholders.authToken')"
                  @input="updateTextField('auth_token', $event)"
                >
                <div class="absolute right-3 top-1/2 flex -translate-y-1/2 items-center gap-1">
                  <button
                    type="button"
                    data-testid="codex-auth-token-visibility"
                    class="editor-icon-button"
                    :title="showToken ? $t('codex.profiles.tokenActions.hide') : $t('codex.profiles.tokenActions.show')"
                    @click="showToken = !showToken"
                  >
                    <SIcon
                      :name="showToken ? 'EyeOff' : 'Eye'"
                      size="w-4 h-4"
                    />
                  </button>
                  <button
                    type="button"
                    data-testid="codex-auth-token-copy"
                    class="editor-icon-button"
                    :disabled="!form.auth_token.trim()"
                    :title="$t('codex.profiles.tokenActions.copy')"
                    @click="copyToken"
                  >
                    <SIcon
                      name="Copy"
                      size="w-4 h-4"
                    />
                  </button>
                </div>
              </div>
              <p class="mt-2 text-xs text-text-muted">
                {{ authTokenHint }}
              </p>
            </div>

            <div
              v-if="requiresEnvKey"
              class="mt-5"
            >
              <label
                for="codex-profile-env-key"
                class="mb-1.5 ml-1 block text-xs font-semibold tracking-wide text-text-muted"
              >
                {{ $t('codex.profiles.fields.envKey') }} <span class="text-red-500">*</span>
              </label>
              <input
                id="codex-profile-env-key"
                :value="form.env_key"
                type="text"
                class="editor-input editor-input--mono w-full rounded-[20px] px-4 py-3 text-sm"
                :placeholder="$t('codex.profiles.placeholders.envKey')"
                @input="updateTextField('env_key', $event)"
              >
              <p class="mt-2 text-xs text-text-muted">
                {{ $t('codex.profiles.envKeyHint') }}
              </p>
            </div>
          </div>
        </section>

        <section
          id="runtime"
          ref="runtimeRef"
          class="editor-section mt-6 space-y-4"
        >
          <div class="editor-section-head flex items-start gap-3">
            <div class="editor-section-icon flex h-10 w-10 items-center justify-center rounded-2xl">
              <SIcon
                name="Bot"
                size="w-5 h-5"
              />
            </div>
            <div class="min-w-0">
              <h3 class="text-base font-semibold text-text-primary">
                {{ $t('codex.profiles.sections.runtime') }}
              </h3>
              <p class="mt-1 text-sm text-text-secondary">
                {{ $t('codex.profiles.sectionHints.runtime') }}
              </p>
            </div>
          </div>

          <div class="editor-panel rounded-[28px] p-5">
            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
              <div>
                <label
                  for="codex-profile-model"
                  class="mb-1.5 ml-1 block text-xs font-semibold tracking-wide text-text-muted"
                >
                  {{ $t('codex.profiles.fields.model') }} <span class="text-red-500">*</span>
                </label>
                <select
                  id="codex-profile-model"
                  :value="selectedModelOption"
                  class="editor-input editor-input--mono w-full rounded-[20px] px-4 py-3 text-sm"
                  @change="emitSelectedModelOption"
                >
                  <option
                    v-for="model in modelCatalog"
                    :key="model"
                    :value="model"
                  >
                    {{ model }}
                  </option>
                  <option :value="CUSTOM_MODEL_OPTION">
                    {{ $t('codex.profiles.customModelOption') }}
                  </option>
                </select>
                <input
                  v-if="selectedModelOption === CUSTOM_MODEL_OPTION"
                  :value="customModelInput"
                  type="text"
                  class="editor-input editor-input--mono mt-3 w-full rounded-[20px] px-4 py-3 text-sm"
                  :placeholder="$t('codex.profiles.placeholders.customModel')"
                  @input="emitCustomModelInput"
                >
                <p class="mt-2 text-xs text-text-muted">
                  {{ selectedModelOption === CUSTOM_MODEL_OPTION ? $t('codex.profiles.customModelHint') : $t('codex.profiles.modelPresetHint') }}
                </p>
              </div>

              <div>
                <label
                  for="codex-profile-reasoning-effort"
                  class="mb-1.5 ml-1 block text-xs font-semibold tracking-wide text-text-muted"
                >
                  {{ $t('codex.profiles.fields.reasoningEffort') }}
                </label>
                <select
                  id="codex-profile-reasoning-effort"
                  :value="form.model_reasoning_effort"
                  data-testid="codex-reasoning-effort-select"
                  class="editor-input w-full rounded-[20px] px-4 py-3 text-sm"
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
                <p class="mt-2 text-xs text-text-muted">
                  {{ $t('codex.profiles.reasoningEffortHint') }}
                </p>
              </div>
            </div>

            <div class="mt-5">
              <label
                for="codex-profile-wire-api"
                class="mb-1.5 ml-1 block text-xs font-semibold tracking-wide text-text-muted"
              >
                {{ $t('codex.profiles.fields.wireApi') }}
              </label>
              <input
                id="codex-profile-wire-api"
                :value="form.wire_api"
                type="text"
                class="editor-input editor-input--mono w-full rounded-[20px] px-4 py-3 text-sm"
                :placeholder="$t('codex.profiles.placeholders.wireApi')"
                @input="updateTextField('wire_api', $event)"
              >
            </div>
          </div>
        </section>

        <section
          id="metadata"
          ref="metadataRef"
          class="editor-section mt-6 space-y-4"
        >
          <div class="editor-section-head flex items-start gap-3">
            <div class="editor-section-icon flex h-10 w-10 items-center justify-center rounded-2xl">
              <SIcon
                name="SlidersHorizontal"
                size="w-5 h-5"
              />
            </div>
            <div class="min-w-0">
              <h3 class="text-base font-semibold text-text-primary">
                {{ $t('codex.profiles.sections.metadata') }}
              </h3>
              <p class="mt-1 text-sm text-text-secondary">
                {{ $t('codex.profiles.sectionHints.metadata') }}
              </p>
            </div>
          </div>

          <div class="editor-panel rounded-[28px] p-5">
            <div class="grid grid-cols-1 gap-4 md:grid-cols-3">
              <div>
                <label
                  for="codex-profile-provider"
                  class="mb-1.5 ml-1 block text-xs font-semibold tracking-wide text-text-muted"
                >
                  {{ $t('codex.profiles.fields.provider') }}
                </label>
                <input
                  id="codex-profile-provider"
                  :value="form.provider"
                  type="text"
                  class="editor-input w-full rounded-[20px] px-4 py-3 text-sm"
                  :placeholder="$t('codex.profiles.placeholders.provider')"
                  @input="updateTextField('provider', $event)"
                >
              </div>
              <div>
                <label
                  for="codex-profile-provider-type"
                  class="mb-1.5 ml-1 block text-xs font-semibold tracking-wide text-text-muted"
                >
                  {{ $t('codex.profiles.fields.providerType') }}
                </label>
                <input
                  id="codex-profile-provider-type"
                  :value="form.provider_type"
                  type="text"
                  class="editor-input w-full rounded-[20px] px-4 py-3 text-sm"
                  :placeholder="$t('codex.profiles.placeholders.providerType')"
                  @input="updateTextField('provider_type', $event)"
                >
              </div>
              <div>
                <label
                  for="codex-profile-tags"
                  class="mb-1.5 ml-1 block text-xs font-semibold tracking-wide text-text-muted"
                >
                  {{ $t('codex.profiles.fields.tags') }}
                </label>
                <input
                  id="codex-profile-tags"
                  :value="form.tags_input"
                  type="text"
                  class="editor-input w-full rounded-[20px] px-4 py-3 text-sm"
                  :placeholder="$t('codex.profiles.placeholders.tags')"
                  @input="updateTextField('tags_input', $event)"
                >
              </div>
            </div>

            <div class="mt-5 flex items-center justify-between rounded-[20px] border border-border-default/35 bg-bg-elevated/34 px-4 py-3">
              <div>
                <p class="text-sm font-semibold text-text-primary">
                  {{ $t('codex.profiles.fields.enabled') }}
                </p>
                <p class="mt-1 text-xs text-text-muted">
                  {{ $t('codex.profiles.enabledHint') }}
                </p>
              </div>
              <label class="inline-flex cursor-pointer items-center gap-3">
                <input
                  :checked="form.enabled"
                  type="checkbox"
                  class="h-5 w-5 rounded border-border-default/50 text-platform-codex focus:ring-platform-codex/30"
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

      <div class="editor-footer mt-5 flex flex-col gap-3 pt-4 sm:flex-row sm:items-center sm:justify-between">
        <p class="text-sm text-text-secondary">
          {{ $t('codex.profiles.modalFooterHint') }}
        </p>
        <div class="flex items-center justify-end gap-3">
          <button
            type="button"
            class="editor-button editor-button--secondary min-h-[44px] rounded-2xl px-5 py-2.5 text-sm disabled:cursor-not-allowed disabled:opacity-50"
            :disabled="saving"
            @click="requestClose"
          >
            {{ $t('common.cancel') }}
          </button>
          <button
            type="button"
            class="editor-button editor-button--primary min-h-[44px] rounded-2xl px-5 py-2.5 text-sm font-medium disabled:cursor-not-allowed disabled:opacity-50"
            :disabled="saving || !form.name.trim()"
            @click="$emit('save')"
          >
            <span class="inline-flex items-center gap-2">
              <SIcon
                v-if="saving"
                name="RefreshCw"
                size="w-4 h-4"
                class="animate-spin"
              />
              {{ $t('codex.actions.save') }}
            </span>
          </button>
        </div>
      </div>
    </div>
  </BaseModal>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import BaseModal from '@/components/common/BaseModal.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { copyToClipboard } from '@/utils/codexHelpers'
import { useUIStore } from '@/stores/ui'
import type { CodexProfileAuthMode } from '@/types'
import type { CodexProfileEditorForm } from '@/utils/codexProfileEditor'
import {
  CUSTOM_MODEL_OPTION,
  REASONING_EFFORT_OPTIONS,
  usesOpenAiAuthMode,
} from '@/utils/codexProfileEditor'

interface SectionItem {
  id: 'identity' | 'auth' | 'runtime' | 'metadata'
  title: string
  icon: string
}

interface Props {
  modelValue: boolean
  editingName: string | null
  saving: boolean
  form: CodexProfileEditorForm
  updateField: (field: keyof CodexProfileEditorForm, value: string | boolean) => void
  availableAuthModeOptions: CodexProfileAuthMode[]
  modelCatalog: string[]
  selectedModelOption: string
  customModelInput: string
  requiresBaseUrl: boolean
  requiresSecret: boolean
  requiresEnvKey: boolean
  authTokenHint: string
  isDeprecatedAuthMode: boolean
  displayOpenAiLoginMethod: string
  authModeLabel: (mode: CodexProfileAuthMode) => string
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'update:selectedModelOption': [value: string]
  'update:customModelInput': [value: string]
  close: []
  save: []
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
  { id: 'identity', title: t('codex.profiles.sections.identity'), icon: 'Layers' },
  { id: 'auth', title: t('codex.profiles.sections.authentication'), icon: 'ShieldCheck' },
  { id: 'runtime', title: t('codex.profiles.sections.runtime'), icon: 'Bot' },
  { id: 'metadata', title: t('codex.profiles.sections.metadata'), icon: 'SlidersHorizontal' },
]))

const activeSectionId = ref<SectionItem['id']>('identity')

const emitSelectedModelOption = (event: Event) => {
  const target = event.target as HTMLSelectElement
  emit('update:selectedModelOption', target.value)
}

const emitCustomModelInput = (event: Event) => {
  const target = event.target as HTMLInputElement
  emit('update:customModelInput', target.value)
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

const scrollToSection = async (id: SectionItem['id']) => {
  activeSectionId.value = id
  await nextTick()

  const map: Record<SectionItem['id'], HTMLElement | null> = {
    identity: identityRef.value,
    auth: authRef.value,
    runtime: runtimeRef.value,
    metadata: metadataRef.value,
  }

  map[id]?.scrollIntoView({ block: 'start', behavior: 'smooth' })
}

const syncActiveSection = () => {
  const container = scrollRef.value
  if (!container) return

  const entries: Array<{ id: SectionItem['id']; el: HTMLElement | null }> = [
    { id: 'identity', el: identityRef.value },
    { id: 'auth', el: authRef.value },
    { id: 'runtime', el: runtimeRef.value },
    { id: 'metadata', el: metadataRef.value },
  ]

  const containerTop = container.getBoundingClientRect().top
  let best: { id: SectionItem['id']; distance: number } | null = null

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

  const ok = await copyToClipboard(token)
  if (ok) {
    uiStore.showSuccess(t('codex.profiles.messages.tokenCopied'))
  } else {
    uiStore.showError(t('codex.profiles.messages.tokenCopyFailed'))
  }
}

watch(() => props.modelValue, (isOpen) => {
  if (!isOpen) {
    showToken.value = false
    return
  }

  showToken.value = false
  activeSectionId.value = 'identity'
  void nextTick(() => {
    scrollRef.value?.scrollTo({ top: 0 })
  })
})
</script>

<style>
.codex-profile-editor-modal {
  --editor-shell-bg: linear-gradient(180deg, rgb(255 253 253 / 96%), rgb(246 255 251 / 92%));
  --editor-shell-border: rgb(var(--color-border-default-rgb) / 82%);
  --editor-shell-shadow: 0 28px 80px rgb(40 160 120 / 10%), 0 12px 32px rgb(104 70 123 / 10%);
  --editor-shell-highlight:
    radial-gradient(circle at top right, rgb(var(--color-platform-codex-rgb) / 12%), transparent 40%),
    radial-gradient(circle at top left, rgb(var(--color-accent-primary-rgb) / 10%), transparent 32%);
  --editor-panel-bg: rgb(255 254 254 / 82%);
  --editor-panel-muted-bg: linear-gradient(180deg, rgb(250 255 252 / 96%), rgb(242 252 247 / 90%));
  --editor-input-bg: rgb(244 252 247 / 94%);
  --editor-input-bg-hover: rgb(251 255 253 / 98%);
  --editor-input-bg-focus: rgb(255 255 255 / 100%);
  --editor-input-border: rgb(var(--color-border-default-rgb) / 84%);
  --editor-input-border-strong: rgb(var(--color-platform-codex-rgb) / 34%);
  --editor-hairline: rgb(var(--color-border-default-rgb) / 72%);
  --editor-hairline-soft: rgb(var(--color-border-default-rgb) / 46%);
  --editor-ink: rgb(var(--color-text-primary-rgb) / 96%);
  --editor-ink-muted: rgb(var(--color-text-secondary-rgb) / 90%);
  --editor-ink-soft: rgb(var(--color-text-muted-rgb) / 86%);
  --editor-placeholder: rgb(var(--color-text-muted-rgb) / 74%);
  --editor-panel-shadow: 0 20px 48px rgb(40 160 120 / 8%), inset 0 1px 0 rgb(255 255 255 / 64%);
  --editor-muted-shadow: 0 14px 34px rgb(40 160 120 / 6%), inset 0 1px 0 rgb(255 255 255 / 42%);
  --editor-ring: 0 0 0 3px rgb(var(--color-platform-codex-rgb) / 14%);
  --editor-scrollbar-thumb: rgb(var(--color-platform-codex-rgb) / 34%);
  --editor-scrollbar-track: rgb(var(--color-bg-overlay-rgb) / 30%);

  position: relative;
  isolation: isolate;
  overflow: hidden;
  background: var(--editor-shell-bg) !important;
  border: 1px solid var(--editor-shell-border) !important;
  box-shadow: var(--editor-shell-shadow) !important;
  color: var(--editor-ink);
}

:root[class~='dark'] .codex-profile-editor-modal,
[data-theme='dark'] .codex-profile-editor-modal {
  --editor-shell-bg: linear-gradient(180deg, rgb(23 18 30 / 96%), rgb(16 14 24 / 94%));
  --editor-shell-border: rgb(96 160 134 / 32%);
  --editor-shell-shadow: 0 32px 90px rgb(8 4 12 / 58%), 0 18px 42px rgb(17 10 24 / 42%);
  --editor-shell-highlight:
    radial-gradient(circle at top right, rgb(var(--color-platform-codex-rgb) / 16%), transparent 44%),
    radial-gradient(circle at top left, rgb(var(--color-accent-primary-rgb) / 11%), transparent 34%);
  --editor-panel-bg: linear-gradient(180deg, rgb(36 30 47 / 86%), rgb(28 24 40 / 82%));
  --editor-panel-muted-bg: linear-gradient(180deg, rgb(44 38 56 / 90%), rgb(34 30 48 / 84%));
  --editor-input-bg: rgb(50 44 66 / 88%);
  --editor-input-bg-hover: rgb(58 52 74 / 92%);
  --editor-input-bg-focus: rgb(64 58 82 / 96%);
  --editor-input-border: rgb(96 160 134 / 34%);
  --editor-input-border-strong: rgb(var(--color-platform-codex-rgb) / 44%);
  --editor-hairline: rgb(96 160 134 / 30%);
  --editor-hairline-soft: rgb(96 160 134 / 22%);
  --editor-ink: rgb(245 255 251 / 98%);
  --editor-ink-muted: rgb(194 232 216 / 86%);
  --editor-ink-soft: rgb(162 206 188 / 78%);
  --editor-placeholder: rgb(162 206 188 / 68%);
  --editor-panel-shadow: 0 24px 60px rgb(6 3 10 / 42%), inset 0 1px 0 rgb(255 255 255 / 6%);
  --editor-muted-shadow: 0 16px 40px rgb(6 3 10 / 34%), inset 0 1px 0 rgb(255 255 255 / 5%);
  --editor-ring: 0 0 0 3px rgb(var(--color-platform-codex-rgb) / 18%);
  --editor-scrollbar-thumb: rgb(var(--color-platform-codex-rgb) / 48%);
  --editor-scrollbar-track: rgb(23 18 30 / 36%);
}

.codex-profile-editor-modal::before {
  content: '';
  position: absolute;
  inset: 0;
  background: var(--editor-shell-highlight);
  pointer-events: none;
  z-index: 0;
}

.codex-profile-editor-modal > * {
  position: relative;
  z-index: 1;
}

.codex-profile-editor-modal .text-text-primary {
  color: var(--editor-ink) !important;
}

.codex-profile-editor-modal .text-text-secondary {
  color: var(--editor-ink-muted) !important;
}

.codex-profile-editor-modal .text-text-muted {
  color: var(--editor-ink-soft) !important;
}

.codex-profile-editor-modal .editor-shell-header,
.codex-profile-editor-modal .editor-footer {
  border-color: var(--editor-hairline-soft);
}

.codex-profile-editor-modal .editor-hero-icon,
.codex-profile-editor-modal .editor-section-icon {
  background: rgb(var(--color-platform-codex-rgb) / 12%);
  color: rgb(var(--color-platform-codex-rgb) / 92%);
  box-shadow: 0 12px 24px rgb(var(--color-platform-codex-rgb) / 12%);
}

.codex-profile-editor-modal .editor-shell-eyebrow {
  color: rgb(var(--color-platform-codex-rgb) / 92%);
}

.codex-profile-editor-modal .editor-close-button,
.codex-profile-editor-modal .editor-nav-button,
.codex-profile-editor-modal .editor-icon-button,
.codex-profile-editor-modal .editor-button,
.codex-profile-editor-modal .editor-inline-chip {
  border: 1px solid var(--editor-hairline-soft);
  background: rgb(var(--color-bg-elevated-rgb) / 56%);
  color: var(--editor-ink-muted);
}

.codex-profile-editor-modal .editor-close-button:hover,
.codex-profile-editor-modal .editor-nav-button:hover,
.codex-profile-editor-modal .editor-icon-button:hover,
.codex-profile-editor-modal .editor-button:hover {
  border-color: var(--editor-hairline);
  background: rgb(var(--color-bg-elevated-rgb) / 78%);
  color: var(--editor-ink);
}

.codex-profile-editor-modal .editor-nav-button__icon {
  border: 1px solid var(--editor-hairline-soft);
  background: rgb(var(--color-bg-elevated-rgb) / 50%);
  color: var(--editor-ink-soft);
}

.codex-profile-editor-modal .editor-nav-button--active {
  border-color: rgb(var(--color-platform-codex-rgb) / 34%);
  background: linear-gradient(180deg, rgb(var(--color-platform-codex-rgb) / 12%), rgb(var(--color-platform-codex-rgb) / 8%));
  color: var(--editor-ink);
  box-shadow: 0 14px 32px rgb(var(--color-platform-codex-rgb) / 12%);
}

.codex-profile-editor-modal .editor-nav-button--active .editor-nav-button__icon {
  border-color: rgb(var(--color-platform-codex-rgb) / 30%);
  background: rgb(var(--color-platform-codex-rgb) / 14%);
  color: rgb(var(--color-platform-codex-rgb) / 98%);
}

.codex-profile-editor-modal .editor-panel {
  border: 1px solid var(--editor-hairline);
  background: var(--editor-panel-bg);
  box-shadow: var(--editor-panel-shadow);
  backdrop-filter: blur(20px) saturate(135%);
}

.codex-profile-editor-modal .editor-panel-muted {
  border: 1px solid var(--editor-hairline-soft);
  background: var(--editor-panel-muted-bg);
  box-shadow: var(--editor-muted-shadow);
}

.codex-profile-editor-modal .editor-scroll-area {
  scrollbar-color: var(--editor-scrollbar-thumb) var(--editor-scrollbar-track);
}

.codex-profile-editor-modal .editor-scroll-area::-webkit-scrollbar {
  width: 10px;
}

.codex-profile-editor-modal .editor-scroll-area::-webkit-scrollbar-track {
  background: var(--editor-scrollbar-track);
  border-radius: 999px;
}

.codex-profile-editor-modal .editor-scroll-area::-webkit-scrollbar-thumb {
  background: var(--editor-scrollbar-thumb);
  border-radius: 999px;
}

.codex-profile-editor-modal .editor-input {
  border: 1px solid var(--editor-input-border);
  background: var(--editor-input-bg);
  color: var(--editor-ink);
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 5%);
  transition: border-color 180ms ease, background-color 180ms ease, box-shadow 180ms ease, color 180ms ease;
}

.codex-profile-editor-modal .editor-input::placeholder {
  color: var(--editor-placeholder);
}

.codex-profile-editor-modal .editor-input:hover {
  border-color: var(--editor-hairline);
  background: var(--editor-input-bg-hover);
}

.codex-profile-editor-modal .editor-input:focus {
  border-color: var(--editor-input-border-strong);
  background: var(--editor-input-bg-focus);
  outline: none;
  box-shadow: var(--editor-ring);
}

.codex-profile-editor-modal .editor-input--mono {
  font-family: var(--font-mono);
  letter-spacing: 0.01em;
}

.codex-profile-editor-modal .editor-icon-button {
  display: inline-flex;
  height: 36px;
  width: 36px;
  align-items: center;
  justify-content: center;
  border-radius: 14px;
}

.codex-profile-editor-modal .editor-icon-button:hover,
.codex-profile-editor-modal .editor-button:hover {
  transform: translateY(-1px);
}

.codex-profile-editor-modal .editor-icon-button:disabled,
.codex-profile-editor-modal .editor-button:disabled,
.codex-profile-editor-modal .editor-close-button:disabled {
  cursor: not-allowed;
  opacity: 0.55;
  transform: none;
}

.codex-profile-editor-modal .editor-pill {
  border: 1px solid transparent;
}

.codex-profile-editor-modal .editor-pill--success {
  border-color: rgb(var(--color-success-rgb) / 20%);
  background: rgb(var(--color-success-rgb) / 14%);
  color: rgb(var(--color-success-rgb) / 100%);
}

.codex-profile-editor-modal .editor-pill--danger {
  border-color: rgb(var(--color-danger-rgb) / 22%);
  background: rgb(var(--color-danger-rgb) / 12%);
  color: rgb(var(--color-danger-rgb) / 100%);
}

.codex-profile-editor-modal .editor-button--primary {
  border-color: rgb(var(--color-platform-codex-rgb) / 28%);
  background: linear-gradient(180deg, rgb(var(--color-platform-codex-rgb) / 18%), rgb(var(--color-platform-codex-rgb) / 10%));
  color: rgb(var(--color-platform-codex-rgb) / 98%);
}
</style>
