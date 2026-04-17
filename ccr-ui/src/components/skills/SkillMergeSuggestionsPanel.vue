<template>
  <section class="merge-panel">
    <header class="merge-panel__header">
      <h3 class="merge-panel__title">
        {{ t('skillsExt.merge.title') }}
      </h3>
      <span class="merge-panel__count">{{ suggestions.length }}</span>
    </header>

    <p
      v-if="suggestions.length === 0"
      class="merge-panel__empty"
    >
      {{ t('skillsExt.merge.empty') }}
    </p>

    <ul
      v-else
      class="merge-panel__list"
    >
      <li
        v-for="(s, idx) in suggestions"
        :key="`${s.skills[0].id}-${s.skills[1].id}-${idx}`"
        class="merge-panel__row"
      >
        <div class="merge-panel__row-main">
          <strong class="merge-panel__pair">
            <span>{{ s.skills[0].name }}</span>
            <span class="merge-panel__arrow">⇄</span>
            <span>{{ s.skills[1].name }}</span>
          </strong>
          <p class="merge-panel__reason">
            {{ s.reason }}
          </p>
        </div>
        <span class="merge-panel__similarity">{{ percent(s.similarity) }}</span>
      </li>
    </ul>
  </section>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import type { MergeSuggestion } from '@/types/skillVersioning'

const { t } = useI18n()

defineProps<{
  suggestions: MergeSuggestion[]
}>()

function percent(sim: number): string {
  return `${Math.round(sim * 100)}%`
}
</script>

<style scoped>
.merge-panel {
  @apply flex flex-col gap-3 rounded-3xl p-4;

  background: var(--surface-workspace-bg);
  border: 1px solid var(--surface-workspace-border);
  box-shadow: var(--elevation-2);
}

.merge-panel__header {
  @apply flex items-center justify-between;
}

.merge-panel__title {
  @apply text-xs font-semibold uppercase tracking-[0.16em] text-text-muted;
}

.merge-panel__count {
  @apply rounded-full border border-border-default/45 bg-bg-base/60 px-2 py-0.5 text-xs text-text-secondary;
}

.merge-panel__empty {
  @apply rounded-2xl border border-border-default/40 p-4 text-center text-xs text-text-muted;
}

.merge-panel__list {
  @apply flex flex-col gap-2;
}

.merge-panel__row {
  @apply flex items-center justify-between gap-3 rounded-2xl border border-border-default/50 p-3;

  background-color: rgb(var(--color-bg-base-rgb) / 55%);
}

.merge-panel__row-main {
  @apply flex min-w-0 flex-col gap-1;
}

.merge-panel__pair {
  @apply flex items-center gap-2 text-sm text-text-primary;
}

.merge-panel__arrow {
  @apply text-text-muted;
}

.merge-panel__reason {
  @apply truncate text-xs text-text-muted;
}

.merge-panel__similarity {
  @apply rounded-full border border-emerald-500/40 px-2 py-0.5 font-mono text-xs text-emerald-300;
}
</style>
