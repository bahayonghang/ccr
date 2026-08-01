<template>
  <BaseModal
    :model-value="modelValue"
    :title="editingName ? t('grok.profiles.editor.editTitle') : t('grok.profiles.editor.createTitle')"
    size="3xl"
    content-class="grok-profile-editor pe-shell"
    :close-on-backdrop="false"
    @update:model-value="requestClose"
  >
    <div class="pe-scroll space-y-5">
      <div
        v-if="error || (showValidation && validationErrors.length > 0)"
        class="pe-summary"
        role="alert"
      >
        <SIcon
          name="AlertTriangle"
          size="w-4 h-4"
        />
        <div>
          <p>{{ error || t('grok.profiles.editor.validationTitle') }}</p>
          <ul v-if="!error">
            <li
              v-for="message in validationErrors"
              :key="message"
            >
              {{ message }}
            </li>
          </ul>
        </div>
      </div>

      <section class="space-y-3">
        <div class="pe-section-heading">
          <div class="pe-panel-icon">
            <SIcon
              name="Fingerprint"
              size="w-4 h-4"
            />
          </div>
          <div>
            <h3 class="pe-section__title">
              {{ t('grok.profiles.editor.identity') }}
            </h3>
            <p class="pe-section__desc">
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
            :class="{ 'md:grid-cols-2': form.profileKind === 'third_party' }"
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
              v-if="form.profileKind === 'third_party'"
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
        v-if="form.profileKind === 'third_party'"
        class="space-y-3"
      >
        <div class="pe-section-heading">
          <div class="pe-panel-icon">
            <SIcon
              name="KeyRound"
              size="w-4 h-4"
            />
          </div>
          <div>
            <h3 class="pe-section__title">
              {{ t('grok.profiles.editor.connection') }}
            </h3>
            <p class="pe-section__desc">
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

      <section class="space-y-3">
        <div class="pe-section-heading">
          <div class="pe-panel-icon">
            <SIcon
              name="Bot"
              size="w-4 h-4"
            />
          </div>
          <div>
            <h3 class="pe-section__title">
              {{ t('grok.profiles.editor.runtime') }}
            </h3>
            <p class="pe-section__desc">
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
                  v-if="form.profileKind === 'third_party'"
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
            v-if="form.profileKind === 'third_party'"
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
            v-if="form.profileKind === 'third_party'"
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

      <section class="space-y-3">
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

    <template #footer>
      <div class="pe-footer">
        <p>{{ t('grok.profiles.editor.footerHint') }}</p>
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
    </template>
  </BaseModal>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
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

const validationErrors = computed(() => {
  const errors: string[] = []
  if (!props.form.name.trim()) errors.push(t('grok.profiles.validation.nameRequired'))
  if (props.form.profileKind === 'third_party') {
    if (!props.form.model.trim()) errors.push(t('grok.profiles.validation.modelRequired'))
    if (!props.form.baseUrl.trim() && !(props.editingName && props.hasExistingBaseUrl)) {
      errors.push(t('grok.profiles.validation.baseUrlRequired'))
    }
    if (!props.editingName && props.form.credentialAction === 'preserve') {
      errors.push(t('grok.profiles.validation.credentialRequired'))
    }
    if (props.form.credentialAction === 'replace_api_key' && !props.form.apiKey.trim()) {
      errors.push(t('grok.profiles.validation.apiKeyRequired'))
    }
    if (props.form.credentialAction === 'replace_env_key' && !props.form.envKey.trim()) {
      errors.push(t('grok.profiles.validation.envKeyRequired'))
    }
  }
  if (props.form.contextWindow.trim()) {
    const contextWindow = Number(props.form.contextWindow)
    if (!Number.isInteger(contextWindow) || contextWindow <= 0) {
      errors.push(t('grok.profiles.validation.contextWindow'))
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

const handleSave = () => {
  showValidation.value = true
  if (validationErrors.value.length === 0) emit('save')
}

watch(() => props.modelValue, (open) => {
  if (open) showValidation.value = false
})
</script>

<style scoped>
.pe-section-heading {
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
}

.pe-panel-icon {
  display: grid;
  width: 2.25rem;
  height: 2.25rem;
  flex: 0 0 auto;
  place-items: center;
}

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

.pe-footer > p {
  margin-right: auto;
  color: var(--color-text-secondary);
  font-size: 0.75rem;
}
</style>
