<template>
  <div class="stat-tile">
    <p class="stat-tile__label">
      <slot name="label">
        {{ label }}
      </slot>
    </p>
    <p
      class="stat-tile__value"
      :class="{ 'stat-tile__value--badge': tone }"
      :data-tone="tone || undefined"
    >
      <span
        v-if="tone"
        class="stat-tile__tone-dot"
        aria-hidden="true"
      />
      <span
        v-if="tone"
        class="stat-tile__value-text"
      >
        <slot name="value">
          {{ displayValue }}
        </slot>
      </span>
      <template v-else>
        <slot name="value">
          {{ displayValue }}
        </slot>
      </template>
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

// 联合写在原语上，避免 UI 组件反向依赖 dashboard 视图模块
export type StatTileTone = 'neutral' | 'success' | 'warning' | 'danger' | 'accent'

interface Props {
  label?: string
  value?: string | number
  hint?: string
  tone?: StatTileTone
}

const props = withDefaults(defineProps<Props>(), {
  label: undefined,
  value: undefined,
  hint: undefined,
  tone: undefined,
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

.stat-tile__value--badge {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;

  /* 列 flex 默认 stretch，否则壳会被拉满栅格列 */
  align-self: start;
  width: auto;
  min-width: 0;
  max-width: 100%;
  padding: 0.15rem 0.5rem;

  /* 卡 12px + pad 16px，内壳取 8px（--radius-lg）；--radius-md 实际是 6px */
  border-radius: var(--radius-lg);
  border: 1px solid var(--color-border-subtle);
  background: var(--color-bg-overlay);
}

.stat-tile__value-text {
  min-width: 0;
  white-space: nowrap;
}

.stat-tile__tone-dot {
  flex-shrink: 0;
  width: 6px;
  height: 6px;
  border-radius: var(--radius-full);
  background: var(--color-text-muted);
}

.stat-tile__value--badge[data-tone='neutral'] {
  background: var(--color-bg-overlay);
  border-color: var(--color-border-subtle);
}

.stat-tile__value--badge[data-tone='neutral'] .stat-tile__tone-dot {
  background: var(--color-text-muted);
}

.stat-tile__value--badge[data-tone='success'] {
  background: rgb(var(--color-success-rgb) / 10%);
  border-color: rgb(var(--color-success-rgb) / 18%);
}

.stat-tile__value--badge[data-tone='success'] .stat-tile__tone-dot {
  background: var(--color-success);
}

.stat-tile__value--badge[data-tone='warning'] {
  background: rgb(var(--color-warning-rgb) / 10%);
  border-color: rgb(var(--color-warning-rgb) / 18%);
}

.stat-tile__value--badge[data-tone='warning'] .stat-tile__tone-dot {
  background: var(--color-warning);
}

.stat-tile__value--badge[data-tone='danger'] {
  background: rgb(var(--color-danger-rgb) / 10%);
  border-color: rgb(var(--color-danger-rgb) / 18%);
}

.stat-tile__value--badge[data-tone='danger'] .stat-tile__tone-dot {
  background: var(--color-danger);
}

.stat-tile__value--badge[data-tone='accent'] {
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  border-color: rgb(var(--color-accent-primary-rgb) / 18%);
}

.stat-tile__value--badge[data-tone='accent'] .stat-tile__tone-dot {
  background: var(--color-accent-primary);
}

.stat-tile__hint {
  margin: 0;
  font-size: 0.8125rem;
  font-weight: 400;
  line-height: 1.24;
  color: var(--color-text-muted);
}
</style>
