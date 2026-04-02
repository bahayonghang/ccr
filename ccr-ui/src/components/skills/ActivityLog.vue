<template>
  <section class="panel">
    <div class="panel__header">
      <h2 class="panel__title">
        Activity
      </h2>
    </div>
    <div class="activity-list">
      <div
        v-for="entry in visibleEntries"
        :key="entry.id"
        class="activity-row"
      >
        <span
          class="activity-row__status"
          :class="`activity-row__status--${entry.status}`"
        />
        <div class="activity-row__body">
          <strong>{{ entry.action }}</strong>
          <span>{{ entry.target }}</span>
          <span class="activity-row__meta">{{ formatTime(entry.timestamp) }}</span>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { SkillLogEntry } from '@/types/skills'

const props = withDefaults(defineProps<{
  entries: SkillLogEntry[]
  maxEntries?: number
}>(), { maxEntries: 8 })

const visibleEntries = computed(() => props.entries.slice(0, props.maxEntries))

function formatTime(timestamp: number) {
  return new Date(timestamp).toLocaleTimeString()
}
</script>

<style scoped>
.activity-list {
  @apply flex flex-col gap-2;
}

.activity-row {
  @apply flex items-start gap-3 rounded-2xl border border-white/10 bg-white/5 p-3;
}

.activity-row__status {
  @apply mt-1 h-2.5 w-2.5 rounded-full bg-white/25;
}

.activity-row__status--success {
  @apply bg-emerald-400;
}

.activity-row__status--error {
  @apply bg-rose-400;
}

.activity-row__status--pending {
  @apply bg-amber-300;
}

.activity-row__body {
  @apply flex flex-col gap-0.5 text-xs text-white/60;
}

.activity-row__body strong {
  @apply text-sm text-white/80;
}
</style>
