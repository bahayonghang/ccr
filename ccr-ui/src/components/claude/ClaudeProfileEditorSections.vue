<template>
  <div class="space-y-5">
    <div
      v-if="saveError"
      class="editor-banner editor-banner--error rounded-xl px-5 py-4"
    >
      <div class="flex items-start gap-3">
        <div class="editor-banner__icon flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl">
          <SIcon
            name="AlertTriangle"
            size="w-4 h-4"
          />
        </div>
        <div class="min-w-0">
          <p class="text-sm font-semibold text-text-primary">
            {{ $t('claudeProfiles.operationFailed') }}
          </p>
          <p class="mt-1 break-words text-sm leading-6 text-text-secondary">
            {{ saveError }}
          </p>
        </div>
      </div>
    </div>

    <section
      :ref="target => registerModalSectionRef('basic', target)"
      class="editor-panel editor-panel--section rounded-xl p-5 lg:p-6"
    >
      <div class="mb-5 flex items-start gap-3">
        <div class="editor-section-icon flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl">
          <SIcon
            name="Layers"
            size="w-5 h-5"
          />
        </div>
        <div class="min-w-0">
          <h3 class="text-base font-semibold text-text-primary">
            {{ $t('claudeProfiles.sections.basic.title') }}
          </h3>
          <p class="mt-1 text-sm leading-6 text-text-secondary">
            {{ $t('claudeProfiles.sections.basic.description') }}
          </p>
        </div>
      </div>

      <div class="grid grid-cols-1 gap-4">
        <div>
          <label
            for="claude-profile-name"
            class="mb-2 block text-sm font-medium text-text-secondary"
          >
            {{ $t('claudeProfiles.nameLabel') }}
          </label>
          <input
            id="claude-profile-name"
            :value="form.name"
            type="text"
            :placeholder="$t('claudeProfiles.namePlaceholder')"
            :class="textFieldClass"
            @input="updateTextField('name', $event)"
          >
          <p class="mt-1.5 text-xs text-text-muted">
            {{ isEditing ? $t('claudeProfiles.renameWarningHint') : $t('claudeProfiles.nameHelper') }}
          </p>
        </div>

        <div>
          <label
            for="claude-profile-description"
            class="mb-2 block text-sm font-medium text-text-secondary"
          >
            {{ $t('claudeProfiles.descLabel') }}
          </label>
          <textarea
            id="claude-profile-description"
            :value="form.description"
            rows="4"
            :placeholder="$t('claudeProfiles.descPlaceholder')"
            :class="textareaClass"
            @input="updateTextAreaField('description', $event)"
          />
          <p class="mt-1.5 text-xs text-text-muted">
            {{ $t('claudeProfiles.descriptionHelper') }}
          </p>
        </div>
      </div>
    </section>

    <section
      :ref="target => registerModalSectionRef('connection', target)"
      class="editor-panel editor-panel--section rounded-xl p-5 lg:p-6"
    >
      <div class="mb-5 flex items-start gap-3">
        <div class="editor-section-icon flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl">
          <SIcon
            name="Globe"
            size="w-5 h-5"
          />
        </div>
        <div class="min-w-0">
          <h3 class="text-base font-semibold text-text-primary">
            {{ $t('claudeProfiles.sections.connection.title') }}
          </h3>
          <p class="mt-1 text-sm leading-6 text-text-secondary">
            {{ $t('claudeProfiles.sections.connection.description') }}
          </p>
        </div>
      </div>

      <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
        <div class="lg:col-span-2">
          <label
            for="claude-profile-base-url"
            class="mb-2 block text-sm font-medium text-text-secondary"
          >
            {{ $t('claudeProfiles.baseUrlLabel') }}
          </label>
          <input
            id="claude-profile-base-url"
            :value="form.base_url"
            type="text"
            :placeholder="$t('claudeProfiles.baseUrlPlaceholder')"
            :class="monospaceFieldClass"
            @input="updateTextField('base_url', $event)"
          >
          <p class="mt-1.5 text-xs text-text-muted">
            {{ $t('claudeProfiles.baseUrlHelper') }}
          </p>
        </div>

        <div class="lg:col-span-2">
          <label
            for="claude-profile-provider"
            class="mb-2 block text-sm font-medium text-text-secondary"
          >
            {{ $t('claudeProfiles.providerLabel') }}
          </label>
          <input
            id="claude-profile-provider"
            :value="form.provider"
            type="text"
            :placeholder="$t('claudeProfiles.providerPlaceholder')"
            :class="textFieldClass"
            @input="updateTextField('provider', $event)"
          >
          <p class="mt-1.5 text-xs text-text-muted">
            {{ $t('claudeProfiles.providerHelper') }}
          </p>
        </div>
      </div>

      <div class="mt-5 editor-panel-muted rounded-xl p-4">
        <div class="flex items-start gap-3">
          <span class="mt-0.5 flex h-6 w-6 shrink-0 items-center justify-center">
            <SIcon
              name="Sparkles"
              size="w-4 h-4"
            />
          </span>
          <div class="min-w-0 flex-1">
            <span class="block text-sm font-semibold text-text-primary">
              {{ $t('claudeProfiles.advancedModelsTitle') }}
            </span>
            <span class="mt-1 block text-xs leading-5 text-text-muted">
              {{ $t('claudeProfiles.advancedModelsDescription') }}
            </span>
          </div>
        </div>

        <div class="mt-4 grid grid-cols-1 gap-4 lg:grid-cols-2">
          <div>
            <label
              for="claude-profile-default-opus-model"
              class="mb-2 block text-sm font-medium text-text-secondary"
            >
              {{ $t('claudeProfiles.defaultOpusModelLabel') }}
            </label>
            <input
              id="claude-profile-default-opus-model"
              :value="form.default_opus_model"
              type="text"
              :placeholder="$t('claudeProfiles.defaultOpusModelPlaceholder')"
              :class="monospaceFieldClass"
              @input="updateTextField('default_opus_model', $event)"
            >
            <p class="mt-1.5 text-xs text-text-muted">
              {{ $t('claudeProfiles.defaultOpusModelHelper') }}
            </p>
          </div>

          <div>
            <label
              for="claude-profile-default-sonnet-model"
              class="mb-2 block text-sm font-medium text-text-secondary"
            >
              {{ $t('claudeProfiles.defaultSonnetModelLabel') }}
            </label>
            <input
              id="claude-profile-default-sonnet-model"
              :value="form.default_sonnet_model"
              type="text"
              :placeholder="$t('claudeProfiles.defaultSonnetModelPlaceholder')"
              :class="monospaceFieldClass"
              @input="updateTextField('default_sonnet_model', $event)"
            >
            <p class="mt-1.5 text-xs text-text-muted">
              {{ $t('claudeProfiles.defaultSonnetModelHelper') }}
            </p>
          </div>

          <div>
            <label
              for="claude-profile-default-haiku-model"
              class="mb-2 block text-sm font-medium text-text-secondary"
            >
              {{ $t('claudeProfiles.defaultHaikuModelLabel') }}
            </label>
            <input
              id="claude-profile-default-haiku-model"
              :value="form.default_haiku_model"
              type="text"
              :placeholder="$t('claudeProfiles.defaultHaikuModelPlaceholder')"
              :class="monospaceFieldClass"
              @input="updateTextField('default_haiku_model', $event)"
            >
            <p class="mt-1.5 text-xs text-text-muted">
              {{ $t('claudeProfiles.defaultHaikuModelHelper') }}
            </p>
          </div>

          <div>
            <label
              for="claude-profile-default-fable-model"
              class="mb-2 block text-sm font-medium text-text-secondary"
            >
              {{ $t('claudeProfiles.defaultFableModelLabel') }}
            </label>
            <input
              id="claude-profile-default-fable-model"
              :value="form.default_fable_model"
              type="text"
              :placeholder="$t('claudeProfiles.defaultFableModelPlaceholder')"
              :class="monospaceFieldClass"
              @input="updateTextField('default_fable_model', $event)"
            >
            <p class="mt-1.5 text-xs text-text-muted">
              {{ $t('claudeProfiles.defaultFableModelHelper') }}
            </p>
          </div>

          <div>
            <label
              for="claude-profile-default-opus-model-name"
              class="mb-2 block text-sm font-medium text-text-secondary"
            >
              {{ $t('claudeProfiles.defaultOpusModelNameLabel') }}
            </label>
            <input
              id="claude-profile-default-opus-model-name"
              :value="form.default_opus_model_name"
              type="text"
              :placeholder="$t('claudeProfiles.defaultModelNamePlaceholder')"
              :class="monospaceFieldClass"
              @input="updateTextField('default_opus_model_name', $event)"
            >
            <p class="mt-1.5 text-xs text-text-muted">
              {{ $t('claudeProfiles.defaultModelNameHelper') }}
            </p>
          </div>

          <div>
            <label
              for="claude-profile-default-sonnet-model-name"
              class="mb-2 block text-sm font-medium text-text-secondary"
            >
              {{ $t('claudeProfiles.defaultSonnetModelNameLabel') }}
            </label>
            <input
              id="claude-profile-default-sonnet-model-name"
              :value="form.default_sonnet_model_name"
              type="text"
              :placeholder="$t('claudeProfiles.defaultModelNamePlaceholder')"
              :class="monospaceFieldClass"
              @input="updateTextField('default_sonnet_model_name', $event)"
            >
            <p class="mt-1.5 text-xs text-text-muted">
              {{ $t('claudeProfiles.defaultModelNameHelper') }}
            </p>
          </div>

          <div>
            <label
              for="claude-profile-default-haiku-model-name"
              class="mb-2 block text-sm font-medium text-text-secondary"
            >
              {{ $t('claudeProfiles.defaultHaikuModelNameLabel') }}
            </label>
            <input
              id="claude-profile-default-haiku-model-name"
              :value="form.default_haiku_model_name"
              type="text"
              :placeholder="$t('claudeProfiles.defaultModelNamePlaceholder')"
              :class="monospaceFieldClass"
              @input="updateTextField('default_haiku_model_name', $event)"
            >
            <p class="mt-1.5 text-xs text-text-muted">
              {{ $t('claudeProfiles.defaultModelNameHelper') }}
            </p>
          </div>

          <div>
            <label
              for="claude-profile-default-fable-model-name"
              class="mb-2 block text-sm font-medium text-text-secondary"
            >
              {{ $t('claudeProfiles.defaultFableModelNameLabel') }}
            </label>
            <input
              id="claude-profile-default-fable-model-name"
              :value="form.default_fable_model_name"
              type="text"
              :placeholder="$t('claudeProfiles.defaultModelNamePlaceholder')"
              :class="monospaceFieldClass"
              @input="updateTextField('default_fable_model_name', $event)"
            >
            <p class="mt-1.5 text-xs text-text-muted">
              {{ $t('claudeProfiles.defaultModelNameHelper') }}
            </p>
          </div>

          <div>
            <label
              for="claude-profile-subagent-model"
              class="mb-2 block text-sm font-medium text-text-secondary"
            >
              {{ $t('claudeProfiles.subagentModelLabel') }}
            </label>
            <input
              id="claude-profile-subagent-model"
              :value="form.subagent_model"
              type="text"
              :placeholder="$t('claudeProfiles.subagentModelPlaceholder')"
              :class="monospaceFieldClass"
              @input="updateTextField('subagent_model', $event)"
            >
            <p class="mt-1.5 text-xs text-text-muted">
              {{ $t('claudeProfiles.subagentModelHelper') }}
            </p>
          </div>

          <div>
            <label
              for="claude-profile-custom-model-option"
              class="mb-2 block text-sm font-medium text-text-secondary"
            >
              {{ $t('claudeProfiles.customModelOptionLabel') }}
            </label>
            <input
              id="claude-profile-custom-model-option"
              :value="form.custom_model_option"
              type="text"
              :placeholder="$t('claudeProfiles.customModelOptionPlaceholder')"
              :class="monospaceFieldClass"
              @input="updateTextField('custom_model_option', $event)"
            >
            <p class="mt-1.5 text-xs text-text-muted">
              {{ $t('claudeProfiles.customModelOptionHelper') }}
            </p>
          </div>

          <div>
            <label
              for="claude-profile-custom-model-option-name"
              class="mb-2 block text-sm font-medium text-text-secondary"
            >
              {{ $t('claudeProfiles.customModelOptionNameLabel') }}
            </label>
            <input
              id="claude-profile-custom-model-option-name"
              :value="form.custom_model_option_name"
              type="text"
              :placeholder="$t('claudeProfiles.customModelOptionNamePlaceholder')"
              :class="monospaceFieldClass"
              @input="updateTextField('custom_model_option_name', $event)"
            >
            <p class="mt-1.5 text-xs text-text-muted">
              {{ $t('claudeProfiles.customModelOptionNameHelper') }}
            </p>
          </div>

          <div class="lg:col-span-2">
            <label
              for="claude-profile-effort-level"
              class="mb-2 block text-sm font-medium text-text-secondary"
            >
              {{ $t('claudeProfiles.effortLevelLabel') }}
            </label>
            <select
              id="claude-profile-effort-level"
              :value="form.effort_level"
              :class="textFieldClass"
              @change="updateTextField('effort_level', $event)"
            >
              <option value="">
                {{ $t('claudeProfiles.effortLevelOptionDefault') }}
              </option>
              <option value="low">
                {{ $t('claudeProfiles.effortLevelOptionLow') }}
              </option>
              <option value="medium">
                {{ $t('claudeProfiles.effortLevelOptionMedium') }}
              </option>
              <option value="high">
                {{ $t('claudeProfiles.effortLevelOptionHigh') }}
              </option>
              <option value="xhigh">
                {{ $t('claudeProfiles.effortLevelOptionXhigh') }}
              </option>
              <option value="max">
                {{ $t('claudeProfiles.effortLevelOptionMax') }}
              </option>
            </select>
            <p class="mt-1.5 text-xs text-text-muted">
              {{ $t('claudeProfiles.effortLevelHelper') }}
            </p>
          </div>
        </div>
      </div>
    </section>

    <section
      :ref="target => registerModalSectionRef('auth', target)"
      class="editor-panel editor-panel--section rounded-xl p-5 lg:p-6"
    >
      <div class="mb-5 flex items-start gap-3">
        <div class="editor-section-icon flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl">
          <SIcon
            name="ShieldCheck"
            size="w-5 h-5"
          />
        </div>
        <div class="min-w-0">
          <h3 class="text-base font-semibold text-text-primary">
            {{ $t('claudeProfiles.sections.auth.title') }}
          </h3>
          <p class="mt-1 text-sm leading-6 text-text-secondary">
            {{ $t('claudeProfiles.sections.auth.description') }}
          </p>
        </div>
      </div>

      <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
        <div class="lg:col-span-2">
          <label
            for="claude-profile-auth-mode"
            class="mb-2 block text-sm font-medium text-text-secondary"
          >
            {{ $t('claudeProfiles.authModeLabel') }}
          </label>
          <select
            id="claude-profile-auth-mode"
            :value="form.auth_mode"
            :class="textFieldClass"
            @change="updateTextField('auth_mode', $event)"
          >
            <option value="subscription">
              {{ $t('claudeProfiles.authModeOptionSubscription') }}
            </option>
            <option value="api_key">
              {{ $t('claudeProfiles.authModeOptionApiKey') }}
            </option>
          </select>
          <p class="mt-1.5 text-xs text-text-muted">
            {{ $t('claudeProfiles.authModeHelper') }}
          </p>

          <div
            v-if="showAuthModeMismatch"
            class="editor-banner editor-banner--warn mt-3 rounded-xl px-4 py-3"
          >
            <div class="flex items-start gap-3">
              <div class="editor-banner__icon flex h-8 w-8 shrink-0 items-center justify-center rounded-xl">
                <SIcon
                  name="AlertTriangle"
                  size="w-4 h-4"
                />
              </div>
              <p class="min-w-0 break-words text-xs leading-5 text-text-secondary">
                {{ $t('claudeProfiles.authModeMismatchWarning') }}
              </p>
            </div>
          </div>
        </div>

        <div>
          <label
            for="claude-profile-account"
            class="mb-2 block text-sm font-medium text-text-secondary"
          >
            {{ $t('claudeProfiles.accountLabel') }}
          </label>
          <input
            id="claude-profile-account"
            :value="form.account"
            type="text"
            :placeholder="$t('claudeProfiles.accountPlaceholder')"
            :class="textFieldClass"
            @input="updateTextField('account', $event)"
          >
          <p class="mt-1.5 text-xs text-text-muted">
            {{ $t('claudeProfiles.accountHelper') }}
          </p>
        </div>

        <div>
          <label
            for="claude-profile-provider-type"
            class="mb-2 block text-sm font-medium text-text-secondary"
          >
            {{ $t('claudeProfiles.providerTypeLabel') }}
          </label>
          <input
            id="claude-profile-provider-type"
            :value="form.provider_type"
            type="text"
            :placeholder="$t('claudeProfiles.providerTypePlaceholder')"
            :class="textFieldClass"
            @input="updateTextField('provider_type', $event)"
          >
          <p class="mt-1.5 text-xs text-text-muted">
            {{ $t('claudeProfiles.providerTypeHelper') }}
          </p>
        </div>

        <div class="lg:col-span-2">
          <label
            for="claude-profile-auth-token"
            class="mb-2 block text-sm font-medium text-text-secondary"
          >
            {{ $t('claudeProfiles.authTokenLabel') }}
          </label>
          <div class="relative">
            <input
              id="claude-profile-auth-token"
              :value="form.auth_token"
              data-testid="claude-auth-token-input"
              :type="showAuthToken ? 'text' : 'password'"
              :placeholder="$t('claudeProfiles.authTokenPlaceholder')"
              :class="[monospaceFieldClass, 'pr-24']"
              @input="updateTextField('auth_token', $event)"
            >
            <div class="absolute right-3 top-1/2 flex -translate-y-1/2 items-center gap-1">
              <button
                type="button"
                data-testid="claude-auth-token-visibility"
                class="editor-icon-button"
                :aria-label="showAuthToken ? $t('claudeProfiles.authTokenActions.hide') : $t('claudeProfiles.authTokenActions.show')"
                :title="showAuthToken ? $t('claudeProfiles.authTokenActions.hide') : $t('claudeProfiles.authTokenActions.show')"
                @click="showAuthToken = !showAuthToken"
              >
                <SIcon
                  :name="showAuthToken ? 'EyeOff' : 'Eye'"
                  size="w-4 h-4"
                />
              </button>
              <button
                type="button"
                data-testid="claude-auth-token-copy"
                class="editor-icon-button"
                :aria-label="$t('claudeProfiles.authTokenActions.copy')"
                :disabled="!form.auth_token.trim()"
                :title="$t('claudeProfiles.authTokenActions.copy')"
                @click="copyAuthToken"
              >
                <SIcon
                  name="Copy"
                  size="w-4 h-4"
                />
              </button>
            </div>
          </div>
          <p class="mt-1.5 text-xs text-text-muted">
            {{ $t('claudeProfiles.authTokenHelper') }}
          </p>
        </div>
      </div>
    </section>

    <section
      :ref="target => registerModalSectionRef('status', target)"
      class="editor-panel editor-panel--section rounded-xl p-5 lg:p-6"
    >
      <div class="mb-5 flex items-start gap-3">
        <div class="editor-section-icon flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl">
          <SIcon
            name="SlidersHorizontal"
            size="w-5 h-5"
          />
        </div>
        <div class="min-w-0">
          <h3 class="text-base font-semibold text-text-primary">
            {{ $t('claudeProfiles.sections.status.title') }}
          </h3>
          <p class="mt-1 text-sm leading-6 text-text-secondary">
            {{ $t('claudeProfiles.sections.status.description') }}
          </p>
        </div>
      </div>

      <div class="grid grid-cols-1 gap-4 lg:grid-cols-[minmax(0,1fr)_280px]">
        <div>
          <label
            for="claude-profile-tags"
            class="mb-2 block text-sm font-medium text-text-secondary"
          >
            {{ $t('claudeProfiles.tagsLabel') }}
          </label>
          <input
            id="claude-profile-tags"
            :value="form.tagsInput"
            type="text"
            :placeholder="$t('claudeProfiles.tagsPlaceholder')"
            :class="textFieldClass"
            @input="updateTextField('tagsInput', $event)"
          >
          <p class="mt-1.5 text-xs text-text-muted">
            {{ $t('claudeProfiles.tagsHelper') }}
          </p>

          <div
            v-if="parsedFormTags.length > 0"
            class="mt-3 flex flex-wrap gap-2"
          >
            <span
              v-for="tag in parsedFormTags"
              :key="tag"
              class="editor-tag rounded-full px-3 py-1 text-xs text-text-secondary"
            >
              #{{ tag }}
            </span>
          </div>
        </div>

        <div class="editor-panel-muted rounded-xl p-4">
          <label
            for="claude-profile-enabled"
            class="flex cursor-pointer items-start gap-3"
          >
            <input
              id="claude-profile-enabled"
              :checked="form.enabled"
              type="checkbox"
              class="mt-1 h-4 w-4 rounded border-border-default text-accent-secondary focus:ring-accent-secondary/30"
              @change="updateCheckboxField('enabled', $event)"
            >
            <div class="min-w-0">
              <span class="block text-sm font-medium text-text-primary">
                {{ $t('claudeProfiles.enabledProfile') }}
              </span>
              <span class="mt-1 block text-xs leading-5 text-text-muted">
                {{ $t('claudeProfiles.enabledHelper') }}
              </span>
            </div>
          </label>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import type { ComponentPublicInstance } from 'vue'
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import SIcon from '@/components/ui/SIcon.vue'
import { useUIStore } from '@/stores/ui'
import type { ClaudeProfileEditorForm, ClaudeProfileFormSectionId } from '@/types/claudeProfileEditor'
import { copyText } from '@/utils/clipboard'

const props = defineProps<{
  editingName: string
  form: ClaudeProfileEditorForm
  isEditing: boolean
  monospaceFieldClass: string
  parsedFormTags: string[]
  registerModalSectionRef: (sectionId: ClaudeProfileFormSectionId, target: Element | ComponentPublicInstance | null) => void
  saveError: string | null
  textareaClass: string
  textFieldClass: string
  updateFormField: (field: keyof ClaudeProfileEditorForm, value: string | boolean) => void
}>()

const { t } = useI18n()
const uiStore = useUIStore()
const showAuthToken = ref(false)

// 「API-key 形态」判定，与后端 is_api_key_shaped 对齐：
// provider_type=third_party_model，或 base_url 与 auth_token 同时非空。
const isApiKeyShaped = computed(() =>
  props.form.provider_type.trim() === 'third_party_model'
  || (props.form.base_url.trim() !== '' && props.form.auth_token.trim() !== ''),
)

// 第三方/API-key 形态却仍选了 subscription：apply 时会被清空，给出内联警告。
const showAuthModeMismatch = computed(() =>
  isApiKeyShaped.value && props.form.auth_mode === 'subscription',
)

function updateTextField(field: keyof ClaudeProfileEditorForm, event: Event) {
  props.updateFormField(field, (event.target as HTMLInputElement).value)
}

function updateTextAreaField(field: keyof ClaudeProfileEditorForm, event: Event) {
  props.updateFormField(field, (event.target as HTMLTextAreaElement).value)
}

function updateCheckboxField(field: keyof ClaudeProfileEditorForm, event: Event) {
  props.updateFormField(field, (event.target as HTMLInputElement).checked)
}

async function copyAuthToken() {
  const token = props.form.auth_token.trim()
  if (!token) return

  const ok = await copyText(token)
  if (ok) {
    uiStore.showSuccess(t('claudeProfiles.authTokenCopied'))
  } else {
    uiStore.showError(t('claudeProfiles.authTokenCopyFailed'))
  }
}

watch(() => props.editingName, () => {
  showAuthToken.value = false
})
</script>
