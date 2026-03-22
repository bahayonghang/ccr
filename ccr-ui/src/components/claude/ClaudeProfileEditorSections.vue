<template>
  <div class="space-y-5">
    <div
      v-if="saveError"
      class="editor-banner editor-banner--error rounded-[24px] px-5 py-4"
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
      class="editor-panel editor-panel--section rounded-[28px] p-5 lg:p-6"
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

      <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
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
            :disabled="isEditing"
            :placeholder="$t('claudeProfiles.namePlaceholder')"
            :class="textFieldClass"
            @input="updateTextField('name', $event)"
          >
          <p class="mt-1.5 text-xs text-text-muted">
            {{ isEditing ? $t('claudeProfiles.readonlyNameHint') : $t('claudeProfiles.nameHelper') }}
          </p>
        </div>

        <div class="editor-panel-muted rounded-[24px] p-4">
          <p class="text-xs font-semibold uppercase tracking-[0.2em] text-text-muted">
            {{ modalStatus }}
          </p>
          <p class="mt-2 text-sm leading-6 text-text-secondary">
            {{ modalDescription }}
          </p>
          <div class="mt-4 flex flex-wrap items-center gap-2">
            <span
              class="editor-pill px-3 py-1 text-xs font-medium"
              :class="modalStatusClass"
            >
              {{ modalStatus }}
            </span>
            <span class="editor-inline-chip px-3 py-1 text-xs text-text-secondary">
              {{ isEditing ? editingName : $t('claudeProfiles.newProfileTitle') }}
            </span>
          </div>
        </div>

        <div class="lg:col-span-2">
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
      class="editor-panel editor-panel--section rounded-[28px] p-5 lg:p-6"
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

        <div>
          <label
            for="claude-profile-model"
            class="mb-2 block text-sm font-medium text-text-secondary"
          >
            {{ $t('claudeProfiles.modelLabel') }}
          </label>
          <input
            id="claude-profile-model"
            :value="form.model"
            type="text"
            :placeholder="$t('claudeProfiles.modelPlaceholder')"
            :class="monospaceFieldClass"
            @input="updateTextField('model', $event)"
          >
          <p class="mt-1.5 text-xs text-text-muted">
            {{ $t('claudeProfiles.modelHelper') }}
          </p>
        </div>

        <div>
          <label
            for="claude-profile-small-fast-model"
            class="mb-2 block text-sm font-medium text-text-secondary"
          >
            {{ $t('claudeProfiles.smallFastModelLabel') }}
          </label>
          <input
            id="claude-profile-small-fast-model"
            :value="form.small_fast_model"
            type="text"
            :placeholder="$t('claudeProfiles.smallFastModelPlaceholder')"
            :class="monospaceFieldClass"
            @input="updateTextField('small_fast_model', $event)"
          >
          <p class="mt-1.5 text-xs text-text-muted">
            {{ $t('claudeProfiles.smallFastModelHelper') }}
          </p>
        </div>

        <div>
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
    </section>

    <section
      :ref="target => registerModalSectionRef('auth', target)"
      class="editor-panel editor-panel--section rounded-[28px] p-5 lg:p-6"
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
          <input
            id="claude-profile-auth-token"
            :value="form.auth_token"
            type="password"
            :placeholder="$t('claudeProfiles.authTokenPlaceholder')"
            :class="monospaceFieldClass"
            @input="updateTextField('auth_token', $event)"
          >
          <p class="mt-1.5 text-xs text-text-muted">
            {{ $t('claudeProfiles.authTokenHelper') }}
          </p>
        </div>
      </div>
    </section>

    <section
      :ref="target => registerModalSectionRef('status', target)"
      class="editor-panel editor-panel--section rounded-[28px] p-5 lg:p-6"
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

        <div class="editor-panel-muted rounded-[24px] p-4">
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

          <div class="editor-inline-card mt-4 rounded-2xl px-4 py-3">
            <p class="text-xs font-semibold uppercase tracking-[0.2em] text-text-muted">
              {{ modalStatus }}
            </p>
            <p class="mt-2 text-sm text-text-primary">
              {{ form.enabled ? $t('claudeProfiles.enabledText') : $t('claudeProfiles.disabledText') }}
            </p>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import type { ComponentPublicInstance } from 'vue'
import SIcon from '@/components/ui/SIcon.vue'
import type { ClaudeProfileEditorForm, ClaudeProfileFormSectionId } from '@/types/claudeProfileEditor'

const props = defineProps<{
  editingName: string
  form: ClaudeProfileEditorForm
  isEditing: boolean
  modalDescription: string
  modalStatus: string
  modalStatusClass: string
  monospaceFieldClass: string
  parsedFormTags: string[]
  registerModalSectionRef: (sectionId: ClaudeProfileFormSectionId, target: Element | ComponentPublicInstance | null) => void
  saveError: string | null
  textareaClass: string
  textFieldClass: string
  updateFormField: (field: keyof ClaudeProfileEditorForm, value: string | boolean) => void
}>()

function updateTextField(field: keyof ClaudeProfileEditorForm, event: Event) {
  props.updateFormField(field, (event.target as HTMLInputElement).value)
}

function updateTextAreaField(field: keyof ClaudeProfileEditorForm, event: Event) {
  props.updateFormField(field, (event.target as HTMLTextAreaElement).value)
}

function updateCheckboxField(field: keyof ClaudeProfileEditorForm, event: Event) {
  props.updateFormField(field, (event.target as HTMLInputElement).checked)
}
</script>
