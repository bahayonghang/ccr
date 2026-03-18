<template>
  <nav
    :class="[
      mobile
        ? 'rounded-2xl border border-border-default/50 bg-bg-surface/70 p-3'
        : 'sticky top-6 rounded-[28px] border border-border-default/50 bg-bg-surface/72 p-4 shadow-xl shadow-black/5 backdrop-blur-xl',
    ]"
    :aria-label="$t('claudeProfiles.providerNavTitle')"
  >
    <div
      v-if="!mobile"
      class="mb-4 space-y-1"
    >
      <p class="flex items-center gap-2 text-xs font-semibold uppercase tracking-[0.24em] text-text-muted">
        <SIcon
          name="PanelLeftOpen"
          size="w-3.5 h-3.5"
        />
        {{ $t('claudeProfiles.providerNavTitle') }}
      </p>
      <p class="text-sm text-text-secondary">
        {{ $t('claudeProfiles.providerNavHint') }}
      </p>
    </div>

    <div
      :class="[
        mobile
          ? 'flex gap-2 overflow-x-auto pb-1'
          : 'flex flex-col gap-2',
      ]"
    >
      <button
        v-for="section in sections"
        :key="section.id"
        type="button"
        :class="[
          mobile
            ? 'min-h-[40px] whitespace-nowrap px-3 py-2 text-sm'
            : 'min-h-[52px] w-full px-4 py-3 text-left',
          'group rounded-2xl border transition-[background-color,border-color,color,transform] duration-200',
          activeSectionId === section.id
            ? 'border-accent-secondary/40 bg-accent-secondary/12 text-text-primary shadow-[0_10px_30px_rgba(96,70,160,0.14)]'
            : 'border-border-default/50 bg-bg-surface/55 text-text-secondary hover:border-border-default hover:bg-bg-elevated/70 hover:text-text-primary',
        ]"
        @click="$emit('navigate', section.id)"
      >
        <div class="flex items-center justify-between gap-3">
          <div class="min-w-0">
            <p
              :class="[
                'truncate font-medium',
                activeSectionId === section.id ? 'text-text-primary' : '',
              ]"
            >
              {{ section.title }}
            </p>
            <p
              v-if="!mobile"
              class="mt-1 text-xs text-text-muted"
            >
              {{ $t('claudeProfiles.providerNavCount', { count: section.count }) }}
            </p>
          </div>
          <div class="flex shrink-0 items-center gap-2">
            <span
              class="rounded-full px-2 py-1 text-xs font-medium"
              :class="activeSectionId === section.id ? 'bg-accent-secondary/14 text-accent-secondary' : 'bg-bg-elevated text-text-muted'"
            >
              {{ section.count }}
            </span>
            <span
              v-if="section.isCurrentProvider && !mobile"
              class="rounded-full bg-accent-success/10 px-2 py-1 text-[11px] font-medium text-accent-success"
            >
              {{ $t('claudeProfiles.currentBadge') }}
            </span>
          </div>
        </div>
      </button>
    </div>
  </nav>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import type { ClaudeProfileSection } from '@/utils/claudeProfiles'

defineProps<{
  sections: ClaudeProfileSection[]
  activeSectionId: string | null
  mobile?: boolean
}>()

defineEmits<{
  navigate: [sectionId: string]
}>()
</script>
