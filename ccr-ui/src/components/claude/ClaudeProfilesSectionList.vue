<template>
  <div
    class="space-y-8 animate-slide-up"
    style="animation-delay: 200ms"
  >
    <section
      v-for="section in providerSections"
      :id="section.id"
      :key="section.id"
      :ref="(element) => registerSectionRef(section.id, element)"
      class="scroll-mt-28 space-y-4"
    >
      <div class="flex flex-col gap-3 md:flex-row md:items-end md:justify-between">
        <div>
          <p class="text-xs font-semibold uppercase tracking-[0.24em] text-text-muted">
            {{ $t('claudeProfiles.providerSectionEyebrow') }}
          </p>
          <div class="mt-2 flex flex-wrap items-center gap-3">
            <h2 class="text-2xl font-semibold tracking-tight text-text-primary">
              {{ section.title }}
            </h2>
            <span
              v-if="section.isCurrentProvider"
              class="rounded-full bg-accent-secondary/10 px-3 py-1 text-xs font-medium text-accent-secondary"
            >
              {{ $t('claudeProfiles.currentProviderBadge') }}
            </span>
          </div>
          <p class="mt-2 text-sm text-text-secondary">
            {{ $t('claudeProfiles.providerSectionSummary', { count: section.count, enabled: section.enabledCount }) }}
          </p>
        </div>

        <div class="flex flex-wrap items-center gap-2">
          <span class="rounded-full bg-bg-elevated px-3 py-1 text-xs font-medium text-text-secondary">
            {{ $t('claudeProfiles.providerNavCount', { count: section.count }) }}
          </span>
          <span class="rounded-full bg-accent-success/10 px-3 py-1 text-xs font-medium text-accent-success">
            {{ $t('claudeProfiles.providerEnabledCount', { count: section.enabledCount }) }}
          </span>
        </div>
      </div>

      <div class="space-y-4">
        <ClaudeProfileRow
          v-for="profile in section.profiles"
          :key="profile.name"
          :profile="profile"
          @apply="$emit('apply', profile.name)"
          @edit="$emit('edit', profile)"
          @delete="$emit('delete', profile.name)"
        />
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import type { ComponentPublicInstance } from 'vue'
import ClaudeProfileRow from '@/components/claude/ClaudeProfileRow.vue'
import type { ClaudeProfile } from '@/types'
import type { ClaudeProfileSection } from '@/utils/claudeProfiles'

defineProps<{
  providerSections: ClaudeProfileSection[]
  registerSectionRef: (sectionId: string, target: Element | ComponentPublicInstance | null) => void
}>()

defineEmits<{
  apply: [name: string]
  delete: [name: string]
  edit: [profile: ClaudeProfile]
}>()
</script>
