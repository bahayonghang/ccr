<template>
  <Teleport to="body">
    <Transition name="tooltip">
      <div
        v-if="tooltip.visible"
        class="heatmap-tooltip"
        :style="{
          left: `${tooltip.x}px`,
          top: `${tooltip.y}px`
        }"
      >
        <div class="tooltip-date">
          {{ tooltip.date }}
        </div>
        <div class="tooltip-value">
          <span class="tooltip-count">{{ formattedCount }}</span>
          <span class="tooltip-unit">tokens</span>
        </div>
        <div class="tooltip-arrow" />
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import type { ActivityHeatmapTooltipState } from '@/types/activityHeatmap'

defineProps<{
  tooltip: ActivityHeatmapTooltipState
  formattedCount: string
}>()
</script>

<style scoped>
.heatmap-tooltip {
  position: fixed;
  z-index: var(--z-tooltip);
  padding: var(--space-2) var(--space-3);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-xl);
  backdrop-filter: var(--glass-blur-md);
  transform: translate(-50%, -100%);
  pointer-events: none;
}

.tooltip-date {
  font-size: var(--text-xs);
  font-weight: var(--font-semibold);
  color: var(--color-text-primary);
  margin-bottom: var(--space-1);
}

.tooltip-value {
  display: flex;
  align-items: baseline;
  gap: var(--space-1);
}

.tooltip-count {
  font-size: var(--text-sm);
  font-weight: var(--font-bold);
  color: var(--color-accent-primary);
  font-family: var(--font-mono);
}

.tooltip-unit {
  font-size: var(--text-xs);
  color: var(--color-text-muted);
}

.tooltip-arrow {
  position: absolute;
  bottom: -6px;
  left: 50%;
  transform: translateX(-50%);
  width: 0;
  height: 0;
  border-left: 6px solid transparent;
  border-right: 6px solid transparent;
  border-top: 6px solid var(--color-bg-elevated);
}

.tooltip-enter-active,
.tooltip-leave-active {
  transition: all var(--duration-fast) var(--ease-out);
}

.tooltip-enter-from,
.tooltip-leave-to {
  opacity: 0;
  transform: translate(-50%, -90%);
}
</style>
