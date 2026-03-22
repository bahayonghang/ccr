<template>
  <div class="stats-row">
    <div
      v-for="item in items"
      :key="item.id"
      class="stat-card"
      :class="item.id === 'activeDays' ? 'stat-card-primary' : 'stat-card-secondary'"
    >
      <div class="stat-icon">
        <svg
          v-if="item.id === 'activeDays'"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
          />
        </svg>
        <svg
          v-else
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6"
          />
        </svg>
      </div>
      <div class="stat-content">
        <span class="stat-label">{{ item.label }}</span>
        <span class="stat-value">{{ item.value }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { ActivityHeatmapStatItem } from '@/types/activityHeatmap'

defineProps<{
  items: ActivityHeatmapStatItem[]
}>()
</script>

<style scoped>
.stats-row {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: var(--space-3);
  margin-top: var(--space-4);
  padding-top: var(--space-4);
  border-top: 1px solid var(--color-border-subtle);
}

.stat-card {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  border-radius: var(--radius-lg);
  border: 1px solid var(--color-border-subtle);
  transition: all var(--duration-normal) var(--ease-out);
}

.stat-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}

.stat-card-primary {
  background: linear-gradient(135deg, rgb(6 182 212 / 10%), rgb(6 182 212 / 5%));
  border-color: rgb(6 182 212 / 20%);
}

.stat-card-primary:hover {
  border-color: rgb(6 182 212 / 40%);
  box-shadow: var(--glow-primary);
}

.stat-card-secondary {
  background: linear-gradient(135deg, rgb(139 92 246 / 10%), rgb(139 92 246 / 5%));
  border-color: rgb(139 92 246 / 20%);
}

.stat-card-secondary:hover {
  border-color: rgb(139 92 246 / 40%);
  box-shadow: var(--glow-secondary);
}

.stat-icon {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-md);
}

.stat-card-primary .stat-icon {
  background: rgb(6 182 212 / 15%);
  color: var(--color-accent-primary);
}

.stat-card-secondary .stat-icon {
  background: rgb(139 92 246 / 15%);
  color: var(--color-accent-secondary);
}

.stat-icon svg {
  width: 18px;
  height: 18px;
}

.stat-content {
  display: flex;
  flex-direction: column;
  gap: var(--space-0-5);
}

.stat-label {
  font-size: var(--text-xs);
  font-weight: var(--font-medium);
  color: var(--color-text-muted);
}

.stat-value {
  font-size: var(--text-xl);
  font-weight: var(--font-bold);
  color: var(--color-text-primary);
  font-family: var(--font-mono);
}

@media (width <= 640px) {
  .stats-row {
    grid-template-columns: 1fr;
  }
}
</style>
