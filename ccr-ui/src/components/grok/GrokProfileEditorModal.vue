<!-- Grok Profile 编辑器模态。
     外壳对齐 Claude/Codex：pe-modal 头 + 限高 pe-shell + pe-nav + pe-scroll + 壳内 pe-footer。
     视图保留表单状态与 buildGrokPatch；模态负责段导航 scroll-spy、保存前校验汇总与写入口展示。 -->
<template>
  <BaseModal
    :model-value="modelValue"
    :persistent="saving"
    :show-close="false"
    size="3xl"
    :scrollable="false"
    content-class="pe-modal grok-profile-editor"
    :close-on-backdrop="false"
    @update:model-value="requestClose"
  >
    <template #header="{ titleId }">
      <div class="pe-modal__head">
        <div class="flex min-w-0 items-start gap-4">
          <div class="pe-panel-icon flex h-12 w-12 shrink-0 items-center justify-center">
            <SIcon
              name="Fingerprint"
              size="w-6 h-6"
            />
          </div>
          <div class="min-w-0">
            <p class="pe-modal__eyebrow">
              {{ eyebrow }}
            </p>
            <div class="mt-1.5 flex flex-wrap items-center gap-2">
              <h2
                :id="titleId"
                class="pe-modal__title"
              >
                {{ title }}
              </h2>
              <span
                class="pe-pill"
                :class="form.enabled ? 'pe-pill--accent' : 'pe-pill--danger'"
              >
                {{ form.enabled ? t('grok.profiles.groups.enabled') : t('grok.profiles.groups.disabled') }}
              </span>
            </div>
            <p class="pe-modal__desc mt-1.5 max-w-3xl">
              {{ t('grok.profiles.subtitle') }}
            </p>
          </div>
        </div>

        <button
          type="button"
          class="pe-icon-btn shrink-0"
          :aria-label="t('common.close')"
          :disabled="saving"
          @click="requestClose(false)"
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
        v-if="error || (showValidation && validationErrors.length > 0)"
        class="pe-summary"
        role="alert"
      >
        <SIcon
          name="AlertTriangle"
          size="w-4 h-4"
        />
        <span>{{ error || validationErrors[0].message }}</span>
        <button
          v-if="!error && showValidation && validationErrors.length > 0"
          type="button"
          class="pe-summary__jump"
          @click="scrollToSection(validationErrors[0].section)"
        >
          {{ t('grok.profiles.editor.validationJump') }}
        </button>
      </div>

      <div
        class="pe-nav"
        role="tablist"
        :aria-label="t('grok.profiles.editor.identity')"
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
      >
        <section
          id="identity"
          ref="identityRef"
          class="space-y-3"
        >
          <div class="flex items-start gap-3">
            <div class="pe-panel-icon flex h-9 w-9 shrink-0 items-center justify-center">
              <SIcon
                name="Fingerprint"
                size="w-4 h-4"
              />
            </div>
            <div class="min-w-0">
              <h3 class="pe-section__title">
                {{ t('grok.profiles.editor.identity') }}
              </h3>
              <p class="pe-section__desc mt-1">
                {{ t('grok.profiles.editor.identityHint') }}
              </p>
            </div>
          </div>

          <div class="pe-panel space-y-4 p-4">
            <div
              class="grok-kind-control"
              role="group"
              :aria-label="t('grok.profiles.fields.profileKind')"
            >
              <button
                v-for="kind in profileKinds"
                :key="kind"
                type="button"
                :class="['grok-kind-control__button', { 'grok-kind-control__button--active': form.profileKind === kind }]"
                :disabled="Boolean(editingName)"
                @click="setField('profileKind', kind)"
              >
                {{ t(`grok.profiles.profileKinds.${kind}`) }}
              </button>
            </div>
            <p
              v-if="editingName"
              class="pe-field__hint"
            >
              {{ t('grok.profiles.editor.kindLocked') }}
            </p>

            <div
              class="grid grid-cols-1 gap-4"
              :class="{ 'md:grid-cols-2': isThirdParty }"
            >
              <div class="pe-field">
                <label
                  class="pe-field__label"
                  for="grok-profile-name"
                >
                  {{ t('grok.profiles.fields.name') }} <span class="text-accent-danger">*</span>
                </label>
                <input
                  id="grok-profile-name"
                  :value="form.name"
                  class="pe-input pe-input--mono"
                  autocomplete="off"
                  @input="setTextField('name', $event)"
                >
              </div>
              <div
                v-if="isThirdParty"
                class="pe-field"
              >
                <label
                  class="pe-field__label"
                  for="grok-profile-provider"
                >
                  {{ t('grok.profiles.fields.provider') }}
                </label>
                <input
                  id="grok-profile-provider"
                  :value="form.provider"
                  class="pe-input"
                  autocomplete="off"
                  @input="setTextField('provider', $event)"
                >
              </div>
            </div>

            <div class="pe-field">
              <label
                class="pe-field__label"
                for="grok-profile-description"
              >
                {{ t('grok.profiles.fields.description') }}
              </label>
              <textarea
                id="grok-profile-description"
                :value="form.description"
                class="pe-input min-h-20 resize-y"
                @input="setTextField('description', $event)"
              />
            </div>
          </div>
        </section>

        <section
          v-if="isThirdParty"
          id="connection"
          ref="connectionRef"
          class="mt-5 space-y-3"
        >
          <div class="flex items-start gap-3">
            <div class="pe-panel-icon flex h-9 w-9 shrink-0 items-center justify-center">
              <SIcon
                name="KeyRound"
                size="w-4 h-4"
              />
            </div>
            <div class="min-w-0">
              <h3 class="pe-section__title">
                {{ t('grok.profiles.editor.connection') }}
              </h3>
              <p class="pe-section__desc mt-1">
                {{ t('grok.profiles.editor.writeOnlyHint') }}
              </p>
            </div>
          </div>

          <div class="pe-panel space-y-4 p-4">
            <div class="pe-field">
              <label
                class="pe-field__label"
                for="grok-profile-base-url"
              >
                {{ t('grok.profiles.fields.baseUrl') }} <span class="text-accent-danger">*</span>
              </label>
              <input
                id="grok-profile-base-url"
                :value="form.baseUrl"
                class="pe-input pe-input--mono"
                :placeholder="baseUrlPlaceholder"
                autocomplete="off"
                @input="setTextField('baseUrl', $event)"
              >
              <p class="pe-field__hint">
                {{ editingName ? t('grok.profiles.editor.keepBaseUrl') : t('grok.profiles.editor.baseUrlHint') }}
              </p>
            </div>

            <div class="grok-credential-status pe-panel-muted">
              <div>
                <span>{{ t('grok.profiles.editor.currentCredential') }}</span>
                <strong>{{ currentAuthLabel }}</strong>
              </div>
              <code v-if="currentEnvKey">{{ currentEnvKey }}</code>
            </div>

            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
              <div class="pe-field">
                <label
                  class="pe-field__label"
                  for="grok-credential-action"
                >
                  {{ t('grok.profiles.fields.credentialAction') }}
                </label>
                <select
                  id="grok-credential-action"
                  :value="form.credentialAction"
                  class="pe-select"
                  @change="setSelectField('credentialAction', $event)"
                >
                  <option
                    v-for="action in credentialActions"
                    :key="action"
                    :value="action"
                  >
                    {{ t(`grok.profiles.credentialActions.${action}`) }}
                  </option>
                </select>
              </div>

              <div
                v-if="form.credentialAction === 'replace_api_key'"
                class="pe-field"
              >
                <label
                  class="pe-field__label"
                  for="grok-profile-api-key"
                >
                  {{ t('grok.profiles.fields.apiKey') }}
                </label>
                <input
                  id="grok-profile-api-key"
                  :value="form.apiKey"
                  type="password"
                  class="pe-input pe-input--mono"
                  autocomplete="new-password"
                  @input="setTextField('apiKey', $event)"
                >
              </div>

              <div
                v-if="form.credentialAction === 'replace_env_key'"
                class="pe-field"
              >
                <label
                  class="pe-field__label"
                  for="grok-profile-env-key"
                >
                  {{ t('grok.profiles.fields.envKey') }}
                </label>
                <input
                  id="grok-profile-env-key"
                  :value="form.envKey"
                  class="pe-input pe-input--mono"
                  placeholder="GROK_API_KEY"
                  autocomplete="off"
                  @input="setTextField('envKey', $event)"
                >
              </div>
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
                {{ t('grok.profiles.editor.runtime') }}
              </h3>
              <p class="pe-section__desc mt-1">
                {{ t('grok.profiles.editor.runtimeHint') }}
              </p>
            </div>
          </div>

          <div class="pe-panel space-y-4 p-4">
            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
              <div class="pe-field">
                <label
                  class="pe-field__label"
                  for="grok-profile-model"
                >
                  {{ t('grok.profiles.fields.model') }}
                  <span
                    v-if="isThirdParty"
                    class="text-accent-danger"
                  >*</span>
                </label>
                <input
                  id="grok-profile-model"
                  :value="form.model"
                  class="pe-input pe-input--mono"
                  autocomplete="off"
                  @input="setTextField('model', $event)"
                >
              </div>
              <div class="pe-field">
                <label
                  class="pe-field__label"
                  for="grok-profile-reasoning"
                >
                  {{ t('grok.profiles.fields.reasoningEffort') }}
                </label>
                <select
                  id="grok-profile-reasoning"
                  :value="form.reasoningEffort"
                  class="pe-select"
                  @change="setSelectField('reasoningEffort', $event)"
                >
                  <option value="">
                    {{ t('grok.profiles.editor.notSet') }}
                  </option>
                  <option
                    v-for="effort in GROK_REASONING_EFFORT_OPTIONS"
                    :key="effort"
                    :value="effort"
                  >
                    {{ effort }}
                  </option>
                </select>
              </div>
            </div>

            <div
              v-if="isThirdParty"
              class="grid grid-cols-1 gap-4 md:grid-cols-2"
            >
              <div class="pe-field">
                <label
                  class="pe-field__label"
                  for="grok-profile-api-backend"
                >
                  {{ t('grok.profiles.fields.apiBackend') }}
                </label>
                <select
                  id="grok-profile-api-backend"
                  :value="form.apiBackend"
                  class="pe-select"
                  @change="setSelectField('apiBackend', $event)"
                >
                  <option value="">
                    {{ t('grok.profiles.editor.notSet') }}
                  </option>
                  <option
                    v-for="backend in GROK_API_BACKEND_OPTIONS"
                    :key="backend"
                    :value="backend"
                  >
                    {{ backend }}
                  </option>
                </select>
              </div>
              <div class="pe-field">
                <label
                  class="pe-field__label"
                  for="grok-profile-context-window"
                >
                  {{ t('grok.profiles.fields.contextWindow') }}
                </label>
                <input
                  id="grok-profile-context-window"
                  :value="form.contextWindow"
                  type="number"
                  min="1"
                  step="1"
                  class="pe-input pe-input--mono"
                  @input="setTextField('contextWindow', $event)"
                >
              </div>
            </div>

            <label
              v-if="isThirdParty"
              class="grok-toggle pe-panel-muted"
            >
              <input
                :checked="form.supportsBackendSearch"
                type="checkbox"
                @change="setCheckboxField('supportsBackendSearch', $event)"
              >
              <span>
                <strong>{{ t('grok.profiles.fields.supportsBackendSearch') }}</strong>
                <small>{{ t('grok.profiles.editor.backendSearchHint') }}</small>
              </span>
            </label>
          </div>
        </section>

        <section
          id="status"
          ref="statusRef"
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
                {{ t('grok.profiles.editor.status') }}
              </h3>
              <p class="pe-section__desc mt-1">
                {{ t('grok.profiles.editor.statusHint') }}
              </p>
            </div>
          </div>

          <div class="pe-panel space-y-4 p-4">
            <div class="pe-field">
              <label
                class="pe-field__label"
                for="grok-profile-tags"
              >
                {{ t('grok.profiles.fields.tags') }}
              </label>
              <input
                id="grok-profile-tags"
                :value="form.tagsInput"
                class="pe-input"
                :placeholder="t('grok.profiles.editor.tagsPlaceholder')"
                @input="setTextField('tagsInput', $event)"
              >
            </div>

            <label class="grok-toggle pe-panel-muted">
              <input
                :checked="form.enabled"
                type="checkbox"
                @change="setCheckboxField('enabled', $event)"
              >
              <span>
                <strong>{{ t('grok.profiles.fields.enabled') }}</strong>
                <small>{{ t('grok.profiles.editor.enabledHint') }}</small>
              </span>
            </label>
          </div>
        </section>
      </div>

      <div class="pe-footer">
        <p class="mr-auto text-xs text-text-secondary">
          {{ t('grok.profiles.editor.footerHint') }}
        </p>
        <button
          type="button"
          class="pe-btn"
          :disabled="saving"
          @click="requestClose(false)"
        >
          {{ t('common.cancel') }}
        </button>
        <button
          type="button"
          class="pe-btn pe-btn--primary"
          :disabled="saving"
          @click="handleSave"
        >
          <SIcon
            v-if="saving"
            name="RefreshCw"
            size="w-4 h-4"
            class="animate-spin"
          />
          {{ t('grok.profiles.actions.save') }}
        </button>
      </div>
    </div>
  </BaseModal>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import BaseModal from '@/components/common/BaseModal.vue'
import SIcon from '@/components/ui/SIcon.vue'
import type { GrokAuthModeDto, GrokCredentialAction, GrokProfileKindDto } from '@/types'
import {
  GROK_API_BACKEND_OPTIONS,
  GROK_REASONING_EFFORT_OPTIONS,
  type GrokProfileEditorForm,
} from '@/utils/grokProfileEditor'
import '@/components/profiles/profile-editor-shell.css'

type GrokEditorSectionId = 'identity' | 'connection' | 'runtime' | 'status'

interface SectionItem {
  id: GrokEditorSectionId
  title: string
}

interface ValidationError {
  section: GrokEditorSectionId
  message: string
}

interface Props {
  modelValue: boolean
  editingName: string | null
  saving: boolean
  error?: string | null
  form: GrokProfileEditorForm
  updateField: (field: keyof GrokProfileEditorForm, value: string | boolean) => void
  baseUrlDisplay?: string | null
  hasExistingBaseUrl?: boolean
  currentAuthMode?: GrokAuthModeDto | null
  currentEnvKey?: string | null
}

const props = withDefaults(defineProps<Props>(), {
  error: null,
  baseUrlDisplay: null,
  hasExistingBaseUrl: false,
  currentAuthMode: null,
  currentEnvKey: null,
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  save: []
}>()

const { t } = useI18n()
const showValidation = ref(false)
const profileKinds: GrokProfileKindDto[] = ['official', 'third_party']
const credentialActions: GrokCredentialAction[] = [
  'preserve',
  'replace_api_key',
  'replace_env_key',
  'clear',
]

const isThirdParty = computed(() => props.form.profileKind === 'third_party')
const isEditing = computed(() => Boolean(props.editingName))

const eyebrow = computed(() => (
  isEditing.value ? t('grok.profiles.editor.editTitle') : t('grok.profiles.editor.createTitle')
))

const title = computed(() => (
  props.form.name.trim()
  || props.editingName
  || t('grok.profiles.editor.createTitle')
))

const visibleSectionIds = computed<GrokEditorSectionId[]>(() => (
  isThirdParty.value
    ? ['identity', 'connection', 'runtime', 'status']
    : ['identity', 'runtime', 'status']
))

const sectionItems = computed<SectionItem[]>(() => (
  visibleSectionIds.value.map((id) => {
    switch (id) {
      case 'identity':
        return { id, title: t('grok.profiles.editor.identity') }
      case 'connection':
        return { id, title: t('grok.profiles.editor.connection') }
      case 'runtime':
        return { id, title: t('grok.profiles.editor.runtime') }
      case 'status':
        return { id, title: t('grok.profiles.editor.status') }
      default: {
        const _exhaustive: never = id
        return _exhaustive
      }
    }
  })
))

const baseUrlPlaceholder = computed(() => (
  props.editingName && props.baseUrlDisplay
    ? props.baseUrlDisplay
    : 'https://api.example.com/v1'
))

const currentAuthLabel = computed(() => (
  props.editingName
    ? t(`grok.profiles.authModes.${props.currentAuthMode ?? 'session'}`)
    : t('grok.states.notSet')
))

const validationErrors = computed<ValidationError[]>(() => {
  const errors: ValidationError[] = []
  if (!props.form.name.trim()) {
    errors.push({ section: 'identity', message: t('grok.profiles.validation.nameRequired') })
  }
  if (props.form.profileKind === 'third_party') {
    if (!props.form.model.trim()) {
      errors.push({ section: 'runtime', message: t('grok.profiles.validation.modelRequired') })
    }
    if (!props.form.baseUrl.trim() && !(props.editingName && props.hasExistingBaseUrl)) {
      errors.push({ section: 'connection', message: t('grok.profiles.validation.baseUrlRequired') })
    }
    if (!props.editingName && props.form.credentialAction === 'preserve') {
      errors.push({ section: 'connection', message: t('grok.profiles.validation.credentialRequired') })
    }
    if (props.form.credentialAction === 'replace_api_key' && !props.form.apiKey.trim()) {
      errors.push({ section: 'connection', message: t('grok.profiles.validation.apiKeyRequired') })
    }
    if (props.form.credentialAction === 'replace_env_key' && !props.form.envKey.trim()) {
      errors.push({ section: 'connection', message: t('grok.profiles.validation.envKeyRequired') })
    }
  }
  if (props.form.contextWindow.trim()) {
    const contextWindow = Number(props.form.contextWindow)
    if (!Number.isInteger(contextWindow) || contextWindow <= 0) {
      errors.push({ section: 'runtime', message: t('grok.profiles.validation.contextWindow') })
    }
  }
  return errors
})

const setField = (field: keyof GrokProfileEditorForm, value: string | boolean) => {
  props.updateField(field, value)
}

const setTextField = (field: keyof GrokProfileEditorForm, event: Event) => {
  setField(field, (event.target as HTMLInputElement | HTMLTextAreaElement).value)
}

const setSelectField = (field: keyof GrokProfileEditorForm, event: Event) => {
  setField(field, (event.target as HTMLSelectElement).value)
}

const setCheckboxField = (field: keyof GrokProfileEditorForm, event: Event) => {
  setField(field, (event.target as HTMLInputElement).checked)
}

const requestClose = (value = false) => {
  if (!props.saving) emit('update:modelValue', value)
}

/* ========================================================================
 * 段导航 + scroll-spy（IntersectionObserver，root = pe-scroll）
 * ======================================================================== */

const scrollRef = ref<HTMLElement | null>(null)
const identityRef = ref<HTMLElement | null>(null)
const connectionRef = ref<HTMLElement | null>(null)
const runtimeRef = ref<HTMLElement | null>(null)
const statusRef = ref<HTMLElement | null>(null)
const activeSectionId = ref<GrokEditorSectionId>('identity')

const sectionElement = (id: GrokEditorSectionId): HTMLElement | null => {
  switch (id) {
    case 'identity':
      return identityRef.value
    case 'connection':
      return connectionRef.value
    case 'runtime':
      return runtimeRef.value
    case 'status':
      return statusRef.value
    default: {
      const _exhaustive: never = id
      return _exhaustive
    }
  }
}

let sectionObserver: IntersectionObserver | null = null

const teardownSectionObserver = () => {
  sectionObserver?.disconnect()
  sectionObserver = null
}

const setupSectionObserver = () => {
  teardownSectionObserver()
  const container = scrollRef.value
  if (!container || typeof IntersectionObserver === 'undefined') return

  const elementToSection = new Map<Element, GrokEditorSectionId>()
  visibleSectionIds.value.forEach((sectionId) => {
    const element = sectionElement(sectionId)
    if (element) elementToSection.set(element, sectionId)
  })
  if (elementToSection.size === 0) return

  const visibility = new Map<GrokEditorSectionId, boolean>()

  sectionObserver = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        const sectionId = elementToSection.get(entry.target)
        if (sectionId) visibility.set(sectionId, entry.isIntersecting)
      }
      const activeId = visibleSectionIds.value.find(id => visibility.get(id))
      if (activeId) activeSectionId.value = activeId
    },
    { root: container, rootMargin: '-140px 0px -70% 0px', threshold: 0 },
  )

  elementToSection.forEach((_sectionId, element) => sectionObserver?.observe(element))
}

const scrollToSection = (sectionId: GrokEditorSectionId) => {
  activeSectionId.value = sectionId

  const container = scrollRef.value
  const element = sectionElement(sectionId)
  if (!container || !element) return

  // jsdom 未实现 Element.scrollTo，缺失时只更新分段高亮
  container.scrollTo?.({ top: Math.max(element.offsetTop - 16, 0), behavior: 'smooth' })
}

const handleSave = () => {
  if (validationErrors.value.length > 0) {
    showValidation.value = true
    scrollToSection(validationErrors.value[0].section)
    return
  }
  showValidation.value = false
  emit('save')
}

watch(
  () => props.modelValue,
  (isOpen) => {
    if (!isOpen) {
      showValidation.value = false
      activeSectionId.value = 'identity'
      teardownSectionObserver()
      return
    }

    showValidation.value = false
    activeSectionId.value = 'identity'
    void nextTick(() => {
      scrollRef.value?.scrollTo?.({ top: 0 })
      setupSectionObserver()
    })
  },
  { immediate: true },
)

watch(
  () => props.form.profileKind,
  (kind) => {
    if (!props.modelValue) return
    if (kind === 'official' && activeSectionId.value === 'connection') {
      activeSectionId.value = 'identity'
    }
    void nextTick(() => {
      setupSectionObserver()
    })
  },
)

onBeforeUnmount(teardownSectionObserver)
</script>

<style scoped>
.grok-kind-control {
  display: inline-flex;
  gap: 0.25rem;
  padding: 0.25rem;
  background: var(--cp-bg-0, var(--color-bg-base));
  border: 1px solid var(--cp-line-2, var(--color-border-default));
  border-radius: var(--radius-md);
}

.grok-kind-control__button {
  padding: 0.5rem 0.875rem;
  color: var(--color-text-secondary);
  background: transparent;
  border: 0;
  border-radius: var(--radius-sm);
  cursor: pointer;
}

.grok-kind-control__button--active {
  color: var(--color-platform-grok);
  background: rgb(var(--color-platform-grok-rgb) / 12%);
}

.grok-kind-control__button:disabled {
  cursor: not-allowed;
  opacity: 0.72;
}

.grok-credential-status,
.grok-toggle {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 0.75rem;
}

.grok-credential-status div,
.grok-toggle span {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 0.25rem;
}

.grok-credential-status span,
.grok-toggle small {
  color: var(--color-text-secondary);
  font-size: 0.75rem;
}

.grok-credential-status strong,
.grok-toggle strong {
  color: var(--color-text-primary);
  font-size: 0.8125rem;
}

.grok-credential-status code {
  overflow-wrap: anywhere;
  color: var(--color-platform-grok);
  font-size: 0.75rem;
}
</style>
