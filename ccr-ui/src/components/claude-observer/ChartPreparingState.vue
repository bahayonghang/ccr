<template>
  <div class="chart-preparing-state">
    <div
      class="chart-preparing-state__bars"
      aria-hidden="true"
    >
      <span
        v-for="height in barHeights"
        :key="height"
        :style="{ height: `${height}%` }"
      />
    </div>
    <span class="chart-preparing-state__label">
      {{ displayLabel }}
    </span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const props = withDefaults(defineProps<{
  label?: string
}>(), {
  label: undefined,
})

const { t } = useI18n()
const barHeights = [34, 58, 46, 76, 52, 68, 40, 60]
const displayLabel = computed(() => props.label ?? t('claudeCode.observer.chart.preparing'))
</script>

<style scoped>
.chart-preparing-state {
  display: grid;
  width: 100%;
  min-height: 100%;
  place-items: center;
  gap: 0.75rem;
  border-radius: 1rem;
  border: 1px dashed rgb(var(--color-border-default-rgb) / 24%);
  background: var(--color-bg-surface);
  color: var(--color-text-muted);
  font-size: 0.85rem;
}

.chart-preparing-state__bars {
  display: flex;
  align-items: end;
  gap: 0.28rem;
  height: 3.25rem;
}

.chart-preparing-state__bars span {
  width: 0.42rem;
  min-height: 0.7rem;
  border-radius: 9999px;
  background: rgb(var(--color-accent-primary-rgb) / 28%);
  animation: chart-preparing-pulse 1.1s ease-in-out infinite;
}

.chart-preparing-state__bars span:nth-child(2n) {
  background: rgb(var(--color-accent-secondary-rgb) / 28%);
  animation-delay: 90ms;
}

.chart-preparing-state__bars span:nth-child(3n) {
  background: rgb(var(--color-info-rgb) / 24%);
  animation-delay: 160ms;
}

.chart-preparing-state__label {
  color: var(--color-text-muted);
  font-weight: 600;
}

@keyframes chart-preparing-pulse {
  0%,
  100% {
    opacity: 0.45;
    transform: scaleY(0.72);
  }

  50% {
    opacity: 1;
    transform: scaleY(1);
  }
}

@media (prefers-reduced-motion: reduce) {
  .chart-preparing-state__bars span {
    animation: none;
  }
}
</style>
