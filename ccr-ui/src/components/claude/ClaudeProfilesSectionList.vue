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
      <div class="border-b border-border-default/35 pb-4">
        <div>
          <p class="text-xs font-semibold uppercase tracking-[0.24em] text-text-muted">
            {{ $t('claudeProfiles.providerSectionEyebrow') }}
          </p>
          <div class="mt-2 flex flex-wrap items-center gap-2.5">
            <h2 class="text-2xl font-semibold tracking-tight text-text-primary">
              {{ section.title }}
            </h2>
            <span class="rounded-full border border-border-default/45 bg-bg-surface/62 px-3 py-1 text-xs text-text-secondary">
              {{ $t('claudeProfiles.providerNavCount', { count: section.count }) }}
            </span>
          </div>
          <p class="mt-2 text-sm text-text-secondary">
            {{ $t('claudeProfiles.providerSectionSummary', { count: section.count, enabled: section.enabledCount }) }}
          </p>
        </div>
      </div>

      <div class="rounded-[30px] border border-border-default/35 bg-bg-surface/42 p-3 shadow-[inset_0_1px_0_rgba(255,255,255,0.03)] md:p-4">
        <div class="space-y-3">
          <ClaudeProfileRow
            v-for="profile in section.profiles"
            :key="profile.name"
            :profile="profile"
            @apply="$emit('apply', profile.name)"
            @edit="$emit('edit', profile)"
            @delete="$emit('delete', profile.name)"
          />
        </div>
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
