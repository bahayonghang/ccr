<template>
  <aside class="space-y-4 xl:sticky xl:top-0 xl:self-start">
    <section class="editor-panel editor-panel--summary overflow-hidden rounded-[28px]">
      <div class="editor-panel-head editor-panel-head--summary border-b px-5 py-5">
        <p class="text-xs font-semibold uppercase tracking-[0.24em] text-text-muted">
          {{ $t('claudeProfiles.editorSummaryTitle') }}
        </p>
        <div class="mt-4 flex items-start gap-3">
          <div class="editor-summary-icon flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl">
            <SIcon
              name="Layers"
              size="w-5 h-5"
            />
          </div>
          <div class="min-w-0">
            <h3 class="truncate text-lg font-semibold text-text-primary">
              {{ modalPreviewTitle }}
            </h3>
            <p class="mt-1 text-sm leading-6 text-text-secondary">
              {{ modalPreviewDescription }}
            </p>
          </div>
        </div>

        <div class="mt-4 flex flex-wrap items-center gap-2">
          <span
            class="editor-pill px-3 py-1 text-xs font-medium"
            :class="modalStatusClass"
          >
            {{ modalStatus }}
          </span>
          <span
            class="editor-pill px-3 py-1 text-xs font-medium"
            :class="enabledBadgeClass"
          >
            {{ formEnabled ? $t('claudeProfiles.enabledText') : $t('claudeProfiles.disabledText') }}
          </span>
        </div>
      </div>

      <div class="space-y-3 px-5 py-5">
        <div
          v-for="item in modalSummaryItems"
          :key="item.label"
          class="editor-info-card rounded-2xl px-4 py-3"
        >
          <div class="flex items-start gap-3">
            <div class="editor-info-icon mt-0.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-2xl">
              <SIcon
                :name="item.icon"
                size="w-4 h-4"
              />
            </div>
            <div class="min-w-0">
              <p class="text-[11px] font-semibold uppercase tracking-[0.2em] text-text-muted">
                {{ item.label }}
              </p>
              <p
                class="mt-1 break-words text-sm text-text-primary"
                :class="item.mono ? 'font-mono text-[13px]' : ''"
              >
                {{ item.value }}
              </p>
            </div>
          </div>
        </div>
      </div>
    </section>

    <section class="editor-panel editor-panel--nav rounded-[28px] p-4">
      <div class="mb-3">
        <p class="text-xs font-semibold uppercase tracking-[0.24em] text-text-muted">
          {{ $t('claudeProfiles.editorSectionsTitle') }}
        </p>
        <p class="mt-1 text-sm leading-6 text-text-secondary">
          {{ $t('claudeProfiles.editorSectionsHint') }}
        </p>
      </div>

      <div class="space-y-2">
        <button
          v-for="section in modalSectionItems"
          :key="section.id"
          type="button"
          class="editor-nav-button flex min-h-[56px] w-full items-start gap-3 rounded-2xl px-3.5 py-3 text-left transition-[background-color,border-color,transform] duration-200 hover:-translate-y-px"
          :class="activeFormSectionId === section.id
            ? 'editor-nav-button--active'
            : 'editor-nav-button--idle'"
          @click="$emit('navigate', section.id)"
        >
          <div class="editor-nav-button__icon mt-0.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-2xl">
            <SIcon
              :name="section.icon"
              size="w-4 h-4"
            />
          </div>
          <div class="min-w-0">
            <p class="font-medium text-inherit">
              {{ section.title }}
            </p>
            <p class="mt-1 text-xs leading-5 text-text-muted">
              {{ section.description }}
            </p>
          </div>
        </button>
      </div>
    </section>

    <section class="editor-panel editor-panel--tags rounded-[28px] p-4">
      <div class="flex items-center gap-3">
        <div class="editor-section-icon flex h-10 w-10 items-center justify-center rounded-2xl">
          <SIcon
            name="Tags"
            size="w-4 h-4"
          />
        </div>
        <div>
          <p class="text-sm font-medium text-text-primary">
            {{ $t('claudeProfiles.tagsLabel') }}
          </p>
          <p class="text-xs leading-5 text-text-muted">
            {{ $t('claudeProfiles.tagsHelper') }}
          </p>
        </div>
      </div>

      <div
        v-if="parsedFormTags.length > 0"
        class="mt-4 flex flex-wrap gap-2"
      >
        <span
          v-for="tag in parsedFormTags"
          :key="tag"
          class="editor-tag rounded-full px-3 py-1 text-xs text-text-secondary"
        >
          #{{ tag }}
        </span>
      </div>
      <p
        v-else
        class="editor-empty-hint mt-4 rounded-2xl px-4 py-3 text-sm text-text-muted"
      >
        {{ $t('claudeProfiles.tagsPreviewEmpty') }}
      </p>
    </section>
  </aside>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import type {
  ClaudeProfileEditorSectionItem,
  ClaudeProfileEditorSummaryItem,
  ClaudeProfileFormSectionId,
} from '@/types/claudeProfileEditor'

defineProps<{
  activeFormSectionId: ClaudeProfileFormSectionId
  enabledBadgeClass: string
  formEnabled: boolean
  modalPreviewDescription: string
  modalPreviewTitle: string
  modalSectionItems: ClaudeProfileEditorSectionItem[]
  modalStatus: string
  modalStatusClass: string
  modalSummaryItems: ClaudeProfileEditorSummaryItem[]
  parsedFormTags: string[]
}>()

defineEmits<{
  navigate: [sectionId: ClaudeProfileFormSectionId]
}>()
</script>
