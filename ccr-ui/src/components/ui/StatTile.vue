<template>
  <div class="stat-tile">
    <p class="stat-tile__label">
      <slot name="label">
        {{ label }}
      </slot>
    </p>
    <p class="stat-tile__value">
      <slot name="value">
        {{ displayValue }}
      </slot>
    </p>
    <p
      v-if="hint || $slots.hint"
      class="stat-tile__hint"
    >
      <slot name="hint">
        {{ hint }}
      </slot>
    </p>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  label?: string
  value?: string | number
  hint?: string
}

const props = withDefaults(defineProps<Props>(), {
  label: undefined,
  value: undefined,
  hint: undefined,
})

const displayValue = computed(() => {
  if (props.value === undefined) return '—'
  return props.value
})
</script>

<style scoped>
.stat-tile {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  min-width: 0;
}

.stat-tile__label {
  margin: 0;
  font-size: 0.8125rem;
  font-weight: 500;
  line-height: 1.24;
  letter-spacing: 0;
  color: var(--color-text-muted);
}

.stat-tile__value {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 600;
  line-height: 1.2;
  letter-spacing: 0;
  font-variant-numeric: tabular-nums;
  color: var(--color-text-primary);
}

.stat-tile__hint {
  margin: 0;
  font-size: 0.8125rem;
  font-weight: 400;
  line-height: 1.24;
  color: var(--color-text-muted);
}
</style>
