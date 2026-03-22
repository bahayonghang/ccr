<template>
  <div class="heatmap-header">
    <div class="header-left">
      <div class="icon-wrapper">
        <svg
          class="icon"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M17.657 18.657A8 8 0 016.343 7.343S7 9 9 10c0-2 .5-5 2.986-7C14 5 16.09 5.777 17.656 7.343A7.975 7.975 0 0120 13a7.975 7.975 0 01-2.343 5.657z"
          />
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M9.879 16.121A3 3 0 1012.015 11L11 14H9c0 .768.293 1.536.879 2.121z"
          />
        </svg>
      </div>
      <h3 class="title">
        {{ $t('activityHeatmap.title', '活动热力图') }}
      </h3>
    </div>

    <div class="legend">
      <span class="legend-label">{{ $t('activityHeatmap.less', '较少') }}</span>
      <div class="legend-cells">
        <div
          v-for="level in 5"
          :key="level"
          class="legend-cell"
          :data-level="level - 1"
        />
      </div>
      <span class="legend-label">{{ $t('activityHeatmap.more', '较多') }}</span>
    </div>
  </div>
</template>

<style scoped>
.heatmap-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space-4);
}

.header-left {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.icon-wrapper {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, var(--color-accent-primary), var(--color-accent-secondary));
  border-radius: var(--radius-lg);
  box-shadow: var(--glow-primary);
}

.icon {
  width: 18px;
  height: 18px;
  color: white;
}

.title {
  font-size: var(--text-base);
  font-weight: var(--font-semibold);
  color: var(--color-text-primary);
  margin: 0;
}

.legend {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-1-5) var(--space-3);
  background: var(--color-bg-surface);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-full);
}

.legend-label {
  font-size: var(--text-xs);
  color: var(--color-text-muted);
  font-weight: var(--font-medium);
}

.legend-cells {
  display: flex;
  gap: 3px;
}

.legend-cell {
  width: 10px;
  height: 10px;
  border-radius: 2px;
  transition: transform var(--duration-fast) var(--ease-out);
}

.legend-cell:hover {
  transform: scale(1.2);
}

.legend-cell[data-level='0'] { background: var(--color-bg-overlay); }
.legend-cell[data-level='1'] { background: rgb(6 182 212 / 30%); }
.legend-cell[data-level='2'] { background: rgb(6 182 212 / 50%); }
.legend-cell[data-level='3'] { background: rgb(6 182 212 / 75%); }
.legend-cell[data-level='4'] { background: var(--color-accent-primary); }

@media (width <= 640px) {
  .heatmap-header {
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-3);
  }
}
</style>
